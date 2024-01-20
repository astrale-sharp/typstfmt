use super::*;

make_test!(list, LIST);
make_test!(enumeration, &LIST.replace('-', "+"));
make_test!(list2, &TERMS.replace('/', "-"));
make_test!(enums, &TERMS.replace('/', "+"));
make_test!(terms, TERMS);

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

const TERMS: &str = "content before
/ Ligature: A merged glyph.
/ Kerning: A spacing adjustment
  between two adjacent letters.
content after";
