use crate::compression::CompressionFormat;

use flate2::bufread::GzEncoder;
use flate2::Compression;
use std::io::Read;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use xz2::read::XzEncoder;

pub struct Sender {
    tx: mpsc::Sender<String>,
    file_name: String,
    buffer: Vec<u8>,
    compression: Option<CompressionFormat>,
}

impl Sender {
    pub fn new(
        tx: mpsc::Sender<String>,
        file_name: String,
        buffer: Vec<u8>,
        compression: Option<CompressionFormat>,
    ) -> Self {
        Self {
            tx,
            file_name,
            buffer,
            compression,
        }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        match self.compression {
            Some(CompressionFormat::Gz) => {
                let compressed_file_name =
                    self.file_name.clone() + self.compression.unwrap().suffix();
                let mut file = tokio::fs::File::create(compressed_file_name.clone()).await?;
                let mut encoder = GzEncoder::new(self.buffer.as_slice(), Compression::default());
                let mut buffer = Vec::new();
                encoder.read_to_end(&mut buffer)?;
                file.write_all(&buffer).await?;
                self.tx.send(compressed_file_name.clone()).await?;
            }
            Some(CompressionFormat::Xz) => {
                let compressed_file_name =
                    self.file_name.clone() + self.compression.unwrap().suffix();
                let mut file = tokio::fs::File::create(compressed_file_name.clone()).await?;
                let mut encoder = XzEncoder::new(self.buffer.as_slice(), 9);
                let mut buffer = Vec::new();
                encoder.read_to_end(&mut buffer)?;
                file.write_all(&buffer).await?;
                self.tx.send(compressed_file_name.clone()).await?;
            }
            None => {
                let mut file = tokio::fs::File::create(&self.file_name).await?;
                file.write_all(&self.buffer).await?;
                self.tx.send(self.file_name.clone()).await?;
            }
        }
        Ok(())
    }
}
