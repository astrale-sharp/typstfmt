Basic formatter for the Typst language ~~with a future~~ without a future 😄!

It's been a fun ride everyone but that's where I stop, feel free to fork etc.

last typst supported version is 0.10.

If I get the formatter fever again I'll probably try contributing to https://github.com/Enter-tainer/typstyle/ and you should check it out ;).

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
  
  The following lines show the contents of a simple `typstfmt.toml` file:
  ```toml
  indent_space = 2
  max_line_length = 80
  experimental_args_breaking_consecutive = false
  line_wrap = true
  use_tabs = false
  ```
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

Optionally, you can setup a git hook to format your files at each commit:

```sh
echo "set -e

for f in \$(git ls-files --full-name -- '*.typ') ; do
    typstfmt --check \$f --verbose
done" > .git/hooks/pre-commit

chmod +x .git/hooks/pre-commit
```

Now if you try to commit unformatted files, they will be caught and the commit will fail, telling you which file should be fixed.

> Notes:
> - You should probably avoid doing this at the moment, as typstfmt is not quite stable yet
> - Be careful if you have another commit hook setup, as the command above will replace it entirely!

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
- @taooceros for the alignment of math block
