use tendermint::block::Block;
use crate::Error;

mod staking;
pub use staking::StakingModule;


pub trait BlockHandle {
    async fn handle_block(&self, block: Block) -> Result<(), Error>;
}


