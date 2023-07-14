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
// todo, fix array formatting trailing comma
make_test!(parenthesized, "#(auto,)");
