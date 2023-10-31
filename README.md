Basic formatter for the Typst language with a future!

- [Goals](#goals)
- [Features](#features)
- [State](#state)
- [Installing](#installing)
  - [Setting up a pre-commit hook](#setting-up-a-pre-commit-hook)
- [Contributing](#contributing)
- [Architecture](#architecture)
  - [Main logic](#main-logic)
  - [Roadmap](#roadmap)
- [Testing and visualizing](#testing-and-visualizing)
  - [Installing Insta](#installing-insta)
  - [Using insta here](#using-insta-here)
    - [Can I see it in action?](#can-i-see-it-in-action)
    - [Is that all I have to help me test?](#is-that-all-i-have-to-help-me-test)
      - [Tracing](#tracing)
      - [Fmttest (TO BE IMPLEMENTED)](#fmttest-to-be-implemented)
- [Thanks (chronological)](#thanks-chronological)

# Goals

- Decent output under any circumstances, anything not decent should be reported
  as a bug!
- Fast, Small, configurable and embeddable library and binary!
- Good default (see [roadmap](#roadmap))

# Features

- Good defaults.
- Config file: run `typstfmt --make-default-config` to create a typstfmt.toml
  file that you can customize!
- Disable the formatting by surrounding code with `// typstfmt::off` and `//
  typstfmt::on`. (Experimental and broken)

# State

It's not always pretty, it sometimes break the code in math mode, but it should
be safe for code and markup.

# Installing

```sh
cargo install --git https://github.com/astrale-sharp/typstfmt.git
```

## Setting up a pre-commit hook

You can set up a git [hook](https://pre-commit.com).

Every `git commit`, will then format automatically every .typ file before
committing.

run:

```sh
echo "\
repos:
  - repo: https://github.com/astrale-sharp/typstfmt
    rev: 1c414de
    hooks:
      - id: typstfmt
" > .pre-commit-config.yaml
```

to add a configured `.pre-commit-config.yaml` file.

You should then run:

```sh
pre-commit install
pre-commit autoupdate
```

And your set up is done!

# Contributing

- feel free to open issue or discuss! I don't have github notifications so also
  feel free to go ping me on the typst discord server (@Astrale).
- once discussed, you may open a PR, not before cause I'm a bit chaotic and
  this is wip so things change fast and I would hate it if you lost your time.

# Architecture

## Main logic

Since we're visiting a AST (which is a tree) we have a recursive function
`visit(node: &LinkedNode, ctx: &mut Ctx)` that meets all the nodes in the tree.

It formats the children first (bottom up), then the parent decide what to do
with their children.

Children have access to arbitrary context (they can know the kind of their
parents, who are their siblings etc).

## Roadmap

Once the test suite is large enough and the formatting is satisfying, create an
abstraction to make the codebase easier to work with.

One person cannot come up with good formatting default. This will first be configurable and
then with experience and opinions from the community, default will be tuned.

# Testing and visualizing

## Installing Insta

We use insta! If you don't have it installed take a look
[here](https://insta.rs/docs/cli/) (hint: use  [`cargo
binstall`](https://github.com/cargo-bins/cargo-binstall))


## Using insta here

### Can I see it in action?

To see how it currently formats all the snippets:

- run `cargo test`, a failing test indicates one of the snippets displayed in
  the next step is not formatted like this anymore.
- run `show_all.sh`

### Is that all I have to help me test?

#### Tracing

Of course not! We have tracing enabled during tests!

If you're contributing tests you should add a test case under `src/tests` for
instance: `make_test!(call_func_empty, "#f()");`

then running your tests: `cargo test && cargo insta review`

If the info log isn't enough, run `DEBUG=true cargo test`. If you wish to pipe
to a file run `NO_COLOR=true cargo test` you may also set the `NOLOG` env
variable if you wish to disable logging entirely.

#### Fmttest (TO BE IMPLEMENTED)

On the fmttest branch, you can see the skeleton of a program that will automate
finding which range, when formatted, was not valid anymore (broke the semantic of the code).

# Thanks (chronological)

- @arnaudgolfouse, for the discussion, designs and the precious friendship.
- @laurmaedje, @reknih and the typst community for the good vibes, the
  interesting talks, the support and ofc, Typst.
- @jeffa5 for contributing ideas on the initial design
- @Andrew15-5, for the many suggestions, issues and feedback.
- @aghriss for a bug fix
- @taooceros for the alignement of math block