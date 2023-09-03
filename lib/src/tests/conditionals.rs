use super::*;

make_test!(short_if_else, "#if true{}else {}");
make_test!(
    long_if_else,
    "#if long_condition {
    some_code()
some_var
    some-stuff(a,b)
}    else {}"
);

make_test!(
    elseif,
    r#"#let _slides-cover(mode, body) = {
  if mode == "invisible" {
    hide(body)
  } else if mode == "transparent" {
    text(gray.lighten(50%), body)
  } else {
    panic("Illegal cover mode: " + mode)
  }
}"#
);

make_test!(
    ifblock,
    "#if k > 0 [
#k / #n
]"
);
