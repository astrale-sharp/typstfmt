//! Some crates are well documented, this crate has a personality instead (please help).
//!
//! This lack is born out of wanting your program to work before documenting it, as long as I'm
//! iterating I don't write docs so much.

use itertools::Itertools;
use log::{debug, trace};
use typst::syntax::SyntaxKind;
use typst::syntax::SyntaxKind::*;
use typst::syntax::{parse, LinkedNode};
use Option::None;

mod config;
use config::Config;

//formatting stuff starts here
mod args;
mod code_blocks;
#[derive(Default)]
struct Ctx {
    config: Config,
    just_spaced: bool,
    consec_new_line: i32,
    // result: String,
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
    /// avoids:
    /// - putting two consecutive spaces.
    /// - putting more than two consecutive newlines.
    fn push_in(&mut self, s: &str, result: &mut String) {
        trace!("PUSH_IN");
        match s {
            " " => {
                if self.just_spaced {
                    debug!("IGNORED space");
                } else {
                    debug!("PUSHED SPACE");
                    self.just_spaced = true;
                    result.push(' ');
                }
            }
            "\n" => {
                if self.consec_new_line <= 1 {
                    debug!("PUSHED NEWLINE");
                    self.consec_new_line += 1;
                    result.push('\n')
                } else {
                    debug!("IGNORED newline");
                }
            }
            _ => {
                debug!("PUSHED {s}");
                result.push_str(s);
                self.lost_context();
            }
        }
    }
    /// makes the context aware it missed info,
    /// should be called when pushing directly in result.
    fn push_raw_in(&mut self, s: &str, result: &mut String) {
        trace!("PUSH_RAW");
        result.push_str(s);
        self.lost_context()
    }

    /// adds an indentation for each line the input except the first to match the current level of identation.
    fn push_raw_indent(&mut self, s: &str, result: &mut String) {
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

fn visit(node: &LinkedNode, ctx: &mut Ctx) -> String {
    let mut res: Vec<String> = vec![];
    for child in node.children() {
        let child_fmt = visit(&child, ctx);
        res.push(child_fmt);
    }
    match node.kind() {
        CodeBlock => code_blocks::format_code_blocks(node, &res, ctx),
        Args => args::format_args(node, &res, ctx),
        Space => String::from(" "),
        _ => format_default(node, &res, ctx),
    }
}

fn format_default(node: &LinkedNode, children: &Vec<String>, ctx: &mut Ctx) -> String {
    debug!("format_default");
    let mut res = String::new();

    match node.kind() {
        // Space => {
        //     for c in node.text().chars() {
        //         match c {
        //             ' '  => res.push_str(config.process(" ")),
        //             '\n' => res
        //         }
        //     }
        // },
        Parbreak => {
            debug!("format_default::ParBreak");
            for _ in 0..node.text().lines().count() {
                debug!("---try push newline");
                ctx.push_in("\n", &mut res);
            }
        }
        _ => {
            ctx.push_raw_in(node.text(), &mut res);
            for s in children {
                ctx.push_raw_in(s, &mut res);
            }
        }
    }
    res
}

/// find any child recursively that fits predicate
fn find_child<'a>(
    node: &LinkedNode<'a>,
    predicate: &impl Fn(&LinkedNode) -> bool,
) -> Option<LinkedNode<'a>> {
    debug!("::find_child of {:?}", node.kind());
    debug!(
        "on children: {:?}",
        node.children().map(|x| x.kind()).collect_vec()
    );
    for child in node.children() {
        debug!("try for {:?}", child.kind());
        if predicate(&child) {
            debug!("predicate accepted");
            return Some(child.clone());
        } else if let Some(f) = find_child(&child, predicate) {
            debug!("predicate accepted for inner of {:?}", child.kind());
            return Some(f);
        }
    }
    None
}

fn next_is_ignoring(node: &LinkedNode, is: SyntaxKind, ignoring: &[SyntaxKind]) -> bool {
    debug!("fn::next_is_ignoring");
    let mut next = node.next_sibling();
    debug!("{:?}", next);
    while let Some(next_inner) = &next {
        debug!("{:?}", next);
        let kind = next_inner.kind();
        if ignoring.contains(&kind) {
            debug!("ignoring {:?}", kind);

            next = next_inner.next_sibling();
            continue;
        }
        let next_is = kind == is;
        debug!("next is: {next_is}");
        return next_is;
    }
    debug!("next is not {is:?}");
    false
}

fn is_trailing_comma(node: &LinkedNode<'_>) -> bool {
    assert!(node.kind() == Comma);
    let next = node.next_sibling();
    let next_skipping_space = match &next {
        Some(x) if x.kind() == Space => next.unwrap().next_sibling(),
        _ => next,
    };
    next_skipping_space.is_some_and(|n| n.kind().is_terminator())
}

fn is_last_comma(node: &LinkedNode) -> bool {
    assert!(node.kind() == Comma);
    let mut next = node.next_sibling().unwrap();
    loop {
        if next.kind() == Comma {
            return false;
        }
        if next.kind().is_terminator() {
            return true;
        }
        next = next.next_sibling().unwrap();
    }
}

fn max_line_length(s: &str) -> usize {
    s.lines()
        .map(|l| l.trim().chars().count())
        .max()
        .unwrap_or(0)
}


#[cfg(test)]
mod tests;
