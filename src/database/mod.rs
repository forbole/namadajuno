use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

use crate::config::DBConfig;
use crate::Error;

mod validator;
pub use validator::{Validator, Validators};
pub use validator::{
    ValidatorCommission, ValidatorCommissions, ValidatorInfo, ValidatorInfos, ValidatorStatus,
    ValidatorStatuses, ValidatorVotingPower, ValidatorVotingPowers,
};

mod block;
pub use block::Block;

mod pre_commmit;
pub use pre_commmit::PreCommit;
pub use pre_commmit::PreCommits;

mod message;
pub use message::Message;

mod tx;
pub use tx::Tx;

#[derive(Clone)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub async fn new(config: &DBConfig) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_open_connections)
            .connect(config.url.as_str())
            .await?;

        Ok(Database {
            pool: Arc::new(pool),
        })
    }

    pub fn pool(&self) -> Arc<PgPool> {
        self.pool.clone()
    }
}
