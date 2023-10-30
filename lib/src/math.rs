use super::*;
use crate::context::Ctx;
use crate::format_comment_handling_disable;
use std::cmp::max;

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

    let align_points: Vec<usize> = retrieve_align_point(parent, children);
    let mut index = 0;
    let mut position = 0usize;

    let mut first_align = true;
    let mut should_indent = false;

    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            MathAlignPoint => {
                debug_assert!(
                    align_points[index] >= position,
                    "align point {} is smaller than position {}",
                    align_points[index],
                    position
                );

                if position == 0 && first_align {
                    should_indent = true;
                }

                if position == 0 && should_indent {
                    ctx.push_raw_in(ctx.get_indent().as_str(), &mut res);
                }

                ctx.push_raw_in(
                    " ".repeat(align_points[index] - position).as_str(),
                    &mut res,
                );
                ctx.push_raw_in(s, &mut res);
                position = align_points[index] + s.len();
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
                position += s.len();
                ctx.push_raw_in(s, &mut res)
            }
        }
    }

    res
}

fn retrieve_align_point(parent: &LinkedNode, children: &[String]) -> Vec<usize> {
    let mut align_points: Vec<usize> = vec![];
    let mut index = 0;
    let mut position = 0usize;

    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            MathAlignPoint => {
                if align_points.len() <= index {
                    align_points.push(position);

                    position += s.len();
                } else {
                    align_points[index] = max(align_points[index], position);
                }
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
                position += s.len();
            }
        }
    }
    align_points
}
