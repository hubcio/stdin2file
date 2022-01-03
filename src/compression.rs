#[derive(Debug, Copy, Clone)]
pub enum CompressionFormat {
    Gz,
    Xz,
}

impl CompressionFormat {
    pub fn suffix(&self) -> &'static str {
        match self {
            CompressionFormat::Gz => ".gz",
            CompressionFormat::Xz => ".xz",
        }
    }
}
