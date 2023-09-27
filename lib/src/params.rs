use typst_syntax::ast::AstNode;

use super::*;
use crate::utils::{get_next_ignoring, next_is_ignoring, Btype};

#[instrument(skip_all)]
/// format args using [format_args_tight] or [format_args_breaking] depending on the context.
/// - if number of args is 0, format tight.
/// - if line gets above max_length - 7 in tight mode, format breaking. (see TODO: why plus 7)
pub(crate) fn format_args(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    // check if any children is markup and contains a linebreak, if so, breaking
    // let mut res = vec![];
    // utils::find_children(&mut res, parent, &|c| {
    //     c.parent_kind() == Some(Markup)
    //         && (c.kind() == Parbreak || (c.kind() == Space) && c.text().contains('\n'))
    // });

    // if !res.is_empty() {
    //     return format_args_breaking(parent, children, ctx);
    // }

    if parent.children().any(|c| c.kind() == LineComment) {
        return format_args_breaking(parent, children, ctx);
    }

    let number_of_args = parent
        .children()
        .filter_map(|node| {
            if [
                Comma,
                Space,
                LeftParen,
                RightParen,
                LineComment,
                BlockComment,
            ]
            .contains(&node.kind())
            {
                None
            } else {
                Some(node)
            }
        })
        .count();

    if number_of_args == 0 {
        return format_args_tight(parent, children, ctx);
    }

    let res = format_args_tight(parent, children, ctx);
    // TODO: why plus 7
    // why plus 7? if you remove it you'll notice the official example
    // fails, since the inner line is broken before reaching the limit,
    // it's difficult to have a condition like "if one of my child had
    // to break in order to not go over the max_len, break" So I had to
    // resort to this hack. A more meaningful approach is desired.
    if utils::max_line_length(&res) + 7 >= ctx.config.max_line_length {
        return format_args_breaking(parent, children, ctx);
    }
    res
}

pub(crate) fn format_args_tight(
    parent: &LinkedNode<'_>,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    let is_destruct_and_one_arg = typst_syntax::ast::Destructuring::from_untyped(parent)
        .is_some_and(|x| x.bindings().count() == 1);
    let mut missing_trailing = is_destruct_and_one_arg;

    for (s, node) in children.iter().zip(parent.children()) {
        let is_last =
            utils::next_is_ignoring(&node, RightParen, &[Space, LineComment, BlockComment]);

        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            Space => {}
            Comma => {
                let is_last_comma = utils::find_next(&node, &|x| x.kind() == Comma).is_none();
                let is_trailing =
                    utils::next_is_ignoring(&node, RightParen, &[Space, BlockComment]);

                missing_trailing = is_last_comma && !is_trailing;
                if utils::next_is_ignoring(&node, RightParen, &[Space]) {
                    // not putting the comma in would result in a parenthesized expression, not an array
                    // "(a,) != (a)"
                    if parent.kind() == Array || is_destruct_and_one_arg {
                        ctx.push_raw_in(",", &mut res);
                    }
                } else {
                    ctx.push_raw_in(s, &mut res);
                    ctx.push_in(" ", &mut res);
                }
            }
            _ => {
                ctx.push_raw_in(s, &mut res);
                if is_last && missing_trailing && is_destruct_and_one_arg {
                    ctx.push_raw_in(",", &mut res);
                }
            }
        }
    }
    res
}

#[derive(Debug, Default)]
struct TrailingBlockDetect {
    pub left_par: bool,
    pub right_par: bool,
}

impl TrailingBlockDetect {
    fn is_trailing_block(&self) -> bool {
        (!self.left_par && !self.right_par) || (self.left_par && self.right_par)
    }
}

pub(crate) fn format_args_breaking(
    parent: &LinkedNode<'_>,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    let mut is_trailing_block = TrailingBlockDetect::default();
    let is_block_math = utils::block_type(parent) == Btype::Math;
    let is_parenthesized = parent.kind() == Parenthesized;
    let mut missing_trailing_comma = !(is_parenthesized || is_block_math);
    // only used with experimental flag in config for now
    let mut consecutive_items = 0;

    for (s, node) in children.iter().zip(parent.children()) {
        let is_last =
            utils::next_is_ignoring(&node, RightParen, &[Space, LineComment, BlockComment]);
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            LeftParen => {
                is_trailing_block.left_par = true;
                ctx.push_raw_in(s, &mut res);
                ctx.push_raw_in("\n", &mut res);
                ctx.push_raw_in(&ctx.get_indent(), &mut res);
            }
            RightParen => {
                is_trailing_block.right_par = true;
                if is_parenthesized || is_block_math {
                    // no trailing comma we don't have a newline!
                    ctx.push_in("\n", &mut res);
                }
                ctx.push_raw_in(s, &mut res);
            }
            LineComment | BlockComment => {
                consecutive_items = 0;
                if utils::prev_is_ignoring(&node, LineComment, &[Space])
                    || utils::prev_is_ignoring(&node, BlockComment, &[Space])
                {
                    ctx.push_raw_in(s, &mut res);
                    ctx.push_in("\n", &mut res);
                } else {
                    let prev = node.prev_sibling().unwrap();
                    let mark = res.rfind(|x| x != ' ' && x != '\n').unwrap() + 1;
                    let prev_maybe_space = get_next_ignoring(&prev, &[]);
                    res = res[..mark].to_string();

                    match prev_maybe_space {
                        Some(space) if space.kind() == Space && space.text().contains('\n') => {
                            ctx.push_raw_in("\n", &mut res);
                            ctx.push_raw_in(&ctx.get_indent(), &mut res);
                            ctx.push_raw_in(s, &mut res);
                        }
                        _ => {
                            ctx.push_raw_in(" ", &mut res);
                            ctx.push_raw_in(s, &mut res);
                        }
                    }
                    ctx.push_raw_in("\n", &mut res);
                    ctx.consec_new_line = 2;
                }

                if !next_is_ignoring(&node, RightParen, &[Space]) {
                    ctx.push_raw_in(&ctx.get_indent(), &mut res);
                    ctx.just_spaced = true;
                }
            }
            Space => {}
            // handles trailing comma
            // handles Line comment
            Comma => {
                let is_last_comma = utils::find_next(&node, &|x| x.kind() == Comma).is_none();
                let is_trailing =
                    utils::next_is_ignoring(&node, RightParen, &[Space, LineComment, BlockComment]);
                missing_trailing_comma = is_last_comma && !is_trailing;

                if is_last_comma && is_trailing {
                    // no indent
                    ctx.push_raw_in(s, &mut res);
                    ctx.push_raw_in("\n", &mut res);
                } else if !(ctx.config.experimental_args_breaking_consecutive || is_block_math)
                    || consecutive_items >= 3
                    || s.contains('\n')
                    || res
                        .lines()
                        .last()
                        .is_some_and(|line| utils::max_line_length(&format!("{line}, ")) >= 10)
                {
                    ctx.push_raw_in(s, &mut res);
                    ctx.push_raw_in("\n", &mut res);
                    ctx.push_raw_in(&ctx.get_indent(), &mut res);

                    consecutive_items = 0;
                } else {
                    consecutive_items += 1;
                    ctx.push_raw_in(s, &mut res);
                    ctx.push_raw_in(" ", &mut res);
                }
            }
            ContentBlock if is_trailing_block.is_trailing_block() => ctx.push_raw_in(s, &mut res),
            _ => {
                ctx.push_raw_indent(s, &mut res);
                if is_last && missing_trailing_comma {
                    ctx.push_raw_in(",\n", &mut res);
                }
            }
        }
    }
    res
}
