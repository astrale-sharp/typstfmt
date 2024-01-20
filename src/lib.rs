#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::dbg_macro
)]

use itertools::Itertools;
use tracing::debug;
use tracing::instrument;
use tracing::warn;
use typst_syntax::ast::BinOp;
use typst_syntax::SyntaxKind;
use typst_syntax::SyntaxKind::*;
use typst_syntax::{parse, LinkedNode};
use Option::None;

mod config;

pub use config::Config;

mod context;

use context::Ctx;

mod utils;

mod binary;
mod code_blocks;
mod markup;
mod math;
mod params;

#[must_use]
pub fn format(s: &str, config: Config) -> String {
    //replace tabs
    let s = &s.replace('\t', &" ".repeat(config.indent_space));

    let init = parse(s);
    let mut context = Ctx::from_config(config);
    let root = LinkedNode::new(&init);
    let s = visit(&root, &mut context);
    regex::Regex::new("( )+\n")
        .unwrap()
        .replace_all(&s, "\n")
        .to_string()
}

/// This is recursively called on the AST, the formatting is bottom up,
/// nodes will decide based on the size of their children and the max line length
/// how they will be formatted.
///
/// One assumed rule is that no kind should be formatting with surrounded space
#[instrument(skip_all, name = "V", fields(kind = format!("{:?}",node.kind())))]
fn visit(node: &LinkedNode, ctx: &mut Ctx) -> String {
    let mut res: Vec<String> = vec![];
    for child in node.children() {
        let child_fmt = visit(&child, ctx);
        res.push(child_fmt);
    }
    let res = match node.kind() {
        LineComment => format_comment_handling_disable(node, &res, ctx),
        _ if ctx.off => no_format(node, &res, ctx),
        Binary => binary::format_bin_left_assoc(node, &res, ctx),
        Named | Keyed => format_named_args(node, &res, ctx),
        ListItem | EnumItem | TermItem => format_list_enum(node, &res, ctx),
        CodeBlock => code_blocks::format_code_blocks(node, &res, ctx),
        Markup => markup::format_markup(node, &res, ctx),
        ContentBlock => markup::format_content_blocks(node, &res, ctx),
        Args | Params | Dict | Array | Destructuring | Parenthesized => {
            params::format_args(node, &res, ctx)
        }
        LetBinding => format_let_binding(node, &res, ctx),
        Conditional => conditional_format(node, &res, ctx),
        Raw | BlockComment => {
            ctx.lost_context();
            node.text().to_string()
        }
        Equation => math::format_equation(node, &res, ctx),
        Math => math::format_math(node, &res, ctx),
        Str => no_format(node, &res, ctx),
        _ => format_default(node, &res, ctx),
    };
    if node.children().count() == 0 {
        debug!("TOKEN : {:?}", node.kind());
    } else {
        debug!("PARENT: {:?}", node.kind());
    }
    res
}

/// formats a node for which no specific function was found. Last resort.
/// For the text of the node:
/// Trim spaces for Space nodes if they contain a linebreak.
/// avoids:
/// - putting two consecutive spaces.
/// - putting more than two consecutive newlines.
///
/// For the already formatted children, change nothing.
#[instrument(skip_all, ret)]
fn format_default(node: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    ctx.push_in(node.text(), &mut res);
    for s in children {
        ctx.push_raw_in(s, &mut res);
    }
    res
}

fn no_format(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    ctx.push_raw_in(parent.text(), &mut res);
    for s in children {
        ctx.push_raw_in(s, &mut res);
    }
    res
}

fn deep_no_format(parent: &LinkedNode) -> String {
    let mut res: Vec<String> = vec![];
    for child in parent.children() {
        let child_fmt = deep_no_format(&child);
        res.push(child_fmt);
    }
    no_format(parent, &res, &mut Ctx::default())
}

fn conditional_format(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    ctx.push_raw_in(parent.text(), &mut res);
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            Space => {}
            If => {
                ctx.push_raw_in(s, &mut res);
                ctx.push_raw_in(" ", &mut res);
            }
            CodeBlock | ContentBlock => {
                ctx.push_raw_in(" ", &mut res);
                ctx.push_raw_in(s, &mut res);
            }
            Else => {
                ctx.push_raw_in(" ", &mut res);
                ctx.push_raw_in(s, &mut res);
                if node.next_sibling_kind() == Some(Conditional) {
                    ctx.push_raw_in(" ", &mut res);
                }
            }
            _ => ctx.push_raw_in(s, &mut res),
        }
    }
    res
}

#[instrument(skip_all, ret)]
pub(crate) fn format_named_args(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            Show | Set => {
                ctx.push_raw_in(s, &mut res);
                ctx.push_in(" ", &mut res);
            }
            Colon => res.push_str(": "),
            Space => {}
            LineComment | BlockComment => ctx.push_raw_in(s, &mut res),
            _ => {
                ctx.push_raw_in(s, &mut res);
            }
        }
    }
    res
}

#[instrument(skip_all, ret)]
pub(crate) fn format_let_binding(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            Eq => {
                ctx.push_in(" ", &mut res);
                ctx.push_in(s, &mut res);
                ctx.push_in(" ", &mut res);
            }
            Space => ctx.push_in(s, &mut res),
            _ => {
                ctx.push_raw_in(s, &mut res);
            }
        }
    }
    res
}

fn format_comment_handling_disable(parent: &LinkedNode, _: &[String], ctx: &mut Ctx) -> String {
    ctx.lost_context();
    if parent.text().contains("typstfmt::off") {
        ctx.off = true;
    } else if parent.text().contains("typstfmt::on") {
        ctx.off = false;
    } else if parent.text().contains("typstfmt::") {
        warn!("your comment contains `typstfmt::` not followed by `on` or `off`, did you make a typo?");
    }
    parent.text().to_string()
}

fn format_list_enum(parent: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
            _ if ctx.off => res.push_str(node.text()),
            EnumMarker | ListMarker | TermMarker => {
                ctx.push_raw_in(node.text(), &mut res);
            }
            _ => {
                ctx.push_raw_indent(s, &mut res);
            }
        }
    }
    res
}

#[cfg(test)]
mod tests;
