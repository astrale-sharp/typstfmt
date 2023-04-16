use super::*;

#[cfg(test)]
fn init() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .is_test(true)
        .try_init();
}

#[test]
fn more_than_one_rule() {
    init();
    similar_asserts::assert_eq!(
        format_with_rules("#{  }  \n", &[OneSpace.as_dyn(), NoSpaceAtEndLine.as_dyn()]),
        "#{ }\n"
    );
}
#[cfg(test)]
mod no_space_when_line_ends;
#[cfg(test)]
mod one_space;

#[cfg(test)]
mod ident_item_func;
mod typst_format;
