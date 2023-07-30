use std::process::ExitCode;

use clap::*;
use domeneshop_client::{
    client::DomeneshopClient,
    endpoints::invoices::{Invoice, InvoiceId, InvoiceStatus},
};

use crate::log_and_fail_with_error;

#[derive(Parser)]
pub struct InvoiceArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    List(ListInvoiceArgs),
    Get(GetInvoiceArgs),
}

#[derive(Parser)]
pub struct ListInvoiceArgs {
    #[arg(short, long, help = "Filters invoice list on given value")]
    status: Option<InvoiceStatusInput>,
}

#[derive(Parser)]
pub struct GetInvoiceArgs {
    id: InvoiceId,
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
        Command::Get(args) => get_invoice(client, args.id).await,
    }
}

async fn list_invoices(client: &DomeneshopClient, args: &ListInvoiceArgs) -> ExitCode {
    info!("Listing invoices with status {:?} ...", args.status);

    let response = match &args.status {
        Some(status) => client.list_invoices_with_status(map_status(status)).await,
        None => client.list_invoices().await,
    };

    match response {
        Err(err) => log_and_fail_with_error("Failed to list invoices", err),
        Ok(invoices) => {
            println!("Got {} invoices.", invoices.len());
            println!("");
            for invoice in invoices {
                println!(
                    "{}: {} {} ({})",
                    invoice.id, invoice.amount, invoice.currency, invoice.status
                );
                println!("");
            }
            ExitCode::SUCCESS
        }
    }
}

async fn get_invoice(client: &DomeneshopClient, id: InvoiceId) -> ExitCode {
    info!("Getting invoice with id {}", id);

    match client.get_invoice(id).await {
        Ok(response) => {
            match response {
                Some(invoice) => {
                    print_invoice(&invoice);
                }
                None => println!("Invoice {} not found", id),
            };
            ExitCode::SUCCESS
        }
        Err(err) => log_and_fail_with_error("Failed to get invoice", err),
    }
}

fn map_status(input: &InvoiceStatusInput) -> InvoiceStatus {
    match input {
        InvoiceStatusInput::Paid => InvoiceStatus::Paid,
        InvoiceStatusInput::Settled => InvoiceStatus::Settled,
        InvoiceStatusInput::Unpaid => InvoiceStatus::Unpaid,
    }
}

fn print_invoice(invoice: &Invoice) {
    println!("Id: {}", invoice.id);
    println!("Type: {}", invoice.r#type);
    println!("Amount: {} {}", invoice.amount, invoice.currency);
    println!("Status: {}", invoice.status);
    println!("Issued at {}", invoice.issued_date);
    if let Some(due_date) = invoice.due_date {
        println!("Due at {}", due_date);
    }
    if let Some(paid_date) = invoice.paid_date {
        println!("Paid at {}", paid_date);
    }
    println!("Url: {}", invoice.url);
}
