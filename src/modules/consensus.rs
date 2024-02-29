use chrono::{Duration, Utc};
use clokwerk::{Scheduler, TimeUnits};
use tracing;
use std::sync::Arc;
use namada_sdk::state::Epoch;

use crate::database::{Database, Block, AverageBlockTime};
use crate::modules::ModuleBasic;
use crate::Error;

#[derive(Clone)]
pub struct ConsensusModule{
    db: Database,
}

impl ConsensusModule {
    pub fn new(db: Database) -> Self {
        Self {
            db,
        }
    }
    async fn update_average_block_time_in_hour(&self) -> Result<(), Error> {
        // Get latest block
        let block = Block::latest_block(&self.db).await?;
        let block = match block {
            Some(block) => block,
            None => return Ok(()),
        };

        // Get block before 1 hour
        let block_before_hour = Block::block_before_time(&self.db, Utc::now() - Duration::hours(1)).await?;
        let block_before_hour = match block_before_hour {
            Some(block) => block,
            None => return Ok(()),
        };

        // Calculate average block time per hour
        let block_time_delta = block.timestamp.timestamp() - block_before_hour.timestamp.timestamp();
        let block_count = block.height - block_before_hour.height;

        let mut average_block_time = 0.0;
        if block_count != 0 {
            average_block_time = block_time_delta as f64 / block_count as f64;
        }

        // Save average block time per hour
        let average_block_time = AverageBlockTime::new(average_block_time, block.height);
        average_block_time.save_average_block_time_per_hour(&self.db).await?;

        Ok(())
    }
}

impl ModuleBasic for ConsensusModule{
    async fn handle_epoch(&self, _: u64, _: Epoch) -> Result<(), Error> {
        // Do nothing
        Ok(())
    }

    fn register_periodic_operations(&self, scheduler: &mut Scheduler) {
        let module = Arc::new(self.clone());
        scheduler.every(1.hour()).run(move || {
            let module = module.clone();
            tokio::spawn(async move {
                match module.update_average_block_time_in_hour().await {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to update average block time in hour: {}", e);
                    }
                }
            });
        });
    }
}