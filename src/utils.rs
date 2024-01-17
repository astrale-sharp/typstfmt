use typst_syntax::ast::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::node::{Content, FmtKind, FmtNode};

pub(crate) fn matches_text(c: Option<&FmtNode>, s: &str) -> bool {
    matches!(c.map(|c| &c.content), Some(Content::Text(t)) if t == &s)
}

pub(crate) fn match_first_unwrapped<'a, T: AstNode>(
    c: &mut impl Iterator<Item = &'a FmtNode<'a>>,
) -> &FmtNode<'a> {
    c.find(|c| c.node.is::<T>()).unwrap()
}

/// ignores parbreak and comments
pub(crate) fn next_is_space<'a>(nodes: impl Iterator<Item = &'a FmtNode<'a>>) -> bool {
    nodes
        .map(|n| &n.kind)
        .take_while(|&kind| matches!(kind, FmtKind::Comment | FmtKind::Space | FmtKind::Parbreak))
        .find(|&kind| kind == &FmtKind::Space)
        .is_some()
}

pub(crate) fn max_line_length(s: &str) -> usize {
    s.lines()
        .map(|l| l.trim().graphemes(true).count())
        .max()
        .unwrap_or(0)
}

pub(crate) fn first_line_length(s: &str) -> usize {
    s.split('\n')
        .next()
        .unwrap_or("")
        .trim()
        .graphemes(true)
        .count()
}
