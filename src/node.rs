use namada_sdk::proof_of_stake::types::{CommissionPair, ValidatorMetaData, ValidatorState};
use tendermint::block::Height;
use tendermint_rpc::{endpoint, Client, HttpClient, Paging};

use namada_sdk::rpc;
use namada_sdk::state::Epoch;
use namada_sdk::types::address::Address;
use namada_sdk::types::token::Amount;

use crate::error::Error;

#[derive(Clone)]
pub struct Node {
    pub rpc_client: HttpClient,
}

impl Node {
    pub fn new(rpc_client: HttpClient) -> Self {
        Node { rpc_client }
    }

    pub async fn latest_height(&self) -> Result<u64, Error> {
        let status = self.rpc_client.status().await?;
        Ok(status.sync_info.latest_block_height.value())
    }

    pub async fn block(&self, height: u64) -> Result<endpoint::block::Response, Error> {
        let block = self.rpc_client.block(Height::try_from(height)?).await?;
        Ok(block)
    }

    pub async fn block_results(
        &self,
        height: u64,
    ) -> Result<endpoint::block_results::Response, Error> {
        let block_results = self
            .rpc_client
            .block_results(Height::try_from(height).unwrap())
            .await?;
        Ok(block_results)
    }

    pub async fn validators(&self, height: u64) -> Result<endpoint::validators::Response, Error> {
        let validator_set = self
            .rpc_client
            .validators(Height::try_from(height)?, Paging::All)
            .await?;
        Ok(validator_set)
    }

    pub async fn validator_infos(
        &self,
        epoch: Epoch,
    ) -> Result<
        Vec<(
            Address,
            Option<ValidatorState>,
            Amount,
            Option<CommissionPair>,
            Option<ValidatorMetaData>,
        )>,
        Error,
    > {
        let validators = rpc::get_all_validators(&self.rpc_client.clone(), epoch).await?;

        let mut tasks = vec![];
        for v in validators {
            let task = self.query_validator_info(epoch, v.clone());
            tasks.push(task);
        }

        let mut validator_infos = vec![];
        futures_util::future::join_all(tasks)
            .await
            .into_iter()
            .map(|v| v.map(|result| validator_infos.push(result)))
            .collect::<Result<(), Error>>()?;

        Ok(validator_infos)
    }

    async fn query_validator_info(
        &self,
        epoch: Epoch,
        addr: Address,
    ) -> Result<
        (
            Address,
            Option<ValidatorState>,
            Amount,
            Option<CommissionPair>,
            Option<ValidatorMetaData>,
        ),
        Error,
    > {
        let (state, stake, metadata) = tokio::join!(
            rpc::get_validator_state(&self.rpc_client, &addr, Some(epoch)),
            rpc::get_validator_stake(&self.rpc_client, epoch, &addr),
            rpc::query_metadata(&self.rpc_client, &addr, Some(epoch))
        );

        let (metadata, commission) = metadata?;
        Ok((addr, state?, stake?, commission, metadata))
    }

    pub async fn epoch(&self, height: u64) -> Result<Epoch, Error> {
        let epoch = rpc::query_epoch_at_height(&self.rpc_client, height.into())
            .await?
            .ok_or(Error::EpochNotFound)?;
        Ok(epoch)
    }
}
