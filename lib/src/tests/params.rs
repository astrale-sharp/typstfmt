use super::*;

test_eq!(call_func_empty, "#f()");
make_test!(call_func_simple, "#f(1,2,3)");
test_eq!(parenthesized_not_array, "#(auto)");
test_eq!(array_not_parenthesized, "#(auto,)");
make_test!(parenthesized_not_array_break, "#(\nauto)");
make_test!(array_not_parenthesized_break, "#(\nauto,)");
make_test!(
    call_func_long,
    "#f(1,this_is_absurdly_loooooooooong,3)",
    Config {
        max_line_length: 1,
        ..Default::default()
    }
);
make_test!(
    call_func_long_trailing,
    "#f(1,this_is_absurdly_loooooooooong,3,)",
    Config {
        max_line_length: 1,
        ..Default::default()
    }
);
make_test!(
    dont_break_for_one_arg,
    "#f(this_is_absurdly_loooooooooong)",
    Config {
        max_line_length: 1,
        ..Default::default()
    }
);
make_test!(
    dont_break_for_one_arg_with_trail,
    "#f(this_is_absurdly_loooooooooong , )",
    Config {
        max_line_length: 1,
        ..Default::default()
    }
);
make_test!(code_func, "#{f(1,2,3)}");
make_test!(
    code_func_break,
    "#{f(1,2,3)}",
    Config {
        max_line_length: 2,
        ..Default::default()
    },
);
make_test!(
    code_func_break_nested,
    "#{{f(1,2,3)}}",
    Config {
        max_line_length: 2,
        ..Default::default()
    },
);
make_test!(parenth_comment_end, "#(\ntrue// comment\n)");
make_test!(func_comment_end, "#f(\ntrue// comment\n)",);
make_test!(
    trailing,
    "#f()[
    something
    ]"
);
make_test!(
    destructuring_comma,
    "#let func((a,)) = {
}"
);
test_eq!(destructuring_no_trailing, "#let (a, b) = (1, 2)");
// this is taken from tablex by Pg Biel whom we love.
make_test!(
    let_closure_params_named,
    r#"#let is-tablex-dict(x) = (
  type(x) == "dictionary"
      and "tablex-dict-type" in x
)
"#,
);
// TODO: don't take into account the trailing args for breaking?
// make_test!(many_trailings, "#f()[][veeeeeeeeeeeeeeeeeeeeeery][][][][][loooooooooooooooooooong]");
make_test!(
    one_long_content_arg,
    "#very-long-long-long-long-long-function-name(
  [Lorem ipsum dolor sit amet, consectetur
  adipiscing elit, sed do eiusmod tempor]
)"
);
test_eq!(
    trailing_one_indent,
    "f[ this loooooooooooooooooooooooooooong text is not supposed to not be indented
at all ]"
);
