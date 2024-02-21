use chrono::{DateTime, Utc};
use sqlx::{Postgres, QueryBuilder};
use tendermint::validator::Info as TmValidatorInfo;
use tracing;

use crate::database::Database;
use crate::utils;
use crate::Error;

pub struct PreCommit {
    pub validator_address: String,
    pub height: i64,
    pub timestamp: DateTime<Utc>,
    pub voting_power: i64,
    pub proposer_priority: i64,
}

impl PreCommit {
    pub fn new(
        validator_address: String,
        height: i64,
        timestamp: DateTime<Utc>,
        voting_power: i64,
        proposer_priority: i64,
    ) -> Self {
        PreCommit {
            validator_address,
            height,
            timestamp,
            voting_power,
            proposer_priority,
        }
    }

    pub fn from_tm_commit_sig(
        height: u64,
        validator_address: tendermint::account::Id,
        validators: Vec<TmValidatorInfo>,
        timestamp: tendermint::Time,
    ) -> Self {
        match utils::find_validator(validators.clone(), validator_address) {
            Some(info) => {
                return PreCommit::new(
                    utils::addr_to_bech32(validator_address),
                    height as i64,
                    DateTime::from_timestamp(timestamp.unix_timestamp(), 0)
                        .expect("invalid timestamp"),
                    info.power.into(),
                    info.proposer_priority.into(),
                )
            }
            //
            None => {
                tracing::error!(
                    "validator not found {} at height {}",
                    utils::addr_to_bech32(validator_address),
                    height,
                );
                return PreCommit::new(
                    utils::addr_to_bech32(validator_address),
                    height as i64,
                    DateTime::from_timestamp(timestamp.unix_timestamp(), 0)
                        .expect("invalid timestamp"),
                    0,
                    0,
                );
            }
        };

        // TODO: Consider this code after investigating the issue that find_validator returns None
        // let validator = utils::find_validator(validators.clone(), validator_address)
        //     .expect(format!("validator not found {}", utils::addr_to_bech32(validator_address)).as_str());

        // PreCommit::new(
        //     utils::addr_to_bech32(validator_address),
        //     height as i64,
        //     DateTime::from_timestamp(timestamp.unix_timestamp(), 0).expect("invalid timestamp"),
        //     validator.power.into(),
        //     validator.proposer_priority.into(),
        // )
    }
}

pub struct PreCommits(Vec<PreCommit>);

impl PreCommits {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        let mut builder: QueryBuilder<Postgres> =
        QueryBuilder::new("INSERT INTO pre_commit (validator_address, height, timestamp, voting_power, proposer_priority)");

        builder.push_values(self.0.iter(), |mut b, p| {
            b.push_bind(p.validator_address.clone())
                .push_bind(p.height)
                .push_bind(p.timestamp)
                .push_bind(p.voting_power)
                .push_bind(p.proposer_priority);
        });
        builder.push("ON CONFLICT DO NOTHING");

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

impl From<Vec<PreCommit>> for PreCommits {
    fn from(pre_commits: Vec<PreCommit>) -> Self {
        PreCommits(pre_commits)
    }
}
