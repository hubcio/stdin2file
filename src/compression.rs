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

    pub fn encoder(self, buf: &[u8]) -> Box<dyn Read + Send + '_> {
        match self {
            CompressionFormat::Gz => Box::new(GzEncoder::new(buf, Compression::default())),
            CompressionFormat::Xz => Box::new(XzEncoder::new(buf, 9)),
            CompressionFormat::None => Box::new(buf),
        }
    }
}

pub struct Compressor {
    tx: mpsc::Sender<String>,       // channel to send file name after completion
    file_name: String,              // raw file name without compression suffix
    buffer: Vec<u8>,                // input buffer with data
    compression: CompressionFormat, // compression format
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
        log::debug!(
            "COMPRESSION({}) starting to compress {}",
            self.compression.suffix(),
            file_name
        );
        let mut encoder = self.compression.encoder(self.buffer.as_slice());
        let mut buffer = Vec::new();
        encoder.read_to_end(&mut buffer)?;
        file.write_all(&buffer).await?;
        log::debug!(
            "COMPRESSION({}) file {} is finished",
            self.compression.suffix(),
            file_name
        );
        self.tx.send(file_name.clone()).await?;
        Ok(())
    }
}
