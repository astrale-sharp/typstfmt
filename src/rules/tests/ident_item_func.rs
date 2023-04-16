use super::*;

test_snippet!(
    basic_func,
    expect = format!("#{{f1(\n{0}1,\n{0}2,\n{0}3,\n)}}", " ".repeat(4)),
    "#{f1(1,2,3,)}",
    &[IdentItemFunc.as_dyn()]
);

test_snippet!(
    reduce_space_trailing,
    expect = format!("#{{f1(\n{0}1,\n{0}2,\n{0}3,\n)}}", " ".repeat(4)),
    "#{f1(\n\n1,\n\n2,\n\n3,\n)}",
    &[IdentItemFunc.as_dyn()]
);

test_snippet!(
    reduce_space_non_trailing,
    expect = format!("#{{f1(\n{0}1,\n{0}2,\n{0}3\n)}}", " ".repeat(4)),
    "#{f1(\n\n1,\n\n2,\n\n3)}",
    &[IdentItemFunc.as_dyn()]
);

test_snippet!(
    nested_func,
    expect = "#{f1(\n    1,\n    2,\n    f(\n        a,\n        b,\n        c,\n    ),\n)}",
    "#{f1(1,2,f(a,b,c,),)}",
    &[IdentItemFunc.as_dyn()]
);

test_snippet!(
    spacing_without_comma,
    expect = "#lorem(\n    9\n)",
    "#lorem(9)",
    &[IdentItemFunc.as_dyn()]
);
