// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! Module for project handling.

use errors::*;
use i3ipc::I3Connection;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;
use toml;
use types::*;
use wait_timeout::ChildExt;
use xdg;

lazy_static! {
    static ref PROJECTS_PREFIX: OsString = OsString::from("projects");
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix("i3nator").expect("couldn't get XDG base directory");
}

/// A structure representing a `i3nator` project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    /// The name of the project.
    ///
    /// As represented by the stem of the filename on disk.
    pub name: String,

    /// The path to the project configuration.
    pub path: PathBuf,

    config: Option<Config>,
}

impl Project {
    /// Create a project given a `name`.
    ///
    /// This will not create the configuration file, but it will ensure a legal XDG path with all
    /// directories leading up to the file existing.
    ///
    /// If you want to pre-fill the configuration file with a template, see
    /// [`Project::create_from_template`][fn-Project-create_from_template].
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the project and the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Project` for the given `name`.
    /// - `Err`: an error, e.g. if the project already exists or couldn't be created.
    ///
    ///
    /// [fn-Project-create_from_template]: #method.create_from_template
    pub fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let path = project_path(name);
        let name = name.as_ref().to_string_lossy().into_owned();

        if XDG_DIRS.find_config_file(&path).is_some() {
            Err(ErrorKind::ProjectExists(name).into())
        } else {
            XDG_DIRS
                .place_config_file(path)
                .map(|path| {
                         Project {
                             name: name,
                             path: path,
                             config: None,
                         }
                     })
                .map_err(|e| e.into())
        }
    }

    /// Create a project given a `name`, pre-filling the configuration file with a given `template`.
    ///
    /// See [`Project::create`][fn-Project-create] for additional information.
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the project and the configuration file on disk.
    /// - `template`: A byte-slice which will be written to the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Project` for the given `name` with the contents of `template`.
    /// - `Err`: an error, e.g. if the project already exists or couldn't be created.
    ///
    ///
    /// [fn-Project-create]: #method.create
    pub fn create_from_template<S: AsRef<OsStr> + ?Sized>(name: &S,
                                                          template: &[u8])
                                                          -> Result<Self> {
        let project = Project::create(name)?;

        // Copy template into config file
        let mut file = File::create(&project.path)?;
        file.write_all(template)?;
        file.flush()?;
        drop(file);

        Ok(project)
    }

    /// Opens an existing project for a given path.
    ///
    /// This will not impose any XDG conventions, but rather allows to open a configuration from
    /// any path.
    ///
    /// See [`Project::open`][fn-Project-open] if you want to open a project by name.
    ///
    /// # Parameters
    ///
    /// - `path`: A `Path` specifiying the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Project` for the given `path`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-Project-open]: #method.open
    pub fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() || !path.is_file() {
            Err(ErrorKind::PathDoesntExist(path.to_string_lossy().into_owned()).into())
        } else {
            Ok(Project {
                   name: "local".to_owned(),
                   path: path.to_path_buf(),
                   config: None,
               })
        }
    }

    /// Opens an existing project using a `name`.
    ///
    /// This will search for a matching project in the XDG directories.
    ///
    /// See [`Project::from_path`][fn-Project-from_path] if you want to open a project using any
    /// path.
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the project and the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Project` for the given `name`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-Project-from_path]: #method.from_path
    pub fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let path = project_path(name);
        let name = name.as_ref().to_string_lossy().into_owned();

        XDG_DIRS
            .find_config_file(&path)
            .map(|path| {
                     Project {
                         name: name.to_owned(),
                         path: path,
                         config: None,
                     }
                 })
            .ok_or_else(|| ErrorKind::UnknownProject(name).into())
    }

    fn load(&self) -> Result<Config> {
        let mut file = BufReader::new(File::open(&self.path)?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        toml::from_str::<Config>(&contents)
            .clone()
            .map_err(|e| e.into())
    }

    /// Gets the project's configuration, loading and storing it in the current project instance if
    /// it hasn't been before.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of [`Config`][struct-Config] for the project.
    /// - `Err`: an error, e.g. if parsing the configuration failed.
    ///
    ///    If you only want to check if the configuration is valid, without modifying the project
    ///    instance, you can use [`Project::verify`][fn-Project-verify].
    ///
    ///
    /// [struct-Config]: ../types/struct.Config.html
    /// [fn-Project-verify]: #method.verify
    pub fn config(&mut self) -> Result<&Config> {
        if self.config.is_none() {
            self.config = Some(self.load()?);
        }

        Ok(self.config.as_ref().unwrap())
    }

    /// Create a copy of the current project, that is a copy of the configuration file on disk,
    /// with a name of `new_name`.
    ///
    /// # Parameters
    ///
    /// - `new_name`: A `OsStr` that is the name of the destination project.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Project` for the new project.
    /// - `Err`: an error, e.g. if a project with `new_name` already exists or copying the file
    /// failed.
    pub fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let new_project = Project::create(new_name)?;
        fs::copy(&self.path, &new_project.path)?;
        Ok(new_project)
    }

    /// Delete this project's configuration from disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: nothing (`()`).
    /// - `Err`: an error if deleting the file failed.
    pub fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)?;
        Ok(())
    }

    /// Rename the current project.
    ///
    /// # Parameters
    ///
    /// - `new_name`: A `OsStr` that is the name of the destination project.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Project` for the renamed project.
    /// - `Err`: an error, e.g. if a project with `new_name` already exists or renaming the file
    /// failed.
    pub fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        // To avoid having to duplicate the path-handling in `create` et al, just copying and
        // deleting is the easiest way to rename. Refactoring this to use renaming is tracked in
        // #48.
        let new_project = self.copy(new_name)?;
        self.delete()?;

        Ok(new_project)
    }

    /// Start the project.
    ///
    /// This will:
    ///
    /// 1. append the specified layout to a given workspace,
    /// 2. start the specified applications.
    /// 3. execute commands in the applications, if specified.
    ///
    /// Command execution is achieved through the use of [`xdotool`][xdotool], which in turn
    /// simulates key-events through X11 in applications. This is not without problems, though.
    /// Some applications do not react to `SendEvent`s, at least by default.
    ///
    /// One example: in `xterm` you have to specifically enable for `SendEvent`s to be processed.
    /// This can be done through the the [`XTerm.vt100.allowSendEvents`][xterm-allow-send-events]
    /// resource, which ensures that `SendEvent`s are activated when `xterm` starts.
    ///
    /// # Parameters:
    ///
    /// - `i3`: An `I3Connection` to append the layout to a given workspace.
    /// - `working_directory`: An optional working directory which overrides any specified working
    /// directories in the project configuration.
    /// - `workspace`: An optional workspace which overrides the specified workspace in the project
    /// configuration.
    ///
    /// # Returns:
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: nothing (`()`).
    /// - `Err`: an error, if:
    ///
    ///   - the configuration is invalid,
    ///   - if a `layout` was specified but could not be stored in a temporary file,
    ///   - an i3-command failed,
    ///   - an application could not be started,
    ///   - a command could not be sent to an application.
    ///
    ///
    /// [xdotool]: https://github.com/jordansissel/xdotool
    /// [xterm-allow-send-events]: https://www.x.org/archive/X11R6.7.0/doc/xterm.1.html#sect6
    pub fn start(&mut self,
                 i3: &mut I3Connection,
                 working_directory: Option<&OsStr>,
                 workspace: Option<&str>)
                 -> Result<()> {
        let config = self.config()?;
        let general = &config.general;

        // Determine if the layout is a path or the actual contents.
        let mut tempfile;
        let path: &Path = if general.layout.find('{').is_some() {
            // We assume that if the layout contains a `{`, that it is not a path.
            tempfile = NamedTempFile::new()?;
            tempfile.write_all(general.layout.as_bytes())?;
            tempfile.flush()?;
            tempfile.path()
        } else {
            Path::new(&general.layout)
        };

        // Change workspace if provided
        let workspace = workspace
            .map(Into::into)
            .or_else(|| general.workspace.as_ref().cloned());
        if let Some(ref workspace) = workspace {
            i3.command(&format!("workspace {}", workspace))?;
        }

        // Append the layout to the workspace
        i3.command(&format!("append_layout {}",
                             path.to_str()
                                 .ok_or_else(|| {
                                                 ErrorKind::InvalidUtF8Path(path.to_string_lossy()
                                                                                .into_owned())
                                             })?))?;

        // Start the applications
        let applications = &config.applications;
        for application in applications {
            let mut cmd = Command::new(&application.command.program);
            cmd.args(&application.command.args);

            // Get working directory. Precedence is as follows:
            // 1. `--working-directory` command-line parameter
            // 2. `working_directory` option in config for application
            // 3. `working_directory` option in the general section of the config
            let working_directory =
                working_directory
                    .map(OsStr::to_os_string)
                    .or_else(|| application.working_directory.as_ref().map(OsString::from))
                    .or_else(|| general.working_directory.as_ref().map(OsString::from));

            if let Some(working_directory) = working_directory {
                cmd.current_dir(working_directory);
            }

            let child = cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;

            // Input text into application, if any
            if let Some(ref exec) = application.exec {
                exec_commands(&child, exec)?;
            }
        }

        Ok(())
    }

    /// This verifies the project's configuration, without storing it in the current project
    /// instance.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: nothing (`()`) if the verification succeeded.
    /// - `Err`: an error if the configuration could not be parsed with details on what failed.
    pub fn verify(&self) -> Result<()> {
        self.load().map(|_| ())
    }
}

/// Get a list of all project names.
///
/// This will check the current users XDG base directories for `i3nator` project configurations,
/// and return a list of their names for use with e.g. [`Project::open`][fn-Project-open].
///
/// [fn-Project-open]: struct.Project.html#method.open
pub fn list() -> Vec<OsString> {
    let mut files = XDG_DIRS.list_config_files_once(PROJECTS_PREFIX.to_string_lossy().into_owned());
    files.sort();
    files
        .iter()
        .map(|file| file.file_stem())
        .filter(Option::is_some)
        .map(Option::unwrap)
        .map(OsStr::to_os_string)
        .collect::<Vec<_>>()
}

fn project_path<S: AsRef<OsStr> + ?Sized>(name: &S) -> PathBuf {
    let mut path = OsString::new();
    path.push(PROJECTS_PREFIX.as_os_str());
    path.push("/");
    path.push(name);
    path.push(".toml");

    path.into()
}

fn exec_text(base_parameters: &[&str], text: &str, timeout: Duration) -> Result<()> {
    let args = &[base_parameters, &["type", "--window", "%1", text]].concat();
    let mut child = Command::new("xdotool")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Return of `wait_timeout` is `None` if the process didn't exit.
    if child.wait_timeout(timeout)?.is_none() {
        // Kill the xdotool process, return error
        child.kill()?;
        child.wait()?;
        Err(ErrorKind::TextOrKeyInputFailed.into())
    } else {
        Ok(())
    }
}

fn exec_keys<S: AsRef<OsStr>>(base_parameters: &[&str],
                              keys: &[S],
                              timeout: Duration)
                              -> Result<()> {
    let args = &[base_parameters, &["key", "--window", "%1"]].concat();
    let mut child = Command::new("xdotool")
        .args(args)
        .args(keys)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Return of `wait_timeout` is `None` if the process didn't exit.
    if child.wait_timeout(timeout)?.is_none() {
        // Kill the xdotool process, return error
        child.kill()?;
        child.wait()?;
        Err(ErrorKind::TextOrKeyInputFailed.into())
    } else {
        Ok(())
    }
}

fn exec_commands(child: &Child, exec: &Exec) -> Result<()> {
    let timeout = exec.timeout;
    let pid = child.id().to_string();
    let base_parameters = &["search",
                            "--sync",
                            "--onlyvisible",
                            "--any",
                            "--pid",
                            &pid,
                            "ignorepattern",
                            "windowfocus",
                            "--sync",
                            "%1"];

    let commands = &exec.commands;
    match exec.exec_type {
        ExecType::Text => {
            for command in commands {
                exec_text(base_parameters, command, timeout)?;
                exec_keys(base_parameters, &["Return"], timeout)?;
            }
        }
        ExecType::TextNoReturn => {
            for command in commands {
                exec_text(base_parameters, command, timeout)?;
            }
        }
        ExecType::Keys => exec_keys(base_parameters, commands.as_slice(), timeout)?,
    }

    Ok(())
}
