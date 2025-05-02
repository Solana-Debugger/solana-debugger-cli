use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Represents `~/.cache/solana_debugger/config.json`
#[derive(Debug, Serialize, Deserialize)]
pub struct DebuggerConfig {
    /// May panic on non-UTF-8 characters
    pub program_path: PathBuf,
    pub input_path: PathBuf
}

impl DebuggerConfig {

    /// Try to create a new config from (possibly relative) paths
    pub fn new_from_input(program_path: &str, input_path: &str) -> Result<Self, Box<dyn std::error::Error>> {

        if ! PathBuf::from(&program_path).is_dir() {
            Err(format!("program_path is not a directory: {}", program_path))?;
        }
        if ! PathBuf::from(&input_path).is_dir() {
            Err(format!("input_path is not a directory: {}", input_path))?;
        }

        let program_path: PathBuf = fs::canonicalize(program_path)?;
        let input_path: PathBuf = fs::canonicalize(input_path)?;

        Ok(
            DebuggerConfig {
                program_path,
                input_path,
            }
        )
    }

    pub fn validate(&self) -> Result<(), String> {
        if ! self.program_path.is_dir() {
            Err(format!("program_path is not a directory: {}", self.program_path.display()))?;
        }
        if ! self.input_path.is_dir() {
            Err(format!("input_path is not a directory: {}", self.input_path.display()))?;
        }
        Ok(())
    }

    pub fn write_to_file(&self, path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> std::io::Result<DebuggerConfig> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}

/// Solana Debugger's cache directory is `~/.cache/solana_debugger`
pub fn get_cache_dir() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    home_dir.join(".cache").join("solana_debugger")
}

pub fn get_build_dir() -> PathBuf {
    get_cache_dir().join("build")
}

/// Custom target dir, outside of `build`
pub fn get_target_dir() -> PathBuf {
    get_cache_dir().join("target")
}

pub fn rm_target_dir() {
    fs::remove_dir_all(get_target_dir()).unwrap()
}

pub fn get_config_path() -> PathBuf {
    get_cache_dir().join("config.json")
}

pub fn ensure_cache_dir() {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");
}