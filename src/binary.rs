use super::*;

/// only format tight cause it would not be supported to format breaking in code blocks
///
/// it could be supported in parenthesized.
#[instrument(skip_all, ret)]
pub(crate) fn format_bin_left_assoc(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let res = format_bin_left_assoc_tight(parent, children, ctx);

    if parent.kind() == Parenthesized
        && (
            crate::utils::max_line_length(&res) >= ctx.config.max_line_length
            // || parent
            // .children()
            // .any(|x| x.kind() == LineComment || x.kind() == BlockComment)
        )
    {
        return format_bin_left_assoc_breaking(parent, children, ctx);
    }
    res
}

/// not integrated and never used for now
#[instrument(skip_all)]
pub(crate) fn format_bin_left_assoc_breaking(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            x if BinOp::from_kind(x).is_some() => {
                ctx.push_in("\n", &mut res);
                ctx.push_raw_indent(s, &mut res);
            }
            LineComment | BlockComment => {
                ctx.push_raw_indent(s, &mut res);
                ctx.push_in("\n", &mut res);
            }
            Space => {}
            _ => {
                ctx.push_in(s, &mut res);
            }
        }
    }
    res
}

#[instrument(skip_all)]
pub(crate) fn format_bin_left_assoc_tight(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),

            x if BinOp::from_kind(x).is_some() => {
                ctx.push_in(" ", &mut res);
                ctx.push_raw_in(s, &mut res);
                ctx.push_in(" ", &mut res);
            }
            // handles `not in` like a pro
            Not => {
                ctx.push_in(" ", &mut res);
                ctx.push_raw_in(s, &mut res);
            }
            Space => {}
            LineComment => {
                ctx.push_in(" ", &mut res);
                ctx.push_raw_in(s, &mut res);
                ctx.push_in("\n", &mut res);
                ctx.push_raw_in(&ctx.get_indent(), &mut res);
                ctx.just_spaced = true;
            }
            _ => {
                ctx.push_raw_in(s, &mut res);
            }
        }
    }
    res
}
