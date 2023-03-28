/// Contains style parameters used while formatting.
#[derive(Clone, Copy)]
pub struct Style {
    /// The amount of space by which to indent.
    indent: usize
}

impl Default for Style {
    fn default() -> Self {
        Style {
            indent: 4,
        }
    }
}

impl Default for Writer {
    fn default() -> Self {
        Writer { 
            result: String::with_capacity(1024),
            buffer: None,
            style: Style::default(),
            indent_level: 0
        }
    }
}

/// A context object used to store state while formatting.
#[derive(Clone)]
pub struct Writer {
    /// The final result of the Writer.
    result: String,
    /// The buffer used to store Strings which are to be appended to the result.
    buffer: Option<String>,
    /// The style to use for formatting the text.
    pub style: Style,
    /// The current indentation level, in spaces.
    indent_level: usize
}

impl Writer {

    pub fn new(init: String, style: Style, indent_level: usize) -> Self {
        Self { 
            result: init,
            buffer: None,
            style,
            indent_level
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

    pub fn with_init(mut self, init: String) -> Self {
        self.result = init;
        self
    }

    #[must_use]
    /// Access the writer's result.
    pub fn result(&self) -> &String {
        &self.result
    }

    #[must_use]
    /// Access the writer's buffer.
    pub fn buffer(&self) -> &Option<String> {
        &self.buffer
    }

    #[must_use]
    pub fn indent_level(&self) -> usize {
        self.indent_level
    }

    /// Empties the current buffer, if there is any.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut writer = Writer::default();
    /// writer.buffered(false, |w| { 
    ///     w.push("hello");
    /// });
    /// assert_eq!(writer.buffer(), Some("hello"));
    /// writer.empty_buffer();
    /// assert_eq!(writer.buffer(), None);
    /// ```
    pub fn empty_buffer(&mut self) -> &mut Self {
        self.buffer = None;
        self
    }

    /// Flushes the buffer by emptying it and push the content onto the
    /// `Writer`'s result.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut writer = Writer::default().with_init("hello, ");
    /// writer.buffered(false, |w| {
    ///     w.push("world");
    /// });
    /// assert_eq!(writer.buffer(), Some("world"));
    /// assert_eq!(writer.result(), "hello, ");
    /// writer.flush();
    /// assert_eq!(writer.buffer(), None);
    /// assert_eq!(writer.result(),"hello, world");
    /// ```
    pub fn flush(&mut self) {
        if self.buffer.is_some() {
            // we may safely unwrap here, because we mande sure the buffer is not `None`.
            self.result.push_str(self.buffer.take().unwrap().as_str());
        }
    }

    /// Apply a function to the current buffer before flushing it.
    pub fn flush_buffer_with<F>(&mut self, with_fn: F)
    where
        F: FnOnce(&String) -> String
    {
        self.buffer.as_ref().map(with_fn);
        self.flush();
    }

    /// Operate the `Writer` in buffered mode. 
    /// 
    /// Every operation in `f` will be compute in a seperate writer with,
    /// however, the same style and indent level. Finally, this seperate writer's
    /// result will be saved in the current writer's buffer or, if the `flush` flag
    /// is set, result will be saved in the current writer's buffer.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut writer = Writer::default();
    /// assert_eq!(writer.result(), "");
    /// assert_eq!(writer.buffer(), None);
    /// writer.buffered(false, |w| {
    ///     w.push("hello,")
    ///         .newline()
    ///         .push("world");
    /// });
    /// assert_eq!(writer.result(), "");
    /// assert_eq!(writer.buffer(), "hello,\nworld");
    /// ```
    pub fn buffered<F>(&mut self, flush: bool, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self)
    {
        let mut w = self.clone();
        w.buffer = None;
        w.result = String::new();
        f(&mut w);
        let buffer = w.result.clone();
        if flush { 
            self.result = buffer;
        } else {
            self.buffer = Some(buffer);
        }
        self
    }

    /// All operations in `f` will not be persistent, that is, any changes in 
    /// the state of the writer will not be saved in the current writer. However,
    /// the writer available in `f` will be identical to the current writer.
    pub fn non_persistent<F>(&self, f: F) -> String
    where
        F: FnOnce(&mut Writer)
    {
        let mut w = self.clone();
        f(&mut w);
        w.result
    }

    /// Appends the amount of spaces defined by the style.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut ctx = Context::default();
    /// assert_eq!(ctx.buffer(), "");
    /// ctx.indent();
    /// // assuming the default indention of 4 spaces is used
    /// assert_eq!(ctx.buffer(), "    ");
    /// ````
    pub fn indent(&mut self) -> &mut Self {
        self.result.push_str(&" ".repeat(self.style.indent));
        self
    }

    /// Appends the given text to the buffer.
    ///
    /// # Arguments
    ///
    /// * `s` - The text to append to the buffer with.
    ///
    /// # Example
    ///
    /// ```
    /// let mut ctx = Context::default();
    /// ctx.push("Hello, world!");
    /// assert_eq!(ctx.buffer, "Hello, world!");
    /// ```
    pub fn push(&mut self, s: &str) -> &mut Self {
        self.result.push_str(&s);
        self
    }

    /// Appends a newline character to the buffer, followed by the current indentation level in spaces.
    ///
    /// # Example
    ///
    /// ```
    /// let mut ctx = Context::new();
    /// ctx.newline_with_indent();
    /// // assuming the default indention of 4 spaces is used
    /// assert_eq!(ctx.buffer, "\n    ");
    /// ```
    pub fn newline_with_indent(&mut self) -> &mut Self {
        self.newline();
        self.push(" ".repeat(self.indent_level).as_str());
        self
    }

    /// Appends a newline character to the buffer.
    ///
    /// # Example
    ///
    /// ```
    /// let mut ctx = Context::new();
    /// ctx.newline();
    /// assert_eq!(ctx.buffer(), "\n");
    /// ```
    pub fn newline(&mut self) -> &mut Self {
        self.push("\n");
        self
    }
    
    /// Appends a space to the buffer.
    pub fn space(&mut self, count: Option<usize>) -> &mut Self {
        self.push(" ".repeat(count.unwrap_or(1)).as_str());
        self
    }

    /// Updates the indentation level.
    /// 
    /// # Example
    /// 
    /// ```
    /// let mut ctx = Context::default();
    /// assert_eq!(ctx.indent_level(), 4);
    /// ctx.update_indent(|i| i * 2);
    /// assert_eq!(ctx.indent_level(), 8);
    /// ````
    pub fn update_indent<F>(&mut self, update_fn: F) -> &mut Self
    where
        F: FnOnce(usize) -> usize
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
    /// # Example
    /// 
    /// ```
    /// let mut ctx = Context::default().with_buffer("f("));
    /// ctx.newline();
    /// ctx.with_indent(|ctx| {
    ///     ctx.push("a, b");
    /// });
    /// ctx.newline_with_indent();
    /// ctx.push(")")
    /// assert_eq!(ctx.buffer(), "f(\n    a, b\n)");
    /// ```
    pub fn indented<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Self) -> ()    
    {
        self.inc_indent();
        f(self);
        self.dec_indent();
        self
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let writer = Writer::new(
            "Hello".to_string(),
            Style { indent: 2 },
            4
        );
        assert_eq!(writer.result(), "Hello");
        assert_eq!(writer.indent_level, 4);
        assert_eq!(writer.style.indent, 2);
    }

    #[test]
    fn test_complex() {
        let mut writer = Writer::default();
        let indent = writer.style.indent;
        writer.push("f(")
            .indented(|w| {
                w.newline_with_indent()
                    .push("a,")
                    .newline_with_indent()
                    .push("b");
            })
            .newline_with_indent()
            .push(")");
        assert_eq!(writer.result, format!("f(\n{}a,\n{}b\n)", " ".repeat(indent), " ".repeat(indent)));
    }

    #[test]
    fn test_indent_change() {
        let mut writer = Writer::default().with_indent_level(0);
        let indent_style = writer.style.indent;
        assert_eq!(writer.indent_level, 0);
        writer.inc_indent();
        assert_eq!(writer.indent_level, 0 + indent_style);
        writer.inc_indent();
        assert_eq!(writer.indent_level, 0 + 2 * indent_style);
        writer.dec_indent();
        assert_eq!(writer.indent_level, 0 + indent_style);
        writer.update_indent(|i| i - indent_style);
        assert_eq!(writer.indent_level, 0);
        writer.dec_indent();
        assert_eq!(writer.indent_level, 0);
    }

    #[test]
    fn test_buffered() {
        let mut writer = Writer::default();
        let indent = writer.style.indent;
        writer.push("let add(x, y) =");
        writer.buffered(false, |w| {
            w.inc_indent()
                .newline_with_indent()
                .push("x + y");
        });
        assert_eq!(writer.buffer, Some(format!("\n{}x + y", " ".repeat(indent))));
        assert_eq!(writer.result, "let add(x, y) =");
        writer.flush();
        assert_eq!(writer.result, format!("let add(x, y) =\n{}x + y", " ".repeat(indent)));
    }
}
