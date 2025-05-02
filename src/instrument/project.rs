use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use crate::instrument::is_hidden_path;
use crate::instrument::source::*;

#[derive(Debug)]
pub struct InstProjectArgs {
    /// The contents of this folder will be overwritten
    pub output_dir: PathBuf,

    /// Info on the project to be instrumented
    pub input_project: InstInputProject,

    /// Which kind of instrumentation to perform
    pub inst_spec: InstProjectSpec,
}

#[derive(Debug)]
pub struct InstInputProject {
    pub project_type: InstInputProjectType,
    pub target_dir: PathBuf,
}

#[derive(Debug)]
pub enum InstInputProjectType {
    Workspace {
        program_path: PathBuf,
        root_path: PathBuf,
    },
    Package {
        program_path: PathBuf,
    },
}

#[derive(Debug)]
pub enum InstProjectSpec {
    SingleLine {
        file: PathBuf,
        line: usize,
    }
}

/// Information on the project that is the instrumented copy of the input project
#[derive(Debug)]
pub struct InstProjectInfo {
    pub program_path: PathBuf,
    pub workspace_root: PathBuf,
    pub is_workspace: bool,
}

pub fn inst_project(args: InstProjectArgs) -> Result<InstProjectInfo, Box<dyn std::error::Error>> {

    // Prepare output dir
    let output_dir = args.output_dir;
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).map_err(|_| "inst_project: Failed to remove output dir")?;
    }
    fs::create_dir_all(&output_dir).map_err(|_| "inst_project: Failed to create output dir")?;

    match args.input_project.project_type {
        InstInputProjectType::Package { program_path } => {
            inst_project_package(&program_path, &output_dir, &args.inst_spec)?;
            Ok(
                InstProjectInfo {
                    program_path: output_dir.clone(),
                    workspace_root: output_dir.clone(),
                    is_workspace: false,
                }
            )
        }
        InstInputProjectType::Workspace { program_path, root_path } => {

            if !program_path.starts_with(&root_path) {
                Err("inst_project: Invalid workspace root")?;
            }

            inst_project_workspace(&root_path, &output_dir, &program_path, &args.input_project.target_dir, &args.inst_spec)?;

            let relative_program_path = program_path.strip_prefix(&root_path).unwrap();
            let output_program_path = output_dir.join(relative_program_path);

            Ok(
                InstProjectInfo {
                    program_path: output_program_path,
                    workspace_root: output_dir.clone(),
                    is_workspace: true,
                }
            )
        }
    }
}

fn inst_project_package(input_path: &Path, output_path: &Path, inst_spec: &InstProjectSpec) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_config_path = input_path.join("Cargo.toml");
    if !cargo_config_path.exists() {
        Err("Cargo.toml not found")?
    }
    let cargo_config_output_path = output_path.join("Cargo.toml");
    //eprintln!("Copy {} to {}", cargo_config_path.display(), cargo_config_output_path.display());
    fs::copy(cargo_config_path, cargo_config_output_path)?;

    let cargo_lock_path = input_path.join("Cargo.lock");
    if cargo_lock_path.exists() {
        let cargo_lock_output_path = output_path.join("Cargo.lock");
        //eprintln!("Copy {} to {}", cargo_lock_path.display(), cargo_lock_output_path.display());
        fs::copy(cargo_lock_path, cargo_lock_output_path)?;
    }

    let source_path = input_path.join("src");
    if !source_path.is_dir() {
        Err(format!("Source directory {} doesn't exist", source_path.display()))?;
    }

    let source_path_out = output_path.join("src");
    fs::create_dir(&source_path_out)?;

    inst_source(&source_path, &source_path_out, inst_spec)?;

    Ok(())
}

fn inst_project_workspace(
    workspace_path: &Path,
    output_path: &Path,
    debugee_path: &Path,
    input_target_dir: &Path,
    inst_spec: &InstProjectSpec
) -> Result<(), Box<dyn std::error::Error>> {

    let mut queue = VecDeque::<(PathBuf, PathBuf)>::new();
    queue.push_back((workspace_path.into(), output_path.into()));

    while let Some((input_dir, output_dir)) = queue.pop_front() {
        for entry in fs::read_dir(&input_dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap();

            // .git etc.
            if is_hidden_path(file_name) {
                continue;
            }

            if path.is_dir() {
                if path == input_target_dir {
                    continue;
                }

                let new_output_dir = output_dir.join(file_name);
                fs::create_dir(&new_output_dir)?;

                if path != debugee_path {
                    queue.push_back((path, new_output_dir));
                } else {
                    inst_project_package(&path, &new_output_dir, inst_spec)?;
                }
            } else if path.is_file() {
                let new_output_file = output_dir.join(file_name);
                fs::copy(&path, &new_output_file)?;
            }
        }
    }

    Ok(())
}