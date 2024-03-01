use crate::database::Database;
use crate::Error;

#[derive(Debug)]
pub struct Tx {
    pub hash: String,
    pub height: i64,
    pub success: bool,

    pub tx_type: String,
    pub memo: String,

    pub gas_wanted: i64,
    pub gas_used: i64,
    pub raw_log: String,
}

impl Tx {
    pub fn new(
        hash: String,
        height: i64,
        success: bool,
        memo: String,
        tx_type: String,
        gas_wanted: i64,
        gas_used: i64,
        raw_log: String,
    ) -> Self {
        Self {
            hash,
            height,
            success,
            memo,
            tx_type,
            gas_wanted,
            gas_used,
            raw_log,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO transaction (hash, height, success, memo, tx_type, gas_wanted, gas_used, raw_log)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT DO NOTHING"#,
        )
        .bind(self.hash.clone())
        .bind(self.height)
        .bind(self.success)
        .bind(self.memo.clone())
        .bind(self.tx_type.clone())
        .bind(self.gas_wanted)
        .bind(self.gas_used)
        .bind(self.raw_log.clone())
        .execute(&db.pool())
        .await?;

        Ok(())
    }
}
