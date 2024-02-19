use namada_sdk::tendermint_proto::Protobuf;
use serde_json::json;
use std::collections::HashMap;
use subtle_encoding::hex;
use tracing::info;

use namada_sdk::ibc::apps::transfer::types::msgs::transfer as ibc_transfer_msg;
use namada_sdk::ibc::core::channel::types::msgs as ibc_channel_msg;
use namada_sdk::ibc::core::client::types::msgs as ibc_client_msg;
use namada_sdk::ibc::core::connection::types::msgs as ibc_connection_msg;
use namada_sdk::ibc::primitives::Msg;

use namada_sdk::account;
use namada_sdk::borsh::BorshDeserialize as NamadaBorshDeserialize;
use namada_sdk::governance;
use namada_sdk::tx::data::pgf;
use namada_sdk::tx::data::pos;
use namada_sdk::tx::data::TxType;
use namada_sdk::tx::Tx as NamadaTx;
use namada_sdk::types::address;
use namada_sdk::types::eth_bridge_pool;
use namada_sdk::types::key::common::PublicKey;
use namada_sdk::types::token;
use sqlx::types::JsonValue;

use crate::database::Database;
use crate::Error;

#[derive(Debug)]
pub struct Message {
    pub height: i64,
    pub tx_hash: String,
    pub message_type: String,
    pub value: JsonValue,
}

impl Message {
    pub fn from_tx(
        checksums_map: &HashMap<String, String>,
        height: i64,
        tx_hash: String,
        tx: NamadaTx,
    ) -> Option<Message> {
        let message = parse_tx_to_message(checksums_map, tx).expect("error when parsing tx");
        if let Some((message_type, value)) = message {
            return Some(Message {
                height,
                tx_hash,
                message_type,
                value,
            });
        }

        None
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO message (height, transaction_hash, type, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(self.height)
        .bind(&self.tx_hash)
        .bind(&self.message_type)
        .bind(&self.value)
        .execute(db.pool().as_ref())
        .await?;

        Ok(())
    }
}

fn parse_tx_to_message(
    checksums_map: &HashMap<String, String>,
    tx: NamadaTx,
) -> Result<Option<(String, JsonValue)>, Error> {
    let mut parsed_message = None;
    if let TxType::Decrypted(..) = tx.header().tx_type {
        let code = tx
            .get_section(tx.code_sechash())
            .and_then(|s| s.code_sec())
            .map(|s| s.code.hash().0)
            .ok_or(Error::InvalidTxData)?;

        let code_hex = String::from_utf8(hex::encode(code.as_slice())).expect("invalid hex");
        let unknown_type = &String::from("unknown");
        let tx_type = checksums_map.get(&code_hex).unwrap_or(unknown_type);

        let data = tx.data().ok_or(Error::InvalidTxData)?;
        parsed_message = match tx_type.as_str() {
            "tx_become_validator" => {
                let msg = pos::BecomeValidator::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_bond" => {
                let msg = pos::Bond::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_bridge_pool" => {
                let msg = eth_bridge_pool::PendingTransfer::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_change_consensus_key" => {
                let msg = pos::ConsensusKeyChange::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_change_validator_commission" => {
                let msg = pos::CommissionChange::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_change_validator_metadata" => {
                let msg = pos::MetaDataChange::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_claim_rewards" => {
                let msg = pos::ClaimRewards::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_deactivate_validator" => {
                let msg = address::Address::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_ibc" => {
                // NOTE: This is a temporary solution to parse IBC messages since IBC messages are not yet supported in JSON format.
                let mut result = (tx_type.clone(), json!({}));
                if let Ok(msg) = ibc_client_msg::MsgCreateClient::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgCreateClient", value);
                } else if let Ok(msg) = ibc_client_msg::MsgUpdateClient::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgUpdateClient", value);
                } else if let Ok(msg) = ibc_client_msg::MsgSubmitMisbehaviour::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgSubmitMisbehaviour", value);
                } else if let Ok(msg) = ibc_connection_msg::MsgConnectionOpenInit::decode(&data[..])
                {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgConnectionOpenInit", value);
                } else if let Ok(msg) = ibc_connection_msg::MsgConnectionOpenTry::decode(&data[..])
                {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgConnectionOpenTry", value);
                } else if let Ok(msg) = ibc_connection_msg::MsgConnectionOpenAck::decode(&data[..])
                {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgConnectionOpenAck", value);
                } else if let Ok(msg) =
                    ibc_connection_msg::MsgConnectionOpenConfirm::decode(&data[..])
                {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgConnectionOpenConfirm", value);
                } else if let Ok(msg) = ibc_client_msg::MsgUpgradeClient::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgUpgradeClient", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgChannelOpenInit::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgChannelOpenInit", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgChannelOpenTry::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgChannelOpenTry", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgChannelOpenAck::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgChannelOpenAck", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgChannelOpenConfirm::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgChannelOpenConfirm", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgChannelCloseInit::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgChannelCloseInit", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgChannelCloseConfirm::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgChannelCloseConfirm", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgRecvPacket::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgRecvPacket", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgAcknowledgement::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgAcknowledgement", value);
                } else if let Ok(msg) = ibc_channel_msg::MsgTimeout::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone(), value);
                } else if let Ok(msg) = ibc_channel_msg::MsgTimeoutOnClose::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgTimeoutOnClose", value);
                } else if let Ok(msg) = ibc_transfer_msg::MsgTransfer::decode(&data[..]) {
                    let value = json!(msg.to_any());
                    result = (tx_type.clone() + ".MsgTransfer", value);
                } else {
                    info!("couldn't parse IBC message: {:?}", tx_type);
                }

                Some(result)
            }
            "tx_init_account" => {
                let msg = account::InitAccount::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_init_proposal" => {
                let msg = governance::InitProposalData::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_reactivate_validator" => {
                let msg = address::Address::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_redelegate" => {
                let msg = pos::Redelegation::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_resign_steward" => {
                let msg = address::Address::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_reveal_pk" => {
                let msg = PublicKey::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_transfer" => {
                let msg = token::Transfer::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_unbond" => {
                let msg = pos::Unbond::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_unjail_validator" => {
                let msg = address::Address::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_update_account" => {
                let msg = account::UpdateAccount::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_update_steward_commission" => {
                let msg = pgf::UpdateStewardCommission::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_vote_proposal" => {
                let msg = governance::VoteProposalData::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            "tx_withdraw" => {
                let msg = pos::Withdraw::try_from_slice(&data[..])?;
                let value = json!(msg);
                Some((tx_type.clone(), value))
            }
            _ => None,
        };
    }

    Ok(parsed_message)
}