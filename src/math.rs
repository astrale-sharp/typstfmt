use super::*;
use crate::context::Ctx;
use crate::format_comment_handling_disable;
use unicode_width::UnicodeWidthStr as _;

#[instrument(skip_all)]
pub(crate) fn format_equation(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    let first_space = parent.children().nth(1);
    let space_type = if first_space
        .as_ref()
        .is_some_and(|s| s.text().contains('\n'))
    {
        "\n"
    } else {
        " "
    };

    let mut first_dollar = true;

    let newline = space_type == "\n";

    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            LineComment | BlockComment => {
                let buf = format_comment_handling_disable(&node, &[], ctx);
                ctx.push_raw_in(&buf, &mut res);
            }
            Dollar if first_dollar => {
                first_dollar = false;
                ctx.push_raw_in(s, &mut res);
            }
            Math => {
                if newline {
                    ctx.push_raw_in(ctx.get_indent().as_str(), &mut res);
                    ctx.push_raw_indent(s, &mut res);
                } else {
                    ctx.push_raw_in(s, &mut res);
                }
            }
            Space => {
                ctx.push_raw_in(space_type, &mut res);
            }
            _ => ctx.push_raw_indent(s, &mut res),
        }
    }
    res
}

#[instrument(skip_all)]
pub(crate) fn format_math(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();

    let align_columns = retrieve_align_columns(parent, children);
    let mut index = 0;
    let mut position = 0usize;

    let mut first_align = true;
    let mut should_indent = false;

    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            MathAlignPoint => {
                debug_assert!(
                    align_columns[index] >= position,
                    "align column {} is smaller than position {}",
                    align_columns[index],
                    position
                );

                if position == 0 && first_align {
                    should_indent = true;
                }

                if position == 0 && should_indent {
                    ctx.push_raw_in(ctx.get_indent().as_str(), &mut res);
                }

                ctx.push_raw_in(
                    " ".repeat(align_columns[index] - position).as_str(),
                    &mut res,
                );
                ctx.push_raw_in(s, &mut res);
                position = align_columns[index] + s.width();
                index += 1;

                first_align = false;
            }
            Space if s.contains('\n') => {
                position = 0;
                index = 0;
                ctx.push_raw_in(s, &mut res);
            }
            Space => {
                position += 1;
                ctx.push_raw_in(" ", &mut res);
            }
            _ => {
                position += s.width();
                ctx.push_raw_in(s, &mut res)
            }
        }
    }

    res
}

/// Calculate the columns the alignment points in the math block should be placed at.
/// The n-th alignment point on a line should be placed at `align_columns[n]`.
fn retrieve_align_columns(parent: &LinkedNode, children: &[String]) -> Vec<usize> {
    let mut align_columns: Vec<usize> = vec![];
    let mut index = 0;
    let mut position = 0usize;

    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            MathAlignPoint => {
                if align_columns.len() <= index {
                    align_columns.push(position);
                } else if align_columns[index] < position {
                    // We found an alignment point that is further away.
                    // We now need to shift this and all the columns after it.
                    let shift = position - align_columns[index];
                    align_columns
                        .iter_mut()
                        .skip(index)
                        .for_each(|pos| *pos += shift);
                } else {
                    // When formatting, this alignment point will be shifted.
                    // Pretend that we inserted the whitespace to ensure correct positions.
                    position = align_columns[index];
                }
                position += s.len();
                index += 1
            }
            Space if s.contains('\n') => {
                position = 0;
                index = 0;
            }
            Space => {
                position += 1;
            }
            _ => {
                position += s.width();
            }
        }
    }
    align_columns
}
