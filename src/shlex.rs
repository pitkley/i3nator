// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use errors::*;
use std::str;
use std::str::Bytes;

// Implementation based in parts on:
//  https://github.com/comex/rust-shlex/blob/95ef6961a2500d89bc065b2873ca3e77850539e3/src/lib.rs
//
// which is dual-licensed under MIT and Apache-2.0:
//  https://github.com/comex/rust-shlex/blob/95ef6961a2500d89bc065b2873ca3e77850539e3/Cargo.toml#L5

struct Shlex<'a> {
    in_str: &'a str,
    in_bytes: Bytes<'a>,
    offset: usize,
}

impl<'a> Shlex<'a> {
    pub fn new(in_str: &'a str) -> Shlex<'a> {
        Shlex {
            in_str: in_str,
            in_bytes: in_str.bytes(),
            offset: 0,
        }
    }

    fn next_word(&mut self) -> Result<Option<&'a str>> {
        let start_offset = self.offset;
        let mut ch = self.next_byte();

        if ch.is_none() {
            return Ok(None);
        }

        loop {
            if ch.is_some() {
                let result = match ch.unwrap() as char {
                    '"' => self.parse_double(),
                    '\'' => self.parse_single(),
                    ' ' | '\t' | '\n' => break,
                    _ => Ok(()),
                };
                if result.is_err() {
                    return result.map(|_| None);
                }
                ch = self.next_byte();
            } else {
                break;
            }
        }

        str::from_utf8(&self.in_str.as_bytes()[start_offset..self.offset - 1])
            .map(|s| Some(s.trim_matches(|c| c == '\'' || c == '"')))
            .map_err(|e| e.into())
    }

    fn parse_double(&mut self) -> Result<()> {
        loop {
            if let Some(ch) = self.next_byte() {
                if let '"' = ch as char {
                    return Ok(());
                }
            } else {
                return Err("".into());
            }
        }
    }

    fn parse_single(&mut self) -> Result<()> {
        loop {
            if let Some(ch) = self.next_byte() {
                if let '\'' = ch as char {
                    return Ok(());
                }
            } else {
                return Err("".into());
            }
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.offset += 1;
        self.in_bytes.next()
    }
}

impl<'a> Iterator for Shlex<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        match self.next_word().ok() {
            None | Some(None) => None,
            Some(o) => o,
        }
    }
}

pub fn split<'a>(in_str: &'a str) -> Option<Vec<&'a str>> {
    let shl = Shlex::new(in_str);
    let res: Vec<&'a str> = shl.collect();

    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}
