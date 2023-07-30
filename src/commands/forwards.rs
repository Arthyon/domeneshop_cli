use std::process::ExitCode;

use clap::*;
use domeneshop_client::{
    client::DomeneshopClient,
    endpoints::{domains::DomainId, forwards::HttpForward},
};

use crate::{
    domain_lookup::{get_domain_id, DomainIdOrHost},
    log_and_fail, log_and_fail_with_error,
};

#[derive(Parser)]
pub struct ForwardArgs {
    #[arg(short, long, help = "Id or name of the domain to manage forwards for")]
    domain: DomainIdOrHost,
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    List,
    Get(GetForwardArgs),
}

// #[derive(Parser)]
// pub struct ListForwardsArgs {}

#[derive(Parser)]
pub struct GetForwardArgs {
    host: String,
}

#[derive(ValueEnum, Clone, Debug)]
enum DnsTypeArg {
    A,
    AAAA,
    CNAME,
    MX,
    SRV,
    TXT,
}

pub async fn handle_forwards(args: &ForwardArgs, client: &DomeneshopClient) -> ExitCode {
    match get_domain_id(&args.domain, client).await {
        Some(domain_id) => match &args.command {
            Command::List => list_forwards(client, domain_id).await,
            Command::Get(args) => get_forward(client, domain_id, &args.host).await,
        },
        None => log_and_fail("Could not resolve --domain input to a domain"),
    }
}

async fn get_forward(client: &DomeneshopClient, domain_id: DomainId, host: &String) -> ExitCode {
    info!(
        "Getting forward with host {} for domain {}",
        host, domain_id
    );

    match client.get_forward(domain_id, host.clone()).await {
        Ok(forward) => {
            print_forward(forward);
            ExitCode::SUCCESS
        }
        Err(err) => log_and_fail_with_error("Failed to get forward", err),
    }
}

async fn list_forwards(client: &DomeneshopClient, domain_id: DomainId) -> ExitCode {
    info!("Listing forwards ...");

    let response = client.list_forwards(domain_id).await;

    match response {
        Err(err) => log_and_fail_with_error("Failed to list forwards", err),
        Ok(forwards) => {
            println!("Got {} forwards.", forwards.len());
            for forward in forwards {
                println!("{} -> {}", forward.host, forward.url);
            }
            ExitCode::SUCCESS
        }
    }
}

fn print_forward(forward: Option<HttpForward>) {
    match forward {
        Some(forward) => println!(
            "{} -> {} (frame: {})",
            forward.host, forward.url, forward.frame
        ),
        None => println!("Forward not found"),
    }
}
