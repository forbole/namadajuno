use sqlx::types::Decimal;
use sqlx::{Postgres, QueryBuilder};
use std::str::FromStr;

use namada_sdk::proof_of_stake::types::ValidatorState;

use crate::database::Database;
use crate::Error;

pub struct Validator {
    consensus_address: String,
    consensus_pubkey: String,
}

impl Validator {
    pub fn new(consensus_address: String, consensus_pubkey: String) -> Self {
        Validator {
            consensus_address,
            consensus_pubkey,
        }
    }
}

pub struct Validators(Vec<Validator>);

impl Validators {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO validator (consensus_address, consensus_pubkey)");

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.consensus_address.clone())
                .push_bind(v.consensus_pubkey.clone());
        });
        builder.push("ON CONFLICT DO NOTHING");

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

impl From<Vec<Validator>> for Validators {
    fn from(validators: Vec<Validator>) -> Self {
        Validators(validators)
    }
}

//--------------------------------------------------------

pub struct ValidatorInfo {
    pub consensus_address: String,
    pub max_change_rate: String,
    pub height: i64,
}

impl ValidatorInfo {
    pub fn new(consensus_address: String, max_change_rate: String, height: u64) -> Self {
        ValidatorInfo {
            consensus_address,
            max_change_rate,
            height: height as i64,
        }
    }
}

pub struct ValidatorInfos(Vec<ValidatorInfo>);

impl From<Vec<ValidatorInfo>> for ValidatorInfos {
    fn from(infos: Vec<ValidatorInfo>) -> Self {
        ValidatorInfos(infos)
    }
}

impl ValidatorInfos {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO validator_info (consensus_address, max_change_rate, height)",
        );

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.consensus_address.clone())
                .push_bind(v.max_change_rate.clone())
                .push_bind(v.height);
        });
        builder.push(
            "ON CONFLICT (consensus_address) DO UPDATE \
            SET max_change_rate = EXCLUDED.max_change_rate, \
                height = EXCLUDED.height \
        WHERE validator_info.height <= EXCLUDED.height",
        );

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

//--------------------------------------------------------

pub struct ValidatorVotingPower {
    pub validator_address: String,
    pub voting_power: i64,
    pub height: i64,
}

impl ValidatorVotingPower {
    pub fn new(validator_address: String, voting_power: i64, height: u64) -> Self {
        ValidatorVotingPower {
            validator_address,
            voting_power,
            height: height as i64,
        }
    }
}

pub struct ValidatorVotingPowers(Vec<ValidatorVotingPower>);

impl From<Vec<ValidatorVotingPower>> for ValidatorVotingPowers {
    fn from(voting_powers: Vec<ValidatorVotingPower>) -> Self {
        ValidatorVotingPowers(voting_powers)
    }
}

impl ValidatorVotingPowers {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO validator_voting_power (validator_address, voting_power, height)",
        );

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.validator_address.clone())
                .push_bind(v.voting_power)
                .push_bind(v.height);
        });
        builder.push(
            "ON CONFLICT (validator_address) DO UPDATE \
            SET voting_power = EXCLUDED.voting_power,\
                height = EXCLUDED.height \
        WHERE validator_voting_power.height <= EXCLUDED.height
            ",
        );

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

//--------------------------------------------------------

pub struct ValidatorCommission {
    pub validator_address: String,
    pub commission_rate: Decimal,
    pub height: i64,
}

impl ValidatorCommission {
    pub fn new(address: String, commission_rate: String, height: u64) -> Self {
        ValidatorCommission {
            validator_address: address,
            commission_rate: Decimal::from_str(&commission_rate).unwrap(),
            height: height as i64,
        }
    }
}

pub struct ValidatorCommissions(Vec<ValidatorCommission>);

impl From<Vec<Option<ValidatorCommission>>> for ValidatorCommissions {
    fn from(commissions: Vec<Option<ValidatorCommission>>) -> Self {
        ValidatorCommissions(commissions.into_iter().filter_map(|x| x).collect())
    }
}

impl ValidatorCommissions {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO validator_commission (validator_address, commission, height)",
        );

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.validator_address.clone())
                .push_bind(v.commission_rate.clone())
                .push_bind(v.height);
        });
        builder.push(
            "ON CONFLICT (validator_address) DO UPDATE \
            SET commission = EXCLUDED.commission, \
                height = EXCLUDED.height \
        WHERE validator_commission.height <= EXCLUDED.height",
        );

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

//--------------------------------------------------------

pub struct ValidatorStatus {
    pub validator_address: String,
    pub status: i64,
    pub jailed: bool,
    pub height: i64,
}

impl ValidatorStatus {
    pub fn new(validator_address: String, state: ValidatorState, height: u64) -> Self {
        ValidatorStatus {
            validator_address,
            status: state as i64,
            jailed: ValidatorState::Jailed == state,
            height: height as i64,
        }
    }
}

pub struct ValidatorStatuses(Vec<ValidatorStatus>);

impl From<Vec<Option<ValidatorStatus>>> for ValidatorStatuses {
    fn from(statuses: Vec<Option<ValidatorStatus>>) -> Self {
        ValidatorStatuses(statuses.into_iter().filter_map(|x| x).collect())
    }
}

impl ValidatorStatuses {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO validator_status (validator_address, status, jailed, height)",
        );

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.validator_address.clone())
                .push_bind(v.status.clone())
                .push_bind(v.jailed)
                .push_bind(v.height);
        });
        builder.push(
            "ON CONFLICT (validator_address) DO UPDATE \
            SET status = EXCLUDED.status, \
                jailed = EXCLUDED.jailed, \
                height = EXCLUDED.height \
        WHERE validator_status.height <= EXCLUDED.height",
        );

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

//--------------------------------------------------------

pub struct ValidatorDescription {
    pub validator_address: String,
    pub avatar_url: String,
    pub website: String,
    pub details: String,
    pub height: i64,
}

impl ValidatorDescription {
    pub fn new(
        validator_address: String,
        avatar_url: String,
        website: String,
        details: String,
        height: u64,
    ) -> Self {
        ValidatorDescription {
            validator_address,
            avatar_url,
            website,
            details,
            height: height as i64,
        }
    }
}

pub struct ValidatorDescriptions(Vec<ValidatorDescription>);

impl From<Vec<Option<ValidatorDescription>>> for ValidatorDescriptions {
    fn from(descriptions: Vec<Option<ValidatorDescription>>) -> Self {
        ValidatorDescriptions(descriptions.into_iter().filter_map(|x| x).collect())
    }
}

impl ValidatorDescriptions {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        if self.0.is_empty() {
            return Ok(());
        }

        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO validator_description (validator_address, avatar_url, website, details, height)",
        );

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.validator_address.clone())
                .push_bind(v.avatar_url.clone())
                .push_bind(v.website.clone())
                .push_bind(v.details.clone())
                .push_bind(v.height);
        });
        builder.push(
            "ON CONFLICT (validator_address) DO UPDATE \
            SET avatar_url = EXCLUDED.avatar_url, \
                website = EXCLUDED.website, \
                details = EXCLUDED.details, \
                height = EXCLUDED.height \
        WHERE validator_description.height <= EXCLUDED.height",
        );

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}

//--------------------------------------------------------

pub struct ValidatorConsensusKey {
    pub consensus_pub_key: String,
    pub validator_address: String,
}

impl ValidatorConsensusKey {
    pub fn new(consensus_pub_key: String, validator_address: String) -> Self {
        ValidatorConsensusKey {
            consensus_pub_key,
            validator_address,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        let query = "UPDATE validator SET validator_address = $2 WHERE consensus_pubkey = $1";
        sqlx::query(query)
            .bind(&self.consensus_pub_key)
            .bind(&self.validator_address)
            .execute(db.pool().as_ref())
            .await?;

        Ok(())
    }
}
