mod args;
mod compression;
mod file_remover;
mod logger;

use crate::args::Args;
use crate::compression::Compressor;
use crate::file_remover::FileRemover;

use anyhow::{Context, Result};
use std::io::Read;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    logger::configure_logger();

    let args = Args::new().with_context(|| "failed to parse args")?;

    let compression = args.compression;
    let max_files = args.max_files.get();
    let base_output_file = args.base_output_file;
    let chunk_bytes = args.chunk.get() * 1024 * 1024;

    log::debug!("START");

    let (tx, rx) = mpsc::channel(16);

    let mut handles = Vec::new();

    handles.push(tokio::spawn(async move {
        FileRemover::new(rx, max_files).run().await
    }));

    let mut buffer = Vec::with_capacity(chunk_bytes);
    let mut current_file_number = 0;
    for byte in std::io::stdin().bytes() {
        match byte {
            Ok(b) => {
                buffer.push(b);

                if buffer.len() == chunk_bytes {
                    let file_name = format!("{}.{}", base_output_file, current_file_number);
                    log::debug!(
                        "Got {} bytes from stdin, sending to Compressor as file {}",
                        buffer.len(),
                        file_name
                    );

                    current_file_number += 1;
                    let tx = tx.clone();

                    handles.push(tokio::spawn(async move {
                        Compressor::new(tx, file_name, buffer, compression)
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

    if !buffer.is_empty() {
        let file_name = base_output_file.clone() + "." + &current_file_number.to_string();
        let tx = tx.clone();
        handles.push(tokio::spawn(async move {
            Compressor::new(tx, file_name, buffer, compression)
                .run()
                .await
        }));
    }

    std::mem::drop(tx);

    log::debug!("Dropped channels, waiting for tasks to finish");

    for handle in handles {
        handle.await??;
    }

    log::debug!("END");

    Ok(())
}
