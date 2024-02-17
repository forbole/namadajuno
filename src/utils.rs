use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::{env, fs};
use subtle_encoding::{bech32, hex};
use tendermint::account::Id as TmAccountId;
use tendermint::validator::Info as ValidatorIfo;

const CHECKSUMS_FILE_PATH_ENV: &str = "CHECKSUMS_FILE_PATH";
const CHECKSUMS_REMOTE_URL_ENV: &str = "CHECKSUMS_REMOTE_URL";
const CHECKSUMS_DEFAULT_PATH: &str = "checksums.json";

const BECH32_PREFIX_CONSENSUS: &str = "cons";
const BECH32_PREFIX_VALIDATOR: &str = "val";
const BECH32_PREFIX_PUBLIC: &str = "pub";
const BECH32_PREFIX_OPERATOR: &str = "oper";

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

pub fn consensus_pub_key_prefix(main_prefix: &str) -> String {
    format!(
        "{}{}{}{}",
        main_prefix, BECH32_PREFIX_VALIDATOR, BECH32_PREFIX_CONSENSUS, BECH32_PREFIX_PUBLIC
    )
}

fn consensus_address_prefix(main_prefix: &str) -> String {
    format!(
        "{}{}{}",
        main_prefix, BECH32_PREFIX_VALIDATOR, BECH32_PREFIX_CONSENSUS
    )
}

pub fn convert_consensus_addr_to_bech32(main_prefix: &str, consensus_addr: TmAccountId) -> String {
    bech32::encode(
        consensus_address_prefix(main_prefix).as_str(),
        consensus_addr.as_bytes(),
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
