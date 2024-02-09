#![allow(clippy::needless_raw_string_hashes)]

use super::*;

make_test!(
    mathblock1,
    r#"$
    #xx(a,b) &= 1 \
    &= 2 \

$"#
);

make_test!(mathblock2, r#"$x$"#);

make_test!(
    mathblock3,
    r#"$
    #xx(a,b) &= 1 \
    &= 2 \
    &=3 \
    &=4 \

$"#
);

make_test!(
    mathblock4,
    r#"$
    #xx(a,b) &= 1 &=3 \
    &= 2 &=2 \
    &=3 \
    &=4 \
    &=5 \
    $"#
);

make_test!(
    mathblock5,
    r#"$
#xx(a,b)
    &= 1 &=3 \
    &= 2 &=2 \
    &=3 \
    &=4 \
    &=5 \
    $"#
);

make_test!(
    mathblock6,
    r#"$
    #xx(a,b) &= 1 &=3 \
                    &= 2 &=2 \
    &=3 \
    &=4 \
    &=5 \
    $"#
);

make_test!(
    mathblock7,
    r#"$
mat(
    1,2,3,4;
5,2,2,3
)
$"#
);

make_test!(mathblock8, r#"$#xx(a,b) &= 1 \ #xx(a,b) &= 1 \ &= 2 \ $"#);

make_test!(
    mathblock9,
    r#"
$
  a
  &= b\
  &= c & d\
  &= &e & f\
  &= && & g
$"#
);

make_test!(
    mathblock10,
    r#"
$
  a&=b & c & d & e\
  &= "a really long string" & pi \
  &= a & #xx & d & e \
  &= "an even longer string!!" & y
$"#
);

make_test!(
    unicode_math_alignment,
    r#"
$
 α   & := β & ("text") \
 α &    := b & ("text") \
 a & ≠  β & ("text") \
 a    & ≠  b &   ("text")
$"#
);
