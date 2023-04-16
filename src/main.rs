use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use typst_fmt::typst_format;

use clap::Parser;
use clap::ValueEnum;

#[derive(Parser, Debug)]
#[command(version = "0.0.1", about="A formatter for the typst language", long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        value_enum, default_value_t = Mode::Format
    )]
    mode: Mode,

    /// input file
    #[arg(help = "A file to format. If not specified, all .typ file will be formatted")]
    input: Option<PathBuf>,

    #[arg(
        short,
        long,
        help = "If specified, the result of output will be put in this file. input *must* be specified if you set output."
    )]
    output: Option<PathBuf>,
    //watch : bool
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    /// formats the file in place
    Format,
    /// puts the formatted result in a __simulate__*.typ next to your inputs.
    Simulate,
}

fn main() {
    let args = Args::parse();
    if args.output.is_some() && args.input.is_none() {
        panic!("Input must be specified to use an output.")
    }
    let paths = if let Some(input) = args.input {
        vec![input]
    } else {
        let glob = globmatch::Builder::new("**.typ").build(".").unwrap();
        glob.into_iter().flat_map(|path| path.ok()).collect()
    };
    for path in paths.into_iter() {
        let mut file = File::options()
            .read(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("Couldn't open input file : {e}"));
        let mut content = String::with_capacity(1024);
        file.read_to_string(&mut content).unwrap();
        drop(file);
        let mut file = File::options()
            .write(true)
            .append(false)
            .truncate(true)
            .open(&path)
            .unwrap();
        file.set_len(0).unwrap();
        let formatted = typst_format(&content);
        if let Some(output) = args.output {
            let mut file =
                File::open(output).unwrap_or_else(|op| panic!("Couldn't open output file: {op}"));
            file.write(formatted.as_bytes())
                .unwrap_or_else(|op| panic!("Couldn't write in the output: {op}"));
            break;
        }
        match args.mode {
            Mode::Format => {
                file.write(formatted.as_bytes())
                    .unwrap_or_else(|op| panic!("Couldn't write in file at {path:?}: {op}"));
            }
            Mode::Simulate => {
                let spath = path
                    .parent()
                    .unwrap_or(&PathBuf::default())
                    .join(path.file_stem().unwrap())
                    .join(&PathBuf::from("__simulate__.typ"));
                let mut file = File::create(&spath)
                    .unwrap_or_else(|e| panic!("Couldn't open input file : {e}"));
                file.write(formatted.as_bytes())
                    .unwrap_or_else(|op| panic!("Couldn't write in file at {path:?}: {op}"));
            }
        }
    }
}
