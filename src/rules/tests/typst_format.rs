use super::*;

mod eof;

test_snippet!(
    later,
    ignore = "need to treat fn first",
    expect = "#{\n    a()[]\n}",
    "#{\na()[]\n}",
    rules().as_slice()
);

test_snippet!(
    test_let,
    expect = r"#let x = 4",
    r"#let x = 4",
    rules().as_slice()
);

test_snippet!(
    complex,
    expect = r##"#import "template.typ": *
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

Best,"##,
    r##"#import "template.typ": *
#show: letter.with(sender:[Jane Smith, Universal Exports, 1 Heavy Plaza, Morristown, NJ 07964,],recipient: [Mr. John Doe \ Acme Corp. \ 123 Glennwood Ave \ Quarto Creek, VA 22438],date: [Morristown, June 9th, 2023,],subject: [test],name: [Jane Smith \Regional Director],)

Dear Joe,

#lorem(9)

Best,"##,
    rules().as_slice()
);
