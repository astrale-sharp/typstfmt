use super::*;
#[test]
fn dont_insert_weird_space() {
    init();

    similar_asserts::assert_eq!(
        format_with_rules("#{  }  \n", &[NoSpaceAtEndLine.as_dyn()]),
        "#{  }\n"
    );
}

#[test]
fn removes_trailing_space() {
    init();

    similar_asserts::assert_eq!(
        format_with_rules(
            r#"Some markup  
                And then some"#,
            &[NoSpaceAtEndLine.as_dyn()]
        ),
        r#"Some markup
                And then some"#
    );
}

#[test]
fn dont_eat_too_much() {
    similar_asserts::assert_eq!(
        format_with_rules("some \n \n  some", &[NoSpaceAtEndLine.as_dyn()]),
        "some\n\n  some"
    );
}
