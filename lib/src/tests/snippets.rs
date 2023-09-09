use super::*;

make_test!(let_stmt_unchanged, "#let ident = variable");
make_test!(let_stmt_period_terminated, "#let ident = variable;");
make_test!(let_stmt_no_spacing, "#let ident=variable");
make_test!(ten_adds, &format!("#{{{}1}}", "1+".repeat(10)));
make_test!(thirty_adds, &format!("#{{{}1}}", "1+".repeat(30)));
make_test!(not_in, "#let page_turned = page not in header_pages");
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

make_test!(
    on_off_indent_func,
    "#figure({
  some-code({
    // typstfmt::off
    // typstfmt::on
    })
  })"
);

make_test!(official, OFFICIAL);
make_test!(raw_text, RAW);
make_test!(tabs, TABS);
make_test!(on_off, ON_OFF);

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
