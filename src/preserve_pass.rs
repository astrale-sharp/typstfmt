use super::node::{Content, FmtKind, FmtNode};

/// Matches `// typstfmt::off` and `// typstfmt::on` Typst's comments
/// and Mark as Preserved all the nodes that are englobed. They will be preserved from
/// modifications during formatting.
///
/// Note that if one of your children has an off command, it doesn't ensure that the on
/// command is in one of your other children or exists at all.
///
/// Example:
/// In the next snippet, everything between the two commands is preserved.
/// ```ignore
/// f(
///     // t::off
///     g([text  text  text]),2,
/// )
/// lorem ipsum
/// // t::on
/// ```
///
/// Note however that starting a node preserved will not give you the option to change your mind halfway, as
/// all the logic of handling a node happens in the beginning of it's formatting process.
///
/// Hence in the next snippet, the whole f function call will be preserved (but not the lorem ispum)
/// ```ignore
/// // t::off
/// f(
///     // t::on
///     g([text  text  text]),2,
/// )
/// lorem ipsum
/// ```
///
/// TODO: TEST lorem ispum is not preserved.
pub(crate) fn preserve_pass(node: &mut FmtNode) -> PreserveData {
    match node.kind {
        FmtKind::Comment => {
            let text = node.text();
            if text.starts_with("// typstfmt::off") {
                return PreserveData::new(1, false);
            } else if text.starts_with("// typstfmt::on") {
                return PreserveData::new(-1, false);
            } else if text.contains("typstfmt::") {
                // TODO WARN HERE
            }
        }
        _ => ()
    }

    match &mut node.content {
        Content::Text(_) => PreserveData::default(),
        Content::Children(c) => {
            let mut node_data = PreserveData::new(0, true);
            c.iter_mut().for_each(|c| {
                let child_data = preserve_pass(c);
                // we ignore typstfmt::on if we weren't preserving already
                node_data.preserve_idx = (node_data.preserve_idx + child_data.preserve_idx).max(0);

                // if one of the children started the preservation, it handled itself already.
                if node_data.preserve_idx > 0 {
                    // We had things to do so we might have already handles things correctly
                    node_data.parent_must_handle = false;
                    match &mut c.content {
                        Content::Text(_) => c.kind = FmtKind::Preserve(node_data.preserve_idx),
                        Content::Children(_) => {
                            if child_data.parent_must_handle || node_data.preserve_idx > 1 {
                                c.kind = FmtKind::Preserve(node_data.preserve_idx)
                            }
                            // else the node handled itself already
                        }
                    }
                }
            });
            node_data
        }
    }
}

/// preserve index and parent must handle me flags
#[derive(Default)]
pub(crate) struct PreserveData {
    preserve_idx: i32,
    parent_must_handle: bool,
}

impl PreserveData {
    fn new(preserve_idx: i32, parent_must_handle: bool) -> Self {
        Self {
            preserve_idx,
            parent_must_handle,
        }
    }
}



macro_rules! tree_has_node_matching {
    (
        $test_name:ident,
        $snippet:expr,
        $kind_fn:expr,
        $parent_fn:expr,
        $($matching:tt)*
    ) => {
        #[test]
        fn $test_name() {
            let snippet = typst_syntax::parse($snippet);
            let root = typst_syntax::LinkedNode::new(&snippet);
            let tree = crate::map_tree(root, None);
            dbg!(&tree);
            let parent : &dyn Fn(Option<&FmtNode>) -> bool = $parent_fn;
            let kind : &dyn Fn(&FmtKind) -> bool = &$kind_fn;
            assert!(
                contains_matching(&tree, &|node| {
                    if !(parent(node.parent.as_deref()) && kind(&node.kind))  {
                        return false
                    }

                    match &node.content {
                        Content::Children(c) => match c.as_slice() {
                            $($matching)* => true,
                            _ => false,
                        },
                        _ => false,
                    }})
                );
            }
        };
        (
            $test_name:ident,
            $snippet:expr,
            $kind_fn:expr,
            $($matching:tt)*
        ) => {
            tree_has_node_matching!(
                $test_name,
                $snippet,
                $kind_fn,
                &|_|true,
                $($matching)*
            );
        };
}

pub fn contains_matching(r: &FmtNode, condition: &impl Fn(&FmtNode) -> bool) -> bool {
    dbg!(r);
    condition(r)
        || if let Content::Children(c) = &r.content {
            !c.is_empty() && c.iter().any(|x| contains_matching(x, condition))
        } else {
            false
        }
}

tree_has_node_matching!(
    basic_with_parent,
    "#[ // typstfmt::off\ntext\n// typstfmt::on\n]text",
    |kind| matches!(kind, FmtKind::Markup),
    x if x.iter().find(|c|c.kind == FmtKind::Comment).is_some()
);

tree_has_node_matching!(
    basic,
    "#[ // typstfmt::off\ntext\n// typstfmt::on\n]text",
    |_| true,
    x if x.iter().find(|c|&c.kind == &FmtKind::FuncCall).is_some()
);
