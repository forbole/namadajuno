use crate::error::Error;
use tendermint::block::Height;
use tendermint::Hash;
use sha2::{Sha256, Digest};
use tendermint_rpc::{endpoint, Client, HttpClient, Paging};

#[derive(Clone)]
pub struct Node {
    rpc_client: HttpClient,
}

impl Node {
    pub fn new(rpc_client: HttpClient) -> Self {
        Node { rpc_client }
    }

    pub async fn latest_height(&self) -> Result<u64, Error> {
        let status = self.rpc_client.status().await?;
        Ok(status.sync_info.latest_block_height.value())
    }

    pub async fn consensus_state(&self) -> Result<endpoint::consensus_state::Response, Error> {
        let state: endpoint::consensus_state::Response = self.rpc_client.consensus_state().await?;
        Ok(state)
    }

    pub async fn block(&self, height: u64) -> Result<endpoint::block::Response, Error> {
        let block = self.rpc_client.block(Height::try_from(height)?).await?;
        Ok(block)
    }

    pub async fn block_results(&self, height: u64) -> Result<endpoint::block_results::Response, Error> {
        let block_results = self
            .rpc_client
            .block_results(Height::try_from(height).unwrap())
            .await
            .unwrap();
        Ok(block_results)
    }

    pub async fn validators(&self, height: u64) -> Result<endpoint::validators::Response, Error> {
        let validator_set = self
            .rpc_client
            .validators(Height::try_from(height)?, Paging::All)
            .await?;
        Ok(validator_set)
    }

    pub async fn tx(&self, hash: Hash) -> Result<endpoint::tx::Response, Error> {
        let tx = self.rpc_client.tx(hash, true).await?;
        Ok(tx)
    }

    pub async fn txs(&self, height: u64) -> Result<Vec<endpoint::tx::Response>, Error> {
        let block = self.block(height).await?;
        let raw_txs = block.block.data;
        let mut txs_responses: Vec<endpoint::tx::Response> = Vec::new();

        for raw_tx in raw_txs.iter() {
            let hash = Sha256::digest(raw_tx.clone()).to_vec();
            println!("hash: {:?}", hash);
            let tx_response = self.tx(Hash::try_from(hash)?).await?;
            txs_responses.push(tx_response);
        }

        Ok(txs_responses)
    }
}
