use std::path::{Path, PathBuf};
use cargo_metadata::{MetadataCommand, Target, TargetKind};

#[derive(Debug)]
pub struct DebugeeProjectInfo {
    pub program_path: PathBuf,
    pub workspace_root: PathBuf,
    pub is_workspace: bool,
    pub target_directory: PathBuf,
    pub target_name: String,
}

pub fn get_program_info(program_path: &Path) -> Result<DebugeeProjectInfo, Box<dyn std::error::Error>> {
    if !program_path.is_dir() {
        Err("program_path is not a directory")?;
    }

    // Run cargo metadata
    // This will create a Cargo.lock file if it doesn't exist yet (there's no option to disable this)
    let program_path = program_path.canonicalize()?;
    let manifest_path = program_path.join("Cargo.toml").canonicalize()?;
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()?;
    let program_package = metadata.packages.iter().find(|package| {
        PathBuf::from(&package.manifest_path) == manifest_path
    }).ok_or("Could not find debug program package in cargo metadata output")?;
    let find_target = program_package.targets.iter().find(|&t|
        t.kind.contains(&TargetKind::CDyLib) && t.kind.contains(&TargetKind::Lib)
    );
    if find_target.is_none() {
        dbg!(&program_package.targets);
        Err(format!("Failed to find a cdylib + lib target in package {}", program_package.name))?;
    }
    let target = find_target.unwrap();
    // For a single Cargo package, this will be it's root folder, i.e. it will be equal to program_path
    let workspace_root = PathBuf::from(metadata.workspace_root);
    let is_workspace = workspace_root != program_path;
    // For a workspace, this is usually $workspace_root/target
    let target_directory = PathBuf::from(metadata.target_directory);
    let target_name = target.name.clone();

    Ok(
        DebugeeProjectInfo {
            program_path,
            workspace_root,
            is_workspace,
            target_directory,
            target_name,
        }
    )
}