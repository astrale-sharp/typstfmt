use super::*;

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

macro_rules! make_test {
    ($test_name:ident, $input:expr) => {
        make_test!($test_name, $input, Config::default());
    };

    ($test_name:ident, $input:expr, $config:expr) => {
        #[test]
        fn $test_name() {
            init();
            let input = $input;
            let formatted = format(input, $config);
            insta::with_settings!({description => format!("INPUT\n===\n{input:?}\n===\n{input}\n===\nFORMATTED\n===\n{formatted}")}, {
                insta::assert_debug_snapshot!(formatted);
            });
        }
    };

    }

#[cfg(test)]
mod basic;

#[cfg(test)]
mod snippets;

#[cfg(test)]
mod args;

#[cfg(test)]
mod code_block;
