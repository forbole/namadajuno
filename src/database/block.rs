use chrono::{DateTime, Utc};
use tendermint::abci::types::ExecTxResult;
use tendermint::block::Block as TmBlock;

use crate::database::Database;
use crate::utils;
use crate::Error;

pub struct Block {
    pub height: i64,
    pub hash: String,
    pub num_txs: i64,
    pub total_gas: i64,
    pub proposer_address: String,
    pub timestamp: DateTime<Utc>,
}

impl Block {
    pub fn from_tm_block(
        block: TmBlock,
        tx_results: Option<Vec<ExecTxResult>>,
        main_prefix: &str,
    ) -> Self {
        Self {
            height: block.header.height.into(),
            hash: block.header.hash().to_string(),
            num_txs: block.data.len() as i64,
            total_gas: sum_total_gas(tx_results),
            proposer_address: utils::convert_consensus_addr_to_bech32(
                main_prefix,
                block.header.proposer_address,
            ),
            timestamp: DateTime::from_timestamp(block.header.time.unix_timestamp(), 0)
                .expect("invalid timestamp"),
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO block (height, hash, num_txs, total_gas, proposer_address, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING"#,
        )
        .bind(self.height)
        .bind(self.hash.clone())
        .bind(self.num_txs)
        .bind(self.total_gas)
        .bind(self.proposer_address.clone())
        .bind(self.timestamp)
        .execute(db.pool().as_ref())
        .await?;

        Ok(())
    }
}

fn sum_total_gas(tx_results: Option<Vec<ExecTxResult>>) -> i64 {
    match tx_results {
        Some(tx_results) => {
            let mut total_gas: i64 = 0;
            for tx_result in tx_results {
                total_gas += tx_result.gas_used;
            }
            total_gas
        }
        None => 0,
    }
}
