use chrono::{Duration, Utc};
use clokwerk::{Scheduler, TimeUnits};
use namada_sdk::state::Epoch;
use tokio::runtime::Handle;
use tracing;

use crate::database::{AverageBlockTime, Block, Database};
use crate::modules::ModuleBasic;
use crate::Error;

#[derive(Clone)]
pub struct ConsensusModule {
    db: Database,
}

impl ConsensusModule {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    async fn update_average_block_time_in_hour(&self) -> Result<(), Error> {
        // Get latest block
        let block = Block::latest_block(&self.db).await?;
        let block = match block {
            Some(block) => block,
            None => return Ok(()),
        };

        // Get block before 1 hour
        let block_before_hour =
            Block::block_before_time(&self.db, (Utc::now() - Duration::hours(1)).naive_utc())
                .await?;
        let block_before_hour = match block_before_hour {
            Some(block) => block,
            None => return Ok(()),
        };

        // Calculate average block time per hour
        let block_time_delta =
            block.timestamp.timestamp() - block_before_hour.timestamp.timestamp();
        let block_count = block.height - block_before_hour.height;

        let mut average_block_time = 0.0;
        if block_count != 0 {
            average_block_time = block_time_delta as f64 / block_count as f64;
        }

        // Save average block time per hour
        let average_block_time = AverageBlockTime::new(average_block_time, 1);
        average_block_time
            .save_average_block_time_per_hour(&self.db)
            .await?;

        Ok(())
    }

    async fn update_average_block_time_in_day(&self) -> Result<(), Error> {
        // Get latest block
        let block = Block::latest_block(&self.db).await?;
        let block = match block {
            Some(block) => block,
            None => return Ok(()),
        };

        // Get block before 1 day
        let block_before_day =
            Block::block_before_time(&self.db, (Utc::now() - Duration::days(1)).naive_utc())
                .await?;
        let block_before_day = match block_before_day {
            Some(block) => block,
            None => return Ok(()),
        };

        // Calculate average block time per day
        let block_time_delta = block.timestamp.timestamp() - block_before_day.timestamp.timestamp();
        let block_count = block.height - block_before_day.height;

        let mut average_block_time = 0.0;
        if block_count != 0 {
            average_block_time = block_time_delta as f64 / block_count as f64;
        }

        // Save average block time per day
        let average_block_time = AverageBlockTime::new(average_block_time, block.height);
        average_block_time
            .save_average_block_time_per_day(&self.db)
            .await?;

        Ok(())
    }
}

impl ModuleBasic for ConsensusModule {
    async fn handle_epoch(&self, _: u64, _: Epoch) -> Result<(), Error> {
        // Do nothing
        Ok(())
    }

    fn register_periodic_operations(&self, scheduler: &mut Scheduler) {
        let module = self.clone();
        scheduler.every(1.hours()).run(move || {
            let module = module.clone();
            Handle::current().spawn_blocking(|| {
                Handle::current().block_on(async move {
                    tracing::info!("Updating average block time in hour");
                    match module.update_average_block_time_in_hour().await {
                        Ok(_) => {
                            tracing::info!("Updated average block time in hour")
                        }
                        Err(e) => {
                            tracing::error!("Failed to update average block time in hour: {}", e);
                        }
                    }
                })
            });
        });

        let module = self.clone();
        scheduler.every(1.day()).run(move || {
            let module = module.clone();
            Handle::current().spawn_blocking(|| {
                Handle::current().block_on(async move {
                    tracing::info!("Updating average block time in day");
                    match module.update_average_block_time_in_day().await {
                        Ok(_) => {
                            tracing::info!("Updated average block time in day")
                        }
                        Err(e) => {
                            tracing::error!("Failed to update average block time in day: {}", e);
                        }
                    }
                })
            });
        });
    }
}
