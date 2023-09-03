use super::*;
make_test!(unchanged, " ");
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
make_test!(content_block_spaced, "[ 4 ]");
make_test!(content_block_too_spaced, "[  4  ]");
make_test!(content_block_tight, "[4]");
make_test!(content_block_only_space, "[   ]");
make_test!(line_wrapping, "Lorem _ipsum_ dolor sit amet, _consectetur_ adipiscing elit, sed do eiusmod tempor incididunt ut labore.");
make_test!(slash_space, r"#[\ ]");
make_test!(
    text_then_list,
    "We have next things:
- thing 1;
- thing 2;
- thing 3."
);
make_test!(bug_quote_space, r#"don't "text" "text"#);
make_test!(bug_space_around_inline_code, "a#lorem(2)b c d");
