use super::*;
make_test!(unchanged, " ");
make_test!(one_space, "  ");
make_test!(one_space_b, "   ");
make_test!(two_line_max, "\n\n\n");
make_test!(let_stmt_unchanged, "#let ident = variable");
make_test!(let_stmt_period_terminated, "#let ident = variable;");
make_test!(let_stmt_no_spacing, "#let ident=variable");
make_test!(
    content_block_spaced,
    "#[\n_Glaciers_ form an important part \nof the earth's climate system.]"
);
make_test!(ten_adds, &format!("#{{{}1}}", "1+".repeat(10)));
make_test!(thirty_adds, &format!("#{{{}1}}", "1+".repeat(30)));
make_test!(markup_block_spaced, "[ 4 ]");
make_test!(markup_block_too_spaced, "[  4  ]");
make_test!(markup_block_tight, "[4]");
make_test!(markup_block_only_space, "[   ]");
make_test!(just, "#let page_turned = page not in header_pages");
make_test!(short_if_else, "#if true{}else {}");
make_test!(
    long_if_else,
    "#if long_condition {
    some_code()
some_var
    some-stuff(a,b)
}    else {}"
);
