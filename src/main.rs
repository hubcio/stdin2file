mod args;
mod compression;
mod file_remover;
mod logger;

use crate::args::Args;
use crate::file_remover::FileRemover;

use anyhow::{Context, Result};
use std::io::Read;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    logger::configure_logger();

    let args = Args::new().with_context(|| "failed to parse args")?;

    let compression = args.compression;
    let max_files = args.max_files.get();
    let base_output_file = args.base_output_file;
    let chunk_bytes = args.chunk.get() * 1024 * 1024;

    log::debug!("START");

    // mpsc
    let (tx, rx) = mpsc::channel(16);

    // handles to all tokio tasks
    let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];

    // create receiver which will remove files from completed_files
    let mut receiver = FileRemover::new(rx, max_files);
    handles.push(tokio::spawn(async move { receiver.run().await }));

    // read from stdin and spawn senders which will process data
    let mut buffer: Vec<u8> = Vec::with_capacity(chunk_bytes);
    let mut current_file_number: usize = 0;
    for byte in std::io::stdin().bytes() {
        match byte {
            Ok(b) => {
                buffer.push(b);

                if buffer.len() == chunk_bytes {
                    let file_name =
                        base_output_file.clone() + "." + &current_file_number.to_string();
                    current_file_number += 1;
                    let tx: mpsc::Sender<String> = tx.clone();

                    handles.push(tokio::spawn(async move {
                        compression::Compressor::new(tx, file_name, buffer, compression)
                            .run()
                            .await
                    }));
                    buffer = Vec::with_capacity(chunk_bytes);
                }
            }
            Err(n) => log::error!("{}", n),
        }
    }

    log::debug!("No more data in stdin");

    // no more data in stdin, but maybe some data is still in buffer
    if !buffer.is_empty() {
        let file_name = base_output_file.clone() + "." + &current_file_number.to_string();
        let tx: tokio::sync::mpsc::Sender<String> = tx.clone();
        handles.push(tokio::spawn(async move {
            compression::Compressor::new(tx, file_name, buffer, compression)
                .run()
                .await
        }));
    }

    // drop the sender so the receiver doesn't listen forever
    std::mem::drop(tx);

    for handle in handles {
        handle.await??;
    }

    log::debug!("END");

    Ok(())
}
