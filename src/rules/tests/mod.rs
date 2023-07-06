use super::*;

macro_rules! test_snippet {
    (
        $test_name:ident,
        $(ignore = $ignore:tt ,)?
        expect = $expected:expr,
        $snippet:expr,
        $rules:expr
    ) => {
        #[test]
        $(#[ignore = $ignore])?
        fn $test_name() {
            init();
            let result = format_with_rules($snippet, $rules);
            similar_asserts::assert_eq!(result, $expected, "first formatting");
            let result2 = format_with_rules(&result, $rules);
            similar_asserts::assert_eq!(result2, $expected, "second formatting");
        }
    };
}

#[cfg(test)]
fn init() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .is_test(true)
        .try_init();
}

test_snippet!(
    more_than_one_rule,
    expect = "#{ }\n",
    "#{  }  \n",
    &[OneSpace.as_dyn(), NoSpaceAtEndLine.as_dyn()]
);

#[cfg(test)]
mod cond_rule;

#[cfg(test)]
mod no_space_when_line_ends;
#[cfg(test)]
mod one_space;

#[cfg(test)]
mod ident_item_func;
mod typst_format;
