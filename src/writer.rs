use crate::Config;
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    mem,
};

// TODO, not a priority 
// remove post process indents, instead we want 2 functions on the writer,
// one that respects indentation and one that doesn't for Preserve NodeFmts.
// This should come with indent and dedent increasing an index in ctx, used to give indentation

/// Writer is used to write your formatted output.
///
/// It comes with the following features :
/// - Markers: you place a mark by calling [Writer::mark], you can use this mark
/// to jump back and redo your formatting should you see it didn't respect some rules.
/// - Todo: rewinding, go back to a position, removing all markers that were introduced after
/// as well as resetting the result to it's past state.
/// Example :
/// ```ignore
/// fn visit_params(/* */) {
///     let mark = self.mark();
///     visit_params_tight(self);
///     if !line_is_too_long(ctx.string_after_mark(mark)) {
///         // we're done yay
///         return
///     } else {
///         ctx.rewind(mark);
///         visit_params_long();
///     }
/// }
/// ```
/// - Indent, Dedent, Preserve: You must specify where you want indent to start and end.
/// It will be applied as a later step respecting the Preserve markers where indentation is
/// not applied (Raw, fmt::off, etc.)
pub(crate) struct Writer<'a> {
    pub(crate) config: Config,
    pub(crate) buffer: &'a mut String,
    pub(crate) marks: Vec<Mark>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Mark {
    kind: MarkKind,
    pos: usize,
}
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum MarkKind {
    Indent,
    Dedent,
    Preserve,
    StopPreserve,
}

impl MarkKind {
    pub(crate) fn to_mark(self, pos: usize) -> Mark {
        Mark { kind: self, pos }
    }
}

impl<'a> Writer<'a> {
    pub(crate) fn new(config: Config, buffer: &'a mut String) -> Self {
        Self {
            config,
            buffer,
            marks: vec![],
        }
    }

    pub fn get_mark(&self) -> usize {
        self.buffer.len()
    }

    pub fn rewind(&mut self, mark: usize) {
        *self.buffer = self.buffer[..mark].to_string();
        while let Some(x) = self.marks.last() {
            if x.pos < mark {
                self.marks.pop();
            }
        }
    }

    // needed when post process adds indentation level.
    pub fn mark_indent(&mut self) {
        self.marks.push(MarkKind::Indent.to_mark(self.buffer.len()))
    }

    pub fn mark_dedent(&mut self) {
        self.marks.push(MarkKind::Dedent.to_mark(self.buffer.len()))
    }

    pub fn mark_preserve(&mut self) {
        self.marks
            .push(MarkKind::Preserve.to_mark(self.buffer.len()))
    }

    pub fn mark_stop_preserve(&mut self) {
        self.marks
            .push(MarkKind::StopPreserve.to_mark(self.buffer.len()))
    }

    /// TODO, check the indexes with BASIC tests
    pub fn post_process_indents(&mut self) {
        let lines = self.buffer.split_inclusive('\n');
        let lines_len = lines.clone().count();
        let sizes = lines.clone().map(|s| 0..str::len(s)).collect_vec();
        // (line number -> bytes idx)
        let sizes = (1..=sizes.len())
            .map(|x| {
                sizes.iter().take(x).fold(0..0, |init, next| {
                    init.end..(init.end + next.end - next.start)
                })
            })
            .collect_vec();
        let line_at_pos = |i: &Mark| (&sizes).iter().position(|range| range.contains(&i.pos));

        // We'll match closed (except typstfmt::off) pairs of marks
        let mut marks = mem::take(&mut self.marks);
        marks.sort_by(|x, y| x.pos.cmp(&y.pos));
        debug_assert!(
            marks
                .iter()
                .map(|x| x.pos)
                .position_min()
                .unwrap_or_default()
                == 0
        );

        let ident_dedent_pairs = get_matching_pairs(&marks, MarkKind::Indent, MarkKind::Dedent);
        let preserve_pairs = get_matching_pairs(&marks, MarkKind::Preserve, MarkKind::StopPreserve);
        dbg!(&preserve_pairs);
        // - A (line number -> is_preserved) register.
        let mut preserved_lines = HashSet::new();
        for pair in preserve_pairs {
            let start = line_at_pos(pair.0).unwrap();
            // it's okay if `//typstfmt::off`` is not closed.
            let end = line_at_pos(pair.1).unwrap(); //_or(lines_len);
            for k in start..end {
                dbg!(k);
                preserved_lines.insert(k);
            }
        }

        let ident_line_range = ident_dedent_pairs
            .into_iter()
            .map(|(i, d)| line_at_pos(i).unwrap()..line_at_pos(d).unwrap())
            .collect_vec();
        // - A (line number -> indent level) HashMap.
        let mut line_to_ident = HashMap::new();
        for r in ident_line_range {
            // we skip the line where the mark appeared cause we only add indentation once we've
            // added a line break.
            for k in r.skip(1) {
                line_to_ident.entry(k).and_modify(|x| *x += 1).or_insert(1);
            }
        }

        let mut res = String::new();
        for (idx, line) in lines.enumerate() {
            if let Some(i) = line_to_ident.get(&idx) {
                if !preserved_lines.contains(&idx) {
                    // todo, config for tabs
                    res.push_str(&self.config.indent.get(*i))
                }
            }
            res.push_str(line);
        }
        *self.buffer = res;
    }

    pub(crate) fn push_str(&mut self, s: &str) {
        // TODO if we remove post process indent
        // check if we added a line feed and add indentation accordingly
        self.buffer.push_str(s)
    }

    /// pushes the thing checking for overflowing the line first and breaking if needed
    ///
    /// returns true if it broke the line
    pub(crate) fn push_str_with_limit(&mut self, s: &str) {
        if s == "" {
            return;
        }

        if self.last_line_length() + s.len() >= self.config.max_line_length {
            self.new_line()
        }
        self.buffer.push_str(s);
    }

    pub(crate) fn new_line(&mut self) {
        self.buffer.push('\n')
    }

    pub(crate) fn space(&mut self) {
        self.buffer.push(' ')
    }

    pub(crate) fn last_line_length(&self) -> usize {
        use unicode_segmentation::UnicodeSegmentation;
        self.buffer
            .split('\n')
            .last()
            .unwrap_or("")
            .trim()
            .graphemes(true)
            .count()
    }
}

fn get_matching_pairs(
    marks: &Vec<Mark>,
    opening_kind: MarkKind,
    closing_kind: MarkKind,
) -> Vec<(&Mark, &Mark)> {
    let mut res = vec![];
    for (i, m) in marks.iter().enumerate() {
        if opening_kind == m.kind {
            let mut c = 0;
            for k in (i + 1)..marks.len() {
                let m2 = &marks[k];
                if m2.kind == opening_kind {
                    c += 1;
                } else if m2.kind == closing_kind {
                    if c == 0 {
                        res.push((m, m2));
                        break;
                    } else {
                        c -= 1;
                    }
                }
            }
        }
    }
    res
}

#[test]
fn test_get_pairs() {
    let mi1 = "\n#[".len();
    let mi2 = "\n#[\ntext #[".len();
    let md2 = "\n#[\ntext #[\ntext\n".len();
    let md1 = "\n#[\ntext #[\ntext\n]\n".len();
    let preserve = "\n#[\ntext #[\n".len();
    let stop_preserve = "\n#[\ntext #[\ntext".len();

    let mut binding = String::new();
    let mut w = Writer::new(Config::default(), &mut binding);

    w.marks = vec![
        MarkKind::Indent.to_mark(mi1),
        MarkKind::Indent.to_mark(mi2),
        MarkKind::Dedent.to_mark(md2),
        MarkKind::Dedent.to_mark(md1),
        MarkKind::Preserve.to_mark(preserve),
        MarkKind::StopPreserve.to_mark(stop_preserve),
    ];

    let res = get_matching_pairs(&w.marks, MarkKind::Indent, MarkKind::Dedent);
    assert!(
        res == vec![
            (
                &MarkKind::Indent.to_mark(mi1),
                &MarkKind::Dedent.to_mark(md1)
            ),
            (
                &MarkKind::Indent.to_mark(mi2),
                &MarkKind::Dedent.to_mark(md2)
            )
        ]
    );
    let res = get_matching_pairs(&w.marks, MarkKind::Preserve, MarkKind::StopPreserve);
    dbg!(&res);
    assert!(
        res == vec![(
            &MarkKind::Preserve.to_mark(preserve),
            &MarkKind::StopPreserve.to_mark(stop_preserve)
        )]
    );
}
