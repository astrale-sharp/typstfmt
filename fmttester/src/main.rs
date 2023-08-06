use std::{
    fs::File,
    io::{stdin, stdout, Read, Write},
    os::fd::AsFd,
    path::Path,
    process::Command,
    thread,
};

use lexopt::prelude::*;
use typstfmt_lib::Config;

const VERSION: &str = "0.0.1";
const HELP: &str = r#"Test Typst formatting

usage: fmtypsttest [options] <file>...

This program will check if your formatter changed the output or failed the compilation
of the files provided. It will then give you the range that caused the problem.

Options:
        -v, --version   Prints the current version.
        -h, --help      Prints this help.
"#;

fn main() -> Result<(), lexopt::Error> {
    let mut parser = lexopt::Parser::from_env();
    let mut paths = vec![];
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
            }
            _ => {
                println!("{}", arg.unexpected());
                println!("use -h or --help");
                return Ok(());
            }
        }
    }
    let mut handlers = vec![];
    // in parallel, try compiling the files,
    for path in paths {
        // if is file
        let Ok(file) = File::options().read(true).open(&path) else {
            println!("[IGNORED] doesn't link to a file:{path:?}");
            continue;
        };

        let handle = thread::spawn(move || deal_with_file(file, path));

        handlers.push(handle)
    }

    Ok(())
}

struct Diagnostic {
    file_name: std::ffi::OsString,
    verdict: Verdict,
}

enum Verdict {
    Ignore(String),
    Good,
    BadResult(std::ops::Range<usize>),
    BadCompile(std::ops::Range<usize>),
}

/// compiles a file with typst, formats a copy and compiles it.
/// If it doesn't compile or outputs a different pdf, errors.
fn deal_with_file(mut initial_file: File, path: &Path) -> Diagnostic {
    // call typst c on the file, ignore if
    let mut initial_compilation = tempfile::NamedTempFile::new()
        .unwrap_or_else(|_| panic!("couldn't create a tempfile while dealing with {path:?}"));
    let mut formatted_compilation = tempfile::NamedTempFile::new()
        .unwrap_or_else(|_| panic!("couldn't create a tempfile while dealing with {path:?}"));

    let mut command = compile(&path, &initial_compilation);

    if command.status().is_err() {
        return Diagnostic {
            file_name: path.into(),
            verdict: Verdict::Ignore("Original file cannot be compiled".to_string()),
        };
    }
    let mut formatted_file = tempfile::NamedTempFile::new()
        .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));
    let mut initial_file_text = String::new();
    let Ok(_) = initial_file.read_to_string(&mut initial_file_text) else {return Diagnostic{ file_name: path.into(), verdict: Verdict::Ignore("Original file coudn't be read".to_string()) };};

    let formatted_file_text = typstfmt_lib::format(&initial_file_text, Config::default());
    formatted_file
        .write_all(formatted_file_text.as_bytes())
        .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));

    let mut original_content = vec![];
    initial_compilation
        .read_to_end(&mut original_content)
        .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));

    let mut command = compile(formatted_file.path(), &formatted_compilation);


    let mut status_is_err = true;

    let mut verdict = None;
    let diff = similar::TextDiff::from_lines(&initial_file_text, &formatted_file_text);

    let old_slices = diff.old_slices().into_iter().collect::<Vec<_>>();
    let new_slices = diff.new_slices().into_iter().collect::<Vec<_>>();
    assert!(old_slices.len() == new_slices.len());
    let mut min_diff = 0;
    let mut max_diff = old_slices.len();

    while status_is_err {
        // apply changes in range to get formatted text
        // compile formatted version.
        // compare text
        // adjust range
        // no change? break
        let mut current_text = old_slices.clone();
        for idx in min_diff..(max_diff + min_diff) / 2 {
            current_text[idx] = new_slices[idx]
        }
        let current_text = {
            let mut buf = String::new();
            for s in current_text {
                buf.push_str(s);
            }
            buf
        };

        let mut command = Command::new("typst");
        let command = command
            .arg("compile")
            .arg(formatted_file.path())
            .arg(formatted_compilation.path());

        // compare outputs
        let mut new_content = vec![];
        formatted_compilation
            .read_to_end(&mut new_content)
            .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));

        match (command.status().is_err(), new_content != original_content) {
            (true, _) if verdict.is_none() => {
                verdict = Some(Verdict::BadCompile(0..0));
                status_is_err = true
            }
            (_, true) if verdict.is_none() => {
                verdict = Some(Verdict::BadResult(0..0));
                status_is_err = true
            }
            (true, _) | (_, true) => {
                status_is_err = true;
            }
            _ => status_is_err = false,
        }
        if status_is_err {
            todo!("modify range + update formatted file, reverting change except on the specified range.")
        }
    }

    todo!()
}

fn compile<'a>(
    path: &'a Path,
    result: &'a tempfile::NamedTempFile,
) -> &'a mut Command {
    let mut command = Command::new("typst");
    let command = command.arg("compile").arg(path).arg(result.path());
    command
}

#[test]
fn feature() {
    let old = "a1b1c1d1e1f1g\na1b1c1\nabc".to_string();
    let new = "abcdefg\nabc\nabc".to_string();
    let diff = similar::TextDiff::from_lines(&old, &new);
    dbg!(diff.old_slices());
    dbg!(diff.new_slices());
    let changes = diff.iter_all_changes();

    for change in changes {
        // println!("{:?}",change.as_str());
        println!("{:?}", change);
        // println!("{:?}-{:?}",change.old_index(),change.new_index());
    }
}
