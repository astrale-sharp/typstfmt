use super::*;
use typst_syntax::ast::AstNode;

#[instrument(skip_all)]
pub(crate) fn format_content_blocks(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    let markup = parent
        .cast_first_match::<typst_syntax::ast::Markup>()
        .unwrap_or_default();
    let first_space = markup.as_untyped().children().next();
    let spaced = first_space.is_some_and(|x| x.kind() == Space);

    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            LineComment | BlockComment => {
                let buf = format_comment_handling_disable(&node, &[], ctx);
                ctx.push_raw_in(&buf, &mut res);
            }
            RightBracket if spaced => {
                let space_type = if first_space.unwrap().text().contains('\n') {
                    '\n'
                } else {
                    ' '
                };
                if !res.ends_with(space_type) {
                    while res.ends_with('\n') || res.ends_with(' ') {
                        res.replace_range(res.len() - 1..res.len(), "");
                    }
                    res.push(space_type);
                }
                ctx.push_raw_in(s, &mut res)
            }
            _ => ctx.push_raw_indent(s, &mut res),
        }
    }
    res
}

// break lines so they won't go over max_line_length
#[instrument(skip_all)]
pub(crate) fn format_markup(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    let mut skip_until = None;
    let parent_is_list = [EnumItem, ListItem, TermItem]
        .map(Some)
        .contains(&parent.parent_kind());

    for (idx, (s, node)) in children.iter().zip(parent.children()).enumerate() {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()), // todo, interaction with line below?
            _ if skip_until.is_some_and(|skip| idx <= skip) => {}
            LineComment | BlockComment => {
                let buf = format_comment_handling_disable(&node, &[], ctx);
                if ctx.off
                    && [Space, Parbreak]
                        .map(Some)
                        .contains(&utils::get_prev_ignoring(&node, &[]).map(|x| x.kind()))
                {
                    let s = utils::get_prev_ignoring(&node, &[])
                        .map(|x| x.text().to_string())
                        .unwrap_or_default();
                    let s = s.split('\n').last().unwrap_or_default();
                    ctx.push_raw_in(s, &mut res);
                }
                ctx.push_raw_in(&buf, &mut res);
            }
            Space => {
                if idx == 0
                    || idx == children.len()
                    || node.prev_sibling_kind() == Some(Linebreak)
                    || [Text, Parbreak, SmartQuote, Hashtag]
                        .map(Some)
                        .contains(&node.next_sibling_kind())
                    || ![Text, Parbreak]
                        .map(Some)
                        .contains(&node.prev_sibling_kind())
                    || [EnumItem, ListItem, TermItem]
                        .map(Some)
                        .contains(&node.next_sibling_kind())
                {
                    ctx.push_raw_in(s, &mut res);
                }
            }
            Text => {
                // We eat all the following nodes if they're in `[Space, Text, Emph, Strong, Label, Ref]`
                // then we format ourselves breaking or spacing.
                skip_until = Some(idx);
                let mut this = node;
                let mut add = s.to_string();
                loop {
                    let next = utils::find_next(&this, &|_| true);
                    match next.as_ref() {
                        Some(next) => {
                            if ![Space, Text, Emph, Strong, Label, Ref].contains(&next.kind()) {
                                break;
                            }
                            if next.kind() == Space
                                && [EnumItem, ListItem, TermItem, SmartQuote, Hashtag]
                                    .map(Some)
                                    .contains(&next.next_sibling_kind())
                            {
                                break;
                            }
                        }
                        None => break,
                    }

                    *skip_until.as_mut().unwrap() += 1;
                    this = next.unwrap();
                    match this.kind() {
                        Space => add.push(' '),
                        _ => add.push_str(&children[skip_until.unwrap()]),
                    }
                }
                let add = add.split(' ').filter(|x| !x.is_empty()).collect_vec();
                for word in add.iter() {
                    if utils::max_line_length(word)
                    + 1 // the space we're adding
                    + utils::max_line_length(res.split('\n').last().unwrap_or(""))
                        <= ctx.config.max_line_length
                        || (parent.parent_kind() == Some(Heading))
                    // we can't break in headings, it would break the ast
                    {
                        ctx.push_raw_in(word, &mut res);
                        ctx.push_in(" ", &mut res);
                    } else {
                        if res.ends_with(' ') {
                            res = res[..res.len() - 1].to_string();
                        }
                        ctx.push_in("\n", &mut res);
                        ctx.push_raw_in(word, &mut res);
                        ctx.push_in(" ", &mut res);
                    }
                }
                // we don't want to end with a space nor to see `don 't`
                if (res.ends_with(' ') || res.ends_with('\n'))
                    && (this.next_sibling().is_none() || this.next_sibling_kind() == Some(Hashtag))
                    || [Some(Text), Some(SmartQuote)].contains(&this.next_sibling_kind())
                {
                    res = res[..res.len() - 1].to_string();
                }

                if parent_is_list && this.next_sibling_kind().is_none() {
                    ctx.push_in("\n", &mut res);
                }
            }
            _ => {
                ctx.push_raw_in(s, &mut res);
            }
        }
    }

    ctx.lost_context();
    res
}
