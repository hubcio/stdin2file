use flate2::bufread::GzEncoder;
use flate2::Compression;
use std::io::Read;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use xz2::read::XzEncoder;

#[derive(Debug, Copy, Clone)]
pub enum CompressionFormat {
    Gz,
    Xz,
    None,
}

impl CompressionFormat {
    pub fn suffix(&self) -> &'static str {
        match self {
            CompressionFormat::Gz => ".gz",
            CompressionFormat::Xz => ".xz",
            CompressionFormat::None => "",
        }
    }
}

pub struct Compressor {
    tx: mpsc::Sender<String>,
    file_name: String,
    buffer: Vec<u8>,
    compression: CompressionFormat,
}

impl Compressor {
    pub fn new(
        tx: mpsc::Sender<String>,
        file_name: String,
        buffer: Vec<u8>,
        compression: CompressionFormat,
    ) -> Self {
        Self {
            tx,
            file_name,
            buffer,
            compression,
        }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        let file_name = self.file_name.clone() + self.compression.suffix();
        let mut file = tokio::fs::File::create(file_name.clone()).await?;

        match self.compression {
            CompressionFormat::Gz => {
                let mut encoder = GzEncoder::new(self.buffer.as_slice(), Compression::default());
                let mut buffer = Vec::new();
                encoder.read_to_end(&mut buffer)?;
                file.write_all(&buffer).await?;
            }
            CompressionFormat::Xz => {
                let mut encoder = XzEncoder::new(self.buffer.as_slice(), 9);
                let mut buffer = Vec::new();
                encoder.read_to_end(&mut buffer)?;
                file.write_all(&buffer).await?;
            }
            CompressionFormat::None => {
                file.write_all(&self.buffer).await?;
            }
        }
        self.tx.send(file_name.clone()).await?;
        Ok(())
    }
}
