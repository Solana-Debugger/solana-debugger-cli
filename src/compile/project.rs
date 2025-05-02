use std::path::PathBuf;

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


    Ok(())
}
