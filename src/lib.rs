use ast::Expr::*;
use itertools;
use regex::Regex;
use typst::syntax::parse;
use typst::syntax::{ast, SyntaxNode};

// Optimize: could return Text edit that should be applied one after the other
// instead of String
pub fn typst_format(s: &str) -> String {
    format_with_rules(s, &[NoSpaceAtEndLine.as_dyn(), OneSpace.as_dyn()])
}

fn format_with_rules(s: &str, rules: &[Box<dyn Rule>]) -> String {
    let init = parse(s);
    let mut parents = vec![&init];
    let mut result = String::new();
    let mut deep = 0;
    while !parents.is_empty() {
        let this_parent = parents.pop().unwrap();
        let children = this_parent.children();
        for this_child in children.clone() {
            let mut to_append = this_child.text().to_string();
            for rule in rules.iter() {
                if rule.accept(this_child, Context) {
                    to_append = rule.eat(to_append, Context);
                }
            }
            result.push_str(&to_append)
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

trait Rule {
    fn accept(&self, syntax_node: &SyntaxNode, context: Context) -> bool;

    fn eat(&self, text: String, context: Context) -> String;

    fn as_dyn(self: Self) -> Box<dyn Rule>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

struct OneSpace;
impl Rule for OneSpace {
    fn accept(&self, syntax_node: &SyntaxNode, context: Context) -> bool {
        syntax_node.is::<ast::Space>() || syntax_node.is::<ast::Markup>()
    }

    fn eat(&self, text: String, context: Context) -> String {
        let rg = Regex::new(r"\s+").unwrap();
        rg.replace_all(&text, " ").to_string()
    }
}

struct NoSpaceAtEndLine;
impl Rule for NoSpaceAtEndLine {
    fn accept(&self, syntax_node: &SyntaxNode, context: Context) -> bool {
        syntax_node.is::<ast::Space>() || syntax_node.is::<ast::Markup>()
    }

    fn eat(&self, text: String, context: Context) -> String {
        let rg = Regex::new(r"(\s)+\n").unwrap();
        rg.replace_all(&text, "\n").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_than_one_rule() {
        similar_asserts::assert_eq!(format_with_rules("#{  }  \n", &[OneSpace.as_dyn(),NoSpaceAtEndLine.as_dyn()]),"#{ }\n");

    }    #[cfg(test)]
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

        #[test]
        fn dont_insert_weird_space() {
        similar_asserts::assert_eq!(format_with_rules("#{  }\n", &[OneSpace.as_dyn()]),"#{ }\n");
        }
    }
    #[cfg(test)]
    mod no_space_when_line_ends {
        use super::*;
        #[test]
        fn dont_insert_weird_space() {
            similar_asserts::assert_eq!(format_with_rules("#{  }  \n", &[NoSpaceAtEndLine.as_dyn()]), "#{  }\n");
        }
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
