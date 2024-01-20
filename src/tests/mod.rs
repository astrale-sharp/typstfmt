use super::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Enables logging.
///
/// Reads different environment variable.
///
/// - `NOLOG`: don't log anything
/// - `DEBUG`: set the logging level to DEBUG
/// - `NO_COLOR`: don't put ainsi colors in the output.
fn init() {
    if std::env::var("NOLOG").is_ok() {
        return;
    }
    let level = if std::env::var("DEBUG").is_ok() {
        Level::DEBUG
    } else {
        Level::INFO
    };
    let ainsi = std::env::var("NO_COLOR").is_err();

    let subscriber = FmtSubscriber::builder()
        .pretty()
        .with_test_writer()
        .without_time()
        .with_line_number(true)
        .with_file(true)
        .with_max_level(level)
        .with_ansi(ainsi)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

/// This makes :
/// - A snapshot test where you're prompted to say if you're snippet is nicely formatted.
/// (see README.md)
/// - A double format test (if an input is formatted twice it should give the same result)
/// - An AST test (if an input is formatted, the output AST should be the same as the input).
///
/// TODO : currently for the AST test, all Space and parbeak are skipped, maybe there is a better way.
/// TODO : AST check when we had a trailing comma, find a way to allow it to be able to run test for these snippets too.
macro_rules! make_test {
    ($test_name:ident, $input:expr $(,)?) => {
        make_test!($test_name, $input, Config::default());
    };
    ($test_name:ident, $input:expr, $config:expr $(,)?) => {
        mod $test_name {
            use super::*;

            #[test]
            fn snapshot()  {
                init();
                let input = $input;
                let formatted = format(input, $config);
                println!("AST: {:?}",parse(input));
                insta::with_settings!({description => format!("INPUT\n===\n{input:?}\n===\n{input}\n===\nFORMATTED\n===\n{formatted}")}, {
                    insta::assert_debug_snapshot!(formatted);
                });
            }

            #[test]
            fn ast() {
                init();
                println!("AST: {:?}",parse($input));
                let input = $input;
                let formatted = format(input, $config);
                assert!(tests::parses_the_same(&input, &formatted));
            }

            #[test]
            fn double_format()  {
                init();
                println!("AST: {:?}",parse($input));
                let input = $input;
                let format_once = format(input, $config);
                let format_twice = format(&format_once, $config);
                similar_asserts::assert_eq!(format_once, format_twice);
            }
        }
    };
}

/// Tests formatting the snippets doesn't change it.
macro_rules! test_eq {
    ($test_name:ident, $input:expr $(,)?) => {
        test_eq!($test_name, $input, Config::default());
    };
    ($test_name:ident, $input:expr, $config:expr $(,)?) => {
        mod $test_name {
            use super::*;

            #[test]
            fn test_eq() {
                init();
                println!("AST: {:?}", parse($input));
                let format_once = format($input, $config);
                similar_asserts::assert_eq!($input, format_once);
            }

            #[test]
            fn double_format() {
                init();
                println!("AST: {:?}", parse($input));
                let input = $input;
                let format_once = format(input, $config);
                let format_twice = format(&format_once, $config);
                similar_asserts::assert_eq!(format_once, format_twice);
            }
        }
    };
}

// allowing modifying trailing comma's, text in markup, space everywhere
// todo, check adding all text from one tree and another equal the same text.
fn tree_are_equal(node: &LinkedNode, other_node: &LinkedNode) -> bool {
    let should_ignore = |x: &LinkedNode| [Space, Parbreak, Comma, Text].contains(&x.kind());

    let node_kind = node.kind();
    let other_kind = other_node.kind();
    if node_kind != other_kind {
        debug!("kind differs! {:?}-{:?}", node_kind, other_kind);
        return false;
    }

    if (node.text() != other_node.text()) && !should_ignore(node) {
        debug!(
            "kind ok {:?}\ntext differ:{:?}-{:?}",
            node.kind(),
            node.text(),
            other_node.text()
        );
        return false;
    }

    let fchildren = node.children().filter(|x| !should_ignore(x)).collect_vec();
    let fchildren_oth = other_node
        .children()
        .filter(|x| !should_ignore(x))
        .collect_vec();
    if fchildren.len() != fchildren_oth.len() {
        debug!(
            "children count differ! {:?}\n{:?}",
            fchildren, fchildren_oth
        );
        return false;
    }
    if node
        .children()
        .filter(|x| !should_ignore(x))
        .zip(other_node.children().filter(|x| !should_ignore(x)))
        .any(|(c, oth)| !tree_are_equal(&c, &oth))
    {
        return false;
    }
    true
}

#[instrument(skip_all)]
fn parses_the_same(s: &str, oth: &str) -> bool {
    let parse1 = parse(s);
    let lkn = LinkedNode::new(&parse1);
    let parse2 = parse(oth);
    let lkn_oth = LinkedNode::new(&parse2);
    debug!("{:?}", parse1);
    debug!("{:?}", parse2);
    tree_are_equal(&lkn, &lkn_oth)
}

#[test]
fn comma_gets_ignored_in_comparison() {
    assert!(parses_the_same("#f(1,2,)", "#f(1,2)"));
    assert!(parses_the_same("#f(1,{g(1,2,3,)},)", "#f(1,{g(1,2,3)})"));
}

mod code_block;
mod comments;
mod conditionals;
mod lists;
mod markup;
mod math;
mod params;
mod snippets;
