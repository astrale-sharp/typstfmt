use super::*;
use crate::utils::{get_next_ignoring, next_is_ignoring};

// #[instrument(skip_all)]
/// format args using [format_args_tight] or [format_args_breaking] depending on the context.
/// - if number of args is 0, format tight.
/// - if line gets above max_length - 7 in tight mode, format breaking. (see TODO: why plus 7)
// pub(crate) fn format_args(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
//     // if number_of_args == 0 {
//     //     return format_args_tight(parent, children, ctx);
//     // }

//     let res = format_args_tight(parent, children, ctx);
//     // TODO: why plus 7
//     // why plus 7? if you remove it you'll notice the official example
//     // fails, since the inner line is broken before reaching the limit,
//     // it's difficult to have a condition like "if one of my child had
//     // to break in order to not go over the max_len, break" So I had to
//     // resort to this hack. A more meaningful approach is desired.
//     if utils::max_line_length(&res) + 7 >= ctx.config.max_line_length {
//         return format_args_breaking(parent, children, ctx);
//     }
//     res
// }

// pub(crate) fn format_args_tight(
//     parent: &LinkedNode<'_>,
//     children: &[String],
//     ctx: &mut Ctx,
// ) -> String {
//     let mut res = String::new();
//     let mut missing_trailing = parent.kind() == Destructuring;

//     for (s, node) in children.iter().zip(parent.children()) {
//         let is_last =
//             utils::next_is_ignoring(&node, RightParen, &[Space, LineComment, BlockComment]);

//         match node.kind() {
//             _ if ctx.off => res.push_str(node.text()),
//             Space => {}
//             Comma => {
//                 let is_last_comma = utils::find_next(&node, &|x| x.kind() == Comma).is_none();
//                 let is_trailing =
//                     utils::next_is_ignoring(&node, RightParen, &[Space, BlockComment]);

//                 missing_trailing = is_last_comma && !is_trailing;
//                 if utils::next_is_ignoring(&node, RightParen, &[Space]) {
//                     // not putting the comma in would result in a parenthesized expression, not an array
//                     // "(a,) != (a)"
//                     if parent.kind() == Array || parent.kind() == Destructuring {
//                         ctx.push_raw_in(",", &mut res);
//                     }
//                 } else {
//                     ctx.push_raw_in(s, &mut res);
//                     ctx.push_in(" ", &mut res);
//                 }
//             }
//             _ => {
//                 ctx.push_raw_in(s, &mut res);
//                 if is_last && missing_trailing && parent.kind() == Destructuring {
//                     ctx.push_raw_in(",", &mut res);
//                 }
//             }
//         }
//     }
//     res
// }

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

/// there are three ways to go around formatting params
/// 1. Everything tight
/// ```
/// #f(a,b)
/// #f(a, // comment after a
///    b)
/// ```
/// 2. Break everything
/// ```
/// #f(
///   a,
///   b,
/// )
/// ```
/// 3. Some combination
/// ```
/// #f(
///   a,b,c,
///   d,e,f,
/// )
/// #f(
///   a,b, // comment after b
///   c,d,e,
///   f,
/// )
/// ```
/// Giving with trailing block
/// ```
/// #f(a,b,c)[content]
/// #f(
///   a,b,c,
///   d,e,f,
/// )[content]
/// ```
/// The algorithm will be the following:
/// - if args number is 0: tight
/// - if table, use combination with n set as best fit
/// - try tight without trailing blocks,
///     - if too long, abort and go breaking
///     - if size <= max_len continue with the code blocks (recomputed with res in the beginning?)
///         if above max_len, abort and go breaking.
///
/// idea? when breaking group named args
#[instrument(skip_all, ret)]
pub(crate) fn format_args(parent: &LinkedNode<'_>, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    let mut missing_trailing_comma = true;
    let mut is_trailing_block = TrailingBlockDetect::default();
    let mut is_breaking = false;
    // when true, does only trailing blocks
    // when false, does everything but trailing blocks
    // if is breaking, this is ignored.
    let mut trailing_block_mode = false;
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
        is_breaking = false;
    }
    is_breaking |= number_of_args >= 4;
    // first iter try tight without trailing blocks
    loop {
        for (s, node) in children.iter().zip(parent.children()) {
            let is_last =
                utils::next_is_ignoring(&node, RightParen, &[Space, LineComment, BlockComment]);
            match node.kind() {
                _ if ctx.off => {res.push_str(&deep_no_format(&node));
                dbg!(s);
                dbg!(node.kind());
                dbg!(deep_no_format(&node));
            
            },
                ContentBlock
                    if is_trailing_block.is_trailing_block()
                        && (trailing_block_mode || is_breaking) =>
                {
                    ctx.push_raw_in(s, &mut res);
                }
                _ if trailing_block_mode => {}

                LeftParen => {
                    is_trailing_block.left_par = true;
                    ctx.push_raw_in(s, &mut res);
                    if is_breaking {
                        ctx.push_raw_in("\n", &mut res);
                        ctx.push_raw_in(&ctx.get_indent(), &mut res);
                    }
                }
                RightParen => {
                    is_trailing_block.right_par = true;
                    if is_breaking && !res.ends_with('\n') {
                        ctx.push_raw_in("\n", &mut res);
                    }
                    ctx.push_raw_in(s, &mut res);
                }
                LineComment | BlockComment => {
                    let last_is_comment = utils::prev_is_ignoring(&node, LineComment, &[Space])
                        || utils::prev_is_ignoring(&node, BlockComment, &[Space]);
                    // let next_is_comment = utils::next_is_ignoring(&node, LineComment, &[Space])
                    //     || utils::next_is_ignoring(&node, BlockComment, &[Space]);

                    if last_is_comment {
                        utils::eat_space(&mut res);
                        ctx.push_raw_in("\n", &mut res);
                        ctx.push_raw_in(&ctx.get_indent(), &mut res);
                        ctx.push_raw_in(s, &mut res);
                        ctx.push_raw_in("\n", &mut res);
                        continue;
                    } else {
                        // todo, check for comma and put it before

                        utils::eat_space(&mut res);
                        let prev = utils::get_prev_ignoring(&node, &[]);
                        match prev {
                            Some(space) if space.kind() == Space && space.text().contains('\n') && space.prev_sibling_kind() != Some(LeftParen) => {
                                ctx.push_raw_in("\n", &mut res);
                                ctx.push_raw_in(&ctx.get_indent(), &mut res);
                                ctx.push_raw_in(s, &mut res);
                            }
                            _ => {
                                ctx.push_raw_in(" ", &mut res);
                                ctx.push_raw_in(s, &mut res);
                            }
                        }
                    }
                    ctx.push_raw_in("\n", &mut res);

                    if !next_is_ignoring(&node, RightParen, &[Space]) && is_breaking {
                        ctx.push_raw_in(&ctx.get_indent(), &mut res);
                        ctx.just_spaced = true;
                    }
                }
                Space => {}
                // handles trailing comma
                // handles Line comment
                Comma => {
                    let is_last_comma = utils::find_next(&node, &|x| x.kind() == Comma).is_none();
                    let is_trailing = utils::next_is_ignoring(
                        &node,
                        RightParen,
                        &[Space, LineComment, BlockComment],
                    );
                    missing_trailing_comma = is_last_comma && !is_trailing;

                    if is_last_comma && is_trailing {
                        // no indent
                        if is_breaking {
                            ctx.push_raw_in(s, &mut res);
                            ctx.push_raw_in("\n", &mut res);
                        } else if parent.kind() == Destructuring
                            || (parent.kind() == Array && number_of_args == 1)
                        {
                            ctx.push_raw_in(s, &mut res);
                        }
                    } else {
                        ctx.push_raw_in(s, &mut res);
                        if is_breaking {
                            ctx.push_raw_in("\n", &mut res);
                            ctx.push_raw_in(&ctx.get_indent(), &mut res);
                        } else {
                            ctx.push_raw_in(" ", &mut res);
                        }
                    }
                    //  else if !ctx.config.experimental_args_breaking_consecutive
                    //     || res
                    //         .lines()
                    //         .last()
                    //         .is_some_and(|line| utils::max_line_length(&format!("{line}, ")) >= 10)
                    // {
                    //     ctx.push_raw_in(s, &mut res);
                    //     ctx.push_raw_in("\n", &mut res);
                    //     ctx.push_raw_in(&ctx.get_indent(), &mut res);}
                }
                _ => {
                    if is_last {
                        debug!(
                            msg = "is last el!",
                            ?missing_trailing_comma,
                            parent = format!("{:?}", parent.kind())
                        )
                    }
                    if is_breaking {
                        if utils::last_line_length(&res) + utils::first_line_length(s)
                            >= ctx.config.max_line_length
                            && ctx.config.pack_params
                        {
                            utils::eat_space(&mut res);
                            ctx.push_raw_in("\n", &mut res);
                            ctx.push_raw_in(&ctx.get_indent(), &mut res);
                            ctx.push_raw_indent(s, &mut res);
                        } else {
                            ctx.push_raw_indent(s, &mut res);
                        }
                    } else {
                        ctx.push_raw_in(s, &mut res);
                        if is_last
                            && missing_trailing_comma
                            && (parent.kind() == Destructuring
                                || (parent.kind() == Array && number_of_args == 1))
                        {
                            debug!("pushed last missing trailing (tight)");
                            ctx.push_raw_in(",", &mut res);
                        }
                    }

                    if is_last
                        && missing_trailing_comma
                        && is_breaking
                        && parent.kind() != Parenthesized
                    {
                        debug!("pushed last missing trailing (breaking)");
                        ctx.push_raw_in(",", &mut res);
                        ctx.push_raw_in("\n", &mut res);
                    }
                }
            }
        }
        debug!(
            ?is_breaking,
            ?trailing_block_mode,
            msg = "ran with:",
            res = res
        );
        // after 1st iter
        if ctx.off {
            break
        }
        if !is_breaking && !trailing_block_mode {
            if utils::max_line_length(&res) > ctx.config.max_line_length {
                res = String::new();
                is_breaking = true;
                debug!("abort and go breaking");
                continue;
            } else {
                trailing_block_mode = true;
                debug!("re run trailing blocks only");
                continue;
            }
        }
        if !is_breaking && trailing_block_mode {
            debug!("finished tight");
            break;
        }
        if is_breaking {
            debug!("finished breaking");
            break;
        }
    }
    res
}
