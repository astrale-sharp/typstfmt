use super::*;

#[test]
fn basic_func() {
    init();
    similar_asserts::assert_eq!(
        format_with_rules("#{f1(1,2,3,)}", &[IdentItemFunc.as_dyn()]),
        format!("#{{f1(\n{0}1,\n{0}2,\n{0}3,\n)}}", " ".repeat(4))
    );
}

#[test]
fn reduce_space_trailing() {
    init();
    similar_asserts::assert_eq!(
        format_with_rules("#{f1(\n\n1,\n\n2,\n\n3,\n)}", &[IdentItemFunc.as_dyn()]),
        format!("#{{f1(\n{0}1,\n{0}2,\n{0}3,\n)}}", " ".repeat(4))
    );
}

#[test]
fn reduce_space_non_trailing() {
    similar_asserts::assert_eq!(
        format_with_rules("#{f1(\n\n1,\n\n2,\n\n3)}", &[IdentItemFunc.as_dyn()]),
        format!("#{{f1(\n{0}1,\n{0}2,\n{0}3\n)}}", " ".repeat(4))
    );
}

#[test]
fn nested_func() {
    init();
    similar_asserts::assert_eq!(
        format_with_rules("#{f1(1,2,f(a,b,c,),)}", &[IdentItemFunc.as_dyn()]),
        "#{f1(\n    1,\n    2,\n    f(\n        a,\n        b,\n        c,\n    ),\n)}"
    );
}
#[test]
fn spacing_without_comma() {
    init();

    similar_asserts::assert_eq!(
        format_with_rules("#lorem(9)", &[IdentItemFunc.as_dyn()]),
        "#lorem(\n    9\n)"
    );
}
