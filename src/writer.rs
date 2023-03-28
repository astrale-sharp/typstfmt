/// Contains style parameters used while formatting.
#[derive(Clone, Copy)]
pub struct Style {
    /// The amount of space by which to indent.
    indent: usize,
}

impl Default for Style {
    fn default() -> Self {
        Style { indent: 4 }
    }
}

impl Default for Writer {
    fn default() -> Self {
        Writer {
            style: Style::default(),
            indent_level: 0,
            value: String::new(),
        }
    }
}

/// A context object used to store state while formatting.
#[derive(Clone)]
pub struct Writer {
    value: String,
    /// The style to use for formatting the text.
    pub style: Style,
    /// The current indentation level, in spaces.
    indent_level: usize,
}

impl Writer {
    pub fn new(init: String, style: Style, indent_level: usize) -> Self {
        Self {
            style,
            indent_level,
            value: init,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_indent_level(mut self, indent: usize) -> Self {
        self.indent_level = indent;
        self
    }

    pub fn with_value(mut self, s: String) -> Self {
        self.value = s;
        self
    }

    #[must_use]
    pub fn indent_level(&self) -> usize {
        self.indent_level
    }

    /// Appends the amount of spaces defined by the style.
    pub fn indent(&mut self) -> &mut Self {
        self.value.push_str(&" ".repeat(self.style.indent));
        self
    }

    /// Appends the given text to the buffer.
    pub fn push(&mut self, s: &str) -> &mut Self {
        self.value.push_str(&s);
        self
    }

    pub fn newline_with_indent(&mut self) -> &mut Self {
        self.newline();
        self.push(" ".repeat(self.indent_level).as_str());
        self
    }

    pub fn newline(&mut self) -> &mut Self {
        self.push("\n");
        self
    }

    /// Appends a space to the buffer.
    pub fn space(&mut self, count: Option<usize>) -> &mut Self {
        self.push(" ".repeat(count.unwrap_or(1)).as_str());
        self
    }

    pub fn update_indent<F>(&mut self, update_fn: F) -> &mut Self
    where
        F: FnOnce(usize) -> usize,
    {
        self.indent_level = update_fn(self.indent_level);
        self
    }

    /// Increases the current indentation level by the amount specified in the style.
    pub fn inc_indent(&mut self) -> &mut Self {
        self.indent_level = self.indent_level.saturating_add(self.style.indent);
        self
    }

    /// Decreases the current indentation level by the amount specified in the style.
    pub fn dec_indent(&mut self) -> &mut Self {
        self.indent_level = self.indent_level.saturating_sub(self.style.indent);
        self
    }

    /// Executes the given function with an increased indentation level and decreases
    /// the indentation level after that the by the same amount.
    ///
    pub fn indented<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self) -> (),
    {
        self.inc_indent();
        f(self);
        self.dec_indent();
        self
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn take(&mut self) -> String {
        let result = self.value.clone();
        self.value = String::new();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_persistent() {
        let mut writer = Writer::default();
        similar_asserts::assert_eq!(writer.value(), "");
        writer.inc_indent();
        writer.newline_with_indent();
        writer.push("Hello, World!");
        similar_asserts::assert_eq!(writer.value(), "\n    Hello, World!");
    }

    #[test]
    fn creation() {
        let writer = Writer::new("Hello".to_string(), Style { indent: 2 }, 4);
        similar_asserts::assert_eq!(writer.value(), "Hello");
        similar_asserts::assert_eq!(writer.indent_level, 4);
        similar_asserts::assert_eq!(writer.style.indent, 2);
    }

    #[test]
    fn complex() {
        let mut writer = Writer::default();
        let indent = writer.style.indent;
        writer
            .push("f(")
            .indented(|w| {
                w.newline_with_indent()
                    .push("a,")
                    .newline_with_indent()
                    .push("b");
            })
            .newline_with_indent()
            .push(")");
        similar_asserts::assert_eq!(
            writer.value(),
            &format!("f(\n{}a,\n{}b\n)", " ".repeat(indent), " ".repeat(indent))
        );
    }

    #[test]
    fn indent_change() {
        let mut writer = Writer::default().with_indent_level(0);
        let indent_style = writer.style.indent;
        similar_asserts::assert_eq!(writer.indent_level, 0);
        writer.inc_indent();
        similar_asserts::assert_eq!(writer.indent_level, 0 + indent_style);
        writer.inc_indent();
        similar_asserts::assert_eq!(writer.indent_level, 0 + 2 * indent_style);
        writer.dec_indent();
        similar_asserts::assert_eq!(writer.indent_level, 0 + indent_style);
        writer.update_indent(|i| i - indent_style);
        similar_asserts::assert_eq!(writer.indent_level, 0);
        writer.dec_indent();
        similar_asserts::assert_eq!(writer.indent_level, 0);
    }
}
