use log::error;
use std::io::Read;

mod args;
mod file_manager;
mod logger;

fn main() {
    logger::configure_logger();
    let args = args::Args::new();

    let mut file_manager = file_manager::FileManager::new(
        String::from(".") + args.compression_mode.as_str(),
        args.max_files,
        args.output_file,
    );

    let chunk_bytes = args.chunk * 1024 * 1024;
    let mut buffer = Vec::with_capacity(chunk_bytes);

    for b in std::io::stdin().bytes() {
        match b {
            Ok(b) => {
                buffer.push(b);

                if buffer.len() == chunk_bytes {
                    file_manager.handle_add_new_file(&mut buffer);
                    buffer.clear();
                }
            }
            Err(n) => error!("{}", n),
        }
    }

    // read is done but some data is still in buffer
    if !buffer.is_empty() {
        file_manager.handle_add_new_file(&mut buffer);
    }

    file_manager.wait_for_finish();
}
