use crate::args::Args;

use anyhow::{Context, Result};

use std::collections::VecDeque;
use std::io::Read;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

mod args;
mod compression;
mod logger;
mod receiver;
mod sender;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    logger::configure_logger();

    let args = Args::new().with_context(|| "failed to parse args")?;

    let compression = args.compression;
    let max_files = args.max_files.get();
    let base_output_file = args.base_output_file;
    let chunk_bytes = args.chunk.get() * 1024 * 1024;

    let completed_files: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    let mut buffer: Vec<u8> = Vec::with_capacity(chunk_bytes);
    let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
    let mut current_file_number: usize = 0;

    log::debug!("START");

    // mpsc
    let (tx, rx) = mpsc::channel(16);

    // create receiver which will remove files from completed_files
    let mut receiver = receiver::Receiver::new(rx, completed_files.clone(), max_files);
    handles.push(tokio::spawn(async move { receiver.run().await }));

    // read from stdin and spawn senders which will process data
    for byte in std::io::stdin().bytes() {
        match byte {
            Ok(b) => {
                buffer.push(b);

                if buffer.len() == chunk_bytes {
                    let file_name =
                        base_output_file.clone() + "." + &current_file_number.to_string();
                    current_file_number += 1;
                    let tx: mpsc::Sender<String> = tx.clone();
                    let mut sender = sender::Sender::new(tx, file_name, buffer, compression);

                    handles.push(tokio::spawn(async move { sender.run().await }));
                    buffer = Vec::with_capacity(chunk_bytes);
                }
            }
            Err(n) => log::error!("{}", n),
        }
    }

    // no more data in stdin, but maybe some data is still in buffer
    if !buffer.is_empty() {
        let file_name = base_output_file.clone() + "." + &current_file_number.to_string();
        let tx: tokio::sync::mpsc::Sender<String> = tx.clone();
        let mut sender = sender::Sender::new(tx, file_name, buffer, compression);
        handles.push(tokio::spawn(async move { sender.run().await }));
    }

    // drop the sender so the receiver doesn't listen forever
    std::mem::drop(tx);

    for handle in handles {
        handle.await??;
    }

    log::info!("END");

    Ok(())
}
