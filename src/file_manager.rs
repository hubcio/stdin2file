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
    task_queue: Option<task_queue::TaskQueue>,
    compression_suffix: String,
    proccessed_files: usize,
}

impl FileManager {
    pub fn new(compression: String, max_files: usize, file_name: String) -> Self {
        let files = VecDeque::new();
        let completed_files = Arc::new(Mutex::new(VecDeque::new()));
        let task_queue = Some(task_queue::TaskQueue::new());
        let compression_suffix = compression;
        let proccessed_files = 0;
        FileManager {
            files,
            completed_files,
            max_files,
            file_name,
            task_queue,
            compression_suffix,
            proccessed_files,
        }
    }

    pub fn handle_add_new_file(&mut self, buffer: &mut Vec<u8>) {
        // create new file
        let file_name =
            FileManager::compose_file_name(self.file_name.clone(), self.proccessed_files);

        self.files.push_back(file_name.clone());
        let mut uncompressed_file = File::create(self.files.back().unwrap().clone()).unwrap();

        info!("CREATE {}", self.files.back().unwrap());

        // dump buffer to file
        uncompressed_file.write(&buffer).unwrap();

        self.proccessed_files = self.proccessed_files + 1;

        // compress newest file
        match self.compression_suffix.as_str() {
            ".xz" => {
                FileManager::enqueue_compress_and_delete(
                    &mut self.task_queue.as_mut().unwrap(),
                    self.files.back().unwrap().clone(),
                    Arc::clone(&self.completed_files),
                );

                FileManager::delete_redundant_compressed_files(
                    self.max_files,
                    &self.files,
                    self.completed_files.clone(),
                );
            }
            ".gz" => {} // todo
            _ => {
                // no compression, so just check if there are files to remove
                FileManager::delete_redundant_files(self.max_files, &mut self.files);
            }
        }
    }

    pub fn wait_for_finish(&mut self) {
        // wait for all tasks to finish
        self.task_queue.take().unwrap().stop_wait();

        FileManager::delete_redundant_compressed_files(
            self.max_files,
            &self.files,
            self.completed_files.clone(),
        );
    }

    fn compose_file_name(path: String, file_number: usize) -> String {
        return path + "." + &file_number.to_string();
    }

    fn enqueue_compress_and_delete(
        task_queue: &mut task_queue::TaskQueue,
        file_name: String,
        cf: Arc<Mutex<VecDeque<String>>>,
    ) {
        task_queue
            .enqueue(move || {
                let mut buf = Vec::<u8>::with_capacity(1024 * 1024 * 20);
                let mut compressor = XzEncoder::new(File::open(file_name.clone()).unwrap(), 6);
                compressor.read_to_end(&mut buf).unwrap();
                let compressed_file_name = file_name.clone() + ".xz";
                info!("CREATE {}", compressed_file_name);
                let mut file = File::create(compressed_file_name.clone()).unwrap();
                file.write_all(buf.as_slice()).unwrap();
                file.flush().unwrap();
                std::fs::remove_file(file_name.clone()).unwrap();
                info!("REMOVE AFTER COMPRESSION {}", compressed_file_name);
                {
                    let mut completed_files = cf.lock().unwrap();
                    completed_files.make_contiguous().sort();
                    completed_files.push_back(compressed_file_name.clone());
                    info!("DONE {}", completed_files.back().clone().unwrap());
                }
            })
            .unwrap();
    }

    fn delete_redundant_compressed_files(
        max_files: usize,
        files: &VecDeque<String>,
        file_list: Arc<Mutex<VecDeque<String>>>,
    ) {
        if max_files < files.len() {
            let mut cf = file_list.lock().unwrap();
            while cf.len() > max_files {
                cf.make_contiguous().sort();
                let file = cf.pop_front().unwrap();
                info!("REMOVE COMPRESSED {}", file.clone());
                std::fs::remove_file(file).unwrap();
            }
        }
    }

    fn delete_redundant_files(max_files: usize, files: &mut VecDeque<String>) {
        while files.len() > max_files {
            info!("REMOVE {}", files.front().clone().unwrap());
            std::fs::remove_file(files.pop_front().unwrap()).unwrap()
        }
    }
}
