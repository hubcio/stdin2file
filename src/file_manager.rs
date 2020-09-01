use log::info;
use std::io::Write;
extern crate task_queue;
use crate::action;

pub struct FileManager {
    files: std::collections::VecDeque<String>,
    max_files: usize,
    file_name: String,
    task_queue: task_queue::TaskQueue,
    compression: String,
    processed_files: usize,
}

impl FileManager {
    pub fn new(compression: String, max_files: usize, file_name: String) -> Self {
        let files = std::collections::VecDeque::new();
        let task_queue = task_queue::TaskQueue::new();
        let processed_files = 0;
        FileManager {
            files,
            max_files,
            file_name,
            task_queue,
            compression,
            processed_files,
        }
    }

    pub fn handle_add_new_file(&mut self, buffer: &mut Vec<u8>) {
        // create new file
        let file_name = FileManager::compose_file_name(self.file_name.clone(), self.processed_files);
        self.files.push_back(file_name.clone());

        info!("creating file {}", self.files.back().unwrap());

        let mut uncompressed_file =
            std::fs::File::create(self.files.back().unwrap().clone()).unwrap();
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
                );
                compression_sufix = ".xz";
            }
            "gz" => {} //todo
            _ => {}
        }

        // remove files over limit
        if self.max_files < self.files.len() {
            FileManager::enqueue_delete(
                &mut self.task_queue,
                self.files.pop_front().unwrap() + compression_sufix
            );
        }
    }

    fn compose_file_name(path: String, file_number: usize) -> String {
        return path + "." + &file_number.to_string();
    }

    fn enqueue_compress_and_delete(task_queue: &mut task_queue::TaskQueue, path: String) {
        task_queue
            .enqueue(move || {
                let mut work = action::Job {
                    action: action::Action::CompressAndDelete,
                    arg: path.clone(),
                };
                action::Job::handle_job(&mut work);
            })
            .unwrap();
    }

    fn enqueue_delete(task_queue: &mut task_queue::TaskQueue, path: String) {
        task_queue
            .enqueue(move || {
                let mut work = action::Job {
                    action: action::Action::BlockingDelete,
                    arg: path.clone(),
                };
                action::Job::handle_job(&mut work);
            })
            .unwrap();
    }

    pub fn wait_for_finish(self) {
        self.task_queue.stop_wait();
    }
}
