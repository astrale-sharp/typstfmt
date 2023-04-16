use super::*;

#[test]
fn one_space_is_unchanged() {
    //    init();

    similar_asserts::assert_eq!(format_with_rules("#{ }", &[OneSpace.as_dyn()]), "#{ }");
    similar_asserts::assert_eq!(
        format_with_rules("some\n\nsome", &[OneSpace.as_dyn()]),
        "some\n\nsome"
    );
    similar_asserts::assert_eq!(
        format_with_rules("some \n \n some", &[OneSpace.as_dyn()]),
        "some \n \n some"
    );
}

#[test]
fn more_than_on_becomes_one() {
    similar_asserts::assert_eq!(format_with_rules("#{  }", &[OneSpace.as_dyn()]), "#{ }");
    init();
    similar_asserts::assert_eq!(
        format_with_rules("some \n  \n some", &[OneSpace.as_dyn()]),
        "some \n \n some"
    );
    similar_asserts::assert_eq!(format_with_rules("#{   }", &[OneSpace.as_dyn()]), "#{ }");
    similar_asserts::assert_eq!(format_with_rules("m  m", &[OneSpace.as_dyn()]), "m m");
    similar_asserts::assert_eq!(
        format_with_rules("some \n \n  some", &[OneSpace.as_dyn()]),
        "some \n \n some"
    );
    similar_asserts::assert_eq!(
        format_with_rules("some  \n \n some", &[OneSpace.as_dyn()]),
        "some \n \n some"
    );
    similar_asserts::assert_eq!(
        format_with_rules("some  \n  \n some", &[OneSpace.as_dyn()]),
        "some \n \n some"
    );
    similar_asserts::assert_eq!(
        format_with_rules("some  \n  \n some  ", &[OneSpace.as_dyn()]),
        "some \n \n some "
    );
}

#[test]
fn dont_insert_weird_space() {
    init();

    similar_asserts::assert_eq!(format_with_rules("#{  }\n", &[OneSpace.as_dyn()]), "#{ }\n");
}
