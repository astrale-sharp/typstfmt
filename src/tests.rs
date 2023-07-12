use super::*;
use tracing_subscriber::FmtSubscriber;

fn init() {
    let subscriber = FmtSubscriber::builder()
        .with_test_writer()
        .without_time()
        .compact()
        // .with_ansi(false)
        // .with_writer(make_writer)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber)
        // .expect("setting tracing default failed")
        ;
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
