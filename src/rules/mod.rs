use super::*;
use typst::syntax::SyntaxKind;
pub(crate) trait Rule: std::fmt::Debug {
    fn accept(&self, context: &Context) -> bool;

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
        NoSpaceBeforeColon.as_dyn(),
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
    fn accept(&self, context: &Context) -> bool {
        context.child().is::<ast::Space>()
            || context.child().is::<ast::Markup>()
            || context.child().is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _: &Context, writer: &mut Writer) {
        let rg = Regex::new(r"( )+").unwrap();
        writer.push(rg.replace_all(&text, " ").to_string().as_str());
    }
}

#[derive(Debug)]
pub(crate) struct NoSpaceAtEndLine;

impl Rule for NoSpaceAtEndLine {
    fn accept(&self, context: &Context) -> bool {
        context.child().is::<ast::Space>()
            || context.child().is::<ast::Markup>()
            || context.child().is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _context: &Context, writer: &mut Writer) {
        let rg = Regex::new(r"( )+\n").unwrap();
        writer.push(rg.replace_all(&text, "\n").to_string().as_str());
    }
}
#[derive(Debug)]
pub(crate) struct TrailingComma;
impl Rule for TrailingComma {
    fn accept(&self, context: &Context) -> bool {
        let Some(parent) = &context.parent else {return false};
        let Some(next_child) = context.next_child() else {return false};

        parent.is::<ast::Args>()
            && !(context.child().kind() == SyntaxKind::Comma)
            && next_child.kind().is_grouping()
    }

    fn eat(&self, text: String, _: &Context, writer: &mut Writer) {
        writer.push(&text).push(",");
    }
}

#[derive(Debug)]
pub(crate) struct SpaceAfterColon;
impl Rule for SpaceAfterColon {
    fn accept(&self, context: &Context) -> bool {
        let Some(next) = context.next_child() else {return false};
        context.child().kind() == SyntaxKind::Colon && !next.is::<ast::Space>()
    }

    fn eat(&self, text: String, _context: &Context, writer: &mut Writer) {
        writer.push(&text).push(" ");
    }
}

#[derive(Debug)]
pub(crate) struct NoSpaceBeforeColon;
impl Rule for NoSpaceBeforeColon {
    fn accept(&self, context: &Context) -> bool {
        let Some(next) = context.next_child() else {return false};
        next.kind() == SyntaxKind::Colon && context.child().is::<ast::Space>()
    }

    fn eat(&self, _: String, _context: &Context, _: &mut Writer) {
        // don't put the space.
    }
}

#[derive(Debug)]
pub(crate) struct JumpTwoLineMax;
impl Rule for JumpTwoLineMax {
    fn accept(&self, context: &Context) -> bool {
        context.child().is::<ast::Text>()
            || context.child().is::<ast::Markup>()
            || context.child().is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _: &Context, writer: &mut Writer) {
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
    fn accept(&self, context: &Context) -> bool {
        let Some(parent) = &context.parent else {return false};
        parent.is::<ast::Args>() || parent.is::<ast::FuncCall>()
    }

    fn eat(&self, text: String, context: &Context, writer: &mut Writer) {
        // todo with last child, if not comma, if last elem, add a comma
        if context.child().kind().is_grouping() {
            // is grouping opening
            if context.next_child().is_some() {
                writer.push(&text).inc_indent().newline_with_indent();
            } else if context.next_child().is_none()
                && context.parent.as_ref().unwrap().is::<ast::Args>()
            {
                // is grouping nested closing
                debug!("GROUPING NESTED CLOSING");
                writer.dec_indent().newline_with_indent().push(&text);
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
        } else if context.child().kind() == SyntaxKind::Comma {
            //todo, ignore if is space and look at the next after the space
            let mut next = context.next_child();
            let mut i = 0;
            while next.is_some() && next.unwrap().kind().is_trivia() {
                i += 1;
                next = context.child_at(i)
            }

            if next.is_some() && next.unwrap().kind().is_grouping() {
                writer.push(&text);
            } else {
                writer.push(&text).newline_with_indent();
            }
        } else if context.child().is::<ast::Space>() {
            // do nothing
        } else {
            writer.push(&text);
        }
    }
}

//#[derive(Debug)]
//pub(crate) struct NoSpaceAtEOF;
//impl Rule for NoSpaceAtEOF {}

#[cfg(test)]
mod tests;
