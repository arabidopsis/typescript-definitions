

use regex::{Captures, Regex};
use std::borrow::Cow;

type N = [(&'static str, &'static str); 7];
const NAMES : N = [("nl", r"\n+"), ("brack", r"\s*\[\s+\]"),
                   ("brace", r"\{\s+\}"), ("colon", r"\s[:]\s"),
                  ("bar", r"\s\|\s+\{"), 
                  ("enl", r"\n+\}"), ("fnl", r"\{\n+"),
                 ];
lazy_static! {
    
    static ref RE: Regex = {
        let v  = NAMES.iter().map(|(n, re)| format!("(?P<{}>{})", n, re))
            .collect::<Vec<_>>().join("|");
        Regex::new(&v).unwrap()
    };
}
// TODO: where does the newline come from? why the double spaces?

trait Has {
    fn has(&self, s: &'static str) -> bool;
    fn key(&self) -> &'static str;
}

impl Has for Captures<'_> {
    #[inline]
    fn has(&self, s: &'static str) -> bool {
        self.name(s).is_some()
    }
    
    fn key(&self) -> &'static str {
        for n in &NAMES {
            if self.has(n.0) {
                return n.0;
            }
        };
        "?"
    }
    /*
    fn key(&self) -> &'static str {
        for n in RE.capture_names() {
            if let Some(m) = n {
                if self.has(m) {
                    return m;
                }
            }
        };

        "?"
    }
    */
}


pub fn debug_patch<'t>(s: &'t str) -> Cow<'t, str> {
    RE.replace_all(s, |c: &Captures| {
        let key = c.key();
        match key {
            "brace" => "{ }",
            "brack" => " [ ]",
            "colon" => " : ",
            "fnl" =>  "{ ",
            "nl" => " ",
            _ => c.get(0).unwrap().as_str()

        }
    })
}

pub fn patch<'t>(s: &'t str) -> Cow<'t, str> {
    RE.replace_all(s, |c: &Captures| {
        let key = c.key();
        match key {
            "brace" => "{}",
            "brack" => "[]",
            "colon" => ": ",
            "fnl" =>  "{ ",
            "bar" => "\n   | {",
            "enl" => " }",
            "nl" => " ",
            _ => c.get(0).unwrap().as_str()

        }      

    })
}
