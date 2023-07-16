use super::*;

make_test!(call_func_empty, "#f()");
make_test!(call_func_simple, "#f(1,2,3)");
make_test!(parenthesized_not_array, "#(auto)");
make_test!(array_not_parenthesized, "#(auto,)");
make_test!(parenthesized_not_array_break, "#(\nauto)");
make_test!(array_not_parenthesized_break, "#(\nauto,)");
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
    args_comment_end,
    ARGS_COMMENT_END,
    Config::default(),
    ignore_ast
);

make_test!(
    parenth_comment_end,
    "#(\ntrue// comment\n)",
    Config::default(),
    ignore_ast
);
make_test!(
    func_comment_end,
    "#f(\ntrue// comment\n)",
    Config::default(),
    ignore_ast
);

const ARGS_COMMENT_END: &str = "#func(
    ..v_or_hline,
    start: start,
    end: end,
    parent: v_or_hline  // the one that generated this
)";
