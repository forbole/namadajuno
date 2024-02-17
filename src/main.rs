use futures::stream::StreamExt;
use futures_util::pin_mut;
use futures_util::Stream;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

use async_channel;
use async_channel::Receiver;
use async_channel::Sender;

use error::Error;
use tendermint_rpc::HttpClient;

mod config;
mod database;
mod error;
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
    start(config).await?;
    Ok(())
}

async fn start(config: config::Config) -> Result<(), Error> {
    // Build the client
    let client = HttpClient::new(config.node.config.rpc.address.as_ref()).unwrap();
    let node = node::Node::new(client.clone());
    let db: database::Database = database::Database::new(&config.database).await?;

    // Get start height and current height
    let start_height = config.parsing.start_height;
    let current_height = node.latest_height().await?;

    let shutdown = Arc::new(AtomicBool::new(false));
    let (tx, rx): (Sender<u64>, Receiver<u64>) = async_channel::bounded(CHANNEL_SIZE);

    // Start workers
    let mut workers: Vec<JoinHandle<Result<(), Error>>> = vec![]; // Array of workers
    let ctx = worker::Context::new(
        rx,
        config.chain.bech32_prefix.clone(),
        node.clone(),
        db.clone(),
        std::collections::HashMap::new(),
    );
    for _ in 0..config.parsing.workers {
        let worker = worker::start(ctx.clone());
        workers.push(worker);
    }

    // Enqueue missing blocks
    let missing_blocks_handler =
        enqueue_blocks(tx.clone(), start_height, current_height, shutdown.clone());
    missing_blocks_handler.await??;

    // Enqueue new blocks
    let new_blocks_handler = enqueue_new_blocks(tx, current_height, node.clone(), shutdown.clone());
    new_blocks_handler.await??;

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
