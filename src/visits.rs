use typst_syntax::ast::*;

use crate::{
    node::{Content, FmtKind, FmtNode, Spacing},
    utils::{match_first_unwrapped, matches_text},
    writer::Writer,
};

/// keeps handling things the basic way:
/// - ignores Space
/// - respects WithSpacing
/// - doesn't check line length or previous existing space.
fn visit_basic(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    match &node.kind {
        FmtKind::Space => (),
        FmtKind::Parbreak => visit_parbreak(node, w),
        FmtKind::WithSpacing(s) => visit_spacing(s, node, w),
        FmtKind::ContentBlock => visit_content_block(node, w),
        FmtKind::Preserve(_) => visit_preserve(node, w),
        FmtKind::Comment => {
            visit_preserve(node, w);
            w.push_str("\n")
        }
        FmtKind::CodeBlock => visit_code_block(node, w),
        FmtKind::Equation => visit_equation(node, w),

        FmtKind::FuncCall
        | FmtKind::ParamsLike
        | FmtKind::ParamsLikeParenthesized
        | FmtKind::Binary
        | FmtKind::Unary
        | FmtKind::OneLineMarkup
        | FmtKind::Conditional
        | FmtKind::Code
        | FmtKind::Math
        | FmtKind::Markup => {
            let Content::Children(children) = &node.content else {
                unreachable!()
            };
            for node in children {
                visit_basic(node, w);
            }
        } // FmtKind::Code => visit_code(node, w),
          // FmtKind::FuncCall => visit_func_call(node, w),
          // FmtKind::ParamsLike => visit_params_like(node, w, false),
          // FmtKind::ParamsLikeParenthesized => visit_params_like(node, w, true),
          // FmtKind::Binary => visit_binary(node, w),
          // FmtKind::Unary => visit_unary(node, w),
          // FmtKind::OneLineMarkup => visit_one_line_markup(node, w),
          // FmtKind::Conditional => visit_conditional(node, w),
          // FmtKind::Math => visit_math(node, w),
    }
}

/// Todo, deals with most common cases that don't need special logic.
///
/// Might not always be called on (Node)[FmtNode] of kind [FmtKind::WithSpacing] if their in
/// another visit function that has it's own logic.
pub(crate) fn visit_spacing(s: &Spacing, node: &FmtNode<'_>, w: &mut Writer<'_>) {
    println!("spacing");
    dbg!(node);
    w.push_str(&node.text().trim());
    match s {
        Spacing::WeakSpace => w.push_str(" "),
        Spacing::Destruct => (),
        Spacing::StrongSpace => w.push_str(" "),
        Spacing::StrongBrkLine => w.push_str("\n"),
    }
}

/// if config.wrap_text Wrap text similar to visit_basic but checking max line length
/// else respect space but replace " "+ -> " " and <whitespace> "\n"+ <whitespace> -> "\n"
pub(crate) fn visit_markup(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

/// simply push respecting everything and tag it as preserve.
/// todo: can we rely on node.text? we should have a recursive use of visit preserve instead.
pub(crate) fn visit_preserve(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    w.mark_preserve();
    w.push_str(&node.text());
    w.mark_stop_preserve();
}

/// todo: respect absence of space just after [ and just before ]
/// - try a tight and expand mode.
pub(crate) fn visit_content_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    let Content::Children(c) = &node.content else {
        unreachable!()
    };
    let c = &mut c.iter().filter(|c| c.kind != FmtKind::Space);
    assert!(matches_text(c.next(), "["));
    assert!(matches_text(c.next_back(), "]"));

    w.push_str("[\n");
    w.mark_indent();
    for node in c {
        visit_basic(node, w)
    }
    w.mark_dedent();
    w.push_str("\n]");
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
        visit_basic(node, w)
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
