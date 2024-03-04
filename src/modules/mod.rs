use clokwerk;
use namada_sdk::state::Epoch;

use crate::Error;
use crate::database::Message;

mod staking;
pub use staking::StakingModule;

mod consensus;
pub use consensus::ConsensusModule;

mod gov;
pub use gov::GovModule;

pub trait ModuleBasic {
    fn register_periodic_operations(&self, scheduler: &mut clokwerk::Scheduler);
    async fn handle_epoch(&self, height: u64, epoch: Epoch) -> Result<(), Error>;
    async fn handle_message(&self, message: Message) -> Result<(), Error>;
}

