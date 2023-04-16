use super::*;

test_snippet!(
    dont_insert_weird_space,
    expect = "#{  }\n",
    "#{  }  \n",
    &[NoSpaceAtEndLine.as_dyn()]
);

// make this test more explicit// invisible space
test_snippet!(
    removes_trailing_space,
    expect = r#"Some markup
    And then some"#,
    r#"Some markup  
    And then some"#,
    &[NoSpaceAtEndLine.as_dyn()]
);

test_snippet!(
    dont_eat_too_much,
    expect = "some\n\n  some",
    "some \n \n  some",
    &[NoSpaceAtEndLine.as_dyn()]
);
