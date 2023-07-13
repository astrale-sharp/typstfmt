*The crate was published with empty Typst to crates.io mostly to reserve the name for the typst dev if they wanted it later (and they can have the name anytime). This means that this formatter will likely change it's name (typfmt) at some point (or become the official one but I likely won't be the main dev then)*

In progress formatter for the Typst language!

- [Goals](#goals)
- [State](#state)
- [Contributing](#contributing)
- [Architecture](#architecture)
  - [Main logic](#main-logic)
- [Testing and visualizing](#testing-and-visualizing)


# Goals

- Decent output under any circumstances, anything not decent should be reported as a bug!
- Fast, Small, configurable and embeddable library and binary ! 

# State

Currently the output is almost never decent, when it will be I'll add a bin target!

# Contributing
- feel free to open issue or discuss! I don't have github notifications so also feel free to go ping me on the typst discord server (at Astrale).
- once discussed, you may open a PR, not before cause I'm a bit chaotic and this is wip so things change fast and I would hate it if you lost your time.

# Architecture

## Main logic

Since we're visiting a AST (which is a tree) we have a recursive function
`visit(node: &LinkedNode, ctx: &mut Ctx)` that meets all the nodes in the tree.

It formats the children first (bottom up), the the parent decide what to do with their children.

Children have access to arbitrary context (they can know the kind of their parents, who are their siblings etc).


# Testing and visualizing

We use insta! If you don't have it installed take a look [here](https://insta.rs/docs/cli/) (I advise installing with [cargo binstall](https://github.com/cargo-bins/cargo-binstall) since I have a small computer and don't like waiting for things to compile)

one liner for cargo binstall : `curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash`

then `cargo binstall cargo-insta`

If you're contributing tests you should add a test case under `src/tests` for instance: `make_test!(call_func_empty, "#f()");`

you may also explore by modifying an existing test (add a space somewhere) and running `cargo insta test && cargo insta review`