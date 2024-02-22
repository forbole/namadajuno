use crate::Error;
use tendermint::block::Block;

mod staking;
pub use staking::StakingModule;

pub trait BlockHandle {
    async fn handle_block(&mut self, block: Block) -> Result<(), Error>;
}
