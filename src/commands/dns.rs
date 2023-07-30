use std::process::ExitCode;

use clap::*;
use domeneshop_client::{
    client::DomeneshopClient,
    endpoints::{
        dns::{DnsId, DnsType, ExistingDnsRecord},
        domains::DomainId,
    },
};

use crate::{
    domain_lookup::{get_domain_id, DomainIdOrHost},
    log_and_fail, log_and_fail_with_error,
};

#[derive(Parser)]
pub struct DnsArgs {
    #[arg(short, long, help = "Id or name of the domain to manage DNS for")]
    domain: DomainIdOrHost,
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    List(ListDnsArgs),
    Get(GetDnsArgs),
}

#[derive(Parser)]
pub struct ListDnsArgs {
    #[arg(long, help = "Filters dns list on given host")]
    host: Option<String>,
    #[arg(long, help = "Filters dns list on given type")]
    r#type: Option<DnsTypeArg>,
}

#[derive(Parser)]
pub struct GetDnsArgs {
    id: DnsId,
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

pub async fn handle_dns(args: &DnsArgs, client: &DomeneshopClient) -> ExitCode {
    match get_domain_id(&args.domain, client).await {
        Some(domain_id) => match &args.command {
            Command::List(args) => list_dns(client, domain_id, args).await,
            Command::Get(args) => get_dns(client, domain_id, args.id).await,
        },
        None => log_and_fail("Could not resolve --domain input to a domain"),
    }
}

async fn get_dns(client: &DomeneshopClient, domain_id: DomainId, id: DnsId) -> ExitCode {
    info!("Getting dns with id {} for domain {}", id, domain_id);

    match client.get_dns_record(domain_id, id).await {
        Ok(dns) => {
            print_dns(dns);
            ExitCode::SUCCESS
        }
        Err(err) => log_and_fail_with_error("Failed to get dns", err),
    }
}

async fn list_dns(client: &DomeneshopClient, domain_id: DomainId, args: &ListDnsArgs) -> ExitCode {
    info!(
        "Listing dns with host {:?} and type {:?} ...",
        args.host, args.r#type
    );

    let mapped_type = args.r#type.clone().map(map_dns_type);

    let response = client
        .list_dns_records_with_filter(domain_id, args.host.clone(), mapped_type)
        .await;

    match response {
        Err(err) => log_and_fail_with_error("Failed to list dns", err),
        Ok(dns_records) => {
            println!("Got {} dns records.", dns_records.len());
            for record in dns_records {
                println!("{}: {:?}", record.id, record.data);
            }
            ExitCode::SUCCESS
        }
    }
}

fn map_dns_type(dns_type: DnsTypeArg) -> DnsType {
    match dns_type {
        DnsTypeArg::A => DnsType::A,
        DnsTypeArg::AAAA => DnsType::AAAA,
        DnsTypeArg::CNAME => DnsType::CNAME,
        DnsTypeArg::MX => DnsType::MX,
        DnsTypeArg::SRV => DnsType::SRV,
        DnsTypeArg::TXT => DnsType::TXT,
    }
}

fn print_dns(dns: ExistingDnsRecord) {
    println!("{:?}", dns);
}
