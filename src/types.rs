// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use deserializers::*;
use serde::de;
use serde::de::{Deserialize, Deserializer};
use shlex;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config<'a> {
    #[serde(borrow)]
    pub general: General<'a>,
    pub applications: Vec<Application<'a>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct General<'a> {
    #[serde(borrow)]
    pub working_directory: Option<&'a str>,
    pub workspace: Option<u16>,
    pub layout: Option<&'a str>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Application<'a> {
    pub command: ApplicationCommand,
    #[serde(borrow)]
    pub working_directory: Option<&'a str>,
    #[serde(borrow, default, deserialize_with="option_string_or_seq_string")]
    pub text: Option<Vec<&'a str>>,
    #[serde(default="default_text_return")]
    pub text_return: bool,
    #[serde(borrow)]
    pub keys: Option<Vec<&'a str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl<'de> Deserialize<'de> for ApplicationCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let empty_command: D::Error = de::Error::custom("command can not be empty");
        let result: Result<Vec<&'de str>, D::Error> = string_or_seq_string(deserializer);
        result
            .and_then(|mut v| match v.len() {
                          0 => Err(empty_command),
                          1 => {
                              match shlex::split(v[0]) {
                                  Some(mut v) => {
                                      if v.is_empty() {
                                          Err(empty_command)
                                      } else {
                                          Ok((v.remove(0), v))
                                      }
                                  }
                                  None => Err(empty_command),
                              }
                          }
                          _ => {
                              Ok((v.remove(0).to_owned(),
                                  v.into_iter()
                                      .map(|s| s.to_owned())
                                      .collect::<Vec<String>>()))
                          }
                      })
            .map(|(program, args)| {
                     ApplicationCommand {
                         program: program,
                         args: args,
                     }
                 })
    }
}

fn default_text_return() -> bool {
    false
}
