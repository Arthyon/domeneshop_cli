#[macro_use]
extern crate simple_log;

mod client;
mod constants;
pub mod domain_lookup;
mod commands {
    pub mod dns;
    pub mod domain;
    pub mod dyndns;
    pub mod forwards;
    pub mod invoices;
}

use client::get_client;
use commands::dns::handle_dns;
use commands::domain::handle_domains;
use commands::dyndns::handle_dyndns;
use commands::forwards::handle_forwards;
use commands::invoices::handle_invoices;
use domeneshop_client::client::DomeneshopClient;
use simple_log::{log_level, LogConfigBuilder, SimpleResult};
use std::fmt::{Debug, Display};
use std::process::ExitCode;
use std::{env, path::PathBuf};

use clap::{arg, Parser};

#[derive(Parser)]
pub enum Command {
    Dyndns(commands::dyndns::Command),
    Domains(commands::domain::DomainArgs),
    Invoices(commands::invoices::InvoiceArgs),
    Dns(commands::dns::DnsArgs),
    Forwards(commands::forwards::ForwardArgs),
}

#[derive(Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
    #[clap(flatten)]
    global_opts: GlobalOpts,
}

#[derive(Debug, Parser)]
pub struct GlobalOpts {
    #[arg(long, global = true, help = "Domeneshop API token")]
    token: Option<String>,
    #[arg(long, global = true, help = "Domeneshop API secret")]
    secret: Option<String>,
    #[arg(
        long,
        action,
        global = true,
        help = "Enables expanded logging and outputs logs to console"
    )]
    debug: bool,
    #[arg(
        long,
        global = true,
        help = "Directory to store CLI data. Defaults to current directory"
    )]
    data_directory: Option<String>,
    #[arg(
        long,
        global = true,
        help = "Directory to store logs. Defaults to current directory"
    )]
    log_directory: Option<String>,
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
        Command::Dns(command) => handle_dns(command, client).await,
        Command::Forwards(command) => handle_forwards(command, client).await,
    }
}

fn initialize_logging(args: &Args) -> SimpleResult<()> {
    // TODO Change to follow XDG-spec and handle error from current_dir

    let log_dir = args
        .global_opts
        .log_directory
        .clone()
        .unwrap_or_else(|| env::current_dir().unwrap().to_string_lossy().to_string());
    let level = if args.global_opts.debug {
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

    if args.global_opts.debug {
        configbuilder = configbuilder.output_console();
    }

    let config = configbuilder.build();

    simple_log::new(config)?;
    SimpleResult::Ok(())
}

fn get_data_directory(args: &Args) -> Option<PathBuf> {
    if let Some(dir) = &args.global_opts.data_directory {
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

pub fn log_and_fail<S>(text: S) -> ExitCode
where
    S: Into<String> + Display,
{
    error!("{}", text);
    eprintln!("{}", text);
    ExitCode::FAILURE
}

pub fn log_and_fail_with_error<S, E>(text: S, error: E) -> ExitCode
where
    S: Into<String> + Display,
    E: Debug,
{
    let error = format!("{}: {:?}", text, error);
    log_and_fail(error)
}
