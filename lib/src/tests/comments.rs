use super::*;

make_test!(param_comment, PARAMS_COMMENT);
make_test!(many_comments, TABLEX_COMMENTS,);
make_test!(parenth_comment, PARENTH_COMMENT,);
make_test!(array_2comment, ARRAY_2COMMENT,);
make_test!(array_end_comment, ARRAY_END_COMMENT,);
make_test!(block_comment, BLOCK_COMMENT);
make_test!(block_comment_nested, BLOCK_COMMENT_NESTED);
make_test!(code_comment, CODE_COMMENT);
make_test!(end_comments, END_COMMENTS);
make_test!(start_with_comment, START_WITH_COMMENT);
make_test!(
    args_comment_end,
    "#func(
  ..v_or_hline,
  start: start,
  end: end,
  parent: v_or_hline  // the one that generated this
)"
);

const PARAMS_COMMENT: &str = r#"#let hlinex(
  stroke-expand: true,
  expand: none, // some comment here
) = ()"#;

const END_COMMENTS: &str = r#"#{
  right-expand += 4 / 2 // comment
  right-expand += 4 / 2 // comment
  }"#;

const TABLEX_COMMENTS: &str = r#"#let convert-length-to-pt(len,styles: none, page_size: none, frac_amount: none, frac_total: none
  ) = { if ratio == none {  // 2em + 5pt  (doesn't contain 100% or something)
  measure(line(length: len), styles).width} else {  // 100% + 2em + 5pt  --> extract the "100%" part
  [  4  ]}}"#;

const PARENTH_COMMENT: &str = r#"#(
    true //some comment
    or false)"#;

const ARRAY_2COMMENT: &str = r#"#(
        true, //some comment
        // some other comment
        false)"#;

const ARRAY_END_COMMENT: &str = r#"#(
    true,
    false // some other comment
)"#;

const CODE_COMMENT: &str = r#"#if col == auto {
  // max cell width
  let col_size = grid-get-column(grid, i)
}"#;

const BLOCK_COMMENT: &str = r#"#if is_last_row {
  row_group_height -= row_gutter_dy
  // one less gutter at the end

}"#;

const BLOCK_COMMENT_NESTED: &str = r#"#for row in group-rows {
  for cell_box in row {
    // place the cell!
    func()
    // let box_h = measure(cell_box.box, styles).height
    // tallest_box_h = calc.max(tallest_box_h, box_h)
  }
}"#;

const START_WITH_COMMENT: &str = r#"#table(
  // Comment
  fill: value,
  [*Pros*],)
"#;
