use tokio::join;
use tokio::task::JoinHandle;

use async_channel::Receiver;

use tendermint::block::commit_sig::CommitSig;
use tendermint::block::Commit;
use tendermint::validator::Info as ValidatorInfo;

use crate::config;
use crate::database;
use crate::node;
use crate::utils;
use crate::Error;

pub fn start(
    rx: Receiver<u64>,
    config: config::ChainConfig,
    node: node::Node,
    db: database::Database,
) -> JoinHandle<Result<(), Error>> {
    println!("worker start");
    tokio::spawn(async move {
        loop {
            let height = rx.recv().await?;
            process_block(height, config.clone(), &node, &db).await?;
        }
    })
}

async fn process_block(
    height: u64,
    config: config::ChainConfig,
    node: &node::Node,
    db: &database::Database,
) -> Result<(), Error> {
    let (tm_block_response, tm_block_results_response, tm_validators_response) = tokio::join!(
        node.block(height),
        node.block_results(height),
        node.validators(height)
    );

    let tm_block = tm_block_response?.block;
    let tm_block_results = tm_block_results_response?;
    let tm_validators = tm_validators_response?.validators;

    // Save validators
    let validators: Vec<_> = tm_validators
        .iter()
        .map(|v| {
            database::Validator::new(
                utils::convert_consensus_addr_to_bech32(&config.bech32_prefix, v.address.clone()),
                v.pub_key
                    .to_bech32(utils::consensus_pub_key_prefix(&config.bech32_prefix).as_str()),
            )
        })
        .collect();
    database::Validators::from(validators).save(db).await?;

    // Save block
    let block = database::Block::from_tm_block(
        tm_block.clone(),
        tm_block_results.txs_results,
        &config.bech32_prefix,
    );
    block.save(db).await?;

    // Save commits
    if let Some(commit) = tm_block.last_commit {
        process_commit(&config, &db, height, commit, tm_validators).await?;
    }

    println!("Processed {}", height);
    Ok(())
}

async fn process_commit(
    config: &config::ChainConfig,
    db: &database::Database,
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
                    &config.bech32_prefix,
                    height,
                    validator_address,
                    validators.clone(),
                    timestamp,
                ));
            }
            CommitSig::BlockIdFlagNil {
                validator_address,
                timestamp,
                signature,
            } => {
                if signature.is_none() {
                    continue;
                }

                pre_commits.push(database::PreCommit::from_tm_commit_sig(
                    &config.bech32_prefix,
                    height,
                    validator_address,
                    validators.clone(),
                    timestamp,
                ));
            }
            _ => {}
        }
    }

    database::PreCommits::from(pre_commits).save(db).await?;

    Ok(())
}
