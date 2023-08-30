use std::{
    ffi::OsStr,
    fs::File,
    io::{stdin, stdout, Read, Write},
    os::fd::AsFd,
    path::Path,
    process::{Command, Stdio},
    thread,
};

use lexopt::prelude::*;
use tempfile::NamedTempFile;
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

    if paths.is_empty() {
        println!("No path provided.");
        println!("{HELP}");
    }

    let mut diags = vec![];
    // in parallel, try compiling the files,
    for path in paths {
        // if is file
        let Ok(mut file) = File::options().read(true).open(&path) else {
            println!("[IGNORED] doesn't link to a file:{path:?}");
            continue;
        };
        println!("register: {path:?}");
        let handle = thread::spawn(move || deal_with_file(&mut file, path.to_str().unwrap()));
        diags.push(handle);
    }
    let mut diags = diags
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();

    diags.sort_by(|x, y| x.file_name.cmp(&y.file_name));
    for d in diags {
        let Diagnostic {
            file_name,
            verdict,
            diffs,
        } = d;
        match verdict {
            Verdict::Good => println!("{file_name:?} ... OK"),
            Verdict::Ignore(msg) => println!("{file_name:?} ... IGNORED : {msg}"),
            Verdict::BadResult(_) | Verdict::BadCompile(_) => {
                let reason = if let Verdict::BadResult(_) = verdict {
                    "Compilation result is different"
                } else {
                    "Compilation error."
                };
                let diffs = diffs.unwrap();
                println!("{file_name:?} ... ERROR: {reason}\n{diffs}\n");
            }
        }
    }

    Ok(())
}

struct Diagnostic {
    file_name: std::ffi::OsString,
    verdict: Verdict,
    diffs: Option<String>,
}

#[derive(Debug)]
enum Verdict {
    Ignore(String),
    Good,
    BadResult(std::ops::Range<usize>),
    BadCompile(std::ops::Range<usize>),
}

/// compiles a file with typst, formats a copy and compiles it.
/// If it doesn't compile or outputs a different pdf, errors.
fn deal_with_file(initial_file: &mut File, path: &str) -> Diagnostic {
    // call typst c on the file, ignore if
    let mut initial_compilation = NamedTempFile::new()
        .unwrap_or_else(|_| panic!("couldn't create a tempfile while dealing with {path:?}"));
    let mut formatted_compilation = NamedTempFile::new()
        .unwrap_or_else(|_| panic!("couldn't create a tempfile while dealing with {path:?}"));

    let mut command = compile(path, initial_compilation.path());

    if command.output().is_err()
        || command.output().is_ok_and(|o| {
            String::from_utf8(o.stderr.to_vec())
                .unwrap()
                .contains("error")
        })
    {
        return Diagnostic {
            file_name: path.into(),
            verdict: Verdict::Ignore("Original file cannot be compiled".to_string()),
            diffs: None,
        };
    }
    println!("{}", command.status().unwrap());
    let mut formatted_file = NamedTempFile::new()
        .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));
    let mut initial_file_text = String::new();
    let Ok(_) = initial_file.read_to_string(&mut initial_file_text) else {
        return Diagnostic {
            file_name: path.into(),
            verdict: Verdict::Ignore("Original file coudn't be read".to_string()),
            diffs: None,
        };
    };

    let formatted_file_text = typstfmt_lib::format(&initial_file_text, Config::default());
    formatted_file
        .write_all(formatted_file_text.as_bytes())
        .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));

    let mut original_content = vec![];
    initial_compilation
        .read_to_end(&mut original_content)
        .unwrap_or_else(|_| panic!("couldn't read tempfile while dealing with {path:?}"));

    let mut command = compile(formatted_file.path(), formatted_compilation.path());

    let mut verdict = None;
    let diff = similar::TextDiff::from_lines(&initial_file_text, &formatted_file_text);
    let mut old_slices = vec![];
    let mut new_slices = vec![];
    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Equal => {
                old_slices.push(change.value());
                new_slices.push(change.value());
            }
            similar::ChangeTag::Delete => {
                old_slices.push(change.value());
                new_slices.push("");
            }
            similar::ChangeTag::Insert => {
                new_slices.push(change.value());
                old_slices.push("");
            }
        }
    }

    assert!(old_slices.len() == new_slices.len(),);

    let mut min_diff = 0;
    let mut max_diff = old_slices.len();
    let mut range = min_diff..max_diff;

    if range.is_empty() {
        panic!("range was empty");
    }

    println!("loop: {}", command.status().unwrap());

    if command.output().is_err()
        || command.output().is_ok_and(|o| {
            String::from_utf8(o.stderr.to_vec())
                .unwrap()
                .contains("error")
        })
    {
        verdict = Some(Verdict::BadCompile(range.clone()))
    }

    if !compare_eq(&mut formatted_compilation, &mut initial_compilation) {
        verdict = Some(Verdict::BadResult(range.clone()))
    }

    if verdict.is_none() {
        return Diagnostic {
            file_name: path.into(),
            verdict: Verdict::Good,
            diffs: None,
        };
    }
    let mut range_size = range.len();

    // apply changes half the change
    // try to find the problematic range
    loop {
        let mut current_text = old_slices.clone();
        current_text[min_diff..((max_diff + min_diff) / 2)]
            .clone_from_slice(&new_slices[min_diff..((max_diff + min_diff) / 2)]);

        let current_text = {
            let mut buf = String::new();
            for s in current_text {
                buf.push_str(s);
            }
            buf
        };

        formatted_file.write_all(current_text.as_bytes()).unwrap();

        let mut command = compile(formatted_file.path(), formatted_compilation.path());

        if compare_eq(&mut formatted_compilation, &mut initial_compilation)
            && !(command.output().is_err()
                || command.output().is_ok_and(|o| {
                    String::from_utf8(o.stderr.to_vec())
                        .unwrap()
                        .contains("error")
                }))
        {
            min_diff = (max_diff + min_diff) / 2
        } else {
            max_diff = (max_diff + min_diff) / 2
        }
        range = min_diff..max_diff;

        if range_size == range.len() {
            // len didn't change
            break;
        }

        range_size = range.len()
    }
    let verdict = match verdict {
        Some(Verdict::BadResult(_)) => Verdict::BadResult(range.clone()),
        Some(Verdict::BadCompile(_)) => Verdict::BadCompile(range.clone()),
        _ => unreachable!(),
    };
    let mut diffs = String::new();

    let mut current_text = old_slices.clone();
    current_text[range.clone()].clone_from_slice(&new_slices[range.clone()]);
    let line = &current_text[0..range.start]
        .concat()
        .to_string()
        .chars()
        .filter(|c| c == &'\n')
        .count();

    diffs.push_str(&format!("at line {line}:\n"));
    diffs.push_str("- ");
    let mut i = range.start;
    let mut count = 0;
    while count < range.len() + 5 {
        if count + i > old_slices.len() {
            break;
        }
        let s = old_slices[i];
        if s.is_empty() {
            i += 1
        } else {
            count += 1;
            diffs.push_str(s)
        }
    }
    diffs.push('\n');
    diffs.push_str("+ ");
    let mut i = range.start;

    let mut count = 0;
    while count < range.len() + 5 {
        if count + i > new_slices.len() as _ {
            break;
        }
        // if  count + i  < 0 {
        //     count += 1;
        //     continue;
        // }
        let s = new_slices[i];
        if s.is_empty() {
            i += 1
        } else {
            count += 1;
            diffs.push_str(s)
        }
    }

    Diagnostic {
        file_name: path.into(),
        verdict,
        diffs: Some(diffs),
    }
}

fn compile<S: AsRef<OsStr>, T: AsRef<OsStr>>(file_path: S, result_path: T) -> Command {
    let mut command = Command::new("typst");
    command
        .arg("compile")
        .arg(file_path)
        .arg(result_path)
        .args(["-f", "pdf"])
        .stderr(Stdio::piped())
        .stdout(Stdio::null());
    command
}

fn compare_eq(file1: &mut NamedTempFile, file2: &mut NamedTempFile) -> bool {
    let mut buf1 = String::new();
    let mut buf2 = String::new();
    file1.read_to_string(&mut buf1).unwrap();
    file2.read_to_string(&mut buf2).unwrap();
    buf1 == buf2
}

// #[test]
// fn feature() {
//     let diag = {

//         let mut initial_compilation = 0;
//         let mut formatted_compilation = 0;
//         let mut command = true;

//         let mut formatted_file = "";
//         let mut initial_file_text = "#let my_var = 4
//         #1
//         #2
//         #3
//         ";

//         let formatted_file_text = "#let my_var = 4
//         #1
//         #5
//         #3
//         ";

//         let mut command = false;

//         let mut verdict = None;
//         let diff = similar::TextDiff::from_lines(initial_file_text, formatted_file_text);
//         let mut old_slices = vec![];
//         let mut new_slices = vec![];
//         for change in diff.iter_all_changes() {
//             match change.tag() {
//                 similar::ChangeTag::Equal => {
//                     old_slices.push(change.value());
//                     new_slices.push(change.value());
//                 }
//                 similar::ChangeTag::Delete => {
//                     old_slices.push(change.value());
//                     new_slices.push("");
//                 }
//                 similar::ChangeTag::Insert => {
//                     new_slices.push(change.value());
//                     old_slices.push("");
//                 }
//             }
//         }

//         assert!(old_slices.len() == new_slices.len(),);
//         let mut min_diff = 0;
//         let mut max_diff = old_slices.len();
//         let mut range = min_diff..max_diff;

//         if range.is_empty() {
//             panic!("range was empty");
//         }

//         if true {
//             verdict = Some(Verdict::BadResult(range.clone()))
//         }

//         let mut range_size = range.len();

//         // apply changes half the change
//         // try to find the problematic range
//         loop {
//             let mut current_text = old_slices.clone();
//             current_text[min_diff..((max_diff + min_diff) / 2)]
//                 .clone_from_slice(&new_slices[min_diff..((max_diff + min_diff) / 2)]);

//             let current_text = {
//                 let mut buf = String::new();
//                 for s in current_text {
//                     buf.push_str(&s);
//                 }
//                 buf
//             };

//             formatted_file = &current_text;

//             let mut command = true;

//             if formatted_file == initial_file_text
//             {
//                 min_diff = (max_diff + min_diff) / 2
//             } else {
//                 max_diff = (max_diff + min_diff) / 2
//             }
//             range = min_diff..max_diff;

//             if range_size == range.len() {
//                 // len didn't change
//                 break;
//             }

//             range_size = range.len()
//         }
//         let verdict = match verdict {
//             Some(Verdict::BadResult(_)) => Verdict::BadResult(range.clone()),
//             Some(Verdict::BadCompile(_)) => Verdict::BadCompile(range.clone()),
//             _ => unreachable!(),
//         };
//         let mut diffs = String::new();
//         diffs.push_str("- ");
//         for i in range.clone() {
//             diffs.push_str(old_slices[i])
//         }
//         diffs.push_str("+ ");
//         for i in range {
//             diffs.push_str(new_slices[i])
//         }

//         Diagnostic {
//             file_name: "PATH".into(),
//             verdict,
//             diffs: Some(diffs),
//         }

//     };

//     let mut diags: Vec<Diagnostic> = vec![diag];
//     diags.sort_by(|x, y| x.file_name.cmp(&y.file_name));
//     for d in diags {
//         let Diagnostic {
//             file_name,
//             verdict,
//             diffs,
//         } = d;
//         match verdict {
//             Verdict::Good => println!("{file_name:?} ... OK"),
//             Verdict::Ignore(msg) => println!("{file_name:?} ... IGNORED : {msg}"),
//             Verdict::BadResult(_) | Verdict::BadCompile(_) => {
//                 let reason = if let Verdict::BadResult(_) = verdict {
//                     "Compilation result is different"
//                 } else {
//                     "Compilation error."
//                 };
//                 let diffs = diffs.unwrap();
//                 println!("{file_name:?} ... ERROR: {reason}\n{diffs}\n");
//             }
//         }
//     }
// }
