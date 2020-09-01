use log::info;
use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
};
use xz2::read::XzEncoder;

pub enum Action {
    BlockingDelete,
    CompressAndDelete,
}

pub struct Job {
    pub action: Action,
    pub arg: String,
}

impl Job {
    pub fn handle_job(data: &mut Job) {
        match data.action {
            Action::BlockingDelete => {
                info!("will delete file {}", data.arg);

                // todo: there must be better way to do this
                loop {
                    let result = std::fs::remove_file(data.arg.clone());
                    match result {
                        Ok(_) => {
                            info!("removed file {}", data.arg);
                            break;
                        }
                        Err(error) => match error.kind() {
                            ErrorKind::NotFound => {}
                            e => std::panic!("failed to remove file: {:?}", e),
                        },
                    };
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            Action::CompressAndDelete => {
                info!("will compress file {}", data.arg);
                let mut buf = Vec::<u8>::with_capacity(1024 * 1024 * 20);
                let mut compressor = XzEncoder::new(File::open(data.arg.clone()).unwrap(), 6);
                compressor.read_to_end(&mut buf).unwrap();
                let compressed_file_name = data.arg.clone() + ".xz";
                let mut file = File::create(compressed_file_name).unwrap();
                file.write_all(buf.as_slice()).unwrap();
                file.flush().unwrap();
                std::fs::remove_file(data.arg.clone()).unwrap();
            }
        }
    }
}
