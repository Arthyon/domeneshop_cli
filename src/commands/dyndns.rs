use std::io::Write;
use std::{fs, net::IpAddr, path::PathBuf, process::ExitCode};

use clap::Parser;
use domeneshop_client::client::DomeneshopClient;

use crate::constants::{DYNDNS_EXECUTION_LOG_FILENAME, LAST_IP_FILENAME};
use crate::log_and_fail_with_error;

#[derive(Parser)]
pub struct Command {
    domain: String,
}

pub async fn handle_dyndns(
    command: &Command,
    client: &DomeneshopClient,
    data_dir: &PathBuf,
) -> ExitCode {
    info!("Updating dyndns ...");
    match public_ip::addr().await {
        None => {
            error!("Unable to resolve ip");
            ExitCode::FAILURE
        }
        Some(ip) => update_dyndns(client, &command.domain, ip, data_dir).await,
    }
}
async fn update_dyndns(
    client: &DomeneshopClient,
    domain: &String,
    ip: IpAddr,
    data_dir: &PathBuf,
) -> ExitCode {
    let last_ip_file = get_last_ip_file(data_dir);
    let last_ip = get_last_ip_address(&last_ip_file);
    if let Some(last_ip) = last_ip {
        if last_ip == ip {
            let message = "IP hasn't changed since last time. Exiting";
            info!("{message}");
            println!("{message}");
            return ExitCode::SUCCESS;
        }
    }

    let result = client.update_dyndns(domain, Some(ip)).await;
    match result {
        Ok(_) => {
            info!("Updated ip to {ip}");
            println!("Updated ip to {ip}");
            update_last_ip(ip, &last_ip_file);
            log_execution(ip, last_ip, &data_dir);
            return ExitCode::SUCCESS;
        }
        Err(err) => log_and_fail_with_error("Error while updating dns settings", err),
    }
}

fn get_last_ip_address(last_ip_file: &PathBuf) -> Option<IpAddr> {
    match fs::read_to_string(last_ip_file) {
        Err(_) => {
            info!("Could not find existing ip");
            None
        }
        Ok(ip) => match ip.parse::<IpAddr>() {
            Ok(ip) => Some(ip),
            Err(err) => {
                warn!("Could not parse ip: {}", err);
                None
            }
        },
    }
}

fn update_last_ip(ip: IpAddr, last_ip_file: &PathBuf) {
    match fs::write(last_ip_file, ip.to_string()) {
        Ok(_) => (),
        Err(err) => error!("Error while persisting last ip: {}", err),
    }
}

fn get_last_ip_file(data_dir: &PathBuf) -> PathBuf {
    let mut last_ip = data_dir.clone();
    last_ip.push(LAST_IP_FILENAME);
    last_ip
}

fn log_execution(current_ip: IpAddr, last_ip: Option<IpAddr>, data_dir: &PathBuf) {
    let mut log_file = data_dir.clone();
    log_file.push(DYNDNS_EXECUTION_LOG_FILENAME);

    let file_result = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file);
    match file_result {
        Err(err) => warn!("Cannot log execution: {}", err),
        Ok(mut file) => {
            if let Err(e) = writeln!(
                file,
                "{}: Updated from ip {:?} to {}",
                chrono::Utc::now(),
                last_ip,
                current_ip
            ) {
                error!("Couldn't write to file: {}", e);
            }
        }
    }
}
