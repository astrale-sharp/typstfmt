use ast::Expr::*;
use regex::Regex;
use typst::syntax::parse;
use typst::syntax::{ast, SyntaxNode};

// Optimize: could return Text edit that should be applied one after the other
// instead of String
pub fn typst_format(s: &str) -> String {
    format_with_rules(s, &[SpaceRule.as_dyn()])
}

fn format_with_rules(s: &str, rules: &[Box<dyn Rule>]) -> String {
    let syntax_node = parse(s);
    format_recursive(&syntax_node, 0, (), rules)
}

// Optimize: consider returning &str instead and other optimisations
//
fn format_recursive(
    syntax_node: &SyntaxNode,
    recurse: usize,
    // feel free to include what your rule needs to know here
    // and change the definition of the function if you need to
    // for instance this could contain the parent if any
    context: (),
    rules: &[Box<dyn Rule>],
) -> String {
    // rules either leave the result unchanged or format it
    // apply rules, append to result, do some for children
    let mut result;
    // currently only the first rule that matches is selected (this behavior could be changed)
    // the most specific rules should come first.
    if let Some(rule) = rules.iter().find(|&rule| rule.accept(syntax_node, context)) {
        result = rule.eat(syntax_node);
    } else {
        // test this returns what I think
        result = String::from(syntax_node.text())
    }

    for child in syntax_node.children() {
        result.push_str(&format_recursive(child, recurse + 1, (), rules))
    }
    result
}

trait Rule {
    fn accept(&self, syntax_node: &SyntaxNode, context: ()) -> bool;

    fn eat(&self, syntax_node: &SyntaxNode) -> String;

    fn as_dyn(self: Self) -> Box<dyn Rule>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

struct SpaceRule;
impl Rule for SpaceRule {
    fn accept(&self, syntax_node: &SyntaxNode, context: ()) -> bool {
        syntax_node.is::<ast::Space>()
    }

    fn eat(&self, syntax_node: &SyntaxNode) -> String {
        //let x = syntax_node.cast::<ast::Space>().unwrap();
        assert!(syntax_node.text() != "");
        let t = syntax_node.text().as_str();
        let rg = Regex::new("\\s+").unwrap();
        rg.replace_all(t, " ").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        similar_asserts::assert_eq!(format_with_rules("#{ }", &[SpaceRule.as_dyn()]), "#{ }");
    }

    #[test]
    fn two_spaces_become_one() {
        similar_asserts::assert_eq!(format_with_rules("#{  }", &[SpaceRule.as_dyn()]), "#{ }");
    }

    #[test]
    fn complex() {
        let expected = 
r##"#import "template.typ": *
#show: letter.with(
    sender: [
        Jane Smith, 
        Universal Exports, 
        1 Heavy Plaza, 
        Morristown, 
        NJ 07964,
    ],
    recipient: [
        Mr. John Doe \
        Acme Corp. \
        123 Glennwood Ave \
        Quarto Creek, VA 22438
    ],
    date: [
        Morristown, 
        June 9th, 2023,
        ],
    subject: [
        test
        ],
    name: [
        Jane Smith \
        Regional Director
        ],
)

Dear Joe,

#lorem(99)

Best,"##;
    let input = 
r##"#import "template.typ": *
#show: letter.with(sender:[Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],date: [Morristown, June 9th, 2023,],subject: [test],name: [Jane Smith \Regional Director],)

Dear Joe,

#lorem(99)

Best,
"##;
    similar_asserts::assert_eq!(typst_format(input), expected);

    }
}


/// rules :
/// ModuleImport, space after colon
/// ImportItems : trailing comma

#[test]
fn feature() {
    for a in [
        "",
        " ",
        r##"#import "template.typ":*"##,
        r##"#import "template.typ": *"##,
        r##"#import "template.typ": func1, func2,"##,
    ] {
        println! {"parsing: {:?}",a};

        let syntax_node = parse(a);
        dbg!(syntax_node.erroneous());
        println!("parse:\n{:?}", &syntax_node);
        println!("--------------");
    }
}
