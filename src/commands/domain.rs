use std::process::ExitCode;

use clap::*;
use domeneshop_client::{
    client::DomeneshopClient,
    endpoints::domains::{Domain, DomainId},
};

use crate::log_and_fail;

#[derive(Parser)]
pub struct DomainArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    List(ListDomainArgs),
    Get(GetDomainArgs),
}

#[derive(Parser)]
pub struct ListDomainArgs {
    #[arg(short, long)]
    filter: Option<String>,
}

#[derive(Parser)]
pub struct GetDomainArgs {
    id: DomainId,
}

pub async fn handle_domains(args: &DomainArgs, client: &DomeneshopClient) -> ExitCode {
    match &args.command {
        Command::List(args) => list_domains(client, args).await,
        Command::Get(args) => get_domain(client, args.id).await,
    }
}

async fn get_domain(client: &DomeneshopClient, id: DomainId) -> ExitCode {
    info!("Getting domain with id {}", id);

    match client.get_domain(id).await {
        Ok(domain) => {
            print_domain(domain);
            ExitCode::SUCCESS
        }
        Err(err) => log_and_fail("Failed to get domain", err),
    }
}

async fn list_domains(client: &DomeneshopClient, args: &ListDomainArgs) -> ExitCode {
    info!("Listing domains with filter {:?} ...", args.filter);

    let response = match &args.filter {
        Some(filter) => client.list_domains_with_filter(filter).await,
        None => client.list_domains().await,
    };

    match response {
        Err(err) => log_and_fail("Failed to list domains", err),
        Ok(domains) => {
            println!("Got {} domains.", domains.len());
            for domain in domains {
                println!("{}: {}", domain.id, domain.domain);
            }
            ExitCode::SUCCESS
        }
    }
}

fn print_domain(domain: Domain) {
    println!("{}: {}", domain.id, domain.domain);
}
