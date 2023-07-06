use super::*;
use typst::syntax::SyntaxKind;
pub(crate) trait Rule: std::fmt::Debug {
    fn accept(&self, node: &LinkedNode) -> bool;

    /// eats the current writer value and replaces it.
    fn eat(&self, text: String, node: &LinkedNode, writer: &mut Writer);

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

pub(crate) struct ConditionalRule<T: Rule> {
    pub rule: T,
    condition: Box<dyn Fn(&String, &LinkedNode, &mut Writer) -> bool>,
}
impl<T: Rule> std::fmt::Debug for ConditionalRule<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConditionalRule")
            .field("rule", &self.rule)
            .finish()
    }
}

impl<T: Rule> ConditionalRule<T> {
    pub(crate) fn new(
        rule: T,
        condition: impl 'static + Fn(&String, &LinkedNode, &mut Writer) -> bool,
    ) -> Self {
        Self {
            rule,
            condition: Box::new(condition),
        }
    }
}

impl<T: Rule> Rule for ConditionalRule<T> {
    fn accept(&self, node: &LinkedNode) -> bool {
        self.rule.accept(node)
    }

    fn eat(&self, text: String, node: &LinkedNode, writer: &mut Writer) {
        if !(self.condition)(&text, node, writer) {
            writer.push(&text);
        } else {
            self.rule.eat(text, node, writer)
        }
    }
}

#[derive(Debug)]
pub(crate) struct OneSpace;

impl Rule for OneSpace {
    fn accept(&self, node: &LinkedNode) -> bool {
        node.is::<ast::Space>() || node.is::<ast::Markup>() || node.is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _: &LinkedNode, writer: &mut Writer) {
        let rg = Regex::new(r"( )+").unwrap();
        writer.push(rg.replace_all(&text, " ").to_string().as_str());
    }
}

#[derive(Debug)]
pub(crate) struct NoSpaceAtEndLine;

impl Rule for NoSpaceAtEndLine {
    fn accept(&self, node: &LinkedNode) -> bool {
        node.is::<ast::Space>() || node.is::<ast::Markup>() || node.is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _: &LinkedNode, writer: &mut Writer) {
        let rg = Regex::new(r"( )+\n").unwrap();
        writer.push(rg.replace_all(&text, "\n").to_string().as_str());
    }
}

/// Check whether the given parent node contains any non-empty non-grouping nodes.
fn is_empty_grouping(parent: &LinkedNode) -> bool {
    let mut children = parent.children();
    let res = children.all(|c| {
        let kind = c.kind();
        if kind.is_grouping() {
            true
        } else {
            match kind {
                SyntaxKind::ContentBlock => is_empty_grouping(&c),
                // TODO: add more types that could contain things.
                _ => {
                    debug!("child not empty: {:?}", c);
                    c.is_empty()
                }
            }
        }
    });
    debug!("is_empty parent:{:?} res:{}", parent, res);
    res
}

#[derive(Debug)]
pub(crate) struct TrailingComma;
impl Rule for TrailingComma {
    fn accept(&self, node: &LinkedNode) -> bool {
        let Some(parent) = node.parent() else {return false};
        let Some(next_child) = node.next_sibling() else {return false};

        parent.is::<ast::Args>()
            && !is_empty_grouping(parent)
            && !(node.kind() == SyntaxKind::Comma)
            && next_child.kind().is_grouping()
    }

    fn eat(&self, text: String, _: &LinkedNode, writer: &mut Writer) {
        writer.push(&text).push(",");
    }
}

#[derive(Debug)]
pub(crate) struct SpaceAfterColon;
impl Rule for SpaceAfterColon {
    fn accept(&self, node: &LinkedNode) -> bool {
        let Some(parent) = node.parent().cloned() else {return false};
        let children = parent.children().collect_vec();
        let Some(next) = children.get(node.index()+1) else {return false};
        node.kind() == SyntaxKind::Colon && !next.is::<ast::Space>()
    }

    fn eat(&self, text: String, _: &LinkedNode, writer: &mut Writer) {
        writer.push(&text).push(" ");
    }
}

#[derive(Debug)]
pub(crate) struct NoSpaceBeforeColon;
impl Rule for NoSpaceBeforeColon {
    fn accept(&self, node: &LinkedNode) -> bool {
        let Some(next) = node.next_sibling() else {return false};
        next.kind() == SyntaxKind::Colon && node.is::<ast::Space>()
    }

    fn eat(&self, _: String, _: &LinkedNode, _: &mut Writer) {
        // don't put the space.
    }
}

#[derive(Debug)]
pub(crate) struct JumpTwoLineMax;
impl Rule for JumpTwoLineMax {
    fn accept(&self, node: &LinkedNode) -> bool {
        node.is::<ast::Text>() || node.is::<ast::Markup>() || node.is::<ast::Parbreak>()
    }

    fn eat(&self, text: String, _: &LinkedNode, writer: &mut Writer) {
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
    fn accept(&self, node: &LinkedNode) -> bool {
        let Some(parent) = &node.parent() else {return false};
        parent.is::<ast::Args>() || parent.is::<ast::FuncCall>()
    }

    fn eat(&self, text: String, node: &LinkedNode, writer: &mut Writer) {
        // todo with last child, if not comma, if last elem, add a comma
        if node.kind().is_grouping() {
            // is grouping opening
            let newline = !is_empty_grouping(node.parent().unwrap());
            if node.next_sibling().is_some() {
                writer.push(&text);
                if newline {
                    writer.inc_indent().newline_with_indent();
                }
            } else if node.next_sibling().is_none()
                && node.parent().as_ref().unwrap().is::<ast::Args>()
            {
                // is grouping nested closing
                debug!("GROUPING NESTED CLOSING");
                if newline {
                    writer.dec_indent().newline_with_indent();
                }
                writer.push(&text);
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
        } else if node.kind() == SyntaxKind::Comma {
            let next = node.next_sibling();
            if next.is_some() && next.unwrap().kind().is_grouping() {
                writer.push(&text);
            } else {
                writer.push(&text).newline_with_indent();
            }
        } else if node.is::<ast::Space>() {
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
