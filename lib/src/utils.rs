use super::*;
use unicode_segmentation::UnicodeSegmentation;

/// like next sibling but doesn't skip trivia.
#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn next_sibling_or_trivia<'a>(node: &LinkedNode<'a>) -> Option<LinkedNode<'a>> {
    node.parent()?.children().nth(node.index() + 1)
}

/// like next sibling but doesn't skip trivia.
#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn prev_sibling_or_trivia<'a>(node: &LinkedNode<'a>) -> Option<LinkedNode<'a>> {
    if node.index() == 0 {
        return None;
    }
    node.parent()?.children().nth(node.index() - 1)
}

/// find any child recursively that fits predicate
#[instrument(ret, skip_all, level = "debug")]
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

/// find all children recursively that fits predicate
// pub(crate) fn find_children<'a>(
//     res: &mut Vec<LinkedNode<'a>>,
//     node: &LinkedNode<'a>,
//     predicate: &impl Fn(&LinkedNode) -> bool,
// ) {
//     for child in node.children() {
//         if predicate(&child) {
//             debug!("predicate accepted");
//             res.push(child);
//         } else {
//             find_children(res, &child, predicate);
//         }
//     }
// }

#[instrument(skip_all, level = "debug")]
pub(crate) fn eat_space(res: &mut String) {
    while res.ends_with('\n') || res.ends_with(' ') {
        res.replace_range(res.len() - 1..res.len(), "");
    }
}

#[instrument(ret, skip_all, level = "debug")]
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

#[instrument(ret, skip_all, level = "debug")]
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

#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn get_prev_ignoring<'a>(
    node: &'a LinkedNode<'a>,
    ignoring: &[SyntaxKind],
) -> Option<LinkedNode<'a>> {
    let mut prev = prev_sibling_or_trivia(node);
    while let Some(prev_inner) = &prev {
        let kind = prev_inner.kind();
        if ignoring.contains(&kind) {
            prev = prev_sibling_or_trivia(&prev_inner.clone());
            continue;
        }
        return Some(prev_inner.clone());
    }
    None
}

#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn next_is_ignoring(node: &LinkedNode, is: SyntaxKind, ignoring: &[SyntaxKind]) -> bool {
    let n = get_next_ignoring(node, ignoring);
    debug!("next is: {:?}", n.as_ref().map(|x| x.kind()));
    n.is_some_and(|n| is == n.kind())
}

#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn prev_is_ignoring(node: &LinkedNode, is: SyntaxKind, ignoring: &[SyntaxKind]) -> bool {
    let n = get_prev_ignoring(node, ignoring);
    debug!("next is: {:?}", n.as_ref().map(|x| x.kind()));
    n.is_some_and(|n| is == n.kind())
}

#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn max_line_length(s: &str) -> usize {
    s.lines()
        .map(|l| l.trim().graphemes(true).count())
        .max()
        .unwrap_or(0)
}

#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn last_line_length(s: &str) -> usize {
    s.split('\n')
        .last()
        .unwrap_or("")
        .trim_start()
        .graphemes(true)
        .count()
}

#[instrument(ret, skip_all, level = "debug")]
pub(crate) fn first_line_length(s: &str) -> usize {
    s.split('\n')
        .next()
        .unwrap_or("")
        .trim_start()
        .graphemes(true)
        .count()
}
