use itertools::Itertools;
use log::{debug, info};
use regex::Regex;
use typst::syntax::ast;
use typst::syntax::{parse, LinkedNode};

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
    let root = LinkedNode::new(&init);
    info!("formats text : {s:?}; with rules {:?}", rules);
    info!("parsed : \n{init:?}\n");
    let mut result = String::with_capacity(1024);
    let mut writer = Writer::default(&mut result);

    format_node(root, &mut writer, rules);
    result
}

fn format_node(node: LinkedNode, mut writer: &mut Writer, rules: &[Box<dyn Rule>]) {
    for child in node.children() {
        format_node(child, &mut writer, rules)
    }
    writer.set_value(node.text().to_string());
    // TODO: this visits this node after the children, may need a way to tell the filter this is
    // after, and have an option to process before the children
    for rule in rules.iter().filter(|&r| r.accept(&node)) {
        //writer simulate node text
        debug!("MATCHED RULE {rule:?}");
        debug!("RULE FROM `{:?}`", writer.value());
        rule.eat(writer.take(), &node, &mut writer);
        debug!("RULE TO `{:?}`", writer.value());
    }
    writer.flush();
}
