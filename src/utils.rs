use unicode_segmentation::UnicodeSegmentation;

use super::*;

/// find any child recursively that fits predicate
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

pub(crate) fn next_is_ignoring(node: &LinkedNode, is: SyntaxKind, ignoring: &[SyntaxKind]) -> bool {
    debug!("fn::next_is_ignoring, current is: {:?}", node);
    let mut next = node.next_sibling();
    debug!("{:?}", next);
    while let Some(next_inner) = &next {
        debug!("{:?}", next);
        let kind = next_inner.kind();
        if ignoring.contains(&kind) {
            debug!("ignoring {:?}", kind);

            next = next_inner.next_sibling();
            continue;
        }
        let next_is = kind == is;
        debug!("next is: {next_is}");
        return next_is;
    }
    debug!("next is not {is:?}");
    false
}

pub(crate) fn max_line_length(s: &str) -> usize {
    s.lines()
        .map(|l| l.trim().graphemes(true).count())
        .max()
        .unwrap_or(0)
}
