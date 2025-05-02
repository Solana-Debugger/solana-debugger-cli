use std::ops::Range;
use std::path::Path;
use std::process::{Command, Stdio};
use cargo_metadata::diagnostic::DiagnosticLevel;
use cargo_metadata::Message;

#[derive(Debug)]
pub struct CompileError {
    pub file_path: String,
    pub error_code: String,
    pub source_spans: Vec<Range<usize>>,
    pub error_message: String,
}

/// Try to compile to SBF, but expect compile errors
/// The Ok value is Vec<CompileError> since compiling with errors is considered expected behavior in our case
pub async fn compile_sbf_with_errors(program_path: &Path, target_dir: Option<&Path>) -> Result<Vec<CompileError>, Box<dyn std::error::Error>> {

    //eprintln!("Compile SBF: {}", program_path.display());

    // This is from cargo-build-sbf's `build_solana_package`
    let mut cargo_build_args = vec![
        // select Solana toolchain
        "+solana",
        "build",
        // Do NOT remove this even if it's faster!
        // Without this, you get compiler warnings like that: "[...] The function call may cause undefined behavior during execution."
        "--release",
        "--target",
        "sbf-solana-solana"
    ];

    // Make sure we only compile the single lib target
    cargo_build_args.extend(["--lib"]);

    // Enable debug output
    cargo_build_args.extend(["--message-format", "json,json-diagnostic-short"]);

    if let Some(target_dir) = target_dir {
        cargo_build_args.extend(["--target-dir", target_dir.to_str().unwrap()]);
    }

    let mut command = Command::new("cargo")
        .args(&cargo_build_args)
        .current_dir(&program_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        // Do NOT set this (it may be faster, but it causes compiler warnings)
        // .env("RUSTFLAGS", "-C opt-level=0")
        .spawn()
        .map_err(|err| format!("Failed to run cargo: {}", err))?;

    let reader = std::io::BufReader::new(command.stdout.take().unwrap());

    let mut errs: Vec<CompileError> = vec![];

    for message in cargo_metadata::Message::parse_stream(reader) {
        match message.unwrap() {
            Message::CompilerMessage(msg) => {
                let msg = msg.message;

                // Ignore warnings etc.
                if msg.level != DiagnosticLevel::Error {
                    continue;
                }
                // This is usually something like "aborting due to ..."
                // We can ignore this
                if msg.code.is_none() {
                    continue;
                }
                //dbg!(&msg);

                let error_code = msg.code.clone().unwrap().code;
                if msg.spans.is_empty() {
                    Err("Cargo returned empty span")?;
                }
                let prim_span = msg.spans.iter().find(|x| x.is_primary).ok_or("No primary span found")?;
                let file_path = prim_span.file_name.clone();
                let source_spans = msg.spans.iter().map(|x| x.byte_start as usize..x.byte_end as usize).collect::<Vec<_>>();
                let error_message = msg.rendered.unwrap_or("N/A".into()).trim().to_string();

                errs.push(CompileError {
                    file_path,
                    error_code,
                    source_spans,
                    error_message
                })
            }
            _ => {}
        }
    }

    let output = command.wait_with_output().map_err(|err| format!("Failed to get output of cargo: {}", err))?;

    if !output.status.success() && errs.is_empty() {
        Err("Compilation failed, but cargo-build-sbf didn't return compile errors")?;
    }

    Ok(errs)
}