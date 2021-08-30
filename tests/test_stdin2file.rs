#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use flate2::write::GzDecoder;
    use rand::{distributions::Uniform, Rng};
    use std::io::{Read, Write};
    use xz2::write::XzDecoder;

    #[test]
    fn split_10_mb_uncompressed() -> Result<(), Box<dyn std::error::Error>> {
        let range = Uniform::new(0, 255);
        let data_size_bytes: usize = 10 * 1024 * 1024; // 10 MB

        let data: Vec<u8> = rand::thread_rng()
            .sample_iter(&range)
            .take(data_size_bytes)
            .collect();

        let original_data = data.clone();

        let mut cmd = Command::cargo_bin("stdin2file")?;

        cmd.timeout(std::time::Duration::from_secs(10))
            .arg("-c")
            .arg("1")
            .arg("-o")
            .arg("test_uncompressed")
            .arg("-m")
            .arg("5")
            .write_stdin(data)
            .ok()?;

        let mut data_from_files: Vec<u8> = vec![];

        for i in 6..=10 {
            let mut file = std::fs::File::open("test_uncompressed.".to_string() + &i.to_string())?;
            file.read_to_end(&mut data_from_files)?;
        }

        assert_eq!(original_data[data_size_bytes / 2..], data_from_files);

        cleanup("test_uncompressed", "")?;
        Ok(())
    }

    #[test]
    fn split_10_mb_compressed_xz() -> Result<(), Box<dyn std::error::Error>> {
        let range = Uniform::new(0, 255);
        let data_size_bytes: usize = 10 * 1024 * 1024; // 10 MB

        let data: Vec<u8> = rand::thread_rng()
            .sample_iter(&range)
            .take(data_size_bytes)
            .collect();

        let original_data = data.clone();
        assert_ne!(original_data, []);

        let mut cmd = Command::cargo_bin("stdin2file")?;

        cmd.timeout(std::time::Duration::from_secs(10))
            .arg("-c")
            .arg("1")
            .arg("-o")
            .arg("test_compressed")
            .arg("-m")
            .arg("5")
            .arg("-s")
            .arg("xz")
            .write_stdin(data)
            .ok()?;

        let mut data_from_files: Vec<u8> = vec![];

        for i in 6..=10 {
            let mut file =
                std::fs::File::open("test_compressed.".to_string() + &i.to_string() + ".xz")?;
            let mut compressed_data: Vec<u8> = vec![];
            file.read_to_end(&mut compressed_data)?;

            let mut decompressor = XzDecoder::new(&mut data_from_files);
            decompressor.write_all(&compressed_data)?;
        }

        assert_ne!(data_from_files, []);
        assert_eq!(original_data[data_size_bytes / 2..], data_from_files);
        cleanup("test_compressed", "xz")?;
        Ok(())
    }

    #[test]
    fn split_10_mb_compressed_gz() -> Result<(), Box<dyn std::error::Error>> {
        let range = Uniform::new(0, 255);
        let data_size_bytes: usize = 10 * 1024 * 1024; // 10 MB

        let data: Vec<u8> = rand::thread_rng()
            .sample_iter(&range)
            .take(data_size_bytes)
            .collect();

        let original_data = data.clone();
        assert_ne!(original_data, []);

        let mut cmd = Command::cargo_bin("stdin2file")?;

        cmd.timeout(std::time::Duration::from_secs(10))
            .arg("-c")
            .arg("1")
            .arg("-o")
            .arg("test_compressed")
            .arg("-m")
            .arg("5")
            .arg("-s")
            .arg("gz")
            .write_stdin(data)
            .ok()?;

        let mut data_from_files: Vec<u8> = vec![];

        for i in 6..=10 {
            let mut file =
                std::fs::File::open("test_compressed.".to_string() + &i.to_string() + ".gz")?;
            let mut compressed_data: Vec<u8> = vec![];
            file.read_to_end(&mut compressed_data)?;

            let mut decompressor = GzDecoder::new(&mut data_from_files);
            decompressor.write_all(&compressed_data)?;
        }

        assert_ne!(data_from_files, []);
        assert_eq!(original_data[data_size_bytes / 2..], data_from_files);
        cleanup("test_compressed", "gz")?;
        Ok(())
    }

    fn cleanup(
        file_base_name: &str,
        compression_suffix: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir("./")? {
            let entry = entry?;
            let file_name = entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            if file_name.contains(file_base_name) && file_name.contains(compression_suffix) {
                eprintln!("removing {:?}", entry.path());
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}
