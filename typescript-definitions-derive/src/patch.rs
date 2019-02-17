// Copyright 2019 Ian Castleden
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Patch
//!
//! we are generating *typescript* from rust tokens so
//! the final result when rendered to a string has a typescript
//! formatting problem. This mod just applies a few patches
//! to make the final result a little more acceptable.
//!

use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;

pub const PATCH: &str = "__XYZZ__patch_me__XYZZ__";
pub const TRIPPLE_EQ: &str = "__eeeeEEEeeee__";

// type N = [(&'static str, &'static str); 10];
const NAMES: [(&str, &str); 12] = [
    ("brack", r"\s*\[\s+\]"),
    ("brace", r"\{\s+\}"),
    ("colon", r"\s+[:]\s"),
    ("bar", r"(^|\s)\|\s+\{"),
    ("enl", r"\n+\}"),
    ("fnl", r"\{\n+"),
    ("result", PATCH),  // for Result...
    ("te", TRIPPLE_EQ), // ===
    ("lt", r"\s<\s"),
    ("gt", r"\s>(\s|$)"),
    ("semi", r"\s+;"),
    // ("number", r"\s[0-9]+usize\s"),
    ("nl", r"\n+"), // last!
];
lazy_static! {
    static ref RE: Regex = {
        let v = NAMES
            .iter()
            .map(|(n, re)| format!("(?P<{}>{})", n, re))
            .collect::<Vec<_>>()
            .join("|");
        Regex::new(&v).unwrap()
    };
}

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
        }
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

// TODO: where does the newline come from? why the double spaces?
// maybe use RegexSet::new(&[.....])
pub fn patch(s: &str) -> Cow<'_, str> {
    RE.replace_all(s, |c: &Captures| {
        let key = c.key();
        let m = match key {
            "brace" => "{}",
            "brack" => "[]",
            "colon" => ": ",
            "fnl" => "{ ",
            "bar" => "\n  | {",
            "enl" => " }",
            "nl" => " ",
            "result" => "|",
            "te" => "===",
            "lt" => "<",
            "gt" => ">",
            // "number" => return Cow::Owned(c.name("number").unwrap().as_str().replace("usize", "")),
            "semi" => ";",
            _ => return Cow::Owned(c.get(0).unwrap().as_str().to_owned()), // maybe should just panic?
        };
        Cow::Borrowed(m)
    })
}
