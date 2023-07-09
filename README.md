*The crate is published with empty Typst*

Howdy! This is a formatter for the Typst language!
This hasn't been worked on for a while but I might start again soon.

# Features

Basic non configurable formatting.

# Contributing

## Contributing via feature request and issue 
To push this formatter further, I need to know what you don't like.

You can :
- request features (something you'd like configurable)
- open an issue with a snippet of code that is not formatted how you'd like it.

## Contributing to the Code Base

The formatting is done using Rules: 

```rust

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
```

To contribute you could:
- Choose an ignored test and make it pass, this will often require you to ad or modify a rule or the order in which they are applied. Sometimes you'll have to modify the Context struct.
- Add a test containing an interesting snippet of code. It will usually look like this: 
```rust
test_snippet!(
    no_apply_if_not_in_code_block,
    ignore = "unimplemented",
    expect = "#f()",
    "#f()",
    &[IdentItemFunc.as_dyn()]
);
```

If there is no open issue, discussion before opening a pull requests is appreciated.
