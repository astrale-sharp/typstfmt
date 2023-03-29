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
    format_with_rules(s, rules().as_slice())
}

fn format_with_rules(s: &str, rules: &[Box<dyn Rule>]) -> String {
    let init = parse(s);
    info!("formats text : {s:?}; with rules {:?}", rules);
    info!("parsed : \n{init:?}\n");
    let mut result = String::with_capacity(1024);

    let mut parents: Vec<(&SyntaxNode, Context)> = vec![(&init, Context::new(&init))];
    let mut writer = Writer::default();
    //let mut deep = 0;

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

        writer = writer.with_value(this_node.text().to_string());
        for rule in rules.iter().filter(|&r| r.accept(this_node, &context)) {
            debug!("MATCHED RULE {rule:?}");
            debug!("RULE FROM `{:?}`", writer.value());
            rule.eat(writer.take(), &context, &mut writer);
            debug!("RULE TO `{:?}`", writer.value());
        }
        result.push_str(writer.value());
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
