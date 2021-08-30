use anyhow::{Context, Result};
use lexical_sort::{natural_lexical_cmp, StringSort};
use log::error;
use std::collections::VecDeque;
use std::io::Read;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use xz2::read::XzEncoder;

mod args;
mod logger;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    logger::configure_logger();

    let args = args::Args::new().with_context(|| "failed to parse args")?;

    let compression_suffix = args.compression;

    let max_files = args.max_files.get();
    let base_output_file = args.base_output_file;

    let chunk_bytes = args.chunk.get() * 1024 * 1024;
    let completed_files: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));

    let mut buffer: Vec<u8> = Vec::with_capacity(chunk_bytes);
    let mut handles: Vec<JoinHandle<Result<(), anyhow::Error>>> = vec![];
    let mut current_file_number: usize = 0;
    log::info!("start");

    // mpsc
    let (tx, mut rx) = mpsc::channel(16);

    // create receiver
    handles.push(tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            log::info!("CONSUMER received! {}", data);

            let mut completed_files = completed_files.lock().await;

            completed_files
                .make_contiguous()
                .string_sort_unstable(natural_lexical_cmp);

            completed_files.push_back(data);

            if completed_files.len() > max_files {
                let file_to_remove = completed_files.pop_front().unwrap();
                log::info!("removing file {}, {:?}", file_to_remove, completed_files);
                tokio::fs::remove_file(file_to_remove).await?
            }
        }
        Ok(())
    }));

    // read from stdin and spawn tokio tasks
    for byte in std::io::stdin().bytes() {
        match byte {
            Ok(b) => {
                buffer.push(b);

                if buffer.len() == chunk_bytes {
                    let file_name = compose_file_name(&base_output_file, current_file_number);
                    current_file_number += 1;
                    process(file_name, &tx, &compression_suffix, &mut handles, buffer)?;
                    buffer = Vec::with_capacity(chunk_bytes);
                }
            }
            Err(n) => error!("{}", n),
        }
    }

    // read is done, but maybe some data is still in buffer
    if !buffer.is_empty() {
        let file_name = compose_file_name(&base_output_file, current_file_number);
        process(file_name, &tx, &compression_suffix, &mut handles, buffer)?;
    }

    // drop the sender so the receiver doesn't listen forever
    std::mem::drop(tx);

    for handle in handles {
        handle.await??;
    }

    log::info!("end");

    Ok(())
}

fn process(
    file_name: String,
    tx: &mpsc::Sender<String>,
    compression: &Option<String>,
    handles: &mut Vec<JoinHandle<Result<(), anyhow::Error>>>,
    buffer: Vec<u8>,
) -> Result<(), anyhow::Error> {
    let tx: tokio::sync::mpsc::Sender<String> = tx.clone();
    match compression {
        Some(mode) => {
            match mode.as_str() {
                // no compression
                "xz" => Ok(handles.push(tokio::spawn(async move {
                    process_compressed(file_name, buffer, tx).await
                }))),

                "gz" => todo!(),

                // Ok(handles.push(tokio::spawn(async move {
                //     process_compressed(file_name, buffer, tx).await
                // }))),
                _ => Ok(()),
            }
        }
        None => Ok(handles.push(tokio::spawn(async move {
            process_uncompressed(file_name, buffer, tx).await
        }))),
    }
}

fn compose_file_name(path: &String, file_number: usize) -> String {
    path.to_owned() + "." + &file_number.to_string()
}

async fn process_uncompressed(
    file_name: String,
    buffer: Vec<u8>,
    tx: tokio::sync::mpsc::Sender<String>,
) -> Result<(), anyhow::Error> {
    let mut file = tokio::fs::File::create(file_name.clone()).await?;
    log::info!("process_uncompressed CREATE {}", file_name);

    file.write_all(&buffer).await?;
    log::info!("process_uncompressed WRITE {}", file_name);

    tx.send(file_name).await?;

    Ok(())
}

async fn process_compressed(
    compressed_file_name: String,
    buffer: Vec<u8>,
    tx: tokio::sync::mpsc::Sender<String>,
) -> Result<(), anyhow::Error> {
    let mut compressor = XzEncoder::new(buffer.as_slice(), 6);
    let mut data = vec![];
    compressor.read_to_end(&mut data).unwrap();
    let compressed_file_name = compressed_file_name.clone() + ".xz";
    let mut file = tokio::fs::File::create(compressed_file_name.clone()).await?;
    log::info!("process_compressed CREATE {}", compressed_file_name);

    file.write_all(data.as_slice()).await?;
    log::info!("process_compressed WRITE {}", compressed_file_name);

    tx.send(compressed_file_name).await?;

    Ok(())
}
