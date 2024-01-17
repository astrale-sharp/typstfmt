#![doc = include_str!("../README.md")]
#![warn(clippy::dbg_macro)]

use std::{
    ffi::OsString,
    fs::File,
    io::{stdin, stdout, Read, Write},
};

use lexopt::prelude::*;
use typstfmt::{format, Config};

const VERSION: &str = env!("TYPSTFMT_VERSION");
// `DOT_CONFIG_FILE_NAME` is not created as a const due to the fact that we
// would have to duplicate the whole string slice, because
// `format!(".{CONFIG_FILE_NAME}")` (non-const function) cannot be applied to
// `const` (or `static`) values in Rust (1.72.1).
const CONFIG_FILE_NAME: &str = "typstfmt.toml";
/// Note: used in [`confy`](https://crates.io/crates/confy) functions.
const APP_NAME: &str = "typstfmt";
const HELP: &str = r#"Format Typst code

usage: typstfmt [options] [file...]

If no file is specified, stdin will be used.
Files will be overwritten unless --output is passed.

Options:
        -o, --output                If not specified, files will be overwritten. '-' for stdout.
        --stdout                    Same as `--output -` (Deprecated, here for compatibility).
        --check                     Run in 'check' mode. Exits with 0 if input is
                                    formatted correctly. Exits with 1 if formatting is required.
        --verbose                   increase verbosity for non errors
        -v, --version               Prints the current version.
        -h, --help                  Prints this help.
        --get-global-config-path    Prints the path of the global configuration file.
        -C, --make-default-config   Create a default config file at typstfmt.toml
"#;

enum Inputs {
    Stdin,
    Files(Vec<OsString>),
}

struct Input {
    name: String,
    content: String,
}

impl Inputs {
    fn read(&self) -> Box<dyn Iterator<Item = Input> + '_> {
        match self {
            Inputs::Stdin => {
                let mut input_buf = String::new();
                stdin()
                    .read_to_string(&mut input_buf)
                    .expect("Couldn't read stdin.");
                Box::new(std::iter::once(Input {
                    name: "stdin".to_owned(),
                    content: input_buf,
                }))
            }
            Inputs::Files(paths) => Box::new(paths.iter().map(|path| {
                let mut input_buf = String::new();
                let mut file = File::options()
                    .read(true)
                    .open(path)
                    .unwrap_or_else(|err| panic!("Failed to open file {path:?}: {err}"));
                file.read_to_string(&mut input_buf)
                    .unwrap_or_else(|err| panic!("Couldn't read file {path:?}: {err}"));
                Input {
                    name: path.to_string_lossy().into_owned(),
                    content: input_buf,
                }
            })),
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
    fn write(&self, input: &Input, formatted: &str, verbose: bool) -> Result<(), ()> {
        match self {
            Output::None => {
                // this is not stdout by the check after parsing the arguments that sets the output
                // to stdout rather than none for stdin.
                let path = &input.name;
                if formatted == input.content {
                    println!("file: {path:?} up to date.");
                    return Ok(());
                }
                let mut file = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .unwrap_or_else(|err| panic!("Couldn't open file: {path:?}: {err}"));
                file.write_all(formatted.as_bytes())
                    .unwrap_or_else(|err| panic!("Failed to write to file {path:?}: {err}"));
                if verbose {
                    println!("file: {path:?} overwritten.");
                };
            }
            Output::Check => {
                if input.content != formatted {
                    if verbose {
                        println!("{} needs formatting.", input.name);
                    }
                    return Err(());
                }
                if verbose {
                    println!("{} is already formatted.", input.name);
                }
            }
            Output::Stdout => {
                if verbose {
                    println!("=== {:?} ===", input.name);
                };
                stdout()
                    .write_all(formatted.as_bytes())
                    .unwrap_or_else(|err| {
                        panic!("Couldn't write to stdout: {}", err);
                    });
            }
            Output::File(output) => {
                let mut file = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output.to_string_lossy().into_owned())
                    .unwrap_or_else(|err| panic!("Couldn't create output file: {output:?}: {err}"));

                file.write_all(formatted.as_bytes())
                    .unwrap_or_else(|err| panic!("Couldn't write to file: {output:?}: {err}"));
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut inputs = Inputs::Stdin;
    let mut output = Output::None;
    let mut verbose = false;
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
            Long("get-global-config-path") => {
                let config_path = confy::get_configuration_file_path(APP_NAME, APP_NAME)
                    .unwrap_or_else(|e| panic!("Error loading global configuration file: {e}"));
                println!("{}", config_path.display());
                return Ok(());
            }
            Long("make-default-config") | Short('C') => {
                let s = Config::default_toml();
                let mut f = File::options()
                    .create_new(true)
                    .write(true)
                    .open(CONFIG_FILE_NAME)
                    .unwrap_or_else(|e| {
                        panic!("Couldn't create a new config file at {CONFIG_FILE_NAME}.\nCaused by {e}")
                    });
                f.write_all(s.as_bytes()).unwrap_or_else(|err| {
                    panic!("Failed to write to file {CONFIG_FILE_NAME:?}: {err}")
                });
                println!("Created config file at: {CONFIG_FILE_NAME}");
                return Ok(());
            }
            Value(v) => {
                inputs = match inputs {
                    Inputs::Stdin => Inputs::Files(vec![v]),
                    Inputs::Files(mut files) => {
                        files.push(v);
                        Inputs::Files(files)
                    }
                };
            }
            Long("stdout") => {
                output = Output::Stdout;
            }
            Long("output") | Short('o') => {
                let value = parser.value()?;
                output = if value == "-" {
                    Output::Stdout
                } else {
                    Output::File(value)
                };
            }
            Long("verbose") => {
                verbose = true;
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

    if matches!(inputs, Inputs::Stdin) && matches!(output, Output::None) {
        output = Output::Stdout;
    }

    let config = {
        let open_config = |file_name| File::options().read(true).open(file_name);
        let config = open_config(CONFIG_FILE_NAME);
        let dot_config_file_name = format!(".{CONFIG_FILE_NAME}");
        let dot_config = open_config(&dot_config_file_name);
        let is_config_ok = config.is_ok();
        if is_config_ok && dot_config.is_ok() {
            eprintln!(
                "Warning! Both {first:?} and {second:?} are present. Using {first:?}.",
                first = CONFIG_FILE_NAME,
                second = dot_config_file_name
            );
        }
        if let Ok(mut f) = config.or(dot_config) {
            let mut buf = String::default();
            let used_config_file_name = if is_config_ok {
                CONFIG_FILE_NAME
            } else {
                &dot_config_file_name
            };
            f.read_to_string(&mut buf).unwrap_or_else(|err| {
                panic!("Failed to read config file {used_config_file_name:?}: {err}");
            });
            Config::from_toml(&buf).unwrap_or_else(|err| {
                panic!(
                    "Config file {used_config_file_name:?} is invalid: {err}.\n{}",
                    "You'll maybe have to delete it and use -C to create a default config file."
                )
            })
        } else {
            let config_path = confy::get_configuration_file_path(APP_NAME, APP_NAME)
                .unwrap_or_else(|e| panic!("Error loading global configuration file: {e}"));
            confy::load(APP_NAME, APP_NAME).unwrap_or_else(|e| {
                panic!(
                    "Error loading global configuration file at {}: {e}",
                    config_path.display()
                )
            })
        }
    };

    let mut exit_status = 0;

    match &inputs {
        Inputs::Stdin => {}
        Inputs::Files(paths) => {
            assert!(
                !(matches!(output, Output::File(_)) && paths.len() > 1),
                "You specified multiple inputs and --output but one output file cannot receive the result of many files.\nAborting."
    );
        }
    }

    for input in inputs.read() {
        let formatted = format(&input.content, config);

        match output.write(&input, &formatted, verbose) {
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
