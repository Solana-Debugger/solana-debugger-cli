use crate::utils::debugger_cache::*;

pub(crate) fn process_init(program_path: &str, input_path: &str) -> Result<(), Box<dyn std::error::Error>> {

    ensure_cache_dir();

    let config = DebuggerConfig::new_from_input(program_path, input_path)?;

    //dbg!(&config);

    config.write_to_file(&get_config_path())?;

    rm_target_dir();

    Ok(())
}