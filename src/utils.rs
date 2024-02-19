use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::{env, fs};
use subtle_encoding::{bech32, hex};
use tendermint::account::Id as TmAccountId;
use tendermint::validator::Info as ValidatorIfo;
pub use namada_sdk::types::string_encoding::ADDRESS_HRP;
pub use namada_sdk::types::string_encoding::COMMON_PK_HRP;

const CHECKSUMS_FILE_PATH_ENV: &str = "CHECKSUMS_FILE_PATH";
const CHECKSUMS_REMOTE_URL_ENV: &str = "CHECKSUMS_REMOTE_URL";
const CHECKSUMS_DEFAULT_PATH: &str = "checksums.json";

pub fn load_checksums() -> Result<HashMap<String, String>, crate::Error> {
    let checksums_file_path = env::var(CHECKSUMS_FILE_PATH_ENV);
    let checksums_remote_url = env::var(CHECKSUMS_REMOTE_URL_ENV);

    let checksums = match (checksums_file_path, checksums_remote_url) {
        (Ok(path), _) => fs::read_to_string(path)?,
        (_, Ok(url)) => ureq::get(&url)
            .call()
            .map_err(|e| crate::Error::Generic(Box::new(e)))?
            .into_string()?,
        _ => fs::read_to_string(CHECKSUMS_DEFAULT_PATH)?,
    };

    let json: serde_json::Value = serde_json::from_str(&checksums)?;
    let obj = json.as_object().ok_or(crate::Error::InvalidChecksum)?;

    let mut checksums_map = HashMap::new();
    for value in obj.iter() {
        let hash = value
            .1
            .as_str()
            .ok_or(crate::Error::InvalidChecksum)?
            .split('.')
            .collect::<Vec<&str>>()[1];
        let type_tx = value.0.split('.').collect::<Vec<&str>>()[0];

        checksums_map.insert(hash.to_string(), type_tx.to_string());
    }

    Ok(checksums_map)
}

pub fn addr_to_bech32(addr: TmAccountId) -> String {
    bech32::encode(
        ADDRESS_HRP,
        addr.as_bytes(),
    )
}

pub fn find_validator(validators: Vec<ValidatorIfo>, address: TmAccountId) -> Option<ValidatorIfo> {
    for validator in validators {
        if validator.address == address {
            return Some(validator);
        }
    }

    None
}

pub fn tx_hash(raw_tx: Vec<u8>) -> String {
    String::from_utf8(hex::encode_upper(Sha256::digest(raw_tx))).expect("Invalid UTF-8 sequence")
}
