use super::*;

make_test!(code_func, "#{f(1,2,3)}");
make_test!(
    code_func_break,
    "#{f(1,2,3)}",
    Config {
        max_line_length: 2,
        ..Default::default()
    }
);
make_test!(
    code_func_break_nested,
    "#{{f(1,2,3)}}",
    Config {
        max_line_length: 2,
        ..Default::default()
    }
);
make_test!(while_loop, WHILE_LOOP);
make_test!(for_loop, FOR_LOOP);

const FOR_LOOP: &str = r#"#for k in range(5) {
    repr(k) + " " 
}"#;

const WHILE_LOOP: &str = r#"#let i = 0
#while true {
  i += 1
  if i > 15 { break }
  repr(i) + " "
}"#;
