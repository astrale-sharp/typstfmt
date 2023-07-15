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

make_test!(param_comment, PARAMS_COMMENT);
make_test!(
    many_comments,
    TABLEX_COMMENTS,
    Config::default(),
    ignore_ast
);
//TODO
// the last line ends in `2// comment` instead of `2 // comment`
// it's probably linked to the trim-line happening in push_raw_indent.
// make_test!(end_comments, END_COMMENTS);

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

const TABLEX_COMMENTS: &str = r#"#let convert-length-to-pt(len,styles: none, page_size: none, frac_amount: none, frac_total: none
) = { if ratio == none {  // 2em + 5pt  (doesn't contain 100% or something)
measure(line(length: len), styles).width} else {  // 100% + 2em + 5pt  --> extract the "100%" part
[  4  ]}}"#;

const END_COMMENTS: &str = r#"#{
right-expand += 4 / 2 // comment
right-expand += 4 / 2 // comment
}"#;
