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

use std::rc::Rc;

use itertools::Itertools;
use typst_syntax::SyntaxKind::*;
use typst_syntax::{parse, LinkedNode};

mod config;
pub use config::Config;

mod context;
use context::Writer;

mod utils;
// mod tests;

/// we visit our [FmtNode] tree, using the [Writer] to specify how we want
/// our formatting to be done.
///
/// Then we apply indentation as post processing.
#[must_use]
pub fn format(s: &str, config: Config) -> String {
    //replace tabs
    let s = &s.replace('\t', &" ".repeat(config.indent_space));

    let init = parse(s);
    let mut s = String::new();
    let mut context = Writer::new(config, &mut s);
    let root = LinkedNode::new(&init);
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

        Heading => FmtKind::OnLineMarkup.with_children(node),

        ListItem | EnumItem | TermItem => FmtKind::ListLike.with_children(node),
        // todo sort the rest of those.
        Equation => FmtKind::Equation.with_children(node),
        Math => todo!(),
        MathIdent => todo!(),
        MathAlignPoint => todo!(),
        MathDelimited => todo!(),
        MathAttach => todo!(),
        MathPrimes => todo!(),
        MathFrac => todo!(),
        MathRoot => todo!(),
        Hashtag => todo!(),
        LeftBrace => todo!(),
        RightBrace => todo!(),
        LeftBracket => todo!(),
        RightBracket => todo!(),
        LeftParen => todo!(),
        RightParen => todo!(),
        Comma => todo!(),
        Semicolon => todo!(),
        Colon => todo!(),
        Star => todo!(),
        Underscore => todo!(),
        Dollar => todo!(),
        Plus => todo!(),
        Minus => todo!(),
        Slash => todo!(),
        Hat => todo!(),
        Prime => todo!(),
        Dot => todo!(),
        Eq => todo!(),
        EqEq => todo!(),
        ExclEq => todo!(),
        Lt => todo!(),
        LtEq => todo!(),
        Gt => todo!(),
        GtEq => todo!(),
        PlusEq => todo!(),
        HyphEq => todo!(),
        StarEq => todo!(),
        SlashEq => todo!(),
        Dots => todo!(),
        Arrow => todo!(),
        Root => todo!(),
        Not => todo!(),
        And => todo!(),
        Or => todo!(),
        None => todo!(),
        Auto => todo!(),
        Let => todo!(),
        Set => todo!(),
        Show => todo!(),
        If => todo!(),
        Else => todo!(),
        For => todo!(),
        In => todo!(),
        While => todo!(),
        Break => todo!(),
        Continue => todo!(),
        Return => todo!(),
        Import => todo!(),
        Include => todo!(),
        As => todo!(),
        Code => todo!(),
        Ident => todo!(),
        Bool => todo!(),
        Int => todo!(),
        Float => todo!(),
        Numeric => todo!(),
        Str => todo!(),
        CodeBlock => todo!(),
        ContentBlock => todo!(),
        Parenthesized => todo!(),
        Array => todo!(),
        Dict => todo!(),
        Named => todo!(),
        Keyed => todo!(),
        Unary => todo!(),
        Binary => todo!(),
        FieldAccess => todo!(),
        FuncCall => todo!(),
        Args => todo!(),
        Spread => todo!(),
        Closure => todo!(),
        Params => todo!(),
        LetBinding => todo!(),
        SetRule => todo!(),
        ShowRule => todo!(),
        Conditional => todo!(),
        WhileLoop => todo!(),
        ForLoop => todo!(),
        ModuleImport => todo!(),
        ImportItems => todo!(),
        ModuleInclude => todo!(),
        LoopBreak => todo!(),
        LoopContinue => todo!(),
        FuncReturn => todo!(),
        Destructuring => todo!(),
        DestructAssignment => todo!(),
        LineComment => todo!(),
        BlockComment => todo!(),
        Error => todo!(),
        Eof => todo!(),
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
