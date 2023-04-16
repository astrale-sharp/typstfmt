use super::*;

test_snippet!(
    dont_insert_weird_space,
    expect = "#{ }\n",
    "#{  }\n",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    one_space_is_unchanged,
    expect = "#{ }",
    "#{ }",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    one_space_is_unchanged_2,
    expect = "some\n\nsome",
    "some\n\nsome",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    one_space_is_unchanged_3,
    expect = "some \n \n some",
    "some \n \n some",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    more_than_on_becomes_one,
    expect = "#{ }",
    "#{  }",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    more_than_on_becomes_one_1,
    expect = "some \n \n some",
    "some \n  \n some",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    more_than_on_becomes_one_2,
    expect = "#{ }",
    "#{   }",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    more_than_on_becomes_one_3,
    expect = "m m",
    "m  m",
    &[OneSpace.as_dyn()]
);

test_snippet!(
    more_than_on_becomes_one_4,
    expect = "some \n \n some",
    "some \n \n  some",
    &[OneSpace.as_dyn()]
);
