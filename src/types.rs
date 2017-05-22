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

use serde::de;
use serde::de::{Deserialize, Deserializer};
use shlex;
use std::fmt;
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
    #[serde(deserialize_with="deserialize_application_command")]
    pub command: ApplicationCommand,

    /// The working directory defines in which directory-context the applications should be
    /// launched in.
    ///
    /// This overrides [`general.working_directory`][general-working_directory].
    ///
    /// [general-working_directory]: struct.General.html#structfield.working_directory
    pub working_directory: Option<PathBuf>,

    /// Commands to execute or keys to simulate after application startup.
    pub exec: Option<Exec>,
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
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

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
/// Commands to execute or keys to simulate after application startup.
///
/// `xdotool` is used to simulate text or keys to input.
pub struct Exec {
    /// List of text or keys to input into the application.
    ///
    /// Text can be defined as simple strings. Keystrokes have to specified in a format `xdotool`
    /// expects them, see `xdotool`'s [official documentation][xdotool-keyboard].
    ///
    /// [xdotool-keyboard]:
    ///   https://github.com/jordansissel/xdotool/blob/master/xdotool.pod#keyboard-commands
    pub commands: Vec<String>,

    /// Defines how the commands above should be interpreted.
    ///
    /// If not specified, [`ExecType::Text`][variant-ExecType-Text] will be used by default.
    ///
    /// [variant-ExecType-Text]: enum.ExecType.html#variant.Text
    pub exec_type: Option<ExecType>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Defines how the commands in [`Exec`][struct-Exec] should be interpreted.
///
/// [struct-Exec]: struct.Exec.html
pub enum ExecType {
    /// Interpret the commands given as separate text-lines, inputting them in order with a
    /// `Return` after each.
    Text,

    /// Interpret the commands given as text, but do not input a `Return` after each element.
    TextNoReturn,

    /// Interpret the commands given as key presses.
    ///
    /// This does not input any `Return`s.
    Keys,
}

fn deserialize_application_command<'de, D>(deserializer: D) -> Result<ApplicationCommand, D::Error>
    where D: Deserializer<'de>
{
    impl<'de> de::Visitor<'de> for ApplicationCommand {
        type Value = ApplicationCommand;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string, sequence of strings or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            match shlex::split(value) {
                Some(mut v) => {
                    if v.is_empty() {
                        Err(de::Error::custom("command can not be empty"))
                    } else {
                        Ok(ApplicationCommand {
                               program: v.remove(0).to_owned(),
                               args: Some(v.into_iter().map(str::to_owned).collect::<Vec<_>>()),
                           })
                    }
                }
                None => Err(de::Error::custom("command can not be empty")),
            }
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
            where S: de::SeqAccess<'de>
        {
            let mut v: Vec<String> =
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))?;
            if v.is_empty() {
                Err(de::Error::custom("command can not be empty"))
            } else {
                Ok(ApplicationCommand {
                       program: v.remove(0),
                       args: Some(v),
                   })
            }
        }

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
            where M: de::MapAccess<'de>
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(ApplicationCommand::default())
}
