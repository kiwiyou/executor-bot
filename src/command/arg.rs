use std::collections::HashMap;

pub struct CommandMatcher<Enum> {
    matcher: HashMap<String, Enum>,
}

impl<Enum: Clone> CommandMatcher<Enum> {
    pub fn new(labels: &[&str], enums: &[Enum]) -> Self {
        let map = labels
            .iter()
            .map(ToString::to_string)
            .zip(enums.iter().cloned())
            .collect();
        Self { matcher: map }
    }

    pub fn find(&self, text: &str) -> Option<Enum> {
        self.matcher.get(text).cloned()
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
