use std::io::stdin;

use typst_fmt::typst_format;

fn main() {
    let res = stdin().lines().map(|l| l.unwrap_or_default()).fold(
        String::with_capacity(1024),
        |mut x, mut y| {
            x.push_str(&mut y);
            x
        },
    );
    println!("{}", typst_format(&res))
}
