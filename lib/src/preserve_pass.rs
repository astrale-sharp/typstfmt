use super::node::{Content, FmtKind, FmtNode};

/// Modifies the tree to isolate preserve nodes
/// of `// typstfmt::off` matching `// typstfmt::on`
///
/// The boolean returned is a preserve flag, the level of preserved nesting.
///
/// Do we need to be able to say I handled my children
///
/// Example:
/// let our snippet correspond to
/// ```ignore
/// f(
///     // t::off
///     f([text  text  text]),2,
///  
/// )
/// // t::on
/// ```
///
/// Here, `f([text  text  text])` wouldn't be tag as Preserve cause it has children
/// and it's parent is not preserved either so we need else
pub(crate) fn preserve_pass(node: &mut FmtNode) -> PreserveData {
    match node.kind {
        FmtKind::Comment => {
            let text = node.text().unwrap();
            if text.contains("typstfmt::off") {
                return PreserveData::new(1, false);
            } else if text.contains("typstfmt::on") {
                return PreserveData::new(-1, false);
            }
        }
        _ => {}
    }

    match &mut node.content {
        Content::Text(_) => PreserveData::default(),
        Content::Children(c) => {
            let mut node_data = PreserveData::new(0, true);
            c.iter_mut().for_each(|c| {
                let child_data = preserve_pass(c);
                // we ignore typstfmt::on if we weren't preserving already
                node_data.preserve_idx = (node_data.preserve_idx + child_data.preserve_idx).max(0);

                #[allow(unused_doc_comments)]
                /// cases: one of my child starts the p process, my parent must not handle me
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
