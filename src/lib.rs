use env_logger;
use itertools::Itertools;
use log::{debug, info, log_enabled, Level};
use regex::Regex;
use std::iter::zip;
use typst::syntax::parse;
use typst::syntax::{ast, SyntaxNode};

mod rules;
use rules::*;
mod writer;
use writer::Writer;

// Optimize: could return Text edit that should be applied one after the other
// instead of String
pub fn typst_format(s: &str) -> String {
    format_with_rules(s, &[NoSpaceAtEndLine.as_dyn(), OneSpace.as_dyn()])
}

fn format_with_rules(s: &str, rules: &[Box<dyn Rule>]) -> String {
    info!("formats text : {s:?}\nwith rules {:?}", rules);
    let init = parse(s);
    let mut result = String::with_capacity(1024);

    let mut parents: Vec<(&SyntaxNode, Context)> = vec![(&init, Context::new(&init))];
    let mut writer = Writer::default();
    //let mut deep = 0;

    debug!(
        "Starting with parent {} and result at {:?}",
        init.text(),
        writer.value()
    );

    while !parents.is_empty() {
        let (this_node, context) = parents.pop().unwrap();
        //let mut children: Vec<_> = this_node.children().map(|c| c).collect_vec();
        let mut children = zip(
            this_node.children(),
            this_node.children().skip(1).map(Some).chain(vec![None]),
        )
        .map(|(now, next)| {
            (
                now,
                Context {
                    parent: Some(this_node),
                    next_child: next,
                    child: now,
                },
            )
        })
        .collect_vec();
        children.reverse();
        parents.append(&mut children);
        debug!("iter on {this_node:?} with context : {context:?}");

        writer = writer.with_value(this_node.text().to_string());
        for rule in rules.iter().filter(|&r| r.accept(this_node, &context)) {
            rule.eat(writer.take(), &context, &mut writer);
        }
        result.push_str(writer.value());
        debug!("result at `{result}`");
    }
    result
}

/// The context needed by a rule to accept the node && produce it's resulting text
// How deep we are in the tree, who's the parent,
// next childen of same level etc can easily be accessed right now.
// todo test if the (parent, next_child) provided are the right one.
#[derive(Debug)]
struct Context<'a> {
    child: &'a SyntaxNode,
    parent: Option<&'a SyntaxNode>,
    next_child: Option<&'a SyntaxNode>,
}

impl<'a> Context<'a> {
    fn new(child: &'a SyntaxNode) -> Self {
        Self {
            child,
            parent: None,
            next_child: None,
        }
    }
}

pub(crate) fn init_log() {
    env_logger::init();
}

// rules :
// ModuleImport, space after colon
// ImportItems : trailing comma
