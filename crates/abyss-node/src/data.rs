//! Devnet account persistence for the node CLI.

use std::fs;
use std::path::{Path, PathBuf};

use abyss_wallet::WalletAccount;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoredAccount {
    pub label: String,
    pub address: String,
    pub public_key: String,
    pub dev_seed: Option<String>,
}

impl StoredAccount {
    pub fn from_account(account: &WalletAccount, dev_seed: Option<String>) -> Self {
        Self {
            label: account.label().to_string(),
            address: account.address().to_string(),
            public_key: account.public_key().to_string(),
            dev_seed,
        }
    }

    pub fn load_wallet(&self) -> Result<WalletAccount, String> {
        match &self.dev_seed {
            Some(seed) => Ok(WalletAccount::from_dev_seed(&self.label, seed)),
            None => Err(format!(
                "account '{}' has no dev_seed; only seed-backed dev accounts can sign",
                self.label
            )),
        }
    }
}

pub fn default_data_dir() -> PathBuf {
    PathBuf::from(".abyss")
}

pub fn chain_path(data_dir: &Path) -> PathBuf {
    data_dir.join("chain.json")
}

pub fn accounts_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("accounts")
}

pub fn account_path(data_dir: &Path, label: &str) -> PathBuf {
    accounts_dir(data_dir).join(format!("{label}.json"))
}

pub fn save_account(data_dir: &Path, account: &StoredAccount) -> Result<(), String> {
    let path = account_path(data_dir, &account.label);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(account).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_account(data_dir: &Path, label: &str) -> Result<StoredAccount, String> {
    let path = account_path(data_dir, label);
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

pub fn list_accounts(data_dir: &Path) -> Result<Vec<StoredAccount>, String> {
    let dir = accounts_dir(data_dir);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out: Vec<StoredAccount> = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
        out.push(serde_json::from_str(&json).map_err(|e| e.to_string())?);
    }
    out.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(out)
}

pub fn parse_data_dir(args: &[String]) -> PathBuf {
    for arg in args {
        if let Some(value) = arg.strip_prefix("--data-dir=") {
            return PathBuf::from(value);
        }
    }
    default_data_dir()
}
