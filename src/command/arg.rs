use aho_corasick::{AhoCorasick, AhoCorasickBuilder};

pub struct CommandMatcher<Enum> {
    matcher: AhoCorasick,
    map: Vec<Enum>,
}

impl<Enum: Clone> CommandMatcher<Enum> {
    pub fn new(labels: &[&str], enums: &[Enum]) -> Self {
        let matcher = AhoCorasickBuilder::new()
            .match_kind(aho_corasick::MatchKind::LeftmostLongest)
            .auto_configure(labels)
            .build(labels);
        Self {
            matcher,
            map: enums.to_vec(),
        }
    }

    pub fn find(&self, text: &str) -> Option<Enum> {
        let first_match = self.matcher.find(text)?;
        if first_match.start() != 0 {
            None
        } else {
            self.map.get(first_match.pattern()).cloned()
        }
    }
}

pub struct Args<'a> {
    s: &'a str,
}

impl<'a> Args<'a> {
    pub fn wrap(s: &'a str) -> Self {
        Self { s }
    }

    pub fn as_str(&self) -> &str {
        self.s.trim_start()
    }
}

impl<'a> Iterator for Args<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.s = self.s.trim_start();
        if let Some(next_ws) = self.s.find(char::is_whitespace) {
            let ret = Some(&self.s[..next_ws]);
            self.s = &self.s[next_ws..];
            ret
        } else if self.s.is_empty() {
            None
        } else {
            let ret = Some(self.s);
            self.s = "";
            ret
        }
    }
}
