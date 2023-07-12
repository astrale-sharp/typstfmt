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

pub(crate) fn is_trailing_comma(node: &LinkedNode<'_>) -> bool {
    assert!(node.kind() == Comma);
    let next = node.next_sibling();
    let next_skipping_space = match &next {
        Some(x) if x.kind() == Space => next.unwrap().next_sibling(),
        _ => next,
    };
    next_skipping_space.is_some_and(|n| n.kind().is_terminator())
}

pub(crate) fn is_last_comma(node: &LinkedNode) -> bool {
    assert!(node.kind() == Comma);
    let mut next = node.next_sibling().unwrap();
    loop {
        if next.kind() == Comma {
            return false;
        }
        if next.kind().is_terminator() {
            return true;
        }
        next = next.next_sibling().unwrap();
    }
}

pub(crate) fn max_line_length(s: &str) -> usize {
    s.lines()
        .map(|l| l.trim().chars().count())
        .max()
        .unwrap_or(0)
}
