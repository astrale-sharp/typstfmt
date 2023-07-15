use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
};

use lexopt::prelude::*;
use typstfmt::{format, Config};

const VERSION: &str = "0.0.1";
const HELP: &str = r#"Format Typst code

usage: typstfmt [options] <file>...

If no file is specified, stdin will be used.
Files will be overwritten except in -o or --stdout is passed.

Options:
        -o, --output    If not specified, files will be overwritten.
        --stdout        If specified, the formatted version of the files will
                        be printed to stdout. 
        -v, --version   Prints the current version.
        -h, --help      Prints this help.
"#;

fn main() -> Result<(), lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut paths = vec![];
    let mut use_stdin = true;
    let mut use_stdout = false;
    let mut output = None;
    let config = Config::default();
    while let Some(arg) = parser.next()? {
        match arg {
            Long("version") | Short('v') => {
                println!("version: {VERSION}");
                return Ok(());
            }
            Long("help") | Short('h') => {
                println!("{HELP}");
                return Ok(());
            }
            Value(v) => {
                paths.push(v);
                use_stdin = false;
            }
            Long("output") | Short('o') => {
                output = Some(parser.value()?);
            }
            Long("stdout") => {
                use_stdout = true;
            }
            _ => {
                println!("{}", arg.unexpected());
                println!("use -h or --help");
                return Ok(());
            }
        }
    }
    if paths.is_empty() && !use_stdin {
        println!("You specified no files to format. If you want to use stdin pass --stdin");
        println!("{HELP}");
        return Ok(());
    }

    if output.is_some() && use_stdout {
        panic!("Both output and stdout are set. You must choose only one.\nAborting.")
    }

    if use_stdin {
        let mut res = String::default();
        stdin()
            .read_to_string(&mut res)
            .expect("Couldn't read stdin.");
        let formatted = &format(&res, config);
        if let Some(output) = output {
            let mut file = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output.to_str().unwrap())
                .unwrap();
            write!(file, "{}", formatted)
                .unwrap_or_else(|err| panic!("Couldn't write to file: {output:?}: {err}"));
        } else {
            write!(stdout(), "{}", formatted)
                .unwrap_or_else(|err| panic!("Couldn't write to stdout: {err}"));
        }
        return Ok(());
    }

    for path in paths {
        let mut res = String::new();
        let mut file = File::options().read(true).open(&path).unwrap();
        file.read_to_string(&mut res).expect("Couldn't read stdin");
        let res = format(&res, config);
        drop(file);

        if use_stdout {
            println!("=== {:?} ===", &path);
            stdout()
                .write_all(res.as_bytes())
                .expect("Couldn't write to stdout");
        } else {
            let mut file = File::options()
                .write(true)
                .truncate(true)
                .open(path.to_str().unwrap())
                .unwrap_or_else(|err| panic!("Couldn't write to file: {path:?}: {err}"));
            file.write_all(res.as_bytes()).unwrap();
            println!("file: {path:?} overwritten.");
        }
    }

    Ok(())
}
