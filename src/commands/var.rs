use std::path::PathBuf;
use log::debug;
use crate::compile::project::{compile_project, CompileProjectArgs};
use crate::utils::debugger_cache::*;
use crate::utils::debugee_project_info::get_program_info;
use crate::utils::program_input::*;
use crate::instrument::*;
use crate::output::*;

pub(crate) async fn process_var(location: &str, line: usize, name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {

    //
    // Input Validation
    //

    if !get_cache_dir().is_dir() {
        Err("Cache directory does not exist. Run 'init' to create it")?
    }

    let config: DebuggerConfig = DebuggerConfig::load_from_file(&get_config_path())?;

    //dbg!(&config);

    config.validate()?;

    // Validate location
    let location_path = config.program_path.join(location);
    if !location_path.is_file() {
        Err(format!("Debug location {} does not exist", location_path.display()))?
    }

    // Must be before load_input_from_folder
    let output_log = set_output_logger()?;

    let program_input = load_input_from_folder(&config.input_path).await?;

    //dbg!(&program_input);

    let debugee_project_info = get_program_info(&config.program_path)?;

    //dbg!(&debugee_project_info);

    //
    // Instrument
    //

    eprintln!("Instrument...");

    let project_type = match debugee_project_info.is_workspace {
        false => InstInputProjectType::Package { program_path: debugee_project_info.program_path.clone() },
        true => InstInputProjectType::Workspace {
            root_path: debugee_project_info.workspace_root.clone(),
            program_path: debugee_project_info.program_path.clone(),
        }
    };

    let inst_args = InstProjectArgs {
        output_dir: get_build_dir(),
        input_project: InstInputProject {
            project_type,
            target_dir: debugee_project_info.target_directory.clone(),
        },
        inst_spec: InstProjectSpec::SingleLine { file: location_path, line },
    };

    let inst_info = inst_project(inst_args)?;

    //dbg!(&inst_info);

    //
    // Compile
    //

    //rm_target_dir();

    eprintln!("Compile...");

    let compile_args = CompileProjectArgs {
        program_path: inst_info.program_path,
        workspace_root: inst_info.workspace_root,
        target_dir: Some(get_target_dir())
    };

    compile_project(compile_args).await?;

    //
    // Output
    //

    eprintln!("Output...");

    let program_output = generate_program_output(
        &get_target_so_dir(),
        &debugee_project_info.target_name,
        program_input,
        output_log
    ).await?;

    //dbg!(&program_output);

    let line_vars = parse_program_output(program_output)?;

    if line_vars.is_empty() {
        eprintln!("No variables data: line was never hit");
    } else {
        println!();
    }

    for (j, item) in line_vars.iter().enumerate() {
        println!("Hit {}", j+1);
        println!("{}:{}", location, item.line_num);
        match name {
            None => {
                for (i, node) in item.nodes.iter().enumerate() {
                    print_debug_node_colored(node, 0);
                    //print_debug_node(node, 0);
                    if i < item.nodes.len() - 1 {
                        println!();
                    }
                }
            },
            Some(name) => {
                let node = item.nodes.iter().find(|n| n.name == name);
                if let Some(node) = node {
                    print_debug_node_colored(node, 0);
                } else {
                    println!("Variable {} not available", name);
                }
            }
        }
        if j < line_vars.len() - 1 {
            println!();
        }
    }

    Ok(())
}