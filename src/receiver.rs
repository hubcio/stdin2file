use lexical_sort::{natural_lexical_cmp, StringSort};
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::{mpsc, Mutex};

pub struct Receiver {
    rx: mpsc::Receiver<String>,
    completed_files: Arc<Mutex<VecDeque<String>>>,
    max_files: usize,
}

impl Receiver {
    pub fn new(rx: mpsc::Receiver<String>, max_files: usize) -> Self {
        Self {
            rx,
            completed_files: Arc::new(Mutex::new(VecDeque::new())),
            max_files,
        }
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Some(data) = self.rx.recv().await {
            log::debug!("RECEIVER received msg that file {} is completed", data);

            let mut completed_files = self.completed_files.lock().await;

            completed_files
                .make_contiguous()
                .string_sort_unstable(natural_lexical_cmp);

            completed_files.push_back(data);

            if completed_files.len() > self.max_files {
                let file_to_remove = completed_files.pop_front().expect("");
                log::debug!("RECEIVER removing file {}", file_to_remove);
                tokio::fs::remove_file(file_to_remove).await?
            }
        }
        Ok(())
    }
}
