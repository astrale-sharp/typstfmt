use super::*;

pub(crate) fn format_code_blocks(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    debug!("format code_blocks!");
    let mut res = format_code_blocks_tight(parent, children, ctx);

    if max_line_length(&res) >= ctx.config.max_line_length {
        debug!("format_args breaking");
        res = format_code_blocks_breaking(parent, children, ctx);
        return res;
    }
    debug!("format_code_blocks tight");
    res
}

pub(crate) fn format_code_blocks_tight(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    debug!("::format_code_blocks_tight");
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            LeftBrace => {
                let code = node.next_sibling().unwrap();
                assert!(code.kind() == Code);
                let non_space_child = find_child(&code, &|c| c.kind() != Space);
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

pub(crate) fn format_code_blocks_breaking(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    debug!("::format_code_blocks_tight");
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            LeftBrace => {
                res.push_str(&format!("{s}\n{}", ctx.get_indent()));
            }
            RightBrace => {
                res.push_str(&format!("\n{s}"));
            }
            Space => {
                // check if ok, can iter
                ctx.push_in(s, &mut res);
            }
            _ => {
                ctx.push_raw_indent(s, &mut res);
                // ctx.push_raw_in(s, &mut res);
            }
        }
    }
    res
}
