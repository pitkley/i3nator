// Copyright Pit Kleyersburg <pitkley@googlemail.com>
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

use configfiles::ConfigFile;
use layouts::Layout as ManagedLayout;
use serde::de;
use serde::de::{Deserialize, Deserializer};
use shlex;
use std::borrow::Cow;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::marker::PhantomData;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// This is the parent type defining the complete project configuration used by i3nator.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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

/// The general configuration section.
///
/// This section defines how a project behaves in general.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct General {
    /// The working directory defines in which directory-context the applications should be
    /// launched in.
    #[serde(default, deserialize_with = "deserialize_opt_pathbuf_with_tilde")]
    pub working_directory: Option<PathBuf>,

    /// If the workspace is `Some`, `i3` will be instructed to open the layout on the specified
    /// workspace. If it is `None`, `i3` will use the currently focused workspace.
    pub workspace: Option<String>,

    /// The layout to append to a workspace.
    ///
    /// This should either be:
    ///
    /// * the quasi-JSON as returned by `i3-save-tree`
    /// * or a file-path containing the quasi-JSON as returned by `i3-save-tree`.
    ///
    /// Either one will be passed to [`append_layout`][append-layout].
    ///
    /// [append-layout]: https://i3wm.org/docs/layout-saving.html#_append_layout_command
    #[serde(deserialize_with = "deserialize_layout")]
    pub layout: Layout,
}

/// This holds the layout, in multiple formats.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    /// The layout is provided directly as a string.
    Contents(String),

    /// The name of a managed layout
    Managed(String),

    /// The layout is provided as a path.
    Path(PathBuf),
}

/// The applications configuration.
///
/// This configuration defines how to start an applications and what potential commands to execute
/// in them.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Application {
    /// The command used for starting an application.
    ///
    /// See [`ApplicationCommand`](struct.ApplicationCommand.html).
    #[serde(deserialize_with = "deserialize_application_command")]
    pub command: ApplicationCommand,

    /// The working directory defines in which directory-context the applications should be
    /// launched in.
    ///
    /// This overrides [`general.working_directory`][general-working_directory].
    ///
    /// [general-working_directory]: struct.General.html#structfield.working_directory
    #[serde(default, deserialize_with = "deserialize_opt_pathbuf_with_tilde")]
    pub working_directory: Option<PathBuf>,

    /// Commands to execute or keys to simulate after application startup.
    #[serde(default, deserialize_with = "deserialize_opt_exec")]
    pub exec: Option<Exec>,
}

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
/// assert_eq!(string.args, vec!["--with".to_owned(), "multiple args".to_owned()]);
/// # }
/// ```
///
/// A string will be split up into separate args, honoring single- and double-quoted elements.
#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct ApplicationCommand {
    /// The executable to start.
    pub program: String,

    /// A list of arguments to pass to the executable.
    #[serde(default)]
    pub args: Vec<String>,
}

/// Commands to execute or keys to simulate after application startup.
///
/// `xdotool` is used to simulate text or keys to input.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
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
    #[serde(default = "default_exec_type")]
    pub exec_type: ExecType,

    /// Specify a timeout after which a command has to be succesfully input into the application.
    ///
    /// The input of commands is done with `xdotool --sync`, that is `xdotool` will block until the
    /// required application starts up. `xdotool` might fail to find a started application, if that
    /// application does not behave well within the X11 standards.
    ///
    /// In this case, `xdotool` would block indefinitely. This timeout will kill the `xdotool`
    /// process if it does not exit (successfully or unsuccessfully).
    #[serde(default = "default_timeout", deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

fn default_exec_type() -> ExecType {
    ExecType::Text
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

/// Defines how the commands in [`Exec`][struct-Exec] should be interpreted.
///
/// [struct-Exec]: struct.Exec.html
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

struct Phantom<T>(PhantomData<T>);

fn deserialize_application_command<'de, D>(deserializer: D) -> Result<ApplicationCommand, D::Error>
where
    D: Deserializer<'de>,
{
    impl<'de> de::Visitor<'de> for Phantom<ApplicationCommand> {
        type Value = ApplicationCommand;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string, sequence of strings or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match shlex::split(value) {
                Some(mut v) => {
                    if v.is_empty() {
                        Err(de::Error::custom("command can not be empty"))
                    } else {
                        Ok(ApplicationCommand {
                            program: v.remove(0).to_owned(),
                            args: v.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                        })
                    }
                }
                None => Err(de::Error::custom("command can not be empty")),
            }
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let mut v: Vec<String> =
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))?;
            if v.is_empty() {
                Err(de::Error::custom("command can not be empty"))
            } else {
                Ok(ApplicationCommand {
                    program: v.remove(0),
                    args: v,
                })
            }
        }

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(Phantom::<ApplicationCommand>(PhantomData))
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    impl<'de> de::Visitor<'de> for Phantom<Duration> {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("integer or map")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_secs(value as u64))
        }

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(Phantom::<Duration>(PhantomData))
}

fn deserialize_exec<'de, D>(deserializer: D) -> Result<Exec, D::Error>
where
    D: Deserializer<'de>,
{
    impl<'de> de::Visitor<'de> for Phantom<Exec> {
        type Value = Exec;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string, sequence of strings or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Exec {
                commands: vec![value.to_owned()],
                exec_type: default_exec_type(),
                timeout: default_timeout(),
            })
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            let v: Vec<String> =
                Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))?;

            if v.is_empty() {
                Err(de::Error::custom("commands can not be empty"))
            } else {
                Ok(Exec {
                    commands: v,
                    exec_type: default_exec_type(),
                    timeout: default_timeout(),
                })
            }
        }

        fn visit_map<M>(self, visitor: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(Phantom::<Exec>(PhantomData))
}

fn deserialize_opt_exec<'de, D>(deserializer: D) -> Result<Option<Exec>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_exec(deserializer).map(Some)
}

fn deserialize_layout<'de, D>(deserializer: D) -> Result<Layout, D::Error>
where
    D: Deserializer<'de>,
{
    impl<'de> de::Visitor<'de> for Phantom<Layout> {
        type Value = Layout;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.find('{').is_some() {
                Ok(Layout::Contents(value.into()))
            } else if ManagedLayout::open(value).is_ok() {
                Ok(Layout::Managed(value.to_owned()))
            } else {
                Ok(Layout::Path(tilde(value).into_owned()))
            }
        }
    }

    deserializer.deserialize_any(Phantom::<Layout>(PhantomData))
}

fn deserialize_pathbuf_with_tilde<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let pathbuf: PathBuf = Deserialize::deserialize(deserializer)?;
    Ok(tilde(&pathbuf).into_owned())
}

fn deserialize_opt_pathbuf_with_tilde<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_pathbuf_with_tilde(deserializer).map(Some)
}

/// Taken from crate "shellexpand", adapted to work with `Path` instead of `str`:
///   https://github.com/netvl/shellexpand/blob/
///     501c4fdd8275fea2e56e71a2659cd90d21d18565/src/lib.rs#L558-L639
///
/// (Linebreak in link to make rustfmt happy...)
///
/// Dual-licensed under MIT/Apache 2.0
/// Copyright (c) 2016 Vladimir Matveev
#[doc(hidden)]
fn tilde_with_context<SI: ?Sized, P, HD>(input: &SI, home_dir: HD) -> Cow<Path>
where
    SI: AsRef<Path>,
    P: AsRef<Path>,
    HD: FnOnce() -> Option<P>,
{
    let input_str = input.as_ref();
    let bytes = input_str.as_os_str().as_bytes();
    if bytes[0] == b'~' {
        let input_after_tilde = &bytes[1..];
        if input_after_tilde.is_empty() || input_after_tilde[0] == b'/' {
            if let Some(hd) = home_dir() {
                let mut s = OsString::new();
                s.push(hd.as_ref().to_path_buf());
                s.push(OsStr::from_bytes(input_after_tilde));
                PathBuf::from(s).into()
            } else {
                // home dir is not available
                input_str.into()
            }
        } else {
            // we cannot handle `~otheruser/` paths yet
            input_str.into()
        }
    } else {
        // input doesn't start with tilde
        input_str.into()
    }
}

fn tilde<SI: ?Sized>(input: &SI) -> Cow<Path>
where
    SI: AsRef<Path>,
{
    tilde_with_context(input, env::home_dir)
}
