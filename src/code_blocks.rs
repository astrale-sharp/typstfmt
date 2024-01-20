use super::*;
use crate::utils::{get_next_ignoring, next_is_ignoring};

#[instrument(skip_all)]
/// format code blocks using [format_code_blocks_breaking] or [format_code_blocks_tight]
/// depending on the context.
pub(crate) fn format_code_blocks(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let children_contains_lines = children.iter().any(|c| c.contains('\n'));
    let parent_is_loop = [Some(ForLoop), Some(WhileLoop)].contains(&parent.parent_kind());
    let code = utils::find_child(parent, &|x| x.kind() == Code).unwrap();

    if parent.children().any(|c| c.kind() == LineComment) {
        return format_code_blocks_breaking(parent, children, ctx);
    }

    if code.is_empty() || code.children().all(|c| c.kind() == Space) {
        return format_code_blocks_tight(parent, children, ctx);
    }

    if children_contains_lines || parent_is_loop {
        debug!("format breaking cause: children contains breakline: {children_contains_lines}");
        debug!("or because parent is loop: {parent_is_loop}");
        return format_code_blocks_breaking(parent, children, ctx);
    }

    let res = format_code_blocks_tight(parent, children, ctx);
    let max_line_length = utils::max_line_length(&res);

    if max_line_length >= ctx.config.max_line_length {
        debug!(
            "format breaking cause max_line_length ({}) above limit",
            max_line_length
        );
        return format_code_blocks_breaking(parent, children, ctx);
    }
    debug!("format tight");
    res
}

#[instrument(skip_all)]
pub(crate) fn format_code_blocks_tight(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            LeftBrace => {
                let code = node.next_sibling().unwrap();
                assert!(code.kind() == Code);
                let non_space_child = utils::find_child(&code, &|c| c.kind() != Space);
                let code_is_empty = non_space_child.is_none();
                if code_is_empty {
                    debug!("format_empty_code_block and exit");
                    ctx.push_raw_in("{}", &mut res);
                    break;
                }
                debug!("leftbrace formatted not empty!");
                res.push_str(s);
                res.push(' ');
            }
            RightBrace => {
                res.push(' ');
                res.push_str(s);
            }
            Space => {}
            _ => {
                ctx.push_raw_in(s, &mut res);
            }
        }
    }
    res
}

#[instrument(skip_all, ret)]
pub(crate) fn format_code_blocks_breaking(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(&deep_no_format(&node)),
            LeftBrace => {
                res.push_str(&format!("{s}\n{}", ctx.get_indent()));
                ctx.just_spaced = true;
            }
            LineComment | BlockComment => {
                let buf = format_comment_handling_disable(&node, &[], ctx);
                if ctx.off {
                    ctx.push_raw_in(&buf, &mut res);
                    continue;
                }
                if utils::prev_is_ignoring(&node, LineComment, &[Space])
                    || utils::prev_is_ignoring(&node, BlockComment, &[Space])
                {
                    ctx.push_raw_in(s, &mut res);
                    if !utils::next_is_ignoring(&node, RightBrace, &[Space]) {
                        ctx.push_in("\n", &mut res);
                        ctx.consec_new_line = 2;
                    }
                } else {
                    let prev = node.prev_sibling().unwrap();
                    let mark = res.rfind(|x| x != ' ' && x != '\n').unwrap() + 1;
                    let prev_maybe_space = get_next_ignoring(&prev, &[]);
                    // go back before
                    res = res[..mark].to_string();
                    match prev_maybe_space {
                        Some(space) if space.kind() == Space && space.text().contains('\n') => {
                            res.push('\n');
                            res.push_str(&ctx.get_indent());
                            res.push_str(s.trim_start());

                            if !utils::next_is_ignoring(&node, RightBrace, &[Space]) {
                                ctx.push_in("\n", &mut res);
                                ctx.consec_new_line = 2;
                                ctx.just_spaced = true;
                            }
                        }
                        _ => {
                            res.push(' ');
                            res.push_str(s);
                            if !utils::next_is_ignoring(&node, RightBrace, &[Space]) {
                                ctx.push_in("\n", &mut res);
                                ctx.consec_new_line = 2;
                            }
                        }
                    }
                }

                if !next_is_ignoring(&node, RightBrace, &[Space]) {
                    ctx.push_raw_in(&ctx.get_indent(), &mut res);
                }
            }
            RightBrace => {
                ctx.push_in("\n", &mut res);
                res.push_str(s);
            }
            Space => {}
            _ => {
                ctx.push_raw_indent(s, &mut res);
            }
        }
    }
    ctx.lost_context();
    res
}
