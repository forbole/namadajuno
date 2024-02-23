use namada_sdk::state::Epoch;
use std::sync::Arc;
use tendermint::block::Block;
use tokio::sync::Mutex;

use crate::database::{self, Database};
use crate::modules::BlockHandle;
use crate::node::Node;
use crate::Error;

#[derive(Clone)]
pub struct StakingModule {
    node: Node,
    db: Database,
    epoch: Arc<Mutex<Option<Epoch>>>,
}

impl StakingModule {
    pub fn new(node: Node, db: Database) -> Self {
        Self {
            node,
            db,
            epoch: Arc::new(Mutex::new(None)),
        }
    }

    async fn update_validators(&self, height: u64, epoch: Epoch) -> Result<(), Error> {
        let validator_infos = self.node.validator_infos(epoch).await?;

        // Save infos
        let validators = validator_infos
            .clone()
            .into_iter()
            .map(|(address, _, _, commission, _)| {
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
            .map(|(address, _, voting_power, _, _)| {
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
            .map(|(address, _, _, commission, _)| {
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
            .map(|(address, state, _, _, _)| {
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
            .into_iter()
            .map(|(address, _, _, _, description)| {
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

        Ok(())
    }
}

impl BlockHandle for StakingModule {
    async fn handle_block(&mut self, block: Block) -> Result<(), Error> {
        // handle block
        let height = block.header.height;
        let epoch = self.node.epoch(height.into()).await?;
        {
            let mut current_epoch = self.epoch.lock().await;
            if Some(epoch) == *current_epoch {
                return Ok(());
            }

            *current_epoch = Some(epoch);
        }

        self.update_validators(height.into(), epoch).await?;

        Ok(())
    }
}
