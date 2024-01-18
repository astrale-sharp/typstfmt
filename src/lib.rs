#![doc = include_str!("../README.md")]
// #![allow(unused)]
// #![allow(warnings)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::dbg_macro
)]

mod config;
/// Our format tree, that we will visit in order to format the code.
mod node;
/// Special operation applied to our tree to take care of giving commands via
/// Typst's comments.
mod preserve_pass;
mod utils;
/// Here lies the formatting logic
mod visits;
/// Handles writing to an output, indentation as a post_process,
/// rewinding if things some condition was not respected.
mod writer;

pub use config::Config;
use node::map_tree;
use preserve_pass::preserve_pass;
use visits::*;
use writer::Writer;
// mod tests;

use typst_syntax::{parse, LinkedNode};

/// we visit our [FmtNode] tree, using the [Writer] to specify how we want
/// our formatting to be done.
///
/// Then we apply indentation as post processing.
#[must_use]
pub fn format(s: &str, config: Config) -> String {
    let init = parse(s);
    let mut s = String::new();
    let mut writer = Writer::new(config, &mut s);
    let root = LinkedNode::new(&init);
    let mut root = map_tree(root, None);
    let _ = preserve_pass(&mut root);
    visit_markup(&root, &mut writer, false);
    writer.post_process_indents();
    regex::Regex::new("( )+\n")
        .unwrap()
        .replace_all(&s, "\n")
        .to_string()
}

/* very basic tests */

#[test]
fn test_indent() {
    let mut snippet = r#"
#[
text #[
text
]
]"#
    .to_string();

    let expected = r#"
#[
  text #[
    text
  ]
]"#;

    let mi1 = "\n#[".len();
    let mi2 = "\n#[\ntext #[".len();
    let md2 = "\n#[\ntext #[\ntext\n".len();
    let md1 = "\n#[\ntext #[\ntext\n]\n".len();

    let mut w = Writer::new(Config::default(), &mut snippet);
    w.marks = vec![
        MarkKind::Indent.to_mark(mi1),
        MarkKind::Indent.to_mark(mi2),
        MarkKind::Dedent.to_mark(md2),
        MarkKind::Dedent.to_mark(md1),
    ];
    w.post_process_indents();
    println!("snippet:");
    println!("{snippet}");
    println!("snippet:?");
    println!("{snippet:?}");

    assert!(snippet == expected);
}

#[cfg(test)]
use crate::writer::MarkKind;

#[test]
fn test_preserve() {
    let mut snippet = r#"
#[
text #[
text
]
]"#
    .to_string();
    println!("init: {snippet}");

    let expected = r#"
#[
  text #[
text
  ]
]"#;

    let mi1 = "\n#[".len();
    let mi2 = "\n#[\ntext #[".len();
    let md2 = "\n#[\ntext #[\ntext\n".len();
    let md1 = "\n#[\ntext #[\ntext\n]\n".len();
    let preserve = "\n#[\ntext #[\n".len();
    let stop_preserve = "\n#[\ntext #[\ntext\n".len();

    let mut w = Writer::new(Config::default(), &mut snippet);
    w.marks = vec![
        MarkKind::Indent.to_mark(mi1),
        MarkKind::Indent.to_mark(mi2),
        MarkKind::Dedent.to_mark(md2),
        MarkKind::Dedent.to_mark(md1),
        MarkKind::Preserve.to_mark(preserve),
        MarkKind::StopPreserve.to_mark(stop_preserve),
    ];
    w.post_process_indents();
    println!("fmt : {snippet}");
    println!("expe: {expected}");

    assert!(snippet == expected);
}

#[test]
fn test_preserve_pass() {
    let snippets = [
        r#"notp // typstfmt::off
#f(x : [p]) 
// typstfmt::on
notp "#,
        r#"#f([notp], // typstfmt::off
)
// typstfmt::off
// typstfmt::on
p
// typstfmt::on
notp"#,
        r#"
// typstfmt::off
#[
// typstfmt::off
#[
// typstfmt::off
#[
// typstfmt::off
text
]
]
]
// typstfmt::on
// typstfmt::on
// typstfmt::on
text
// typstfmt::on
text
"#,
    ];

    for snippet in snippets {
        let parse = parse(snippet);
        let root = LinkedNode::new(&parse);
        println!("===");
        println!("{snippet}");
        println!("===");
        println!("{:?}", root);
        let mut root = map_tree(root, None);
        println!("===");
        println!("{:?}", root);
        println!("===");
        preserve_pass(&mut root);
        println!("{:?}", root);
        println!("===");
        insta::assert_debug_snapshot!(root)
    }
}
