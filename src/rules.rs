use super::*;
use typst::syntax::SyntaxKind;
pub(crate) trait Rule: std::fmt::Debug {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool;

    fn eat(&self, text: String, context: &Context, writer: &mut Writer);

    fn as_dyn(self: Self) -> Box<dyn Rule>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

pub(crate) fn rules() -> Vec<Box<dyn rules::Rule>> {
    vec![
        SpaceAfterColon.as_dyn(),
        TrailingComma.as_dyn(),
        IdentItemFunc.as_dyn(),
        JumpTwoLineMax.as_dyn(),
        OneSpace.as_dyn(),
        NoSpaceAtEndLine.as_dyn(),
    ]
}

#[derive(Debug)]
pub(crate) struct OneSpace;

impl Rule for OneSpace {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool {
        syntax_node.is::<ast::Space>()
            || syntax_node.is::<ast::Markup>()
            || syntax_node.is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        let rg = Regex::new(r"( )+").unwrap();
        writer.push(rg.replace_all(&text, " ").to_string().as_str());
    }
}

#[derive(Debug)]
pub(crate) struct NoSpaceAtEndLine;

impl Rule for NoSpaceAtEndLine {
    fn accept(&self, syntax_node: &SyntaxNode, _context: &Context) -> bool {
        syntax_node.is::<ast::Space>()
            || syntax_node.is::<ast::Markup>()
            || syntax_node.is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _context: &Context, writer: &mut Writer) {
        let rg = Regex::new(r"( )+\n").unwrap();
        writer.push(rg.replace_all(&text, "\n").to_string().as_str());
    }
}
#[derive(Debug)]
pub(crate) struct TrailingComma;
impl Rule for TrailingComma {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool {
        let Some(parent) = context.parent else {return false};
        let Some(next_child) = context.next_child else {return false};

        parent.is::<ast::Args>()
            && !(syntax_node.kind() == SyntaxKind::Comma)
            && next_child.kind().is_grouping()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        writer.push(&text).push(",");
    }
}

#[derive(Debug)]
pub(crate) struct SpaceAfterColon;
impl Rule for SpaceAfterColon {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool {
        let Some(next) = context.next_child else {return false};
        syntax_node.kind() == SyntaxKind::Colon && !next.is::<ast::Space>()
    }

    fn eat(&self, text: String, _context: &Context, writer: &mut Writer) {
        writer.push(&text).push(" ");
    }
}

#[derive(Debug)]
pub(crate) struct JumpTwoLineMax;
impl Rule for JumpTwoLineMax {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool {
        syntax_node.is::<ast::Text>()
            || syntax_node.is::<ast::Markup>()
            || syntax_node.is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        let rg_one_line = Regex::new(r"(\s)*\n(\s)*").unwrap();
        let rg_two_line = Regex::new(r"(\s)*\n(\s)*\n(\s)*").unwrap();
        let to_add = if rg_two_line.is_match(&text) {
            rg_two_line.replace_all(&text, "\n\n").to_string()
        } else {
            rg_one_line.replace_all(&text, "\n").to_string()
        };
        writer.push(&to_add);
    }
}

#[derive(Debug)]
pub(crate) struct IdentItemFunc;

impl Rule for IdentItemFunc {
    fn accept(&self, _syntax_node: &SyntaxNode, context: &Context) -> bool {
        let Some(parent) = context.parent else {return false};
        parent.is::<ast::Args>() || parent.is::<ast::FuncCall>()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        // todo with last child, if not comma, if last elem, add a comma
        if context.child.kind().is_grouping() {
            // is grouping opening
            if context.next_child.is_some() {
                writer.push(&text).inc_indent().newline_with_indent();
            } else if context.next_child.is_none() && context.parent.unwrap().is::<ast::Args>() {
                // is grouping nested closing
                debug!("GROUPING NESTED CLOSING");
                writer.newline_with_indent().push(&text).dec_indent();
            //                writer.newline_with_indent();
            } else {
                debug!("GROUPING CLOSING GOOD");
                // is grouping closing

                writer
                    .newline_with_indent()
                    .push(&text)
                    .dec_indent()
                    .newline_with_indent();
            }
        } else if context.child.kind() == SyntaxKind::Comma {
            //todo, ignore if is space and look at the next after the space
            if context.next_child.is_some() && context.next_child.unwrap().kind().is_grouping() {
                writer.push(&text);
            } else {
                writer.push(&text).newline_with_indent();
            }
        } else if context.child.is::<ast::Space>() {
            // do nothing
        } else {
            writer.push(&text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .is_test(true)
                .try_init();
    }

    #[test]
    fn more_than_one_rule() {
        init();
        similar_asserts::assert_eq!(
            format_with_rules("#{  }  \n", &[OneSpace.as_dyn(), NoSpaceAtEndLine.as_dyn()]),
            "#{ }\n"
        );
    }
    #[cfg(test)]
    mod one_space {
        use super::*;

        #[test]
        fn one_space_is_unchanged() {
            init();

            similar_asserts::assert_eq!(format_with_rules("#{ }", &[OneSpace.as_dyn()]), "#{ }");
            similar_asserts::assert_eq!(
                format_with_rules("some\n\nsome", &[OneSpace.as_dyn()]),
                "some\n\nsome"
            );
            similar_asserts::assert_eq!(
                format_with_rules("some \n \n some", &[OneSpace.as_dyn()]),
                "some \n \n some"
            );
        }

        #[test]
        fn more_than_on_becomes_one() {
            similar_asserts::assert_eq!(format_with_rules("#{  }", &[OneSpace.as_dyn()]), "#{ }");
            init();
            similar_asserts::assert_eq!(
                format_with_rules("some \n  \n some", &[OneSpace.as_dyn()]),
                "some \n \n some"
            );
            similar_asserts::assert_eq!(format_with_rules("#{   }", &[OneSpace.as_dyn()]), "#{ }");
            similar_asserts::assert_eq!(format_with_rules("m  m", &[OneSpace.as_dyn()]), "m m");
            similar_asserts::assert_eq!(
                format_with_rules("some \n \n  some", &[OneSpace.as_dyn()]),
                "some \n \n some"
            );
            similar_asserts::assert_eq!(
                format_with_rules("some  \n \n some", &[OneSpace.as_dyn()]),
                "some \n \n some"
            );
            similar_asserts::assert_eq!(
                format_with_rules("some  \n  \n some", &[OneSpace.as_dyn()]),
                "some \n \n some"
            );
            similar_asserts::assert_eq!(
                format_with_rules("some  \n  \n some  ", &[OneSpace.as_dyn()]),
                "some \n \n some "
            );
        }

        #[test]
        fn dont_insert_weird_space() {
            init();

            similar_asserts::assert_eq!(
                format_with_rules("#{  }\n", &[OneSpace.as_dyn()]),
                "#{ }\n"
            );
        }
    }
    #[cfg(test)]
    mod no_space_when_line_ends {
        use super::*;
        #[test]
        fn dont_insert_weird_space() {
            init();

            similar_asserts::assert_eq!(
                format_with_rules("#{  }  \n", &[NoSpaceAtEndLine.as_dyn()]),
                "#{  }\n"
            );
        }

        #[test]
        fn removes_trailing_space() {
            init();

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

        #[test]
        fn dont_eat_too_much() {
            similar_asserts::assert_eq!(
                format_with_rules("some \n \n  some", &[NoSpaceAtEndLine.as_dyn()]),
                "some\n\n  some"
            );
        }
    }

    #[cfg(test)]
    mod func {
        use super::*;

        #[test]
        fn basic_func() {
            init();
            similar_asserts::assert_eq!(
                format_with_rules("#{f1(1,2,3,)}", &[IdentItemFunc.as_dyn()]),
                format!("#{{f1(\n{0}1,\n{0}2,\n{0}3,\n{0})}}", " ".repeat(4))
            );
        }

        #[test]
        fn reduce_space() {
            init();
            similar_asserts::assert_eq!(
                format_with_rules("#{f1(\n\n1,\n\n2,\n\n3,\n)}", &[IdentItemFunc.as_dyn()]),
                format!("#{{f1(\n{0}1,\n{0}2,\n{0}3,\n{0})}}", " ".repeat(4))
            );
        }

        #[test]
        fn nested_func() {
            init();
            similar_asserts::assert_eq!(
                format_with_rules("#{f1(1,2,f(a,b,c,),)}", &[IdentItemFunc.as_dyn()]),
                "#{f1(\n    1,\n    2,\n    f(\n        a,\n        b,\n        c,\n        ),\n    )}"
            );
        }
        #[test]
        fn spacing_without_comma() {
            init();

            similar_asserts::assert_eq!(
                format_with_rules("#lorem(9)", &[IdentItemFunc.as_dyn()]),
                "#lorem(\n    9\n    )"
            );
        }
    }

    #[test]
    fn complex() {
        init();

        let expected = r##"#import "template.typ": *
#show: letter.with(
    sender: [Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],
    recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],
    date: [Morristown, June 9th, 2023,],
    subject: [test],
    name: [Jane Smith \Regional Director],
    )

Dear Joe,

#lorem(
    9,
    )

Best,"##;
        let input = r##"#import "template.typ": *
#show: letter.with(sender:[Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],date: [Morristown, June 9th, 2023,],subject: [test],name: [Jane Smith \Regional Director],)

Dear Joe,

#lorem(9)

Best,"##;
        similar_asserts::assert_eq!(typst_format(input), expected);
    }
}
