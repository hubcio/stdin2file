use log::info;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use xz2::read::XzEncoder;

pub struct FileManager {
    files: VecDeque<String>,
    completed_files: Arc<Mutex<VecDeque<String>>>,
    max_files: usize,
    file_name: String,
    task_queue: task_queue::TaskQueue,
    compression: String,
    processed_files: usize,
}

impl FileManager {
    pub fn new(compression: String, max_files: usize, file_name: String) -> Self {
        let files = VecDeque::new();
        let files_to_delete = Arc::new(Mutex::new(VecDeque::new()));
        let task_queue = task_queue::TaskQueue::new();
        let processed_files = 0;
        FileManager {
            files,
            completed_files: files_to_delete,
            max_files,
            file_name,
            task_queue,
            compression,
            processed_files,
        }
    }

    pub fn handle_add_new_file(&mut self, buffer: &mut Vec<u8>) {
        // create new file
        let file_name =
            FileManager::compose_file_name(self.file_name.clone(), self.processed_files);
        self.files.push_back(file_name.clone());

        let mut uncompressed_file =
            std::fs::File::create(self.files.back().unwrap().clone()).unwrap();

        info!("CREATE {}", self.files.back().unwrap());

        self.processed_files = self.processed_files + 1;

        // dump buffer to file
        uncompressed_file.write(&buffer).unwrap();

        // compress newest file
        let mut compression_sufix = "";
        match self.compression.as_str() {
            "xz" => {
                FileManager::enqueue_compress_and_delete(
                    &mut self.task_queue,
                    self.files.back().unwrap().clone(),
                    Arc::clone(&self.completed_files),
                );
                compression_sufix = ".xz";
            }
            "gz" => {} //todo
            _ => {}
        }

        // remove files over-limit
        if self.max_files < self.files.len() {
            let mut cf = self.completed_files.lock().unwrap();
            while cf.len() > self.max_files {
                let file_to_delete = cf.pop_front().unwrap() + compression_sufix;
                info!("REMOVE {}", file_to_delete.clone());
                std::fs::remove_file(file_to_delete).unwrap();
            }
        }
    }

    fn compose_file_name(path: String, file_number: usize) -> String {
        return path + "." + &file_number.to_string();
    }

    fn enqueue_compress_and_delete(
        task_queue: &mut task_queue::TaskQueue,
        path: String,
        cf: Arc<Mutex<VecDeque<String>>>,
    ) {
        task_queue
            .enqueue(move || {
                let mut buf = Vec::<u8>::with_capacity(1024 * 1024 * 20);
                let mut compressor = XzEncoder::new(File::open(path.clone()).unwrap(), 6);
                compressor.read_to_end(&mut buf).unwrap();
                let compressed_file_name = path.clone() + ".xz";
                info!("CREATE {}", compressed_file_name.clone());

                let mut file = File::create(compressed_file_name).unwrap();
                file.write_all(buf.as_slice()).unwrap();
                file.flush().unwrap();
                std::fs::remove_file(path.clone()).unwrap();
                {
                    let mut completed_files = cf.lock().unwrap();
                    completed_files.make_contiguous().sort();
                    completed_files.push_back(path.clone());
                    info!("DONE {}.xz", completed_files.back().clone().unwrap());
                }
            })
            .unwrap();
    }

    pub fn wait_for_finish(self) {
        self.task_queue.stop_wait();

        if self.max_files < self.files.len() {
            let mut cf = self.completed_files.lock().unwrap();
            while cf.len() > self.max_files {
                let file = cf.pop_front().unwrap();
                info!("REMOVE END {}", file.clone());
                std::fs::remove_file(file + ".xz").unwrap();
            }
        }
    }
}
