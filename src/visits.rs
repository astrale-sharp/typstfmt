// the question is to whom
use crate::{
    node::{Content, FmtKind, FmtNode, Spacing},
    utils::{self, match_first_unwrapped, matches_text},
    writer::Writer,
};

use typst_syntax::ast::*;

/// respects everything but spaces
fn visit(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    match &node.kind {
        FmtKind::Space => (),
        FmtKind::FuncCall => visit_func_call(node, w),
        FmtKind::ParamsLike => visit_params_like(node, w, false),
        FmtKind::ParamsLikeParenthesized => visit_params_like(node, w, true),
        FmtKind::Binary => visit_binary(node, w),
        FmtKind::Unary => visit_unary(node, w),
        FmtKind::Preserve(_) => visit_preserve(node, w),
        FmtKind::Parbreak => visit_parbreak(node, w),
        FmtKind::WithSpacing(s) => visit_spacing(s, node, w),
        FmtKind::Markup => visit_markup(node, w, true),
        FmtKind::ContentBlock => visit_content_block(node, w),
        FmtKind::OneLineMarkup => visit_one_line_markup(node, w),
        FmtKind::Conditional => visit_conditional(node, w),
        FmtKind::Code => visit_code(node, w),
        FmtKind::CodeBlock => visit_code_block(node, w),
        FmtKind::Math => visit_math(node, w),
        FmtKind::Equation => visit_equation(node, w),
        FmtKind::Comment => visit_comment(node, w),
    }
}

fn visit_comment(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    visit_preserve(node, w);
    w.push_str("\n")
}

/// Todo, deals with most common cases that don't need special logic.
///
/// Might not always be called on (Node)[FmtNode] of kind [FmtKind::WithSpacing] if their in
/// another visit function that has it's own logic.
pub(crate) fn visit_spacing(s: &Spacing, node: &FmtNode<'_>, w: &mut Writer<'_>) {
    w.push_str(&node.text().trim());
    match s {
        Spacing::WeakSpace => w.push_str(" "),
        Spacing::Destruct => (),
        Spacing::StrongSpace => w.push_str(" "),
        Spacing::StrongBrkLine => w.push_str("\n"),
    }
}

pub(crate) fn visit_spacing_line_wrapped(s: &Spacing, node: &FmtNode<'_>, w: &mut Writer<'_>) {
    w.push_str_with_limit(&node.text().trim());
    match s {
        Spacing::WeakSpace => w.push_str(" "),
        Spacing::Destruct => (),
        Spacing::StrongSpace => w.push_str(" "),
        Spacing::StrongBrkLine => w.push_str("\n"),
    }
}

/// simply push respecting everything and tag it as preserve.
/// todo: can we rely on node.text? we should have a recursive use of visit preserve instead.
pub(crate) fn visit_preserve(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    w.mark_preserve();
    w.push_str(&node.text());
    w.mark_stop_preserve();
}

pub(crate) fn visit_content_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let mark = w.get_mark();
    visit_content_block_with(node, w, true);
    // todo or go above max length
    if w.buffer[mark..].contains("\n") {
        w.rewind(mark);
        visit_content_block_with(node, w, false);
    }
}

/// todo: respect absence of space just after [ and just before ]
/// - try a tight and expand mode.
pub(crate) fn visit_content_block_with(node: &FmtNode<'_>, w: &mut Writer<'_>, tight: bool) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    // todo rm this clone
    assert!(matches_text(c.last(), "["));
    assert!(matches_text(c.first(), "]"));

    w.push_str("[");
    w.mark_indent();
    let mut c = c.iter();
    c.next();
    if let Some(c) = c.next() {
        debug_assert_eq!(c.kind, FmtKind::Markup);
        visit_markup(node, w, tight)
    } else {
        debug_assert!(false);
    }
    w.mark_dedent();
    w.push_str("]");
}

/// if config.wrap_text Wrap text similar to visit_basic but checking max line length
/// else respect space but replace " "+ -> " " and <whitespace> "\n"+ <whitespace> -> "\n"
// optimize, return a bool to fail tight in advance
pub(crate) fn visit_markup(node: &FmtNode<'_>, w: &mut Writer<'_>, tight: bool) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    let start_space = utils::next_is_space(c.iter());
    let end_space = utils::next_is_space(c.iter().rev());

    let c = &mut c.iter().filter(|c| c.kind != FmtKind::Space).peekable();
    let space_kind = if tight { " " } else { "\n" };
    if start_space {
        w.push_str(space_kind)
    }
    for node in c {
        match &node.kind {
            FmtKind::Space if w.config.line_wrap => (),
            FmtKind::WithSpacing(s) if w.config.line_wrap => visit_spacing_line_wrapped(s, node, w),
            FmtKind::Space => {
                if node.text().contains("\n") {
                    w.push_str("\n")
                }
            }
            FmtKind::ContentBlock if tight => visit_content_block_with(node, w, tight),
            _ => visit(node, w),
        }
    }
    if end_space {
        w.push_str(space_kind)
    }
}

pub(crate) fn visit_code(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_code_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    let c = &mut c.iter().filter(|c| c.kind != FmtKind::Space);
    assert!(matches_text(c.next(), "{"));
    assert!(matches_text(c.next_back(), "}"));
    // todo put a mark try one line code else rewind

    w.push_str("{\n");
    w.mark_indent();
    for node in c {
        visit(node, w)
    }
    w.mark_dedent();
    w.push_str("\n}");
}

pub(crate) fn visit_math(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_equation(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    let c = &mut c.iter().filter(|c| c.kind != FmtKind::Space);
    assert!(matches_text(c.next(), "$\n"));
    assert!(matches_text(c.next_back(), "$\n"));

    w.push_str("$\n");
    w.mark_indent();
    visit_math(&match_first_unwrapped::<Math>(c), w);
    w.mark_dedent();
    w.push_str("\n$");
}

pub(crate) fn visit_parbreak(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    w.push_str("\n\n")
}

pub(crate) fn visit_conditional(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_one_line_markup(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_unary(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_binary(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_params_like(node: &FmtNode<'_>, w: &mut Writer<'_>, parenthesized: bool) {
    todo!()
}

pub(crate) fn visit_func_call(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}
