#[macro_use]
extern crate simple_log;

mod client;
mod constants;
mod commands {
    pub mod domain;
    pub mod dyndns;
    pub mod invoices;
}

use client::get_client;
use commands::domain::handle_domains;
use commands::dyndns::handle_dyndns;
use commands::invoices::handle_invoices;
use domeneshop_client::client::DomeneshopClient;
use simple_log::{log_level, LogConfigBuilder, SimpleResult};
use std::fmt::{Debug, Display};
use std::process::ExitCode;
use std::{env, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
pub enum Command {
    Dyndns(commands::dyndns::Command),
    Domains(commands::domain::DomainArgs),
    Invoices(commands::invoices::InvoiceArgs),
}

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,

    #[arg(short, long)]
    data_directory: Option<String>,
    #[arg(short, long)]
    log_directory: Option<String>,
    #[arg(long)]
    token: Option<String>,
    #[arg(long)]
    secret: Option<String>,
    #[arg(long, action)]
    debug: bool,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    match initialize_logging(&args) {
        Err(err) => {
            eprintln!("Could not initialize logging: {}", err);
            ExitCode::FAILURE
        }
        Ok(_) => match get_data_directory(&args) {
            None => ExitCode::FAILURE,
            Some(data_dir) => match get_client(&args, &data_dir) {
                None => ExitCode::FAILURE,
                Some(client) => run_command(&client, &args, &data_dir).await,
            },
        },
    }
}

async fn run_command(client: &DomeneshopClient, args: &Args, data_dir: &PathBuf) -> ExitCode {
    match &args.command {
        Command::Dyndns(command) => handle_dyndns(command, client, data_dir).await,
        Command::Domains(command) => handle_domains(command, client).await,
        Command::Invoices(command) => handle_invoices(command, client).await,
    }
}

fn initialize_logging(args: &Args) -> SimpleResult<()> {
    // TODO Change to follow XDG-spec and handle error from current_dir

    let log_dir = args
        .log_directory
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap().to_string_lossy().to_string());
    let level = if args.debug {
        log_level::DEBUG
    } else {
        log_level::INFO
    };

    let log_file = format!("{}/domeneshop_cli.log", log_dir);

    let mut configbuilder = LogConfigBuilder::builder()
        .path(&log_file)
        .size(1 * 100)
        .roll_count(10)
        .level(level)
        .output_file();

    if args.debug {
        configbuilder = configbuilder.output_console();
    }

    let config = configbuilder.build();

    simple_log::new(config)?;
    println!("Logging to {}", &log_file);
    SimpleResult::Ok(())
}

fn get_data_directory(args: &Args) -> Option<PathBuf> {
    if let Some(dir) = &args.data_directory {
        debug!("Using data directory from arguments");
        Some(PathBuf::from(dir))
    } else if let Ok(current_dir) = env::current_dir() {
        debug!("Using current dir as data directory");
        Some(current_dir)
    } else {
        warn!("Could not resolve data directory");
        None
    }
}

pub fn log_and_fail<S, E>(text: S, error: E) -> ExitCode
where
    S: Into<String> + Display,
    E: Debug,
{
    let error = format!("{}: {:?}", text, error);
    error!("{}", error);
    eprintln!("{}", error);
    ExitCode::FAILURE
}
