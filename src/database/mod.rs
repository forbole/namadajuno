use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::config::DBConfig;
use crate::Error;

mod validator;
pub use validator::{Validator, Validators};
pub use validator::{
    ValidatorCommission, ValidatorCommissions, ValidatorConsensusKey, ValidatorDescription,
    ValidatorDescriptions, ValidatorInfo, ValidatorInfos, ValidatorStatus, ValidatorStatuses,
    ValidatorVotingPower, ValidatorVotingPowers,
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

mod consensus;
pub use consensus::AverageBlockTime;

mod gov;
pub use gov::{Proposal, ProposalVote, ProposalTallyResult};


#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &DBConfig) -> Result<Self, Error> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_open_connections)
            .connect(config.url.as_str())
            .await?;

        Ok(Database {
            pool,
        })
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}
