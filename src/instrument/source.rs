use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::instrument::*;

pub fn inst_source(input_path: &Path, output_path: &Path, inst_spec: &InstProjectSpec) -> Result<(), Box<dyn std::error::Error>> {
    if let InstProjectSpec::SingleLine { .. } = inst_spec {
        write_fixed_serialization_file(&output_path.join("_solana_debugger_serialize.rs"))?;
    }

    let mut queue = VecDeque::<(PathBuf, PathBuf)>::new();
    queue.push_back((input_path.into(), output_path.into()));

    while let Some((input_dir, output_dir)) = queue.pop_front() {
        for entry in fs::read_dir(&input_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();

            if is_hidden_path(file_name) {
                continue;
            }

            if path.is_dir() {
                let new_output_dir = output_dir.join(file_name);
                fs::create_dir(&new_output_dir)?;
                queue.push_back((path, new_output_dir));
            } else if path.is_file() {
                let new_output_file = output_dir.join(file_name);

                if let InstProjectSpec::SingleLine { file, line } = &inst_spec {

                    // TODO: path should be dynamically obtained
                    let is_main_module = new_output_file.ends_with("src/lib.rs");

                    // TODO: should be something like "src/lib.rs"
                    let file_path_str = "".to_string();

                    let line_inst = if path == file.to_owned() { Some(*line) } else { None };

                    let ast_spec = InstAstSpec {
                        mod_fixed_serialization: is_main_module,
                        feature_min_specialization: is_main_module,
                        debugee_file_path: file_path_str,
                        line_inst,
                        custom_type_serialization: true
                    };

                    inst_source_file(&path, &new_output_file, &ast_spec)?;
                }
            }
        }
    }

    Ok(())
}

fn inst_source_file(input_file_path: &Path, output_file_path: &Path, spec: &InstAstSpec) -> Result<(), Box<dyn std::error::Error>> {
    //eprintln!("Process {}", input_file_path.display());
    let input_file_contents = fs::read_to_string(input_file_path)?;
    let input_ast = syn::parse_file(&input_file_contents)?;
    let output_ast = inst_ast(input_ast, spec);
    let output_file_contents = prettyplease::unparse(&output_ast);

    //eprintln!("Write {}", output_file_path.display());
    let mut output_file = File::create(output_file_path)?;
    output_file.write_all(output_file_contents.as_bytes())?;

    Ok(())
}

fn write_fixed_serialization_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut output_file = File::create(&path)?;
    let trait_code = crate::instrument::get_fixed_serialization();
    let contents = prettyplease::unparse(&trait_code);
    output_file.write_all(contents.as_bytes())?;
    Ok(())
}
