use typst_syntax::LinkedNode;

use super::utils;
use crate::Config;

/// Writer is used to write your formatted output.
///
/// It comes with the following features :
/// - Markers: you place a mark by calling [Writer::mark], you can use this mark
/// to jump back and redo your formatting should you see it didn't respect some rules.
///
/// Example :
/// ```ignore
/// fn visit_params(/* */) {
///     let mark = self.mark();
///     visit_params_tight(self);
///     if !line_is_too_long(ctx.string_after_mark(mark)) {
///         // we're done yay
///         return
///     } else {
///         ctx.rewind(mark);
///         visit_params_long();
///     }
/// }
/// ```
/// - Indent, Dedent, Preserve: You must specify where you want indent to start and end.
/// It will be applied as a later step respecting the Preserve markers where indentation is
/// not applied (Raw, fmt::off, etc.)
pub(crate) struct Writer<'a> {
    pub(crate) config: Config,
    pub(crate) off: bool,
    pub(crate) buffer: &'a mut String,
}

pub(crate) struct Mark;

impl<'a> Writer<'a> {
    pub(crate) fn new(config: Config, buffer: &'a mut String) -> Self {
        Self {
            config,
            off: false,
            buffer,
        }
    }
    // needed when post process adds indentation level.
    pub fn mark_indent(&mut self) {
        // TODO
    }
    pub fn mark_dedent(&mut self) {
        // TODO
    }

    pub fn mark_preserve(&mut self) -> Mark {
        todo!()
    }

    pub fn wrap_preserve(&mut self, mark: Mark) {}

    pub(crate) fn push_node(&mut self, node: &LinkedNode) {
        self.buffer.push_str(node.text())
    }

    pub(crate) fn push_node_spaced(&mut self, n: &LinkedNode) {
        self.push_spaced(n.text())
    }

    pub(crate) fn push_str(&mut self, s: &str) {
        self.buffer.push_str(s)
    }

    pub(crate) fn push_spaced(&mut self, s: &str) {
        if s == "" {
            return;
        }

        self.buffer.push_str(s);

        if utils::last_line_length(self.buffer) >= self.config.max_line_length {
            self.new_line()
        } else {
            self.space()
        }
    }

    pub(crate) fn new_line(&mut self) {
        self.buffer.push('\n')
    }

    pub(crate) fn space(&mut self) {
        self.buffer.push(' ')
    }
}
