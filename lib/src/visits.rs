use crate::{
    node::{Content, FmtKind, FmtNode, Spacing},
    writer::Writer,
};

pub(crate) fn visit_markup(node: &FmtNode<'_>, w: &mut Writer<'_>) {
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

pub(crate) fn visit_content_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
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
pub(crate) fn visit_code_block(node: &FmtNode<'_>, w: &mut Writer<'_>) {
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

pub(crate) fn matches_text(c: Option<&FmtNode>, s: &str) -> bool {
    matches!(c.map(|c| &c.content), Some(Content::Text(t)) if t == &s)
}

pub(crate) fn visit_spacing(s: &Spacing, node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_equation(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_math(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_code(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_parbreak(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_conditional(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_one_line_markup(node: &FmtNode<'_>, w: &mut Writer<'_>) {
    todo!()
}

pub(crate) fn visit_preserve(node: &FmtNode<'_>, w: &mut Writer<'_>) {
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
    let Content::Children(children) = &node.content else {
        unreachable!()
    };
    let mut children = children.iter();
    // todo, more checks? like is it an ident etc
    w.push_node(children.next().unwrap());
    visit_params_like(children.next().unwrap(), w, false)
}
