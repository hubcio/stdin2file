use clap::{App, AppSettings, Arg};
pub struct Args {
    pub chunk: usize,
    pub output_file: String,
    pub compression_mode: String,
    pub max_files: usize,
}

impl Args {
    pub fn new() -> Self {
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
            .about("Compression mode (currently only 'xz' is supported)")
            .required(false);

        let max_files_option = Arg::new("max-files")
            .long("max-files")
            .short('m')
            .takes_value(true)
            .about("Number of rotated files")
            .required(false);

        let app = app
            .arg(chunk_option)
            .arg(output_option)
            .arg(compress_option)
            .arg(max_files_option);

        let matches = app.get_matches();

        let chunk = matches
            .value_of("chunk")
            .expect("This can't be None, we said it was required")
            .parse::<usize>()
            .expect("Failed to parse chunk number as usize");

        let output_file = matches.value_of("output").unwrap_or("");

        let compression = match matches.value_of("compress") {
            Some("xz") => "xz",
            // Some(_) => std::panic!("unsuported compression value"),
            Some(_) => {
                App::new("app").setting(AppSettings::ArgRequiredElseHelp);
                std::process::exit(-1);
            }
            None => "",
        };

        let max_files = match matches.value_of("max-files") {
            Some(n) => n
                .parse::<usize>()
                .expect("Failed to parse max-files as usize"),
            None => usize::MAX,
        };

        Args {
            chunk,
            output_file: String::from(output_file),
            compression_mode: String::from(compression),
            max_files,
        }
    }
}
