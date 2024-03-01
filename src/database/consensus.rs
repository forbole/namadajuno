use crate::database::Database;

use crate::Error;

pub struct AverageBlockTime {
    pub average_block_time: f64,
    pub height: i64,
}

impl AverageBlockTime {
    pub fn new(average_block_time: f64, height: i64) -> Self {
        Self {
            average_block_time,
            height,
        }
    }

    pub async fn save_average_block_time_per_hour(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO average_block_time_per_hour (average_block_time, height)
            VALUES ($1, $2) ON CONFLICT (one_row_id) DO UPDATE
                SET average_time = EXCLUDED.average_block_time,
                    height = EXCLUDED.height
            WHERE average_block_time_per_hour <= EXCLUDED.height"#,
        )
        .bind(self.average_block_time)
        .bind(self.height)
        .execute(db.pool().as_ref())
        .await?;

        Ok(())
    }

    pub async fn save_average_block_time_per_day(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO average_block_time_per_day (average_block_time, height)
            VALUES ($1, $2) ON CONFLICT (one_row_id) DO UPDATE
                SET average_time = EXCLUDED.average_block_time,
                    height = EXCLUDED.height
            WHERE average_block_time_per_day <= EXCLUDED.height"#,
        )
        .bind(self.average_block_time)
        .bind(self.height)
        .execute(db.pool().as_ref())
        .await?;

        Ok(())
    }
}