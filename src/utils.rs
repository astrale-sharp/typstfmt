use super::*;
use unicode_segmentation::UnicodeSegmentation;

/// like next sibling but doesn't skip trivia.
pub(crate) fn next_sibling_or_trivia<'a>(node: &LinkedNode<'a>) -> Option<LinkedNode<'a>> {
    node.parent()?.children().nth(node.index() + 1)
}

/// find any child recursively that fits predicate
#[instrument(ret, skip_all)]
pub(crate) fn find_child<'a>(
    node: &LinkedNode<'a>,
    predicate: &impl Fn(&LinkedNode) -> bool,
) -> Option<LinkedNode<'a>> {
    debug!("::find_child of {:?}", node.kind());
    debug!(
        "on children: {:?}",
        node.children().map(|x| x.kind()).collect_vec()
    );
    for child in node.children() {
        debug!("try for {:?}", child.kind());
        if predicate(&child) {
            debug!("predicate accepted");
            return Some(child.clone());
        } else if let Some(f) = find_child(&child, predicate) {
            debug!("predicate accepted for inner of {:?}", child.kind());
            return Some(f);
        }
    }
    None
}

#[instrument(ret, skip_all)]
pub(crate) fn find_next<'a>(
    node: &LinkedNode<'a>,
    predicate: &impl Fn(&LinkedNode) -> bool,
) -> Option<LinkedNode<'a>> {
    let mut next = next_sibling_or_trivia(node);
    while let Some(next_inner) = next {
        if predicate(&next_inner) {
            return Some(next_inner);
        }
        next = next_sibling_or_trivia(&next_inner);
    }
    None
}

#[instrument(ret, skip_all)]
pub(crate) fn get_next_ignoring<'a>(
    node: &'a LinkedNode<'a>,
    ignoring: &[SyntaxKind],
) -> Option<LinkedNode<'a>> {
    let mut next = next_sibling_or_trivia(node);
    while let Some(next_inner) = &next {
        let kind = next_inner.kind();
        if ignoring.contains(&kind) {
            next = next_sibling_or_trivia(&next_inner.clone());
            continue;
        }
        return Some(next_inner.clone());
    }
    None
}

#[instrument(ret, skip_all)]
pub(crate) fn next_is_ignoring(node: &LinkedNode, is: SyntaxKind, ignoring: &[SyntaxKind]) -> bool {
    let n = get_next_ignoring(node, ignoring);
    debug!("next is: {:?}", n.as_ref().map(|x| x.kind()));
    n.is_some_and(|n| is == n.kind())
}

pub(crate) fn max_line_length(s: &str) -> usize {
    s.lines()
        .map(|l| l.trim().graphemes(true).count())
        .max()
        .unwrap_or(0)
}
