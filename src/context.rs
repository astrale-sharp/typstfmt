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

impl Default for Context {
    fn default() -> Self {
        Context { 
            buffer: String::with_capacity(1024),
            style: Style::default(),
            indent_level: 0
        }
    }
}

/// A context object used to store state while formatting.
#[derive(Clone)]
pub struct Context {
    /// The buffer used to store the formatted text.
    buffer: String,
    /// The style to use for formatting the text.
    style: Style,
    /// The current indentation level, in spaces.
    indent_level: usize
}

impl Context {

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_indent_level(mut self, indent: usize) -> Self {
        self.indent_level = indent;
        self
    }

    pub fn with_buffer(mut self, buffer: &str) -> Self {
        self.buffer = buffer.to_string();
        self
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn indent_level(&self) -> usize {
        self.indent_level
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
    pub fn indent(&mut self) {
        self.buffer.push_str(&" ".repeat(self.style.indent));
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
    pub fn push(&mut self, s: &str) {
        self.buffer.push_str(s);
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
    pub fn newline_with_indent(&mut self) {
        self.newline();
        self.push(" ".repeat(self.indent_level).as_str());
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
    pub fn newline(&mut self) {
        self.push("\n");
    }
    
    /// Appends a space to the buffer.
    pub fn space(&mut self, count: Option<usize>) {
        self.push(" ".repeat(count.unwrap_or(1)).as_str());
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
    pub fn update_indent<F>(&mut self, update_fn: F)
    where
        F: FnOnce(usize) -> usize
    {
        self.indent_level = update_fn(self.indent_level);
    }

    /// Increases the current indentation level by the amount specified in the style.
    pub fn inc_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_add(self.style.indent);
    }

    /// Decreases the current indentation level by the amount specified in the style.
    pub fn dec_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(self.style.indent);
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
    pub fn with_indent<F>(ctx: &mut Self, f: F)
    where
        F: FnOnce(&mut Self) -> ()    
    {
        ctx.inc_indent();
        f(ctx);
        ctx.dec_indent();
    }
}
