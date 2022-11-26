use std::fs;
use std::panic;

const TEST_DIR: &str = "./tests/tmp";

fn setup() {
    fs::create_dir(TEST_DIR).ok();
}

fn run_test<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    setup();
    let result = panic::catch_unwind(test);
    teardown();
    assert!(result.is_ok())
}

fn teardown() {
    fs::remove_dir_all(TEST_DIR).ok();
}

#[cfg(test)]
mod tests {
    use super::run_test;
    use assert_cmd::Command;
    use flate2::write::GzDecoder;
    use rand::{distributions::Uniform, Rng};
    use serial_test::serial;
    use std::fs::File;
    use std::io::{Read, Write};
    use xz2::write::XzDecoder;

    #[test]
    #[serial]
    fn split_10_mb_uncompressed() {
        run_test(|| {
            let range = Uniform::new(0, 255);
            let data_size_bytes: usize = 10 * 1024 * 1024; // 10 MB

            let data: Vec<u8> = rand::thread_rng()
                .sample_iter(&range)
                .take(data_size_bytes)
                .collect();

            let original_data = data.clone();

            let mut cmd = Command::cargo_bin("stdin2file").expect("failed to run exe");

            cmd.timeout(std::time::Duration::from_secs(10))
                .arg("-c")
                .arg("1") // 1 MB chunk
                .arg("-o")
                .arg("./tests/tmp/test_uncompressed") // output path
                .arg("-m")
                .arg("5") // 5 files
                .write_stdin(data)
                .assert();

            let mut data_from_files: Vec<u8> = vec![];

            for i in 5..=9 {
                let mut file =
                    File::open("tests/tmp/test_uncompressed.".to_string() + &i.to_string())
                        .expect("failed to open file");

                file.read_to_end(&mut data_from_files)
                    .expect("failed to read_to_end");
            }

            assert_eq!(original_data[data_size_bytes / 2..], data_from_files);
        })
    }

    #[test]
    #[serial]
    fn split_10_mb_compressed_xz() {
        run_test(|| {
            let range = Uniform::new(0, 255);
            let data_size_bytes: usize = 10 * 1024 * 1024; // 10 MB

            let data: Vec<u8> = rand::thread_rng()
                .sample_iter(&range)
                .take(data_size_bytes)
                .collect();

            let original_data = data.clone();
            assert_ne!(original_data, []);

            let mut cmd = Command::cargo_bin("stdin2file").expect("failed to run exe");

            cmd.timeout(std::time::Duration::from_secs(10))
                .arg("-c")
                .arg("1") // 1 MB chunk
                .arg("-o")
                .arg("./tests/tmp/test_compressed_xz")
                .arg("-m")
                .arg("5") // 5 files
                .arg("-s")
                .arg("xz") // use xz
                .write_stdin(data)
                .assert();

            let mut decompressed_data: Vec<u8> = vec![];

            for i in 5..=9 {
                let mut compressed_data: Vec<u8> = vec![];

                File::open("./tests/tmp/test_compressed_xz.".to_string() + &i.to_string() + ".xz")
                    .expect("failed to open file")
                    .read_to_end(&mut compressed_data)
                    .expect("failed to read_to_end");

                XzDecoder::new(&mut decompressed_data)
                    .write_all(&compressed_data)
                    .expect("write_all failed");
            }

            assert_ne!(decompressed_data, []);
            assert_eq!(original_data[(data_size_bytes / 2)..], decompressed_data);
        });
    }

    #[test]
    #[serial]
    fn split_10_mb_compressed_gz() {
        run_test(|| {
            let range = Uniform::new(0, 255);
            let data_size_bytes: usize = 10 * 1024 * 1024; // 10 MB

            let data: Vec<u8> = rand::thread_rng()
                .sample_iter(&range)
                .take(data_size_bytes)
                .collect();

            let original_data = data.clone();
            assert_ne!(original_data, []);

            let mut cmd = Command::cargo_bin("stdin2file").expect("failed to run exe");

            cmd.timeout(std::time::Duration::from_secs(10))
                .arg("-c")
                .arg("1")
                .arg("-o")
                .arg("./tests/tmp/test_compressed_gz")
                .arg("-m")
                .arg("5")
                .arg("-s")
                .arg("gz")
                .write_stdin(data)
                .assert();

            let mut decompressed_data: Vec<u8> = vec![];

            for i in 5..=9 {
                let mut compressed_data: Vec<u8> = vec![];

                File::open("./tests/tmp/test_compressed_gz.".to_string() + &i.to_string() + ".gz")
                    .expect("failed to open file")
                    .read_to_end(&mut compressed_data)
                    .expect("failed to read_to_end");

                GzDecoder::new(&mut decompressed_data)
                    .write_all(&compressed_data)
                    .expect("write_all failed");
            }

            assert_ne!(decompressed_data, []);
            assert_eq!(original_data[(data_size_bytes / 2)..], decompressed_data);
        });
    }
}
