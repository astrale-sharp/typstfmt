#![doc = include_str!("../README.md")]


use itertools::Itertools;
use tracing::debug;
use tracing::instrument;
use typst::syntax::SyntaxKind;
use typst::syntax::SyntaxKind::*;
use typst::syntax::{parse, LinkedNode};
use Option::None;

mod config;
use config::Config;
mod utils;

//formatting stuff starts here
mod args;
mod code_blocks;

#[derive(Default)]
struct Ctx {
    config: Config,
    just_spaced: bool,
    consec_new_line: i32,
}

/// you may push into your own buffer using this to ensure you push considering context
///
/// you may then push said buffer the final result.
impl Ctx {
    fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Trim spaces for Space nodes if they contain a linebreak.
    /// avoids:
    /// - putting two consecutive spaces.
    /// - putting more than two consecutive newlines.
    #[instrument(skip_all)]
    fn push_in(&mut self, s: &str, res: &mut String) {
        let s = if s.contains('\n') {
            s.trim_end_matches(' ')
        } else {
            s
        };
        for c in s.chars() {
            match c {
                ' ' => {
                    if self.just_spaced {
                        debug!("IGNORED space");
                    } else {
                        debug!("PUSHED SPACE");
                        self.just_spaced = true;
                        res.push(' ');
                    }
                }
                '\n' => {
                    if self.consec_new_line <= 1 {
                        debug!("PUSHED NEWLINE");
                        self.consec_new_line += 1;
                        res.push('\n')
                    } else {
                        debug!("IGNORED newline");
                    }
                }
                _ => {
                    // debug!("PUSHED {c}");
                    res.push(c);
                    self.lost_context();
                }
            }
        }
    }

    /// makes the context aware it missed info,
    /// should be called when pushing directly in result.
    fn push_raw_in(&mut self, s: &str, result: &mut String) {
        debug!("PUSH_RAW: {s:?}");
        result.push_str(s);
        self.lost_context()
    }

    /// adds an indentation for each line the input except the first to match the current level of identation.
    fn push_raw_indent(&mut self, s: &str, result: &mut String) {
        debug!("push::raw::indent");
        let mut is_first = true;
        for s in s.lines() {
            if is_first {
                is_first = false;
                self.push_raw_in(s, result);
                continue;
            }
            self.push_raw_in("\n", result);
            self.push_raw_in(&self.get_indent(), result);
            self.push_raw_in(s, result)
        }
    }

    /// must be called when you cannot keep track of what you pushed
    /// so that context doesn't refuse your next pushes for no reasons.
    fn lost_context(&mut self) {
        self.just_spaced = false;
        self.consec_new_line = 0;
    }

    /// returns an indent using config to get it's length.
    fn get_indent(&self) -> String {
        " ".repeat(self.config.ident_space)
    }
}

pub fn format(s: &str, config: Config) -> String {
    let init = parse(s);
    let mut context = Ctx::from_config(config);
    let root = LinkedNode::new(&init);
    visit(&root, &mut context)
}

/// This is recursively called on the AST, the formatting is bottom up,
/// nodes will decide based on the size of their children and the max line length
/// how they will be formatted.
///
/// One assumed rule is that no kind should be formatting with surrounded space
#[instrument(skip_all,name = "V" , ret, fields(kind = format!("{:?}",node.kind())))]
fn visit(node: &LinkedNode, ctx: &mut Ctx) -> String {
    let mut res: Vec<String> = vec![];
    for child in node.children() {
        let child_fmt = visit(&child, ctx);
        res.push(child_fmt);
    }
    let res = match node.kind() {
        CodeBlock => code_blocks::format_code_blocks(node, &res, ctx),
        Args => args::format_args(node, &res, ctx),
        LetBinding => format_let_binding(node, &res, ctx),
        _ => format_default(node, &res, ctx),
    };
    if node.children().count() == 0 {
        debug!("visiting token {:?}", node.kind());
    } else {
        debug!("visiting parent: {:?}", node.kind());
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
///
#[instrument(skip_all)]
fn format_default(node: &LinkedNode, children: &[String], ctx: &mut Ctx) -> String {
    debug!("::format_default: {:?}", node.kind());
    let mut res = String::new();
    debug!(
        "with children: {:?}",
        node.children().map(|c| c.kind()).collect_vec()
    );

    ctx.push_in(node.text(), &mut res);
    for s in children {
        ctx.push_raw_in(s, &mut res);
    }
    res
}

#[instrument(skip_all)]
pub(crate) fn format_let_binding(
    parent: &LinkedNode,
    children: &[String],
    ctx: &mut Ctx,
) -> String {
    let mut res = String::new();
    for (s, node) in children.iter().zip(parent.children()) {
        match node.kind() {
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

#[cfg(test)]
mod tests;
