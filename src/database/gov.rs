use serde_json::json;
use namada_sdk::governance::ProposalType;
use sqlx::types::JsonValue;
use chrono::NaiveDateTime;

use crate::database::Database;
use crate::Error;

pub struct Proposal {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub content: JsonValue,
    pub submit_time: NaiveDateTime,
    pub voting_start_epoch: i64,
    pub voting_end_epoch: i64,
    pub grace_epoch: i64,
    pub proposer_address: String,
    pub status: String,
}

impl Proposal {
    pub fn new(
        id: u64,
        title: String,
        description: String,
        content: ProposalType,
        submit_time: String,
        voting_start_epoch: u64,
        voting_end_epoch: u64,
        grace_epoch: u64,
        proposer: String,
        status: String,
    ) -> Self {
        Proposal {
            id: id as i64,
            title,
            description,
            content: json!(content),
            submit_time: NaiveDateTime::parse_from_str(&submit_time, "%Y-%m-%dT%H:%M:%SZ").expect("invalid timestamp"),
            voting_start_epoch: voting_start_epoch as i64,
            voting_end_epoch: voting_end_epoch as i64,
            grace_epoch: grace_epoch as i64,
            proposer_address: proposer,
            status,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        let mut tx = db.pool().begin().await?;
        sqlx::query(
            r#"
            INSERT INTO proposal (id, title, description, content, submit_time, voting_start_epoch, voting_end_epoch, grace_epoch, proposer_address, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            content = EXCLUDED.content,
            submit_time = EXCLUDED.submit_time,
            voting_start_epoch = EXCLUDED.voting_start_epoch,
            voting_end_epoch = EXCLUDED.voting_end_epoch,
            grace_epoch = EXCLUDED.grace_epoch,
            proposer_address = EXCLUDED.proposer_address,
            status = EXCLUDED.status
            WHERE proposal.submit_time <= EXCLUDED.submit_time
            "#,
        )
        .bind(&self.id)
        .bind(&self.title)
        .bind(&self.description)
        .bind(&self.content)
        .bind(&self.submit_time)
        .bind(&self.voting_start_epoch)
        .bind(&self.voting_end_epoch)
        .bind(&self.grace_epoch)
        .bind(&self.proposer_address)
        .bind(&self.status)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}

pub struct ProposalVote {
    pub proposal_id: i64,
    pub voter_address: String,
    pub option: String,
    pub height: i64,
}

impl ProposalVote {
    pub fn new(proposal_id: u64, voter: String, option: String, height: i64) -> Self {
        ProposalVote {
            proposal_id: proposal_id as i64,
            voter_address: voter,
            option: match option.as_str() {
                "yay" => "yes".to_string(),
                "nay" => "no".to_string(),
                _ => option.to_string(),
            },
            height,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        let mut tx = db.pool().begin().await?;
        sqlx::query(
            r#"
            INSERT INTO proposal_vote (proposal_id, voter_address, option, height)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (proposal_id, voter_address) DO UPDATE SET
            option = EXCLUDED.option,
            height = EXCLUDED.height
            WHERE proposal_vote.height <= EXCLUDED.height
            "#,
        )
        .bind(&self.proposal_id)
        .bind(&self.voter_address)
        .bind(&self.option)
        .bind(&self.height)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}


pub struct ProposalTallyResult {
    pub proposal_id: i64,
    pub yes: i64,
    pub abstain: i64,
    pub no: i64,
    pub height: i64,
}

impl ProposalTallyResult {
    pub fn new(proposal_id: i64, yes: u64, abstain: u64, no: u64, height: u64) -> Self {
        ProposalTallyResult {
            proposal_id,
            yes: yes as i64,
            abstain: abstain as i64,
            no: no as i64,
            height: height as i64,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        let mut tx = db.pool().begin().await?;
        sqlx::query(
            r#"
            INSERT INTO tally (proposal_id, yes, abstain, no, height)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (proposal_id) DO UPDATE SET
            yes = EXCLUDED.yes,
            abstain = EXCLUDED.abstain,
            no = EXCLUDED.no,
            height = EXCLUDED.height
            WHERE tally.height <= EXCLUDED.height
            "#,
        )
        .bind(&self.proposal_id)
        .bind(&self.yes)
        .bind(&self.abstain)
        .bind(&self.no)
        .bind(&self.height)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
}