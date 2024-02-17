use std::error::Error as StdError;
use std::num::ParseIntError;
use thiserror::Error as ThisError;
use tokio::task::JoinError;
use tendermint::Error as TError;
use tendermint_rpc::Error as TRpcError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Invalid Transaction data")]
    InvalidTxData,
    
    #[error("Tendermint error: {0}")]
    TendermintError(#[from] TError),
    #[error("Tendermint rpc_error: {0}")]
    TendermintRpcError(#[from] TRpcError),

    #[error("Configuration error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Address parsing error: {0}")]
    AddrError(#[from] std::net::AddrParseError),
    #[error("Database error: {0}")]
    DB(#[from] sqlx::Error),
    #[error("std::env error: {0}")]
    EnvError(#[from] std::env::VarError),

    #[error("async channel SendError: {0}")]
    SenderError(#[from] async_channel::SendError<u64>),
    #[error("async channel RecvError: {0}")]
    RecvError(#[from] async_channel::RecvError),
    #[error("tokio_error: {0}")]
    JoinError(#[from] JoinError),

    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("serde_json error: {0}")]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error("Invalid checksum data")]
    InvalidChecksum,
    #[error("Unknow error: {0}")]
    Generic(Box<dyn StdError + Send>),
    #[error("ParseInt error")]
    ParseIntError(#[from] ParseIntError),
}