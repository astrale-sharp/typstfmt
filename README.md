*The crate is published with empty Typst*

Howdy! This is a formatter for the Typst language!

# Features

Basic non configurable formatting.

# Contributing

## Contributing via feature request and issue 
To push this formatter further, I need to know what you don't like.

You can :
- request features (something you'd like configurable)
- show a problematic snippet of code.

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

To contribute you could help making new rules, choosing the order in which they are selected, write test, get familiar with typst AST etc or take a look at the issue on this repo!

Another possible axe of contribution is  optimisation, which I have given 0 thoughts about so far.

If there is no issue open, discussion before pull requests is appreciated.