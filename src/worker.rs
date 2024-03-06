use async_channel::Receiver;
use async_channel::Sender;
use std::sync::{Arc, Mutex};
use tracing::info;

use namada_sdk;
use namada_sdk::state::Epoch;
use namada_sdk::tx::data::TxType;
use namada_sdk::tx::Tx as NamadaTx;
use tendermint::abci::types::ExecTxResult;
use tendermint::abci::Code;
use tendermint::block::commit_sig::CommitSig;
use tendermint::block::Commit;
use tendermint::validator::Info as ValidatorInfo;

use crate::database;
use crate::modules::ModuleBasic;
use crate::modules::{StakingModule, GovModule};
use crate::node::Node;
use crate::utils;
use crate::Error;

#[derive(Clone)]
pub struct Context {
    tx: Sender<u64>,
    rx: Receiver<u64>,
    node: Node,
    db: database::Database,
    checksums_map: std::collections::HashMap<String, String>,
    epoch: Arc<Mutex<Option<Epoch>>>,

    // TODO: use trait object when the Namada RPC provides thread-safe client methods
    //modules: Vec<Box<dyn ModuleBasic>>,
    staking: StakingModule,
    gov: GovModule,
}

impl Context {
    pub fn new(
        tx: Sender<u64>,
        rx: Receiver<u64>,
        node: Node,
        db: database::Database,
        checksums_map: std::collections::HashMap<String, String>,
        staking: StakingModule,
        gov: GovModule,
    ) -> Self {
        Context {
            tx,
            rx,
            node,
            db,
            checksums_map,
            epoch: Arc::new(Mutex::new(None)),
            staking,
            gov,
        }
    }
}

pub async fn start(ctx: Arc<Context>) -> Result<(), Error> {
    loop {
        let height = ctx.rx.recv().await?;
        match process_block(&ctx, height).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to processing block {}: {}", height, e);
                tracing::info!("Reenqueuing block {}", height);
                ctx.tx.send(height).await?;
            }
        }
    }
}

async fn process_block(ctx: &Context, height: u64) -> Result<(), Error> {
    // Query the node
    let (tm_block_response, tm_block_results_response, tm_validators_response) = tokio::join!(
        ctx.node.block(height),
        ctx.node.block_results(height),
        ctx.node.validators(height),
    );

    let tm_block = tm_block_response?.block;
    let tm_block_results = tm_block_results_response?;
    let tm_validators = tm_validators_response?.validators;
    let txs_results = tm_block_results.txs_results.unwrap_or_default();

    // Save validators
    let validators: Vec<_> = tm_validators
        .iter()
        .map(|v| {
            database::Validator::new(
                utils::addr_to_bech32(v.address.clone()),
                v.pub_key.to_bech32(utils::COMMON_PK_HRP),
            )
        })
        .collect();
    database::Validators::from(validators).save(&ctx.db).await?;

    // Save block
    let block = database::Block::from_tm_block(tm_block.clone(), txs_results.clone());
    block.save(&ctx.db).await?;

    // Handle epoch for modules
    let height = tm_block.header.height.into();
    if let Some(epoch) = update_epoch(ctx, height).await? {
        ctx.staking.handle_epoch(height.into(), epoch).await?;
    }

    // Save commits
    if let Some(commit) = tm_block.last_commit {
        process_commit(ctx, height, commit, tm_validators).await?;
    }

    // Save transactions
    for (i, tx) in tm_block.data.iter().enumerate() {
        process_tx(ctx, height, txs_results[i].clone(), tx.clone()).await?;
    }

    info!("Processed {}", height);
    Ok(())
}

async fn process_tx(
    ctx: &Context,
    height: u64,
    tx_results: ExecTxResult,
    raw_tx: Vec<u8>,
) -> Result<(), Error> {
    let namada_tx: NamadaTx = NamadaTx::try_from(raw_tx.as_slice())
        .map_err(|_| Error::InvalidTxData("failed to parse raw transaction".into()))?;

    let tx_type = match namada_tx.header.tx_type {
        TxType::Raw => "raw",
        TxType::Wrapper(_) => "wrapper",
        TxType::Decrypted(_) => "decrypted",
        TxType::Protocol(_) => "protocol",
    };

    let tx_hash = utils::tx_hash(raw_tx);
    let tx = database::Tx::new(
        tx_hash.clone(),
        height as i64,
        tx_results.code == Code::Ok,
        String::from_utf8(namada_tx.memo().unwrap_or_default()).expect("Invalid UTF-8 sequence"),
        tx_type.into(),
        tx_results.gas_wanted,
        tx_results.gas_used,
        tx_results.log,
    );
    tx.save(&ctx.db).await?;

    if !tx.success {
        return Ok(());
    }

    // Save message
    let msg = database::Message::from_tx(&ctx.checksums_map, height as i64, tx_hash, namada_tx);
    if let Some(msg) = msg {
        msg.save(&ctx.db).await?;

        // Handle message for modules
        ctx.gov.handle_message(msg).await?;
    }

    Ok(())
}

async fn process_commit(
    ctx: &Context,
    height: u64,
    commit: Commit,
    validators: Vec<ValidatorInfo>,
) -> Result<(), Error> {
    let mut pre_commits: Vec<database::PreCommit> = vec![];

    for commit_sig in commit.signatures {
        match commit_sig {
            CommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => {
                if signature.is_none() {
                    continue;
                }

                pre_commits.push(database::PreCommit::from_tm_commit_sig(
                    height,
                    validator_address,
                    validators.clone(),
                    timestamp,
                ));
            }
            _ => {}
        }
    }

    database::PreCommits::from(pre_commits)
        .save(&ctx.db)
        .await?;

    Ok(())
}

async fn update_epoch(ctx: &Context, height: u64) -> Result<Option<Epoch>, Error> {
    let epoch = ctx.node.epoch(height.into()).await?;
    let mut current_epoch = ctx.epoch.lock().unwrap();
    if Some(epoch) <= *current_epoch {
        return Ok(None);
    }

    *current_epoch = Some(epoch);
    Ok(Some(epoch))
}
