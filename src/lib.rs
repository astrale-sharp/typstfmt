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
    let mut parents: Vec<(&SyntaxNode, Context)> = vec![(&init, Context::default())];
    let mut writer = Writer::default();
    let mut deep = 0;

    debug!(
        "Starting with parent {} and result at {:?}",
        init.text(),
        writer.result()
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
                },
            )
        })
        .collect_vec();
        children.reverse();
        parents.append(&mut children);
        debug!("iter on {this_node:?} with context : {context:?}");

        let mut to_append = this_node.text().to_string();
        for rule in rules.iter() {
            if rule.accept(this_node, &context) {
                // we use the writer in buffered mode since there may be multiple rules that match
                writer.buffered(false, |w| {
                    rule.eat(to_append.clone(), &context, w);
                    let result = w.result();
                    if log_enabled!(Level::Debug) {
                        let diff = similar_asserts::SimpleDiff::from_str(
                            to_append.as_str(), result, "before", "after",
                        );
                        debug!("MATCHED RULE: {rule:?} \ntransforms {to_append:?} in {result:?}\nwith diff:\n {diff}");
                    }
                    // we update the string representing this with the formatted string applied by the current rule
                    to_append = result.clone();
                });
            }
        }
        // since all rules have been applied to this rule, we now may push the final result to the writer
        writer.push(&to_append);
        debug!("result at `{}`", writer.result());

        deep += 1;
    }
    //format_recursive(&syntax_node, 0, (), rules)
    String::from(writer.result())
}

/// The context needed by a rule to accept the node && produce it's resulting text
// How deep we are in the tree, who's the parent,
// next childen of same level etc can easily be accessed right now.
// todo test if the (parent, next_child) provided are the right one.
#[derive(Debug, Default)]
struct Context<'a> {
    parent: Option<&'a SyntaxNode>,
    next_child: Option<&'a SyntaxNode>,
}

pub(crate) fn init_log() {
    env_logger::init();
}

// rules :
// ModuleImport, space after colon
// ImportItems : trailing comma
