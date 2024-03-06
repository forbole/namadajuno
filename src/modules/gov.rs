use clokwerk::{Scheduler, TimeUnits};
use tokio::runtime::Handle;

use namada_sdk::governance::utils::TallyResult;
use namada_sdk::governance::{InitProposalData, VoteProposalData};
use namada_sdk::state::Epoch;

use crate::database::{Database, Message};
use crate::database::{Block, Proposal, ProposalTallyResult, ProposalVote};
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

    async fn tally_active_proposals(&self) -> Result<(), Error> {
        let proposals = Proposal::voting_proposals(&self.db).await?;
        let height = self.node.latest_height().await?;
        for proposal in proposals {
            let tally = self.node.proposal_result(proposal.id as u64).await?;
            if let Some(tally) = tally {
                // Save proposal tally result
                ProposalTallyResult::new(
                    proposal.id as i64,
                    tally.total_yay_power.to_string(),
                    tally.total_nay_power.to_string(),
                    tally.total_abstain_power.to_string(),
                    height,
                )
                .save(&self.db)
                .await?;
            }
        }
        Ok(())
    }
}

impl ModuleBasic for GovModule {
    async fn handle_epoch(&self, height: u64, epoch: Epoch) -> Result<(), Error> {
        // Update active proposal from init to voting
        Proposal::update_active_proposals_statuses_from_init(&self.db, epoch.into()).await?;

        // Update ended proposals
        let ended_proposals = Proposal::voting_ended_proposals(&self.db, epoch.into()).await?;
        for mut proposal in ended_proposals {
            let tally = self.node.proposal_result(proposal.id as u64).await?;

            if let Some(tally) = tally {
                // Save proposal tally result
                ProposalTallyResult::new(
                    proposal.id as i64,
                    tally.total_yay_power.to_string(),
                    tally.total_nay_power.to_string(),
                    tally.total_abstain_power.to_string(),
                    height,
                )
                .save(&self.db)
                .await?;

                // Update proposal status
                match tally.result {
                    TallyResult::Passed => {
                        proposal.status = "PROPOSAL_STATUS_PASSED".to_string();
                    }
                    TallyResult::Rejected => {
                        proposal.status = "PROPOSAL_STATUS_REJECTED".to_string();
                    }
                }
                proposal.save(&self.db).await?;
            }
        }

        Ok(())
    }

    fn register_periodic_operations(&self, scheduler: &mut Scheduler) {
        let module = self.clone();
        scheduler.every(10.minutes()).run(move || {
            let module = module.clone();
            Handle::current().spawn_blocking(|| {
                Handle::current().block_on(async move {
                    match module.tally_active_proposals().await {
                        Ok(_) => {
                            tracing::info!("Tallied active proposals");
                        }
                        Err(e) => {
                            tracing::error!("Failed to tally active proposals: {}", e);
                        }
                    }
                })
            });
        });
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
                        Block::block_at_height(&self.db, message.height).await?.map(|b| b.timestamp).expect("Block not found"),
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
