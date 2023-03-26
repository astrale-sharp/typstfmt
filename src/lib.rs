use ast::Expr::*;
use regex::Regex;
use typst::syntax::parse;
use typst::syntax::{ast, SyntaxNode};

// Optimize: could return Text edit that should be applied one after the other
// instead of String
pub fn typst_format(s: &str) -> String {
    format_with_rules(s, &[OneSpace.as_dyn()])
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

struct OneSpace;
impl Rule for OneSpace {
    fn accept(&self, syntax_node: &SyntaxNode, context: ()) -> bool {
        syntax_node.is::<ast::Space>() || syntax_node.is::<ast::Markup>()
    }

    fn eat(&self, syntax_node: &SyntaxNode) -> String {
        let rg = Regex::new("\\s+").unwrap();
        rg.replace_all(syntax_node.text().as_str(), " ").to_string()
    }
}

struct NoSpaceAtEndLine;
impl Rule for NoSpaceAtEndLine {
    fn accept(&self, syntax_node: &SyntaxNode, context: ()) -> bool {
        syntax_node.is::<ast::Space>() || syntax_node.is::<ast::Markup>()
    }

    fn eat(&self, syntax_node: &SyntaxNode) -> String {
        let rg = Regex::new("(\\s+)\\n").unwrap();
        rg.replace_all(syntax_node.text().as_str(), "\n")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(test)]
    mod one_space {
        use super::*;

        #[test]
        fn one_space_is_unchanged() {
            similar_asserts::assert_eq!(format_with_rules("#{ }", &[OneSpace.as_dyn()]), "#{ }");
        }

        #[test]
        fn more_than_on_becomes_one() {
            similar_asserts::assert_eq!(format_with_rules("#{  }", &[OneSpace.as_dyn()]), "#{ }");
            similar_asserts::assert_eq!(format_with_rules("#{   }", &[OneSpace.as_dyn()]), "#{ }");
            similar_asserts::assert_eq!(format_with_rules("m  m", &[OneSpace.as_dyn()]), "m m");
        }
    }
    #[cfg(test)]
    mod no_space_when_line_ends {
        use super::*;

        #[test]
        fn removes_trailing_space() {
            similar_asserts::assert_eq!(
                format_with_rules(
                    r#"Some markup  
                And then some"#,
                    &[NoSpaceAtEndLine.as_dyn()]
                ),
                r#"Some markup
                And then some"#
            );
        }
    }

    #[test]
    fn complex() {
        let expected = r##"#import "template.typ": *
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
        let input = r##"#import "template.typ": *
#show: letter.with(sender:[Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],date: [Morristown, June 9th, 2023,],subject: [test],name: [Jane Smith \Regional Director],)

Dear Joe,

#lorem(99)

Best,
"##;
        similar_asserts::assert_eq!(typst_format(input), expected);
    }
}

// rules :
// ModuleImport, space after colon
// ImportItems : trailing comma
