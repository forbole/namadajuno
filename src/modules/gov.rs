use namada_sdk::state::Epoch;

use crate::node::Node;
use crate::modules::ModuleBasic;
use crate::error::Error;

pub struct GovModule {
    node: Node,
}

impl GovModule {
    pub fn new(node: Node) -> Self {
        Self {
            node,
        }
    }
}

impl ModuleBasic for GovModule {
    async fn handle_epoch(&self, _height: u64, _epoch: namada_sdk::state::Epoch) -> Result<(), Error> {
        Ok(())
    }

    fn register_periodic_operations(&self, _scheduler: &mut clokwerk::Scheduler) {
        // Do nothing
    }

    async fn handle_message(&self, message: crate::database::Message) -> Result<(), Error> {
        // Do nothing
        Ok(())
    }
}