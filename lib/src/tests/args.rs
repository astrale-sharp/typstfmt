use super::*;

make_test!(call_func_empty, "#f()");
make_test!(call_func_simple, "#f(1,2,3)");
make_test!(
    call_func_long,
    "#f(1,this_is_absurdly_loooooooooong,3)",
    Config {
        max_line_length: 1,
        ..Default::default()
    },
    ignore_ast
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
    },
    ignore_ast
);

make_test!(
    parenthesized_comment,
    PARENTHESIZED_COMMENT,
    Config::default(),
    ignore_ast
);

const PARENTHESIZED_COMMENT: &str = "#func(
    ..v_or_hline,
    start: start,
    end: end,
    parent: v_or_hline  // the one that generated this
)";
