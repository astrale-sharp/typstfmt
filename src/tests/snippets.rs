use super::*;

test_eq!(let_stmt, "#let ident = variable");
test_eq!(let_stmt_period_terminated, "#let ident = variable;");
make_test!(let_stmt_no_spacing, "#let ident=variable");
make_test!(ten_adds, &format!("#{{{}1}}", "1+".repeat(10)));
make_test!(thirty_adds, &format!("#{{{}1}}", "1+".repeat(30)));
test_eq!(not_in, "#let page_turned = page not in header_pages");
make_test!(
    while_loop,
    r#"#let i = 0
#while true {
  i += 1
  if i > 15 { break }
  repr(i) + " "
}"#
);
make_test!(
    for_loop,
    r#"#for k in range(5) {
  repr(k) + " " 
}"#
);

make_test!(
    on_off_indent_bug,
    r#"#let template(doc) = {
  //typstfmt::off
  let         a      =    ""
  //typstfmt::on
  doc
}"#
);
make_test!(official, OFFICIAL);
make_test!(raw_text, RAW);
make_test!(tabs, TABS);
make_test!(on_off, ON_OFF);
test_eq!(string_literal_in_math_mode, r#"$ a "        x" $"#);
test_eq!(string_literal_in_code_mode, r#"#raw("   foo   ");"#);
test_eq!(
    line_wrap_off,
    "a very very very very very very very very very very very very very long line",
    Config {
        line_wrap: false,
        max_line_length: 50,
        ..Default::default()
    }
);
make_test!(
    trailing_com_math_bug0,
    "$mat(
  1111111111111111;,
)$",
    Config {
        max_line_length: 9,
        ..Default::default()
    }
);
make_test!(
    trailing_com_math_bug2,
    "$mat(11111111111111111111111111111111111111111111111111111111111111111111111111;)$"
);

test_eq!(
    stable_raw_indents,
    "#focus-slide[
#fit-to-height(3em)[Introduction]

#pdfpc.speaker-note(```
    Let's start the introduction with a quote from Foo Bar
  ```)
]"
);

// TODO: wait for parser fix
//    $step(&>= ceil(phi.alt (n+1)) / (n+1) >= phi.alt. )$
// vs $step(&>= ceil(phi.alt (n+1)) / (n+1) >= phi.alt.)$

const OFFICIAL: &str = r#"Glaciers as the one shown in
@glaciers will cease to exist if
we don't take action soon!

#figure(
  image("glacier.jpg", width: 70%),
  caption: [
    _Glaciers_ form an important part
    of the earth's climate system.
  ],
) <glaciers>"#;

const RAW: &str = r#"```
fn main() {
 println!(hello world);

 let bob = 0;
 if bob == {
  println("bob is 0");
 }
}```"#;

const TABS: &str = r#"
#{
	{
    v(0pt)
		v(0pt)
	}
}"#;

const ON_OFF: &str = r#"// typstfmt::off
#{{4}}
// typstfmt::on
#{{4}}
"#;
