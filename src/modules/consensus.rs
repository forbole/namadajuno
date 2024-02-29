
use crate::database::{Database, Block, AverageBlockTime};
use chrono::{Duration, Utc};
use clokwerk::{Scheduler, TimeUnits};
use tracing;
use std::sync::Arc;

use tendermint::block::Block as TmBlock;


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
        let average_block_time = block.timestamp.timestamp() - block_before_hour.timestamp.timestamp();
        let average_block_time = average_block_time as f64 / 3600.0;

        // Save average block time per hour
        let average_block_time = AverageBlockTime::new(average_block_time, block.height);
        average_block_time.save_average_block_time_per_hour(&self.db).await?;

        Ok(())
    }
}

impl ModuleBasic for ConsensusModule{
    async fn handle_block(&mut self, _: TmBlock) -> Result<(), Error> {
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