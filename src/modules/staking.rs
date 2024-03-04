use namada_sdk::state::Epoch;
use tendermint::PublicKey;

use crate::database::{self, Database};
use crate::modules::ModuleBasic;
use crate::node::Node;
use crate::utils;
use crate::Error;

#[derive(Clone)]
pub struct StakingModule {
    node: Node,
    db: Database,
    
}

impl StakingModule {
    pub fn new(node: Node, db: Database) -> Self {
        Self {
            node,
            db,
        }
    }

    async fn update_validators(&self, height: u64, epoch: Epoch) -> Result<(), Error> {
        let validator_infos = self.node.validator_infos(epoch).await?;

        // Save infos
        let validators = validator_infos
            .clone()
            .into_iter()
            .map(|(address, _, _, commission, _, _)| {
                database::ValidatorInfo::new(
                    address.encode(),
                    commission
                        .unwrap()
                        .max_commission_change_per_epoch
                        .to_string(),
                    height,
                )
            })
            .collect::<Vec<_>>();
        database::ValidatorInfos::from(validators)
            .save(&self.db)
            .await?;

        // Save voting powers
        let validators = validator_infos
            .clone()
            .into_iter()
            .map(|(address, _, voting_power, _, _, _)| {
                database::ValidatorVotingPower::new(
                    address.encode(),
                    voting_power.to_string().parse::<i64>().unwrap(),
                    height,
                )
            })
            .collect::<Vec<_>>();
        database::ValidatorVotingPowers::from(validators)
            .save(&self.db)
            .await?;

        // Save commissions
        let validators_commissions = validator_infos
            .clone()
            .into_iter()
            .map(|(address, _, _, commission, _, _)| {
                if let Some(commission) = commission {
                    return Some(database::ValidatorCommission::new(
                        address.encode(),
                        commission.commission_rate.to_string(),
                        height,
                    ));
                }

                None
            })
            .collect::<Vec<_>>();
        database::ValidatorCommissions::from(validators_commissions)
            .save(&self.db)
            .await?;

        // Save statuses
        let validators_statuses = validator_infos
            .clone()
            .into_iter()
            .map(|(address, state, _, _, _, _)| {
                if let Some(state) = state {
                    return Some(database::ValidatorStatus::new(
                        address.encode(),
                        state,
                        height,
                    ));
                }

                None
            })
            .collect::<Vec<_>>();
        database::ValidatorStatuses::from(validators_statuses)
            .save(&self.db)
            .await?;

        // Save descriptions
        let validators_descriptions = validator_infos
            .clone()
            .into_iter()
            .map(|(address, _, _, _, description, _)| {
                if let Some(description) = description {
                    return Some(database::ValidatorDescription::new(
                        address.encode(),
                        description.clone().avatar.unwrap_or_default(),
                        description.clone().website.unwrap_or_default(),
                        description.clone().description.unwrap_or_default(),
                        height,
                    ));
                }

                None
            })
            .collect::<Vec<_>>();
        database::ValidatorDescriptions::from(validators_descriptions)
            .save(&self.db)
            .await?;

        for (address, _, _, _, _, pub_key) in validator_infos {
            if let Some(pub_key) = pub_key {
                let consensus_key = database::ValidatorConsensusKey::new(
                    Into::<PublicKey>::into(pub_key).to_bech32(utils::COMMON_PK_HRP),
                    address.encode(),
                );
                consensus_key.save(&self.db).await?;
            }
        }

        Ok(())
    }
}

impl ModuleBasic for StakingModule {
    async fn handle_epoch(&self, height: u64, epoch: Epoch) -> Result<(), Error> {
        tracing::info!(
            "Updating validators for epoch {}, it will take seconds",
            epoch
        );
        self.update_validators(height.into(), epoch).await?;

        Ok(())
    }

    fn register_periodic_operations(&self, _: &mut clokwerk::Scheduler) {
        // Do nothing
    }

    async fn handle_message(&self, _message: crate::database::Message) -> Result<(), Error> {
        // Do nothing
        Ok(())
    }
}
