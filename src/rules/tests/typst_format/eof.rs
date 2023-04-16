use super::*;

test_snippet!(
    test_eof,
    ignore = "might get implemented another way",
    expect = r"#{}",
    "#{} \n",
    rules().as_slice()
);
test_snippet!(
    test_eof_1,
    ignore = "might get implemented another way",
    expect = r"#{}",
    "#{} \n ",
    rules().as_slice()
);
test_snippet!(test_eof_2, expect = r"#{}", r"#{}", rules().as_slice());

test_snippet!(
    test_eof_3,
    ignore = "might get implemented another way",
    expect = r"#{}",
    r"#{} ",
    rules().as_slice()
);