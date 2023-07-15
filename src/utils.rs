use unicode_segmentation::UnicodeSegmentation;

use super::*;

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
    let mut next = node.next_sibling();
    while let Some(next_inner) = next {
        if predicate(&next_inner) {
            return Some(next_inner);
        }
        next = next_inner.next_sibling();
    }
    None
}

#[instrument(ret, skip_all)]
pub(crate) fn get_next_ignoring<'a>(
    node: &LinkedNode<'a>,
    ignoring: &[SyntaxKind],
) -> Option<LinkedNode<'a>> {
    let mut next = node.next_sibling();
    while let Some(next_inner) = &next {
        let kind = next_inner.kind();
        if ignoring.contains(&kind) {
            next = next_inner.next_sibling();
            continue;
        }
        return Some(next_inner.clone());
    }
    None
}

#[instrument(ret, skip_all)]
pub(crate) fn next_is_ignoring(
    node: &LinkedNode,
    is: SyntaxKind,
    ignoring: &[SyntaxKind],
) -> bool {
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
