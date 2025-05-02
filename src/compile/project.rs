use std::collections::HashMap;
use std::path::PathBuf;
use crate::compile::correct_file::correct_file;
use crate::compile::sbf_with_errors::{compile_sbf_with_errors, CompileError};

#[derive(Debug)]
pub struct CompileProjectArgs {
    /// The path to the program to be compiled
    pub program_path: PathBuf,

    /// The path to the workspace of the program to be compiled (needed to find the files in which an error occurs)
    pub workspace_root: PathBuf,

    /// Custom `target` dir for cargo build
    pub target_dir: Option<PathBuf>,
}

pub async fn compile_project(args: CompileProjectArgs) -> Result<(), Box<dyn std::error::Error>> {
    let CompileProjectArgs { program_path, workspace_root, target_dir } = args;
    let target_dir = target_dir.as_ref().map(|dir| dir.as_path());

    /// Compile and correct approach
    /// If the compiler returns an error, correct the respective files. Try to compile again. Do this until it compiles.
    loop {
        let compile_errors = compile_sbf_with_errors(&program_path, target_dir).await?;
        //dbg!(&compile_errors);

        if compile_errors.is_empty() {
            return Ok(())
        }

        let files_map = files_to_errors(compile_errors);
        //dbg!(&files_map);

        for (file_path, errors) in  files_map {
            let full_path = workspace_root.join(file_path);
            correct_file(&full_path, errors)?;
        }
    }
}

fn files_to_errors(errs: Vec<CompileError>) -> HashMap<String, Vec<CompileError>>{
    let mut result: HashMap<String, Vec<CompileError>> = HashMap::new();
    for err in errs {
        result.entry(err.file_path.clone()).or_default().push(err);
    }
    result
}