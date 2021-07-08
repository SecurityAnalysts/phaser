use crate::{
    modules::{Module, ModuleName, ModuleVersion, SubdomainModule},
    report::Severity,
    Error,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub struct Crtsh {}

impl Crtsh {
    pub fn new() -> Self {
        Crtsh {}
    }
}

impl Module for Crtsh {
    fn name(&self) -> ModuleName {
        ModuleName::SubdomainsCrtsh
    }

    fn version(&self) -> ModuleVersion {
        ModuleVersion(1, 0, 0)
    }

    fn description(&self) -> String {
        String::from("Use crt.sh/ to find subdomains")
    }

    fn is_aggressive(&self) -> bool {
        false
    }

    fn severity(&self) -> Severity {
        Severity::Informative
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CrtShEntry {
    name_value: String,
}

#[async_trait]
impl SubdomainModule for Crtsh {
    async fn enumerate(&self, domain: &str) -> Result<Vec<String>, Error> {
        let url = format!("https://crt.sh/?q=%25.{}&output=json", domain);
        let res = reqwest::get(&url).await?;

        if !res.status().is_success() {
            return Err(Error::InvalidHttpResponse(self.name().to_string()));
        }

        let crtsh_entries: Vec<CrtShEntry> = match res.json().await {
            Ok(info) => info,
            Err(_) => return Err(Error::InvalidHttpResponse(self.name().to_string())),
        };

        // clean and dedup results
        let subdomains: HashSet<String> = crtsh_entries
            .into_iter()
            .map(|entry| {
                entry
                    .name_value
                    .split("\n")
                    .map(|subdomain| subdomain.trim().to_string())
                    .collect::<Vec<String>>()
            })
            .flatten()
            .filter(|subdomain: &String| !subdomain.contains("*"))
            .collect();

        Ok(subdomains.into_iter().collect())
    }
}
