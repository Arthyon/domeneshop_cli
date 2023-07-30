use std::str::FromStr;

use domeneshop_client::{client::DomeneshopClient, endpoints::domains::DomainId};

#[derive(Clone)]
pub enum DomainIdOrHost {
    DomainId(DomainId),
    Host(String),
}

impl FromStr for DomainIdOrHost {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<i32>()
            .map(DomainIdOrHost::DomainId)
            .unwrap_or_else(|_| DomainIdOrHost::Host(s.to_string())))
    }
}

pub async fn get_domain_id(
    domain_input: &DomainIdOrHost,
    client: &DomeneshopClient,
) -> Option<DomainId> {
    match domain_input {
        DomainIdOrHost::DomainId(id) => Some(id.clone()),
        DomainIdOrHost::Host(host) => match client.list_domains_with_filter(host).await {
            Ok(domains) => domains.iter().next().map(|d| d.id),
            Err(err) => {
                warn!(
                    "Error while fetching domain id for domain {}: {}",
                    host, err
                );
                None
            }
        },
    }
}
