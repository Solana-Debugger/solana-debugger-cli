use std::path::Path;
use crate::compile::sbf_with_errors::CompileError;

pub fn correct_file(path: &Path, errors: Vec<CompileError>) -> Result<(), Box<dyn std::error::Error>> {
   Ok(())
}
