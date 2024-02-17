use crate::database::Database;
use crate::Error;
use sqlx::{Postgres, QueryBuilder};

pub struct Validator {
    consensus_address: String,
    consensus_pubkey: String,
}

impl Validator {
    pub fn new(consensus_address: String, consensus_pubkey: String) -> Self {
        Validator {
            consensus_address,
            consensus_pubkey,
        }
    }
}

pub struct Validators(Vec<Validator>);

impl Validators {
    pub async fn save(&self, db: &Database) -> Result<(), Error> {
        let mut builder: QueryBuilder<Postgres> =
            QueryBuilder::new("INSERT INTO validator (consensus_address, consensus_pubkey)");

        builder.push_values(self.0.iter(), |mut b, v| {
            b.push_bind(v.consensus_address.clone())
                .push_bind(v.consensus_pubkey.clone());
        });
        builder.push("ON CONFLICT DO NOTHING");

        let query = builder.build();
        query.execute(db.pool().as_ref()).await?;

        Ok(())
    }
}


impl From<Vec<Validator>> for Validators {
    fn from(validators: Vec<Validator>) -> Self {
        Validators(validators)
    }
}