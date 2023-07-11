use super::*;

make_test!(call_func_empty, "#f()");
make_test!(call_func_simple, "#f(1,2,3)");
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
