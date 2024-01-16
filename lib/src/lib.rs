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
/// - Binary: like conditional for for binary operation.
#[derive(Clone)]
struct FmtNode<'a> {
    parent: Option<Rc<FmtNode<'a>>>,
    content: Content<'a>,
    kind: FmtKind,
}

impl<'a> FmtNode<'a> {
    pub fn text(&self) -> Option<&'a str> {
        match self.content {
            Content::Text(s) => Some(s),
            Content::Children(_) => None,
        }
    }
}

impl Debug for FmtNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.kind, self.content)
    }
}

// todo a LetBinding, ForLoop, WhileLoop node forcing a linebreak in code mode or a ; in markup?
#[derive(Debug, Clone, PartialEq)]
enum FmtKind {
    // used only for when in preserve Node, ignored otherwise.
    FuncCall,
    ParamsLike,
    ParamsLikeParenthesized,
    Binary,
    Unary,
    /// We must preserve the content as is
    ///
    /// We smartly only tag the text nodes in the preserve_pass, so we can rely on it when
    /// `evaluating Preserve`
    /// TODO: panic if the node has children
    Preserve(i32),
    /// if else chain that we must format all at the same time for maximum style
    Space,
    Parbreak,

    WithSpacing(Spacing),

    Markup,
    ContentBlock,
    OneLineMarkup,
    Conditional,

    // todo: force line break after loops
    Code,
    CodeBlock,

    Math,
    Equation,

    Comment,
}

#[derive(Clone)]
enum Content<'a> {
    Text(&'a str),
    Children(Vec<FmtNode<'a>>),
}

impl Debug for Content<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(arg0) => f.debug_tuple("Text").field(arg0).finish(),
            Self::Children(arg0) => arg0.fmt(f),
        }
    }
}

impl FmtKind {
    pub fn with_children<'a>(
        self,
        node: LinkedNode<'a>,
        parent: Option<Rc<FmtNode<'a>>>,
    ) -> FmtNode<'a> {
        let mut fnode = FmtNode {
            parent,
            content: Content::Children(vec![]),
            kind: self,
        };
        fnode.content = Content::Children(
            node.children()
                .map(|c| convert(c, Some(Rc::new(fnode.clone()))))
                .collect(),
        );

        fnode
    }

    pub fn with_text<'a>(
        self,
        node: LinkedNode<'a>,
        parent: Option<Rc<FmtNode<'a>>>,
    ) -> FmtNode<'a> {
        FmtNode {
            parent,
            content: Content::Text(node.get().text()),

            kind: self,
        }
    }
}

fn convert<'a>(node: LinkedNode<'a>, parent: Option<Rc<FmtNode<'a>>>) -> FmtNode<'a> {
    use typst_syntax::SyntaxKind::{self, *};

    match node.kind() {
        Raw => FmtKind::Preserve(1).with_text(node, parent),
        Space => FmtKind::Space.with_text(node, parent),
        Linebreak => FmtKind::WithSpacing(Spacing::StrongBrkLine).with_text(node, parent),
        Parbreak => FmtKind::Parbreak.with_text(node, parent),
        Markup => FmtKind::Markup.with_children(node, parent),
        Code => FmtKind::Code.with_children(node, parent),
        CodeBlock => FmtKind::CodeBlock.with_children(node, parent),
        ContentBlock => FmtKind::ContentBlock.with_children(node, parent),
        Math => FmtKind::Math.with_children(node, parent),
        Equation => FmtKind::Equation.with_children(node, parent),
        FuncCall => FmtKind::FuncCall.with_children(node, parent),
        Array | Dict | Args | ListItem | EnumItem | TermItem | Params | Destructuring => {
            FmtKind::ParamsLike.with_children(node, parent)
        }
        Parenthesized => FmtKind::ParamsLikeParenthesized.with_children(node, parent),
        Heading => FmtKind::OneLineMarkup.with_children(node, parent),
        Unary => FmtKind::Unary.with_children(node, parent),
        Binary => FmtKind::Binary.with_children(node, parent),

        LetBinding | ModuleImport | ImportItems | ModuleInclude | SetRule | ShowRule => {
            FmtKind::WithSpacing(Spacing::StrongBrkLine).with_children(node, parent)
        }

        Eof | Spread | Root | Dots | Dot | Star | Underscore | Hashtag | RefMarker | SmartQuote => {
            FmtKind::WithSpacing(Spacing::Destruct).with_text(node, parent)
        }

        LoopBreak
        | LoopContinue
        | FuncReturn
        | Bool
        | Int
        | Float
        | Numeric
        | Str
        | Arrow
        | Not
        | And
        | Or
        | SyntaxKind::None
        | Auto
        | Let
        | Set
        | Show
        | If
        | Else
        | For
        | In
        | While
        | Break
        | Continue
        | Return
        | Import
        | Include
        | As
        | Eq
        | EqEq
        | ExclEq
        | Lt
        | LtEq
        | Gt
        | GtEq
        | PlusEq
        | HyphEq
        | StarEq
        | SlashEq
        | Plus
        | Minus
        | Hat
        | Prime
        | Slash
        | Semicolon
        | Dollar
        | LeftParen
        | RightParen
        | LeftBrace
        | RightBrace
        | LeftBracket
        | RightBracket
        | MathAlignPoint
        | Error
        | Text
        | Shorthand
        | Escape
        | MathIdent => FmtKind::WithSpacing(Spacing::WeakSpace).with_text(node, parent),

        DestructAssignment | WhileLoop | ForLoop | Closure | FieldAccess | Named | Keyed
        | MathFrac | MathRoot | MathAttach | Link | Label | Strong | Emph => {
            FmtKind::WithSpacing(Spacing::WeakSpace).with_children(node, parent)
        }

        MathDelimited | ListMarker | EnumMarker | TermMarker | HeadingMarker | Ref => {
            FmtKind::WithSpacing(Spacing::WeakSpace).with_children(node, parent)
        }

        Ident | Colon | Comma | MathPrimes => {
            FmtKind::WithSpacing(Spacing::StrongSpace).with_text(node, parent)
        }

        Conditional => FmtKind::Conditional.with_children(node, parent),

        BlockComment | LineComment => FmtKind::Comment.with_text(node, parent),
    }
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

fn visit_content_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    let mut c = c.iter().filter(|c| c.kind != FmtKind::Space);
    assert!(matches_text(c.next(), "["));
    assert!(matches_text(c.last(), "]"));
    w.push_str("[");
    w.mark_indent();
    visit_markup(node, w);
    w.mark_dedent();
    w.push_str("]");
}

fn visit_code_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    let mut c = c.iter().filter(|c| c.kind != FmtKind::Space);
    assert!(matches_text(c.next(), "{"));
    assert!(matches_text(c.last(), "}"));
    // todo put a mark try one line code else rewind

    w.push_str("{");
    w.mark_indent();
    visit_code(node, w);
    w.mark_dedent();
    w.push_str("}");
}

fn matches_text(c: Option<&FmtNode>, s: &str) -> bool {
    matches!(c.map(|c| &c.content), Some(Content::Text(t)) if t == &s)
}

fn visit_markup(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    // debug_assert!(node.is::<typst_syntax::ast::Markup>());
    let Content::Children(children) = &node.content else {
        unreachable!()
    };
    for node in children {
        match &node.kind {
            FmtKind::Markup => visit_markup(node, w),
            FmtKind::ContentBlock => visit_content_block(node, w),
            FmtKind::FuncCall => visit_func_call(node, w),
            FmtKind::ParamsLike => visit_params_like(node, w, false),
            FmtKind::ParamsLikeParenthesized => visit_params_like(node, w, true),
            FmtKind::Binary => visit_binary(node, w),
            FmtKind::Unary => visit_unary(node, w),
            FmtKind::Preserve(_) | FmtKind::Comment => visit_preserve(node, w),
            FmtKind::Space => (),
            FmtKind::OneLineMarkup => visit_one_line_markup(node, w),
            FmtKind::Conditional => visit_conditional(node, w),
            FmtKind::Parbreak => visit_parbreak(node, w),
            FmtKind::Code => visit_code(node, w),
            FmtKind::CodeBlock => visit_code_block(node, w),
            FmtKind::Math => visit_math(node, w),
            FmtKind::Equation => visit_equation(node, w),
            FmtKind::WithSpacing(s) => visit_spacing(s, node, w),
        }
    }
}

fn visit_spacing(s: &Spacing, node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_equation(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_math(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_code(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_parbreak(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_conditional(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_one_line_markup(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_preserve(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_unary(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_binary(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

fn visit_params_like(node: &FmtNode<'_>, w: &mut Writer<'_>, parenthesized: bool) {
    todo!()
}

fn visit_func_call(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let Content::Children(children) = &node.content else {
        unreachable!()
    };
    let mut children = children.iter();
    // todo, more checks? like is it an ident etc
    w.push_node(children.next().unwrap());
    visit_params_like(children.next().unwrap(), w, false)
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
