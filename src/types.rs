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
pub struct Config {
    pub general: General,
    pub applications: Vec<Application>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct General {
    pub working_directory: Option<String>,
    pub workspace: Option<String>,
    pub layout: Option<String>,
    pub layout_path: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Application {
    pub command: ApplicationCommand,
    pub working_directory: Option<String>,
    #[serde(default, deserialize_with="option_string_or_seq_string")]
    pub text: Option<Vec<String>>,
    #[serde(default="default_text_return")]
    pub text_return: bool,
    pub keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationCommand {
    pub program: String,
    pub args: Option<Vec<String>>,
}

impl<'de> Deserialize<'de> for ApplicationCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let empty_command: D::Error = de::Error::custom("command can not be empty");
        let result: Result<Vec<String>, D::Error> = string_or_seq_string(deserializer);
        result
            .and_then(|mut v| match v.len() {
                          0 => Err(empty_command),
                          1 => {
                              match shlex::split(&v[0]) {
                                  Some(mut v) => {
                                      if v.is_empty() {
                                          Err(empty_command)
                                      } else {
                                          Ok((v.remove(0).to_owned(),
                                              v.into_iter().map(str::to_owned).collect::<Vec<_>>()))
                                      }
                                  }
                                  None => Err(empty_command),
                              }
                          }
                          _ => {
                              Ok((v.remove(0), v.iter().map(|s| s.to_owned()).collect::<Vec<_>>()))
                          }
                      })
            .map(|(program, args)| {
                     ApplicationCommand {
                         program: program,
                         args: if args.is_empty() { None } else { Some(args) },
                     }
                 })
    }
}

fn default_text_return() -> bool {
    true
}
