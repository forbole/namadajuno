use sqlx::types::Decimal;

use crate::database::Database;
use crate::Error;

pub struct AverageBlockTime {
    pub average_block_time: Decimal,
    pub height: i64,
}

impl AverageBlockTime {
    pub fn new(average_block_time: f64, height: i64) -> Self {
        Self {
            average_block_time: Decimal::from_f64_retain(average_block_time).unwrap(),
            height,
        }
    }

    pub async fn save_average_block_time_per_hour(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO average_block_time_per_hour (average_time, height)
            VALUES ($1, $2) ON CONFLICT (one_row_id) DO UPDATE
                SET average_time = EXCLUDED.average_time,
                    height = EXCLUDED.height
            WHERE average_block_time_per_hour.height <= EXCLUDED.height"#,
        )
        .bind(self.average_block_time)
        .bind(self.height)
        .execute(&db.pool())
        .await?;

        Ok(())
    }

    pub async fn save_average_block_time_per_day(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO average_block_time_per_day (average_time, height)
            VALUES ($1, $2) ON CONFLICT (one_row_id) DO UPDATE
                SET average_time = EXCLUDED.average_time,
                    height = EXCLUDED.height
            WHERE average_block_time_per_day.height <= EXCLUDED.height"#,
        )
        .bind(self.average_block_time)
        .bind(self.height)
        .execute(&db.pool())
        .await?;

        Ok(())
    }
}