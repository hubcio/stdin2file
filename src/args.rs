use std::num::NonZeroUsize;

use anyhow::Context;
use clap::{App, Arg};
use log::debug;

#[derive(Debug)]
pub struct Args {
    pub chunk: NonZeroUsize,
    pub base_output_file: String,
    pub compression: Option<String>,
    pub max_files: NonZeroUsize,
    pub execute_command: Option<String>,
}

impl Args {
    pub fn new() -> Result<Args, anyhow::Error> {
        let app = App::new("stdin2file")
            .version("1.0")
            .author("hugruu <h.gruszecki@gmail.com>")
            .about("Write from stdin to file(s), optionally compresses it using given algorithm");

        let chunk_option = Arg::new("chunk")
            .long("chunk")
            .short('c')
            .takes_value(true)
            .about("Maximum size of single file size [MiB]")
            .required(true);

        let output_option = Arg::new("output")
            .long("output")
            .short('o')
            .takes_value(true)
            .about("Output file")
            .required(true);

        let compress_option = Arg::new("compress")
            .long("compress")
            .short('s')
            .takes_value(true)
            .about("Compression algorithm (supported: gz, xz)")
            .required(false)
            .possible_values(&["xz", "gz"]);

        let max_files_option = Arg::new("max-files")
            .long("max-files")
            .short('m')
            .takes_value(true)
            .about("Number of rotated files")
            .required(false);

        let execute_command_option = Arg::new("execute")
            .long("execute")
            .short('e')
            .takes_value(true)
            .about("Command to execute (instead of stdin)")
            .required(false);

        let app = app
            .arg(chunk_option)
            .arg(output_option)
            .arg(compress_option)
            .arg(max_files_option)
            .arg(execute_command_option);

        let matches = app.get_matches();

        let chunk = matches
            .value_of("chunk")
            .expect("This can't be None, we said it was required")
            .parse::<NonZeroUsize>()
            .expect("Failed to parse chunk number as usize");

        let output_file = matches
            .value_of("output")
            .with_context(|| format!("output_file is none"))?
            .to_string();

        let compression_mode = matches.value_of("compress").map(String::from);

        let max_files = match matches.value_of("max-files") {
            Some(n) => n
                .parse::<NonZeroUsize>()
                .expect("Failed to parse max-files as usize"),
            None => NonZeroUsize::new(usize::MAX).unwrap(),
        };

        let execute_command = matches.value_of("execute").map(String::from);

        let args = Args {
            chunk,
            base_output_file: output_file,
            compression: compression_mode,
            max_files,
            execute_command,
        };

        debug!("{:#?}", args);

        Ok(args)
    }
}
