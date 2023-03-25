*PUBLISHING WITH EMPTY TYPST*

Howdy, this *will be* a formatter for the typst language.

Currently on a very early state, Pull Requests are more than welcome

To contribute the main thing you should help with is making new rules, choosing the order in which they are selected, write test, get familiar with typst AST etc.

When we have a working formatter it will be time to think about optimisations although all ideas are welcomed now as well!

```rust


trait Rule {
    fn accept(&self, syntax_node: &SyntaxNode, context: ()) -> bool;

    fn eat(&self, syntax_node: &SyntaxNode) -> String;

    fn as_dyn(self: Self) -> Box<dyn Rule>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

```