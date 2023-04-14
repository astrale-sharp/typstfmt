use itertools::Itertools;
use log::{debug, info};
use regex::Regex;
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

    let mut parents: Vec<Context> = vec![(Context::new(None, 0, vec![init]))];
    let mut writer = Writer::default();
    while !parents.is_empty() {
        let context = parents.pop().unwrap();
        let node = context.child();
        let mut children = node
            .children()
            .cloned()
            .into_iter()
            .enumerate()
            .map(|(idx, _)| Context {
                parent: Some(node.clone()),
                child_idx: idx,
                children: node.children().cloned().collect_vec(),
            })
            .collect_vec();
        children.reverse();
        parents.append(&mut children);

        writer = writer.with_value(node.text().to_string());
        for rule in rules.iter().filter(|&r| r.accept(&context)) {
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
// next children of same level etc can easily be accessed right now.
// todo test if the (parent, next_child) provided are the right one.
#[derive(Debug)]
struct Context {
    parent: Option<SyntaxNode>,
    child_idx: usize,
    children: Vec<SyntaxNode>,
}

impl Context {
    fn new(parent: Option<SyntaxNode>, child_idx: usize, children: Vec<SyntaxNode>) -> Self {
        Self {
            parent,
            child_idx,
            children,
        }
    }

    pub fn child(&self) -> &SyntaxNode {
        self.children.get(self.child_idx).unwrap()
    }

    pub fn next_child(&self) -> Option<&SyntaxNode> {
        self.children.get(self.child_idx + 1)
    }

    pub fn child_at(&self, idx_rel: usize) -> Option<&SyntaxNode> {
        self.children.get(self.child_idx + idx_rel)
    }
}
