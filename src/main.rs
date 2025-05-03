mod commands;
mod utils;
mod instrument;
mod compile;
mod output;

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
                .arg(Arg::new("variable_names")
                    .help("Name of variables to inspect. Leave empty to show all")
                    .required(false))
        );

    let mut processed_args = get_processed_args();

    let matches = cli.try_get_matches_from_mut(processed_args).unwrap_or_else(|e| e.exit());

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

fn get_processed_args() -> Vec<String> {
    let mut args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && try_get_file_line_format(&args[1]).is_ok() {
        args.insert(1, "var".to_string());
    }

    args
}

fn subcommand_init(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let program_path = matches.get_one::<String>("program_path").unwrap();
    let input_path = matches.get_one::<String>("input_path").unwrap();

    commands::init::process_init(program_path, input_path)?;

    Ok(())
}

async fn subcommand_var(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let location_str = matches.get_one::<String>("location").unwrap();

    let (file_path, line_number) = try_get_file_line_format(&location_str)?;

    // We assume that the source files are stored in `src`
    // We prepend `src/` for convenience
    let file_path = "src/".to_string() + file_path.as_str();

    let variable_name = matches.get_many::<String>("variable_names").map(|v| v.as_str());

    //commands::var::process_var(&file_path, line_number, variable_name).await?;

    Ok(())
}

fn try_get_file_line_format(input: &str) -> Result<(String, usize), String> {

    let split: Vec<String> = input.rsplitn(2, ':').map(|v| v.to_string()).collect();

    if split.len() != 2 {
        Err("Invalid format of location")?;
    }

    let line_number = split[0].parse::<usize>()
        .map_err(|_| format!("Invalid line number: {}", split[0]))?;

    let file_path = split[1].clone();

    Ok((file_path, line_number))
}