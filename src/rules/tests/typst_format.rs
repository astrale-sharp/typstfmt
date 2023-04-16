use super::*;

#[test]
#[ignore]
fn test_eof() {
    similar_asserts::assert_eq!(typst_format("#{} \n"), r"#{}");
    similar_asserts::assert_eq!(typst_format("#{} \n "), r"#{}");
    //pass
    // todo new rules, No /n before end of file.
    similar_asserts::assert_eq!(typst_format(r"#{}"), r"#{}");
    similar_asserts::assert_eq!(typst_format(r"#{} "), r"#{}");
}

#[test]
fn test_let() {
    init();

    similar_asserts::assert_eq!(typst_format(r"#let x = 4"), r"#let x = 4");
}

#[test]
fn complex() {
    init();

    let expected = r##"#import "template.typ": *
#show: letter.with(
    sender: [Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],
    recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],
    date: [Morristown, June 9th, 2023,],
    subject: [test],
    name: [Jane Smith \Regional Director],
)

Dear Joe,

#lorem(
    9,
)

Best,"##;
    let input = r##"#import "template.typ": *
#show: letter.with(sender:[Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],date: [Morristown, June 9th, 2023,],subject: [test],name: [Jane Smith \Regional Director],)

Dear Joe,

#lorem(9)

Best,"##;
    similar_asserts::assert_eq!(typst_format(input), expected);
}
