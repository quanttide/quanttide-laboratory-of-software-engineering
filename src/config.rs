use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ContractConfig {
    pub code: Option<CodeConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CodeConfig {
    pub rules: Option<Vec<String>>,
}

pub fn load_contract(path: &Path) -> Option<ContractConfig> {
    let mut current = Some(path.to_path_buf());
    while let Some(dir) = current {
        let config_path = dir.join(".quanttide").join("code").join("contract.yaml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).ok()?;
            let config: ContractConfig = serde_yaml::from_str(&content).ok()?;
            return Some(config);
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

pub fn resolve_enabled_rules(
    cli_rules: &Option<Vec<String>>,
    config: &Option<ContractConfig>,
    all_rules: &[&str],
) -> Vec<String> {
    if let Some(rules) = cli_rules {
        return rules.clone();
    }

    if let Some(config) = config {
        if let Some(code) = &config.code {
            if let Some(rules) = &code.rules {
                return rules.clone();
            }
        }
    }

    all_rules.iter().map(|s| s.to_string()).collect()
}
