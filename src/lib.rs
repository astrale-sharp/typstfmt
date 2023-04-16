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

    let mut parents: Vec<LinkedNode> = vec![root];

    let mut writer = Writer::default(&mut result);
    while !parents.is_empty() {
        let node = parents.pop().unwrap();
        let mut children = node.children().collect_vec();
        children.reverse();
        parents.append(&mut children);

        writer = writer.with_value(node.text().to_string());
        for rule in rules.iter().filter(|&r| r.accept(&node)) {
            //writer simulate node text
            debug!("MATCHED RULE {rule:?}");
            debug!("RULE FROM `{:?}`", writer.value());
            rule.eat(writer.take(), &node, &mut writer);
            debug!("RULE TO `{:?}`", writer.value());
        }
        writer.flush();
    }
    result
}
