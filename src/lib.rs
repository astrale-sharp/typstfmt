use ast::Expr::*;
use env_logger;
use log::{debug, info, log_enabled, Level};
use regex::Regex;
use typst::syntax::parse;
use typst::syntax::{ast, SyntaxNode};

mod rules;
use rules::*;

// Optimize: could return Text edit that should be applied one after the other
// instead of String
pub fn typst_format(s: &str) -> String {
    format_with_rules(s, &[NoSpaceAtEndLine.as_dyn(), OneSpace.as_dyn()])
}

fn format_with_rules(s: &str, rules: &[Box<dyn Rule>]) -> String {
    info!("formats text : {s:?}\nwith rules {:?}", rules);
    let init = parse(s);
    let mut parents = vec![&init];
    let mut result = String::new();
    let mut deep = 0;

    debug!(
        "Starting with parent {} and result at {result:?}",
        init.text()
    );

    while !parents.is_empty() {
        let this_parent = parents.pop().unwrap();
        let children = this_parent.children();
        debug!("iter at deep: {deep} with parent: `{this_parent:?}`");

        for this_child in children.clone() {
            debug!("-> for child {this_child:?}");

            let mut to_append = this_child.text().to_string();
            for rule in rules.iter() {
                if rule.accept(this_child, Context) {
                    if log_enabled!(Level::Debug) {
                        let to_append = to_append.as_str();
                        let result = rule.eat(to_append.clone().to_owned(), Context);
                        let diff = similar_asserts::SimpleDiff::from_str(
                            to_append, &result, "before", "after",
                        );
                        debug!("MATCHED RULE: {rule:?} \ntransforms {to_append:?} in {result:?}\nwith diff:\n {diff}");
                    }
                    to_append = rule.eat(to_append, Context);
                }
            }
            result.push_str(&to_append);
            debug!("result at `{result}`");
        }
        parents.append(&mut children.collect());
        deep += 1;
    }
    //format_recursive(&syntax_node, 0, (), rules)
    String::from(result)
}

/// The context needed by a rule to accept the node && produce it's resulting text
// How deep we are in the tree, who's the parent,
// next childen of same level etc can easily be accessed right now
struct Context;

pub(crate) fn init_log() {
    env_logger::init();
}

// rules :
// ModuleImport, space after colon
// ImportItems : trailing comma
