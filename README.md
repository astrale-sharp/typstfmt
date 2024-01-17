# Typstfmt

Powerful formatter for the Typst language with a future!

- [Typstfmt](#typstfmt)
- [Goals](#goals)
- [Roadmap and State](#roadmap-and-state)
- [Features](#features)
- [Installing](#installing)
- [Contributing](#contributing)
  - [Using insta here](#using-insta-here)
  - [Architecture](#architecture)
- [Setting up a pre-commit hook](#setting-up-a-pre-commit-hook)
- [Showcase](#showcase)
- [Thanks (chronological)](#thanks-chronological)

# Goals

- Decent output under any circumstances, anything not decent should be reported
  as a bug!
- Fast, Small, configurable and embeddable library and binary!
- Good default (see [roadmap](#roadmap))

# Roadmap and State

This project is at the end of it's first big refactoring (on the refactor branch).
This was needed in order to fix long standing bug like nest raw blocks formatting and such.

- The groundwork is almost done and we have example of visit functions, 
  the priority is to perfect our visits functions to get awesome results and see if the current design is sufficient.
- help wanted to reintroduce the features.
- Maybe dump our config files and just read typst.toml


# Features

- Good defaults.
- Tries different strategy and strives for an optimal result.
- Disable the formatting by surrounding code with `// typstfmt::off` and `//
  typstfmt::on`.
- Config file: run `typstfmt --make-default-config` to create a typstfmt.toml
  file that you can customize! Support for global configuration as well.


# Installing

```sh
cargo install --git https://github.com/astrale-sharp/typstfmt.git
```


# Contributing

- feel free to open issue or discuss! I don't have github notifications so also
  feel free to go ping me on the typst discord server (@Astrale).
- once discussed, you may open a PR, not before cause I'm a bit chaotic and
  this is wip so things change fast and I would hate it if you lost your time.
- I would encourage you to take a look at the existing code and ask questions, or try to 
  remove todos.
  A lot of the old features can probably be easily adapted.

## Using insta here

We use insta! If you don't have it installed take a look
[here](https://insta.rs/docs/cli/) (hint: use  [`cargo
binstall`](https://github.com/cargo-bins/cargo-binstall))


`cargo insta test && cargo insta review`


## Architecture

1. We modify the Ast to get a Format Tree containing only relevant info.
2. We apply a preserve pass on the tree to mark nodes that should be preserved
3. We visit the tree with a &mut Writer, we have the capacity to rewind time if things didn't go well
4. We format from top to bottom with (todo) optional back and forth between a parent and it's child for maximum style


# Setting up a pre-commit hook

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


# Showcase

To see how it currently formats all the snippets:

- run `cargo test --workspace`, a failing test indicates one of the snippets displayed in
  the next step is not formatted like this anymore.
- run `show_all.sh`


# Thanks (chronological)

- @arnaudgolfouse, for the discussion, designs and the precious friendship.
- @Dherse for the support and testing.
- The Typst community for the good vibes, the
  interesting talks, the support and ofc, Typst.
- @jeffa5 for contributing ideas on the initial design.
- @Andrew15-5, for the many suggestions, issues and feedback.
- @aghriss for a bug fix.
- @taooceros for the alignement of math block.