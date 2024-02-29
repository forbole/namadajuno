use crate::Error;
use tendermint::block::Block;
use clokwerk;

mod staking;
pub use staking::StakingModule;

mod consensus;
pub use consensus::ConsensusModule;

pub trait ModuleBasic {
    async fn handle_block(&mut self, block: Block) -> Result<(), Error>;
    fn register_periodic_operations(&self, scheduler: &mut clokwerk::Scheduler);
}