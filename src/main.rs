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
        -o, --output    If not specified, files will be overwritten. '-' for stdout.
        --check         Run in 'check' mode. Exits with 0 if input is
                        formatted correctly. Exits with 1 if formatting is required.
        -v, --version   Prints the current version.
        -h, --help      Prints this help.
        -C, --make-default-config   Create a default config file at typstfmt-config.toml
"#;

enum Input {
    Stdin,
    File(OsString),
}

impl Input {
    fn read(&self) -> String {
        match self {
            Input::Stdin => {
                let mut input_buf = String::default();
                stdin()
                    .read_to_string(&mut input_buf)
                    .expect("Couldn't read stdin.");
                input_buf
            }
            Input::File(path) => {
                let mut input_buf = String::new();
                let mut file = File::options().read(true).open(path).unwrap();
                file.read_to_string(&mut input_buf)
                    .expect("Couldn't read stdin");
                input_buf
            }
        }
    }

    fn name(&self) -> String {
        match self {
            Input::Stdin => "input".to_owned(),
            Input::File(p) => p.to_string_lossy().into_owned(),
        }
    }
}

enum Output {
    None,
    Check,
    Stdout,
    File(OsString),
}

impl Output {
    fn write(&self, input: &Input, input_buf: &str, formatted: &str) -> Result<(), ()> {
        match self {
            Output::None => {
                if let Input::File(path) = input {
                    let mut file = File::options()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(path.to_str().unwrap())
                        .unwrap_or_else(|err| panic!("Couldn't write to file: {path:?}: {err}"));
                    file.write_all(formatted.as_bytes()).unwrap();
                    println!("file: {path:?} overwritten.");
                }
            }
            Output::Check => {
                if input_buf != formatted {
                    println!("{} needs formatting.", input.name());
                    return Err(());
                } else {
                    println!("{} is already formatted.", input.name());
                }
            }
            Output::Stdout => {
                if let Input::File(path) = input {
                    println!("=== {:?} ===", path);
                }
                stdout()
                    .write_all(formatted.as_bytes())
                    .unwrap_or_else(|err| panic!("Couldn't write to stdout: {err}"));
            }
            Output::File(output) => {
                let mut file = File::options()
                    .write(true)
                    .truncate(true)
                    .open(output.to_str().unwrap())
                    .unwrap_or_else(|err| panic!("Couldn't write to output: {output:?}: {err}"));

                file.write_all(formatted.as_bytes())
                    .unwrap_or_else(|err| panic!("Couldn't write to file: {output:?}: {err}"));
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut inputs = vec![];
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
                inputs.push(if v == "-" {
                    Input::Stdin
                } else {
                    Input::File(v)
                });
            }
            Long("output") | Short('o') => {
                let value = parser.value()?;
                output = if value == "-" {
                    Output::Stdout
                } else {
                    Output::File(value)
                };
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

    if inputs.is_empty() {
        println!("You specified no files to format.");
        println!("{HELP}");
        return Ok(());
    }

    let mut exit_status = 0;

    assert!(
        !(matches!(output, Output::File(_)) && inputs.len() > 1),
        "You specified multiple inputs and --output but one output file cannot receive the result of many files.\nAborting."
    );

    for input in &inputs {
        let input_buf = input.read();
        let formatted = format(&input_buf, config);

        match output.write(input, &input_buf, &formatted) {
            Ok(()) => {}
            Err(()) => {
                exit_status = 1;
            }
        }
    }
    if exit_status == 0 {
        Ok(())
    } else {
        std::process::exit(exit_status);
    }
}
