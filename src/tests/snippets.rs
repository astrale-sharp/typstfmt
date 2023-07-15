use super::*;

make_test!(code_func, "#{f(1,2,3)}");
make_test!(
    code_func_break,
    "#{f(1,2,3)}",
    Config {
        max_line_length: 2,
        ..Default::default()
    },
    ignore_ast
);
make_test!(
    code_func_break_nested,
    "#{{f(1,2,3)}}",
    Config {
        max_line_length: 2,
        ..Default::default()
    },
    ignore_ast
);
make_test!(while_loop, WHILE_LOOP);
make_test!(for_loop, FOR_LOOP);
make_test!(official, OFFICIAL);
make_test!(
    let_closure_params_named,
    TABLEX,
    Config::default(),
    ignore_ast
);

// todo
// this is not passing since we'd like the last line to look like this:
// `  expand: none, // some comment here`
make_test!(param_comment, PARAMS_COMMENT);

const FOR_LOOP: &str = r#"#for k in range(5) {
    repr(k) + " " 
}"#;

const WHILE_LOOP: &str = r#"#let i = 0
#while true {
  i += 1
  if i > 15 { break }
  repr(i) + " "
}"#;

const PARAMS_COMMENT: &str = r#"#let hlinex(
  stroke-expand: true,
  expand: none, // some comment here
) = ()"#;

const OFFICIAL: &str = r#"Glaciers as the one shown in
@glaciers will cease to exist if
we don't take action soon!

#figure(
  image("glacier.jpg", width: 70%),
  caption: [
    _Glaciers_ form an important part
    of the earth's climate system.
  ],
) <glaciers>"#;

// this is taken from tablex by Pg Biel whom we love.
const TABLEX: &str = r#"#let is-tablex-dict(x) = (
  type(x) == "dictionary"
      and "tablex-dict-type" in x
)
"#;
