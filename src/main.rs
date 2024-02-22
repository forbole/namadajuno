use modules::StakingModule;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tracing;
use tracing_subscriber::layer::SubscriberExt;

use async_channel;
use async_channel::Receiver;
use async_channel::Sender;
use futures::stream::StreamExt;
use futures_util::pin_mut;
use futures_util::Stream;
use tokio::task::{JoinHandle, JoinSet};

use error::Error;
use tendermint_rpc::HttpClient;

mod config;
mod database;
mod error;
mod modules;
mod node;
mod utils;
mod worker;

// Max number of queued blocks in channel.
// this can be adjusted for optimal performance, however
// either http request or database_queries are both slow
// processes.
const CHANNEL_SIZE: usize = 100;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Error> {
    let config = config::load_config()?;

    let std_out = tracing_subscriber::fmt::layer().pretty();
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level));
    let subscriber = tracing_subscriber::Registry::default()
        .with(std_out)
        .with(env_filter);
    tracing::subscriber::set_global_default(subscriber).expect("Could not set global logger");

    // Build the client
    let client = HttpClient::new(config.node.config.rpc.address.as_ref()).unwrap();
    let node = node::Node::new(client.clone());

    start(config, node).await?;
    Ok(())
}

async fn start(config: config::Config, node: node::Node) -> Result<(), Error> {
    let db: database::Database = database::Database::new(&config.database).await?;

    // Get start height and current height
    let start_height = config.parsing.start_height;
    let current_height = node.latest_height().await?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx, rx): (Sender<u64>, Receiver<u64>) = async_channel::bounded(CHANNEL_SIZE);

    // Setup worker context
    let ctx = Arc::new(worker::Context::new(
        tx.clone(),
        rx,
        node.clone(),
        db.clone(),
        utils::load_checksums()?,
        vec![StakingModule::new(node.clone(), db.clone())],
    ));

    // Start workers
    let mut workers: JoinSet<Result<(), Error>> = JoinSet::new(); // Array of workers
    for _ in 0..config.parsing.workers {
        workers.spawn(worker::start(ctx.clone()));
    }

    // Enqueue missing blocks
    let missing_blocks_handler =
        enqueue_blocks(tx.clone(), start_height, current_height, shutdown.clone());
    missing_blocks_handler.await??;

    // Enqueue new blocks
    if config.parsing.listen_new_blocks {
        let new_blocks_handler =
            enqueue_new_blocks(tx, current_height, node.clone(), shutdown.clone());
        new_blocks_handler.await??;
    }

    // Wait for workers to finish
    while let Some(worker) = workers.join_next().await {
        worker??;
    }

    Ok(())
}

fn blocks_stream(start_height: u64, end_height: u64) -> impl Stream<Item = u64> {
    futures::stream::iter(start_height..end_height).then(move |i| async move { i })
}

fn enqueue_blocks(
    tx: Sender<u64>,
    start_height: u64,
    current_height: u64,
    producer_shutdown: Arc<AtomicBool>,
) -> JoinHandle<Result<(), Error>> {
    // Spawn the task
    let handler = tokio::spawn(async move {
        let stream = blocks_stream(start_height, current_height);
        pin_mut!(stream);

        while let Some(height) = stream.next().await {
            if producer_shutdown.load(Ordering::Relaxed) {
                break;
            }

            tx.send(height).await?;
        }

        Ok(())
    });

    handler
}

fn enqueue_new_blocks(
    tx: Sender<u64>,
    current_height: u64,
    node: node::Node,
    producer_shutdown: Arc<AtomicBool>,
) -> JoinHandle<Result<(), Error>> {
    let mut start_height = current_height;
    let handler = tokio::spawn(async move {
        loop {
            let new_height = node.latest_height().await.unwrap();
            enqueue_blocks(
                tx.clone(),
                start_height,
                new_height,
                producer_shutdown.clone(),
            )
            .await??;
            tokio::time::sleep(Duration::from_secs(5)).await;
            start_height = new_height.clone();
        }
    });

    handler
}
