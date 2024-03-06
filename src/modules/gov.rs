use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::state::Epoch;

use crate::database::{Database, Message};
use crate::database::{Proposal, ProposalTallyResult, ProposalVote};
use crate::error::Error;
use crate::modules::ModuleBasic;
use crate::node::Node;

#[derive(Clone)]
pub struct GovModule {
    node: Node,
    db: Database,
}

impl GovModule {
    pub fn new(node: Node, db: Database) -> Self {
        Self { node, db }
    }
}

impl ModuleBasic for GovModule {
    async fn handle_epoch(&self, _height: u64, _epoch: Epoch) -> Result<(), Error> {
        Ok(())
    }

    fn register_periodic_operations(&self, _scheduler: &mut clokwerk::Scheduler) {
        // Do nothing
    }

    async fn handle_message(&self, message: Message) -> Result<(), Error> {
        match message.message_type.as_str() {
            "tx_init_proposal" => {
                let msg = serde_json::from_value::<InitProposalData>(message.value)?;
                let proposal = self.node.proposal(msg.id).await?;
                if let Some(proposal) = proposal {
                    Proposal::new(
                        proposal.id,
                        proposal
                            .content
                            .get("title")
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "No title".to_string()),
                        proposal
                            .content
                            .get("details")
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "No description".to_string()),
                        proposal.r#type,
                        proposal.content.get("created").unwrap().to_string(),
                        proposal.voting_start_epoch.into(),
                        proposal.voting_end_epoch.into(),
                        proposal.grace_epoch.into(),
                        proposal.author.encode(),
                        "PROPOSAL_STATUS_INIT".to_string(),
                    )
                    .save(&self.db)
                    .await?;
                }
            }
            "tx_vote_proposal" => {
                let msg = serde_json::from_value::<VoteProposalData>(message.value)?;
                ProposalVote::new(
                    msg.id,
                    msg.voter.encode(),
                    msg.vote.to_string(),
                    message.height,
                )
                .save(&self.db)
                .await?;
            }
            _ => {}
        }
        Ok(())
    }
}
