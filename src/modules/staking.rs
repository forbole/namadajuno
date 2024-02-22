use namada_sdk::state::Epoch;
use tendermint::block::Block;

use crate::database::{self, Database};
use crate::modules::BlockHandle;
use crate::node::Node;
use crate::Error;

#[derive(Clone)]
pub struct StakingModule {
    node: Node,
    db: Database,
    epoch: Option<Epoch>,
}

impl StakingModule {
    pub fn new(node: Node, db: Database) -> Self {
        Self {
            node,
            db,
            epoch: None,
        }
    }

    async fn update_validators(&self, height: u64) -> Result<(), Error> {
        let epoch = self.node.clone().epoch(height).await?;
        let validator_infos = self.node.validator_infos(epoch).await?;

        // Save infos
        let validators = validator_infos
            .clone()
            .into_iter()
            .map(|(address, _, _, commission)| {
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
            .map(|(address, _, voting_power, _)| {
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
            .map(|(address, _, _, commission)| {
                database::ValidatorCommission::new(
                    address.encode(),
                    commission.unwrap().commission_rate.to_string(),
                    height,
                )
            })
            .collect::<Vec<_>>();
        database::ValidatorCommissions::from(validators_commissions)
            .save(&self.db)
            .await?;

        // Save statuses
        let validators_statuses = validator_infos
            .into_iter()
            .map(|(address, state, _, _)| {
                database::ValidatorStatus::new(address.encode(), state.unwrap(), height)
            })
            .collect::<Vec<_>>();
        database::ValidatorStatuses::from(validators_statuses)
            .save(&self.db)
            .await?;

        Ok(())
    }
}

impl BlockHandle for StakingModule {
    async fn handle_block(&self, block: Block) -> Result<(), Error> {
        // handle block
        let height = block.header.height;
        self.update_validators(height.into()).await?;

        Ok(())
    }
}
