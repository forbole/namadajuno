use chrono::NaiveDateTime;
use namada_sdk::governance::utils::TallyType;
use namada_sdk::governance::ProposalType;
use serde_json::json;
use sqlx::types::JsonValue;
use sqlx::FromRow;
use std::collections::BTreeMap;

use crate::database::Database;
use crate::Error;

#[derive(FromRow)]
pub struct Proposal {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub metadata: String,
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
        metadata: BTreeMap<String, String>,
        content: ProposalType,
        submit_time: NaiveDateTime,
        voting_start_epoch: u64,
        voting_end_epoch: u64,
        grace_epoch: u64,
        proposer: String,
        status: String,
    ) -> Self {
        Proposal {
            id: id as i32,
            title,
            description,
            metadata: json!(&metadata).to_string(),
            content: json!(content),
            submit_time,
            voting_start_epoch: voting_start_epoch as i64,
            voting_end_epoch: voting_end_epoch as i64,
            grace_epoch: grace_epoch as i64,
            proposer_address: proposer,
            status,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO proposal (id, title, description, metadata, content, submit_time, voting_start_epoch, voting_end_epoch, grace_epoch, proposer_address, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(&self.id)
        .bind(&self.title)
        .bind(&self.description)
        .bind(&self.metadata)
        .bind(&self.content)
        .bind(&self.submit_time)
        .bind(&self.voting_start_epoch)
        .bind(&self.voting_end_epoch)
        .bind(&self.grace_epoch)
        .bind(&self.proposer_address)
        .bind(&self.status)
        .execute(&db.pool())
        .await?;

        Ok(())
    }

    pub async fn update_active_proposals_statuses_from_init(
        db: &Database,
        epoch: u64,
    ) -> Result<(), Error> {
        sqlx::query(
            r#"UPDATE proposal SET status = 'PROPOSAL_STATUS_VOTING_PERIOD' WHERE voting_start_epoch <= $1 AND status = 'PROPOSAL_STATUS_INIT'"#,
        )
        .bind(epoch as i64)
        .execute(&db.pool())
        .await?;

        Ok(())
    }

    pub async fn update_ended_proposal_status(
        db: &Database,
        epoch: u64,
        id: i32,
        status: String,
    ) -> Result<(), Error> {
        sqlx::query(r#"UPDATE proposal SET status = $1 WHERE id = $2 AND voting_end_epoch <= $3"#)
            .bind(status)
            .bind(id)
            .bind(epoch as i64)
            .execute(&db.pool())
            .await?;

        Ok(())
    }

    pub async fn voting_proposals(db: &Database) -> Result<Vec<Proposal>, Error> {
        let proposals = sqlx::query_as(
            r#"SELECT * FROM proposal WHERE status = 'PROPOSAL_STATUS_VOTING_PERIOD'"#,
        )
        .fetch_all(&db.pool())
        .await?;

        Ok(proposals)
    }

    pub async fn voting_ended_proposals(db: &Database, epoch: u64) -> Result<Vec<Proposal>, Error> {
        let proposals = sqlx::query_as(r#"SELECT * FROM proposal WHERE voting_end_epoch <= $1 AND status != 'PROPOSAL_STATUS_PASSED' AND status != 'PROPOSAL_STATUS_REJECTED'"#)
            .bind(epoch as i64)
            .fetch_all(&db.pool())
            .await?;

        Ok(proposals)
    }
}

pub struct ProposalVote {
    pub proposal_id: i32,
    pub voter_address: String,
    pub option: String,
    pub height: i64,
}

impl ProposalVote {
    pub fn new(proposal_id: u64, voter: String, option: String, height: i64) -> Self {
        ProposalVote {
            proposal_id: proposal_id as i32,
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
        .execute(&db.pool())
        .await?;

        Ok(())
    }
}

pub struct ProposalTallyResult {
    pub proposal_id: i32,
    pub tally_type: String,
    pub total: String,
    pub yes: String,
    pub no: String,
    pub abstain: String,
    pub height: i64,
}

impl ProposalTallyResult {
    pub fn new(
        proposal_id: i64,
        tally_type: TallyType,
        total: String,
        yes: String,
        no: String,
        abstain: String,
        height: u64,
    ) -> Self {
        ProposalTallyResult {
            proposal_id: proposal_id as i32,
            tally_type: match tally_type {
                TallyType::TwoThirds => "two_thirds".to_string(),
                TallyType::OneHalfOverOneThird => "one_half_over_one_third".to_string(),
                TallyType::LessOneHalfOverOneThirdNay => {
                    "less_one_half_over_one_third_nay".to_string()
                }
            },
            total,
            yes,
            no,
            abstain,
            height: height as i64,
        }
    }

    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO proposal_tally_result (proposal_id, tally_type, total, yes, abstain, no, height)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (proposal_id) DO UPDATE SET
            total = EXCLUDED.total,
            yes = EXCLUDED.yes,
            abstain = EXCLUDED.abstain,
            no = EXCLUDED.no,
            height = EXCLUDED.height
            WHERE proposal_tally_result.height <= EXCLUDED.height
            "#,
        )
        .bind(&self.proposal_id)
        .bind(&self.tally_type)
        .bind(&self.total)
        .bind(&self.yes)
        .bind(&self.abstain)
        .bind(&self.no)
        .bind(&self.height)
        .execute(&db.pool())
        .await?;

        Ok(())
    }
}
