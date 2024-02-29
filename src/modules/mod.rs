use crate::Error;
use clokwerk;
use namada_sdk::state::Epoch;

mod staking;
pub use staking::StakingModule;
mod consensus;
pub use consensus::ConsensusModule;

pub trait ModuleBasic {
    fn register_periodic_operations(&self, scheduler: &mut clokwerk::Scheduler);
    async fn handle_epoch(&self, height: u64, epoch: Epoch) -> Result<(), Error>;
}

