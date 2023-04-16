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

/// A context object used to store state while formatting.
pub struct Writer<'a> {
    final_result: &'a mut String,
    value: String,
    /// The style to use for formatting the text.
    pub style: Style,
    /// The current indentation level, in spaces.
    indent_level: usize,
}

impl<'a> Writer<'a> {
    pub fn default(s: &'a mut String) -> Self {
        Self {
            final_result: s,
            indent_level: 0,
            style: Default::default(),
            value: Default::default(),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[allow(dead_code)]
    pub fn with_indent_level(mut self, indent: usize) -> Self {
        self.indent_level = indent;
        self
    }

    pub fn with_value(mut self, s: impl Into<String>) -> Self {
        self.value = s.into();
        self
    }

    // todo test me
    /// Ignore whitespace.
    pub fn current_line_length(&self, s: &String) -> usize {
        fn len_no_space(s: &str) -> usize {
            s.len() - s.chars().filter(|x| x == &' ').count()
        }
        let Some(last_line) = self.final_result.lines().last() else {
            if let Some(app) = s.lines().last() {
                println!("no last line");
               return len_no_space(app);
        } else {
                println!("no last line and no app lines");
                return 0;
            }
        };
        if !s.contains('\n') {
            len_no_space(last_line) + len_no_space(s)
        } else {
            len_no_space(s.split('\n').last().unwrap())
        }
    }

    pub fn flush(&mut self) {
        self.final_result.push_str(&self.value)
    }
    // #[must_use]
    // pub fn indent_level(&self) -> usize {
    //     self.indent_level
    // }

    /// Appends the amount of spaces defined by the style.
    // pub fn indent(&mut self) -> &mut Self {
    //     self.value.push_str(&" ".repeat(self.style.indent));
    //     self
    // }

    /// Appends the given text to the buffer.
    /// # Arguments
    ///
    /// * `s` - The text to append to the buffer with.
    pub fn push(&mut self, s: &str) -> &mut Self {
        self.value.push_str(&s);
        self
    }

    /// Appends a newline character to the buffer, followed by
    /// the current indentation level in spaces.
    pub fn newline_with_indent(&mut self) -> &mut Self {
        self.newline();
        self.push(" ".repeat(self.indent_level).as_str());
        self
    }

    /// Appends a newline character to the buffer.
    pub fn newline(&mut self) -> &mut Self {
        self.push("\n");
        self
    }

    /// Appends a space to the buffer.
    // pub fn space(&mut self, count: Option<usize>) -> &mut Self {
    //     self.push(" ".repeat(count.unwrap_or(1)).as_str());
    //     self
    // }

    /// Updates the indentation level.
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn indented<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self) -> (),
    {
        self.inc_indent();
        f(self);
        self.dec_indent();
        self
    }

    /// The current value holded by the writer
    pub fn value(&self) -> &String {
        &self.value
    }

    /// empties the writer and takes it's value but preserve it's state.
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
        let mut res = String::from("");
        let mut writer = Writer::default(&mut res);
        similar_asserts::assert_eq!(writer.value(), "");
        writer.inc_indent();
        writer.newline_with_indent();
        writer.push("Hello, World!");
        similar_asserts::assert_eq!(writer.value(), "\n    Hello, World!");
    }

    #[test]
    fn creation() {
        let mut res = String::from("");
        let writer = Writer::default(&mut res)
            .with_value("Hello")
            .with_style(Style { indent: 2 });
        similar_asserts::assert_eq!(writer.value(), "Hello");
        similar_asserts::assert_eq!(writer.style.indent, 2);
    }

    #[test]
    fn complex() {
        let mut res = String::from("");
        let mut writer = Writer::default(&mut res);
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
        let mut res = String::from("");
        let mut writer = Writer::default(&mut res).with_indent_level(0);
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
