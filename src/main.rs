mod commands;
mod utils;
mod instrument;
mod compile;

use clap::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = clap::Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .subcommand(
            Command::new("init")
                .about("Set a debug configuration")
                .arg(Arg::new("program_path")
                    .help("Path to the Solana program to be debugged")
                    .required(true))
                .arg(Arg::new("input_path")
                    .help("Path to a folder containing the input to the program")
                    .required(true))
        )
        .subcommand(
            Command::new("var")
                .about("Inspect the value of variables")
                .arg(Arg::new("location")
                    .help("Location to inspect. Format: FILE:LINE, e.g. lib.rs:33 (without `src/`)")
                    .required(true))
                .arg(Arg::new("variable_name")
                    .help("Name of the variable to inspect. Leave empty to show all")
                    .required(false))
        );

    let matches = cli.get_matches_mut();

    match matches.subcommand() {
        Some(("init", sub_m)) => subcommand_init(sub_m),
        Some(("var", sub_m)) => subcommand_var(sub_m).await,
        _ => {
            eprintln!("Invalid subcommand. Help:");
            eprintln!();
            cli.print_help().unwrap();
            std::process::exit(1)
            // Don't need to return Err
        }
    }
}

fn subcommand_init(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let program_path = matches.get_one::<String>("program_path").unwrap();
    let input_path = matches.get_one::<String>("input_path").unwrap();

    commands::init::process_init(program_path, input_path)?;

    Ok(())
}

async fn subcommand_var(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let location_str = matches.get_one::<String>("location").unwrap();

    let split: Vec<&str> = location_str.rsplitn(2, ':').collect();

    if split.len() != 2 {
        Err("Invalid format of location")?;
    }

    let line_number = split[0].parse::<usize>()
        .map_err(|_| format!("Invalid line number: {}", split[0]))?;

    let file_path = split[1];

    // We assume that the source files are stored in `src`
    // We prepend `src/` for convenience
    let file_path = "src/".to_string() + file_path;

    let variable_name = matches.get_one::<String>("variable_name").map(|v| v.as_str());

    commands::var::process_var(&file_path, line_number, variable_name).await?;

    Ok(())
}