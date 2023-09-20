use super::*;

#[derive(Default)]
pub(crate) struct Ctx {
    pub(crate) config: Config,
    pub(crate) just_spaced: bool,
    pub(crate) consec_new_line: i32,
    pub(crate) off: bool,
}

/// you may push into your own buffer using this to ensure you push considering context
///
/// you may then push said buffer the final result.
impl Ctx {
    pub(crate) fn from_config(config: Config) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Pushes the string in the result avoiding:
    /// - putting two consecutive spaces.
    /// - putting more than two consecutive newlines.
    /// - trims the string if it DOES contain a newline.
    /// Won't work for indents.
    #[instrument(skip_all)]
    pub(crate) fn push_in(&mut self, s: &str, result: &mut String) {
        let s = if s.contains('\n') {
            s.trim_end_matches(' ')
        } else {
            s
        };
        for c in s.chars() {
            match c {
                ' ' => {
                    if self.just_spaced || result.ends_with(' ') {
                        debug!("IGNORED space");
                    } else {
                        debug!("PUSHED SPACE");
                        self.just_spaced = true;
                        result.push(' ');
                    }
                }
                '\n' => {
                    if self.consec_new_line <= 1 {
                        debug!("PUSHED NEWLINE");
                        self.consec_new_line += 1;
                        result.push('\n');
                    } else {
                        debug!("IGNORED newline");
                    }
                }
                _ => {
                    // debug!("PUSHED {c}");
                    result.push(c);
                    self.lost_context();
                }
            }
        }
    }

    /// makes the context aware it missed info,
    /// should be called when pushing directly in result.
    pub(crate) fn push_raw_in(&mut self, s: &str, result: &mut String) {
        result.push_str(s);
        self.lost_context();
    }

    /// adds an indentation for each line the input except the first to match the current level of indentation.
    pub(crate) fn push_raw_indent(&mut self, s: &str, result: &mut String) {
        let mut is_first = true;
        for s in s.split('\n') {
            if is_first {
                is_first = false;
                self.push_raw_in(s, result);
                continue;
            }
            self.push_raw_in("\n", result);
            self.push_raw_in(
                format!("{}{}", self.get_indent(), s).trim_end_matches(' '),
                result,
            );
        }
    }

    // pub(crate) fn push_in_indent(&mut self, s: &str, result: &mut String) {
    //     let mut is_first = true;
    //     for s in s.lines() {
    //         let s = s.trim_end();
    //         if is_first {
    //             is_first = false;
    //             self.push_in(s, result);
    //             continue;
    //         }
    //         self.push_in("\n", result);
    //         self.push_in(
    //             format!("{}{}", self.get_indent(), s).trim_end_matches(' '),
    //             result,
    //         );
    //     }
    // }

    /// must be called when you cannot keep track of what you pushed
    /// so that context doesn't refuse your next pushes for no reasons.
    pub(crate) fn lost_context(&mut self) {
        self.just_spaced = false;
        self.consec_new_line = 0;
    }

    /// returns an indent using config to get it's length.
    pub(crate) fn get_indent(&self) -> String {
        " ".repeat(self.config.indent_space)
    }
}
