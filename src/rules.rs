use super::*;

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

#[derive(Debug)]
pub(crate) struct OneSpace;

impl Rule for OneSpace {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool {
        syntax_node.is::<ast::Space>() || syntax_node.is::<ast::Markup>()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        let rg = Regex::new(r"( )+").unwrap();
        writer.push(rg.replace_all(&text, " ").to_string().as_str());
    }
}

#[derive(Debug)]
pub(crate) struct NoSpaceAtEndLine;

impl Rule for NoSpaceAtEndLine {
    fn accept(&self, syntax_node: &SyntaxNode, context: &Context) -> bool {
        syntax_node.is::<ast::Space>() || syntax_node.is::<ast::Markup>()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        let rg = Regex::new(r"( )+\n").unwrap();
        writer.push(rg.replace_all(&text, "\n").to_string().as_str());
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
        }

        #[test]
        fn more_than_on_becomes_one() {
            init();

            similar_asserts::assert_eq!(format_with_rules("#{  }", &[OneSpace.as_dyn()]), "#{ }");
            //  similar_asserts::assert_eq!(format_with_rules("#{   }", &[OneSpace.as_dyn()]), "#{ }");
            //  similar_asserts::assert_eq!(format_with_rules("m  m", &[OneSpace.as_dyn()]), "m m");
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
    }

    #[test]
    fn complex() {
        init();

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
