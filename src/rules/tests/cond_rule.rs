use super::*;

#[derive(Debug)]
pub struct DumbRule;
impl Rule for DumbRule {
    fn accept(&self, node: &LinkedNode) -> bool {
        !node.kind().is_trivia() && node.text() != "" && node.parent().is_some()
    }

    fn eat(&self, _: String, _: &LinkedNode, writer: &mut Writer) {
        writer.push("dumb");
    }
}

#[test]
fn cond_rule() {
    init();
    let never_rule = ConditionalRule::new(DumbRule {}, |_, _, _| false);
    let always_rule = ConditionalRule::new(DumbRule {}, |_, _, _| true);
    similar_asserts::assert_eq!(
        format_with_rules("anything", &[always_rule.as_dyn()]),
        "dumb"
    );
    similar_asserts::assert_eq!(
        format_with_rules("anything", &[never_rule.as_dyn()]),
        "anything"
    );
}

#[test]
fn apply_if_len_above_threshold() {
    init();
    let len_rule = ConditionalRule::new(DumbRule {}, |t, _, w| w.current_line_length(t) >= 5);
    similar_asserts::assert_eq!(format_with_rules("any", &[len_rule.as_dyn()]), "any");

    let len_rule = ConditionalRule::new(DumbRule {}, |t, _, w| {
        println!("{}", w.current_line_length(t));
        w.current_line_length(t) >= 5
    });
    similar_asserts::assert_eq!(format_with_rules("anything", &[len_rule.as_dyn()]), "dumb");
}
