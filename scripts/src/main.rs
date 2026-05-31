use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod help;
mod xml_to_yaml;

#[derive(Parser)]
#[command(name = "scripts")]
#[command(about = "L2Shablya management scripts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Prints help information
    #[command(name = "info")]
    Help,

    /// Converts XML files to YAML
    XmlToYaml {
        /// Input file or directory
        input: PathBuf,
        /// Output file or directory
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Help => {
            help::print_help();
        }
        Commands::XmlToYaml { input, output } => {
            if let Err(e) = xml_to_yaml::run(input, output) {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
