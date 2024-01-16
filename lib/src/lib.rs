#![doc = include_str!("../README.md")]
#![allow(unused)]
#![allow(warnings)]
// #![warn(
//     clippy::all,
//     clippy::pedantic,
//     clippy::print_stdout,
//     clippy::print_stderr,
//     clippy::dbg_macro,
// )]

use std::fmt::Debug;
use std::rc::Rc;

use typst_syntax::{parse, LinkedNode};
mod config;
pub use config::Config;

mod writer;
use writer::Writer;

mod spacing;
use spacing::*;

mod utils;
// mod tests;

/// we visit our [FmtNode] tree, using the [Writer] to specify how we want
/// our formatting to be done.
///
/// Then we apply indentation as post processing.
#[must_use]
pub fn format(s: &str, config: Config) -> String {
    //replace tabs
    let s = &s.replace('\t', &config.indent.get(1));

    let init = parse(s);
    let mut s = String::new();
    let mut context = Writer::new(config, &mut s);
    let root = LinkedNode::new(&init);
    let mut root = convert(root, None);
    preserve_pass(&mut root);
    visit_markup(&root, &mut context);
    regex::Regex::new("( )+\n")
        .unwrap()
        .replace_all(&s, "\n")
        .to_string()
}

/// We translate the syntax tree, simplifying it for the
/// purpose of formatting only, then we apply rules to it to simplify
/// the formatting part.
///
/// Rules :
/// - Preserve: Raw blocks and area delimited by `// fmt::off` and `// fmt::on` shouldn't be formatted.
/// - Condition: We collapse all nested conditional nodes into one in
/// order to be able to format consistently across long if else chains.
/// - Binops: like conditional for for binary operation.
struct FmtNode<'a> {
    inner: Inner<'a>,
    kind: FmtKind,
}

enum FmtKind {
    // used only for when in preserve Node, ignored otherwise.
    Space,
    Parbreak,
    /// Must not be followed by a space, i.e : `#`
    NoSpaceAfter,
    /// Must be followed by a space, i.e : `let`
    SpaceAfter,
    /// Must be followed by a line_break, i.e : `let`
    BrkLineAfter,
    MaySpaceAfter,
    Markup,
    MarkupBlock,
    OnLineMarkup,
    Code,
    CodeBlock,
    Equation,
    ListLike,
    ParamsLike,
    // We must preserve the content as is
    Preserve,
    /// if else chain that we must format all at the same time for maximum style
    Condition,
    Binops,
}

struct Inner<'a> {
    parent: Option<Rc<FmtNode<'a>>>,
    content: Content<'a>,
}

enum Content<'a> {
    Text(&'a str),
    Children(Vec<FmtNode<'a>>),
}

impl FmtKind {
    pub fn with_children<'a>(self, node: LinkedNode<'a>) -> FmtNode<'a> {
        FmtNode {
            inner: Inner {
                parent: node.parent().cloned().map(convert).map(Rc::new),
                content: Content::Children(node.children().map(|c| convert(c)).collect()),
            },
            kind: self,
        }
    }

    pub fn with_text<'a>(self, node: LinkedNode<'a>) -> FmtNode<'a> {
        FmtNode {
            inner: Inner {
                parent: node.parent().cloned().map(convert).map(Rc::new),
                content: Content::Text(node.get().text()),
            },
            kind: self,
        }
    }
}

pub fn convert<'a>(node: LinkedNode<'a>) -> FmtNode<'a> {
    match node.kind() {
        Markup => FmtKind::Markup.with_children(node),

        Space => FmtKind::Space.with_text(node),
        Linebreak => FmtKind::BrkLineAfter.with_text(node),
        Parbreak => FmtKind::Parbreak.with_text(node),

        RefMarker | SmartQuote => FmtKind::NoSpaceAfter.with_text(node),

        Text | Shorthand | Escape => FmtKind::MaySpaceAfter.with_text(node),
        Link | Label | Strong | Emph => FmtKind::MaySpaceAfter.with_children(node),
        Raw => FmtKind::Preserve.with_text(node),

        ListMarker | EnumMarker | TermMarker | HeadingMarker | Ref => {
            FmtKind::SpaceAfter.with_children(node)
        }

/// preserve index and parent must handle me flags
#[derive(Default)]
struct PreserveData {
    preserve_idx: i32,
    parent_must_handle: bool,
}

impl PreserveData {
    fn new(preserve_idx: i32, parent_must_handle: bool) -> Self {
        Self {
            preserve_idx,
            parent_must_handle,
        }
    }
}

/// Modifies the tree to isolate preserve nodes
/// of `// typstfmt::off` matching `// typstfmt::on`
///
/// The boolean returned is a preserve flag, the level of preserved nesting.
///
/// Do we need to be able to say I handled my children
///
/// Example:
/// let our snippet correspond to
/// ```ignore
/// f(
///     // t::off
///     f([text  text  text]),2,
///  
/// )
/// // t::on
/// ```
///
/// Here, `f([text  text  text])` wouldn't be tag as Preserve cause it has children
/// and it's parent is not preserved either so we need else
///
fn preserve_pass(node: &mut FmtNode) -> PreserveData {
    match node.kind {
        FmtKind::Comment => {
            let text = node.text().unwrap();
            if text.contains("typstfmt::off") {
                return PreserveData::new(1, false);
            } else if text.contains("typstfmt::on") {
                return PreserveData::new(-1, false);
            }
        }
        _ => {}
    }

    match &mut node.content {
        Content::Text(_) => PreserveData::default(),
        Content::Children(c) => {
            let mut node_data = PreserveData::new(0, true);
            c.iter_mut().for_each(|c| {
                let child_data = preserve_pass(c);
                // we ignore typstfmt::on if we weren't preserving already
                node_data.preserve_idx = (node_data.preserve_idx + child_data.preserve_idx).max(0);

                #[allow(unused_doc_comments)]
                /// cases: one of my child starts the p process, my parent must not handle me
                if node_data.preserve_idx > 0 {
                    // We had things to do so we might have already handles things correctly
                    node_data.parent_must_handle = false;
                    match &mut c.content {
                        Content::Text(_) => c.kind = FmtKind::Preserve(node_data.preserve_idx),
                        Content::Children(_) => {
                            if child_data.parent_must_handle || node_data.preserve_idx > 1 {
                                c.kind = FmtKind::Preserve(node_data.preserve_idx)
                            }
                            // else the node handled itself already
                        }
                    }
                }
            });
            node_data
        }
    }
}

fn visit_content_block(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    // TODO put marker to come back here if too long
    w.push_str("[");
    // TODO ADD MARK IDENT
    visit_markup(parent, w) // todo try markup_short then markup_long
                            // TODO ADD MARK DEDENT
                            // TODO Check from marker if too long : markup_long.
}

fn visit_code_block(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    w.mark_indent();
    todo!();
    w.mark_dedent();
}

fn visit_code(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_set_rule(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_show_rule(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_math(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_equation(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_conditional(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_while(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_for(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_import(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_include(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

/// makes sure it's on one line
fn visit_heading(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    todo!("headings")
}

fn visit_markup(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    debug_assert!(parent.is::<typst_syntax::ast::Markup>());
    for parent in parent.children() {
        match parent.kind() {
            CodeBlock => visit_code_block(&parent, w),
            ContentBlock => visit_content_block(&parent, w),
            Equation => visit_equation(&parent, w),
            Parbreak => {
                w.new_line();
                w.new_line();
            }
            Raw => visit_raw(&parent, w),
            Heading => visit_heading(&parent, w),
            Hashtag => {
                w.push_node(&parent);
                visit_code(&parent, w)
            }

            TermItem | TermMarker | HeadingMarker | ListItem | ListMarker | EnumItem | Math
            | MathIdent | MathAlignPoint | MathDelimited | MathAttach | MathPrimes | MathFrac
            | MathRoot | EnumMarker => {
                unreachable!()
            }
            LineComment => {
                w.push_node(&parent);
                w.new_line();
            }
            BlockComment => w.push_node(&parent),
            Space => (),
            _ => {
                w.push_node_spaced(&parent);
            }
        }
    }
}

fn visit_raw(parent: &LinkedNode<'_>, w: &mut Writer<'_>) {
    let m = w.mark_preserve();
    w.push_node(&parent)
    // w no indent close
}

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
        let mut root = convert(root, None);
        println!("===");
        println!("{:?}", root);
        println!("===");
        preserve_pass(&mut root);
        println!("{:?}", root);
        println!("===");
        insta::assert_debug_snapshot!(root)
    }
}
