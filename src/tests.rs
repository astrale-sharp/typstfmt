use super::*;

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(true)
        .try_init();
}

macro_rules! make_test {
    ($test_name:ident, $input:literal) => {
        make_test!($test_name, $input, Config::default());
    };

    ($test_name:ident, $input:literal, $config:expr) => {
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

make_test!(unchanged, "#{ }");
make_test!(on_space, "#{  }");
make_test!(on_space_b, "#{   }");
make_test!(two_line_max, "\n\n\n");
make_test!(call_func_empty, "#f()");
make_test!(call_func_simple, "#f(1,2,3)");
make_test!(
    call_func_long,
    "#f(1,this_is_absurdly_loooooooooong,3)",
    Config {
        max_line_length: 1,
        ..Default::default()
    }
);
make_test!(
    call_func_long_trailing,
    "#f(1,this_is_absurdly_loooooooooong,3,)",
    Config {
        max_line_length: 1,
        ..Default::default()
    }
);
