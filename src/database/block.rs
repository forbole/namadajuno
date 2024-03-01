use chrono::NaiveDateTime;
use tendermint::abci::types::ExecTxResult;
use tendermint::block::Block as TmBlock;
use sqlx::FromRow;

use crate::database::Database;
use crate::utils;
use crate::Error;

#[derive(FromRow)]
pub struct Block {
    pub height: i64,
    pub hash: String,
    pub num_txs: i32,
    pub total_gas: i64,
    pub proposer_address: String,
    pub timestamp: NaiveDateTime,
}

impl Block {
    pub fn from_tm_block(block: TmBlock, tx_results: Vec<ExecTxResult>) -> Self {
        Self {
            height: block.header.height.into(),
            hash: block.header.hash().to_string(),
            num_txs: block.data.len() as i32,
            total_gas: sum_total_gas(tx_results),
            proposer_address: utils::addr_to_bech32(block.header.proposer_address),
            timestamp: NaiveDateTime::from_timestamp_opt(block.header.time.unix_timestamp(), 0)
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
        .execute(&db.pool())
        .await?;

        Ok(())
    }

    pub async fn latest_block(db: &Database) -> Result<Option<Self>, Error> {
        let block = sqlx::query_as::<_, Self>(
            r#"SELECT * FROM block
            ORDER BY height DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&db.pool())
        .await?;
    
       Ok(block)
    }

    pub async fn block_before_time(db: &Database, time: NaiveDateTime) -> Result<Option<Self>, Error> {
        let block = sqlx::query_as::<_, Self>(
            r#"
            SELECT * FROM block
            WHERE timestamp <= $1
            ORDER BY height DESC
            LIMIT 1
            "#,
        )
        .bind(time)
        .fetch_optional(&db.pool())
        .await?;
    
       Ok(block)
    }
}

fn sum_total_gas(tx_results: Vec<ExecTxResult>) -> i64 {
    tx_results.iter().map(|tx_result| tx_result.gas_used).sum()
}
