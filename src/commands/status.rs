use crate::utils::debugger_cache::*;

pub(crate) fn process_status() -> Result<(), Box<dyn std::error::Error>> {
    if !get_cache_dir().is_dir() {
        Err("Cache directory does not exist. Run 'init' to create it")?
    }
    let config: DebuggerConfig = DebuggerConfig::load_from_file(&get_config_path())?;
    config.validate()?;
    println!("{:#?}", config);
    Ok(())
}