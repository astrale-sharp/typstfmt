use super::*;

#[instrument(skip_all)]
pub(crate) fn format_args(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = format_args_one_line(parent, children, ctx);

    if children.iter().any(|c| c.contains('\n')) {
        return format_args_breaking(parent, children, ctx);
    }

    let number_of_args = parent
        .children()
        .filter_map(|node| {
            if [Comma, Space, LeftParen, RightParen].contains(&node.kind()) {
                None
            } else {
                Some(node)
            }
        })
        .count();

    if number_of_args <= 1 {
        return res;
    }

    if utils::max_line_length(&res) >= ctx.config.max_line_length {
        debug!("format_args::breaking");
        res = format_args_breaking(parent, children, ctx);
        return res;
    }
    debug!("format_args::one_line");
    res
}

pub(crate) fn format_args_one_line(
    parent: &LinkedNode<'_>,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            Space => {}
            Comma => {
                if utils::is_trailing_comma(&node) {
                    // don't print
                } else {
                    ctx.push_raw_in(s, &mut res);
                    ctx.push_in(" ", &mut res);
                }
            }
            _ => {
                ctx.push_raw_in(&s, &mut res);
            }
        }
    }
    res
}

pub(crate) fn format_args_breaking(
    parent: &LinkedNode<'_>,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    debug!("format arg breaking");
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        debug!("formatting kind: {:?}", node.kind());
        match node.kind() {
            LeftParen => {
                res.push_str(s);
                res.push('\n');
                res.push_str(&ctx.get_indent());
            }
            Space => {}
            Comma => {
                // print the last comma but don't indent
                if utils::is_last_comma(&node) && utils::is_trailing_comma(&node) {
                    ctx.push_raw_in(&s, &mut res);
                    ctx.push_in("\n", &mut res);
                } else {
                    ctx.push_raw_in(&format!("{s}\n{}", ctx.get_indent()), &mut res);
                }
            }
            _ => {
                // cannot be a comma
                // so last and no trailing comma, adding a trailing comma.
                debug!(
                    "decide if indent according to: {:?}",
                    utils::next_is_ignoring(&node, RightParen, &[Space])
                );
                //or s contains n
                if utils::next_is_ignoring(&node, RightParen, &[Space]) || s.contains('\n') {
                    ctx.push_raw_indent(s, &mut res);
                    ctx.push_raw_in(",\n", &mut res);
                } else {
                    ctx.push_raw_in(s, &mut res);
                }
            }
        }
    }
    res
}
