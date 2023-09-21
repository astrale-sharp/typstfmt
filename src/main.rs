#![doc = include_str!("../README.md")]

use std::{
    ffi::OsString,
    fs::File,
    io::{stdin, stdout, Read, Write},
};

use lexopt::prelude::*;
use typstfmt_lib::{format, Config};

const VERSION: &str = env!("TYPSTFMT_VERSION");
const CONFIG_PATH: &str = "typstfmt-config.toml";
const HELP: &str = r#"Format Typst code

usage: typstfmt [options] <file>...

If no file is specified, stdin will be used.
Files will be overwritten except in -o or --stdout is passed.

Options:
        -o, --output    If not specified, files will be overwritten.
        --stdout        If specified, the formatted version of the files will
                        be printed to stdout.
        --check         Run in 'check' mode. Exits with 0 if input is
                        formatted correctly. Exits with 1 if formatting is required.
        -v, --version   Prints the current version.
        -h, --help      Prints this help.
        -C, --make-default-config   Create a default config file at typstfmt-config.toml
"#;

enum Output {
    None,
    Check,
    Stdout,
    File(OsString),
}

fn main() -> Result<(), lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut paths = vec![];
    let mut use_stdin = true;
    let mut output = Output::None;
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
            Long("make-default-config") | Short('C') => {
                let s = Config::default_toml();
                let mut f = File::options()
                    .create_new(true)
                    .write(true)
                    .open(CONFIG_PATH)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Couldn't create a new config file at {}.\nCaused by {}",
                            CONFIG_PATH, e
                        )
                    });
                f.write_all(s.as_bytes()).unwrap();
                println!("Created config file at: {CONFIG_PATH}");
                return Ok(());
            }
            Value(v) => {
                paths.push(v);
                use_stdin = false;
            }
            Long("output") | Short('o') => {
                output = Output::File(parser.value()?);
            }
            Long("stdout") => {
                output = Output::Stdout;
            }
            Long("check") => {
                output = Output::Check;
            }
            _ => {
                println!("{}", arg.unexpected());
                println!("use -h or --help");
                return Ok(());
            }
        }
    }

    let config = {
        if let Ok(mut f) = File::options().read(true).open(CONFIG_PATH) {
            let mut buf = String::default();
            f.read_to_string(&mut buf).unwrap();
            Config::from_toml(&buf).unwrap_or_else(|e| panic!("Config file invalid: {e}.\nYou'll maybe have to delete it and use -C to create a default config file."))
        } else {
            Config::default()
        }
    };

    if paths.is_empty() && !use_stdin {
        println!("You specified no files to format. If you want to use stdin pass --stdin");
        println!("{HELP}");
        return Ok(());
    }

    assert!(
        !(matches!(output, Output::File(_))),
        "Both output and stdout are set. You must choose only one.\nAborting."
    );

    let mut exit_status = 0;

    if use_stdin {
        let mut input_buf = String::default();
        stdin()
            .read_to_string(&mut input_buf)
            .expect("Couldn't read stdin.");
        let formatted = format(&input_buf, config);
        match output {
            Output::None => {}
            Output::Check =>
            {
                #[allow(unused_assignments)]
                if input_buf != formatted {
                    println!("input needs formatting.");
                    exit_status = 1;
                } else {
                    println!("input is already formatted.")
                }
            }
            Output::Stdout => {
                write!(stdout(), "{}", formatted)
                    .unwrap_or_else(|err| panic!("Couldn't write to stdout: {err}"));
            }
            Output::File(output) => {
                let mut file = File::options()
                    .write(true)
                    .truncate(true)
                    .open(output.to_str().unwrap())
                    .unwrap_or_else(|err| panic!("Couldn't write to output: {output:?}: {err}"));

                write!(file, "{}", formatted)
                    .unwrap_or_else(|err| panic!("Couldn't write to file: {output:?}: {err}"));
            }
        }
        return Ok(());
    }

    assert!(
        !(matches!(output, Output::File(_)) && paths.len() > 1),
        "You specified multiple input files and --output but one output file cannot receive the result of many files.\nAborting."
    );

    for path in &paths {
        let mut input_buf = String::new();
        let mut file = File::options().read(true).open(path).unwrap();
        file.read_to_string(&mut input_buf)
            .expect("Couldn't read stdin");
        let formatted = format(&input_buf, config);
        drop(file);

        match &output {
            Output::None => {
                let mut file = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path.to_str().unwrap())
                    .unwrap_or_else(|err| panic!("Couldn't write to file: {path:?}: {err}"));
                file.write_all(formatted.as_bytes()).unwrap();
                println!("file: {path:?} overwritten.");
            }
            Output::Check => {
                if input_buf != formatted {
                    println!("{path:?} needs formatting.");
                    exit_status = 1;
                } else {
                    println!("{path:?} is already formatted.")
                }
            }
            Output::Stdout => {
                println!("=== {:?} ===", path);
                stdout()
                    .write_all(formatted.as_bytes())
                    .expect("Couldn't write to stdout");
            }
            Output::File(output) => {
                let mut file = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output)
                    .unwrap();
                file.write_all(formatted.as_bytes()).unwrap();
                println!("file: {output:?} overwritten.");
            }
        }
    }
    if exit_status == 0 {
        Ok(())
    } else {
        std::process::exit(exit_status);
    }
}
