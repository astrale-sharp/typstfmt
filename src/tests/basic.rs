use super::*;
make_test!(unchanged, "#{ }");
make_test!(on_space, "#{  }");
make_test!(on_space_b, "#{   }");
make_test!(two_line_max, "\n\n\n");
