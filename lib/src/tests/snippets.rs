use super::*;

make_test!(code_func, "#{f(1,2,3)}");
make_test!(
    code_func_break,
    "#{f(1,2,3)}",
    Config {
        max_line_length: 2,
        ..Default::default()
    },
);
make_test!(
    code_func_break_nested,
    "#{{f(1,2,3)}}",
    Config {
        max_line_length: 2,
        ..Default::default()
    },
);
make_test!(while_loop, WHILE_LOOP);
make_test!(for_loop, FOR_LOOP);
make_test!(official, OFFICIAL);
make_test!(let_closure_params_named, TABLEX,);
make_test!(raw_text, RAW);
make_test!(tabs, TABS);
make_test!(on_off, ON_OFF);
make_test!(list, LIST);
make_test!(enumeration, &LIST.replace('-', "+"));
make_test!(line_wrapping, "Lorem _ipsum_ dolor sit amet, _consectetur_ adipiscing elit, sed do eiusmod tempor incididunt ut labore.");
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

const FOR_LOOP: &str = r#"#for k in range(5) {
    repr(k) + " " 
}"#;

const WHILE_LOOP: &str = r#"#let i = 0
#while true {
  i += 1
  if i > 15 { break }
  repr(i) + " "
}"#;

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

// this is taken from tablex by Pg Biel whom we love.
const TABLEX: &str = r#"#let is-tablex-dict(x) = (
  type(x) == "dictionary"
      and "tablex-dict-type" in x
)
"#;

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

const LIST: &str = r#"
- 000
 some text 
 badly broken for no _reason_ which is a @very long line and should be broken up in at least three bits in my opinion.
// not broken by comments
 - 010
  - 011
  - 012
   inner content

- 003
-     10 not too spaced
  inner content
outer content
"#;
