use std::process::ExitCode;

use clap::*;
use domeneshop_client::{client::DomeneshopClient, endpoints::invoices::InvoiceStatus};

use crate::log_and_fail;

#[derive(Parser)]
pub struct InvoiceArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    List(ListInvoiceArgs),
}

#[derive(Parser)]
pub struct ListInvoiceArgs {
    #[arg(short, long)]
    status: Option<InvoiceStatusInput>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum InvoiceStatusInput {
    Unpaid,
    Paid,
    Settled,
}

pub async fn handle_invoices(args: &InvoiceArgs, client: &DomeneshopClient) -> ExitCode {
    match &args.command {
        Command::List(args) => list_invoices(client, args).await,
    }
}

async fn list_invoices(client: &DomeneshopClient, args: &ListInvoiceArgs) -> ExitCode {
    info!("Listing domains with status {:?} ...", args.status);

    let response = match &args.status {
        Some(status) => client.list_invoices_with_status(map_status(status)).await,
        None => client.list_invoices().await,
    };

    match response {
        Err(err) => log_and_fail("Failed to list invoices", err),
        Ok(invoices) => {
            println!("Got {} invoices.", invoices.len());
            for invoice in invoices {
                println!("{}: {}", invoice.id, invoice.amount);
            }
            ExitCode::SUCCESS
        }
    }
}

fn map_status(input: &InvoiceStatusInput) -> InvoiceStatus {
    match input {
        InvoiceStatusInput::Paid => InvoiceStatus::Paid,
        InvoiceStatusInput::Settled => InvoiceStatus::Settled,
        InvoiceStatusInput::Unpaid => InvoiceStatus::Unpaid,
    }
}
