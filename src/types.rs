// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! The types in this module make up the structure of the project configuration files.
//!
//! # Example
//!
//! The following is an examplary TOML configuration, which will be parsed into this modules types.
//!
//! ```toml
//! # i3nator project
//!
//! # General configuration items
//! [general]
//! # Working directory to use
//! working_directory = "/path/to/my/working/directory"
//!
//! # Name of the workspace the layout should be applied to
//! workspace = "1"
//!
//! # Path to your layout-file
//! layout_path = "/path/to/my/layout.json"
//!
//! # Alternatively, you can include the JSON-contents of the layout directly:
//! # layout = "{ ... }"
//!
//! # List of applications to start
//! [[applications]]
//! command = "mycommand --with 'multiple args'"
//! working_directory = "/path/to/a/different/working/directory"
//! ```

use deserializers::*;
use serde::de;
use serde::de::{Deserialize, Deserializer};
use shlex;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// This is the parent type defining the complete project configuration used by i3nator.
pub struct Config {
    /// The general configuration section.
    ///
    /// This section defines how a project behaves in general.
    pub general: General,

    /// The applications configuration list.
    ///
    /// This list defines what applications to start and how to start them.
    pub applications: Vec<Application>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// The general configuration section.
///
/// This section defines how a project behaves in general.
pub struct General {
    /// The working directory defines in which directory-context the applications should be
    /// launched in.
    pub working_directory: Option<PathBuf>,

    /// If the workspace is `Some`, `i3` will be instructed to open the layout on the specified
    /// workspace. If it is `None`, `i3` will use the currently focused workspace.
    pub workspace: Option<String>,

    /// The layout to append to a workspace, given as the quasi-JSON `i3-save-tree` returns and
    /// [`append_layout`][append-layout] expects.
    ///
    /// [append-layout]: https://i3wm.org/docs/layout-saving.html#_append_layout_command
    pub layout: Option<String>,

    /// A file-path containing a quasi-JSON as returned by `i3-save-tree` and expected by
    /// [`append_layout`][append-layout].
    ///
    /// [append-layout]: https://i3wm.org/docs/layout-saving.html#_append_layout_command
    pub layout_path: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// The applications configuration.
///
/// This configuration defines how to start an applications and what potential commands to execute
/// in them.
pub struct Application {
    /// The command used for starting an application.
    ///
    /// See [`ApplicationCommand`](struct.ApplicationCommand.html).
    pub command: ApplicationCommand,

    /// The working directory defines in which directory-context the applications should be
    /// launched in.
    ///
    /// This overrides [`general.working_directory`][general-working_directory].
    ///
    /// [general-working_directory]: struct.General.html#structfield.working_directory
    pub working_directory: Option<PathBuf>,

    #[serde(default, deserialize_with="option_string_or_seq_string")]
    /// *WIP:* List of strings to input into the started application
    pub text: Option<Vec<String>>,
    #[serde(default="default_text_return")]
    /// *WIP:* Wether to simulate a `Return` after every string entered from `text`.
    pub text_return: bool,
    /// *WIP:* Specific keys to input into the started application (such as hotkeys).
    pub keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The command used for starting an application.
///
/// # Example
///
/// This struct can be deserialized (from TOML) either from a string or a sequence of strings. The
/// following are equivalent:
///
/// ```toml
/// command = "myprogram --with 'multiple args'"
/// command = ["myprogram, "--with", "multiple args"]
/// ```
///
/// ```rust
/// # extern crate i3nator;
/// # extern crate toml;
/// # use i3nator::types::*;
/// # fn main() {
/// let string: ApplicationCommand = toml::from_str::<Application>(r#"
///         command = "myprogram --with 'multiple args'"
///     "#).unwrap().command;
/// let sequence_of_strings: ApplicationCommand = toml::from_str::<Application>(r#"
///         command = ["myprogram", "--with", "multiple args"]
///     "#).unwrap().command;
///
/// assert_eq!(string, sequence_of_strings);
/// assert_eq!(string.program, "myprogram");
/// assert_eq!(string.args, Some(vec!["--with".to_owned(), "multiple args".to_owned()]));
/// # }
/// ```
///
/// A string will be split up into separate args, honoring single- and double-quoted elements.
pub struct ApplicationCommand {
    /// The executable to start.
    pub program: String,

    /// A list of arguments to pass to the executable.
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
