use super::*;
test_eq!(unchanged, " ");
make_test!(one_space, "  ");
make_test!(one_space_b, "   ");
make_test!(two_line_max, "\n\n\n");
make_test!(
    content_block_basic,
    "#[\n_Glaciers_ form an important part \nof the earth's climate system.]"
);
make_test!(
    content_block2,
    "#[
    something
    ]"
);
test_eq!(content_block_spaced, "[ 4 ]");
make_test!(content_block_too_spaced, "[  4  ]");
test_eq!(content_block_tight, "[4]");
make_test!(content_block_only_space, "[   ]");
make_test!(line_wrapping, "Lorem _ipsum_ dolor sit amet, _consectetur_ adipiscing elit, sed do eiusmod tempor incididunt ut labore.");
test_eq!(slash_space, r"#[\ ]");
make_test!(
    text_then_list,
    "We have next things:
- thing 1;
- thing 2;
- thing 3."
);
test_eq!(bug_quote_space, r#"don't "text" "text"#);
test_eq!(bug_space_around_inline_code, "a#lorem(2)b c d");
// TODO: before or at 49b62e8 this shouldn't pass the ast test but it does.
test_eq!(
    dont_break_heading,
    "= my loong loong loong loong loong loong loong loong loong loong loong loong heading"
);
test_eq!(backticks, "`Makefile`.");
test_eq!(math, "$$.");
test_eq!(escape, "C\\#");
test_eq!(
    dont_break_smartquote,
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa69'420"
);
test_eq!(equation_spaced, "aaa $ a b c $ bbb");
test_eq!(last_space_conserved, "#[. ]");
make_test!(last_space_conserved_as_space, "#[.\n]");
// TODO: first line doesn't respect last line length
make_test!(
    children_respect_max_line_length,
    r#"#[ Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et #[ Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris ] dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla ]"#
);
