// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! Module for project handling.

use configfiles::{self, ConfigFile, ConfigFileImpl};
use errors::*;
use i3ipc::I3Connection;
use layouts::Layout as ManagedLayout;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;
use toml;
use types::*;
use wait_timeout::ChildExt;

lazy_static! {
    static ref PROJECTS_PREFIX: OsString = OsString::from("projects");
}

/// A structure representing a `i3nator` project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    configfile: ConfigFileImpl,

    /// The name of the project.
    ///
    /// As represented by the stem of the filename on disk.
    pub name: String,

    /// The path to the project configuration.
    pub path: PathBuf,

    config: Option<Config>,
}

impl Deref for Project {
    type Target = ConfigFileImpl;

    fn deref(&self) -> &ConfigFileImpl {
        &self.configfile
    }
}

impl Project {
    fn from_configfile(configfile: ConfigFileImpl) -> Self {
        let name = configfile.name.to_owned();
        let path = configfile.path.clone();

        Project {
            configfile: configfile,
            name: name,
            path: path,
            config: None,
        }
    }

    fn load(&self) -> Result<Config> {
        let mut file = BufReader::new(File::open(&self.path)?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        toml::from_str::<Config>(&contents).clone().map_err(
            |e| e.into(),
        )
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
    pub fn start(
        &mut self,
        i3: &mut I3Connection,
        working_directory: Option<&OsStr>,
        workspace: Option<&str>,
    ) -> Result<()> {
        let config = self.config()?;
        let general = &config.general;

        // Determine if the layout is a path or the actual contents.
        let mut tempfile;
        let managed_layout_path;
        let path: &Path = match general.layout {
            Layout::Contents(ref contents) => {
                tempfile = NamedTempFile::new()?;
                tempfile.write_all(contents.as_bytes())?;
                tempfile.flush()?;
                tempfile.path()
            }
            Layout::Managed(ref name) => {
                managed_layout_path = ManagedLayout::open(&name)?.path;
                &managed_layout_path
            }
            Layout::Path(ref path) => path,
        };

        // Change workspace if provided
        let workspace = workspace.map(Into::into).or_else(|| {
            general.workspace.as_ref().cloned()
        });
        if let Some(ref workspace) = workspace {
            i3.command(&format!("workspace {}", workspace))?;
        }

        // Append the layout to the workspace
        i3.command(&format!(
            "append_layout {}",
            path.to_str().ok_or_else(|| {
                ErrorKind::InvalidUtF8Path(path.to_string_lossy().into_owned())
            })?
        ))?;

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
                    .or_else(|| {
                        application.working_directory.as_ref().map(OsString::from)
                    })
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
}

impl ConfigFile for Project {
    fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let configfile = ConfigFileImpl::create(PROJECTS_PREFIX.as_os_str(), name.as_ref())?;
        Ok(Project::from_configfile(configfile))
    }

    fn create_from_template<S: AsRef<OsStr> + ?Sized>(name: &S, template: &[u8]) -> Result<Self> {
        let configfile = ConfigFileImpl::create_from_template(
            PROJECTS_PREFIX.as_os_str(),
            name.as_ref(),
            template,
        )?;
        Ok(Project::from_configfile(configfile))
    }

    fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self> {
        let configfile = ConfigFileImpl::from_path(path)?;
        Ok(Project::from_configfile(configfile))
    }

    fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let configfile = ConfigFileImpl::open(PROJECTS_PREFIX.as_os_str(), name.as_ref())?;
        Ok(Project::from_configfile(configfile))
    }

    fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let configfile = self.configfile.copy(new_name)?;
        Ok(Project::from_configfile(configfile))
    }

    fn delete(&self) -> Result<()> {
        self.configfile.delete()?;
        Ok(())
    }

    fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let configfile = self.configfile.rename(new_name)?;
        Ok(Project::from_configfile(configfile))
    }

    fn verify(&self) -> Result<()> {
        // Verify configuration can be loaded
        let config = self.load()?;

        // Collect all loaded paths
        let mut paths: Vec<&Path> = vec![];
        if let Some(ref p) = config.general.working_directory {
            paths.push(p);
        }

        match config.general.layout {
            Layout::Contents(_) => (),
            Layout::Managed(ref name) => {
                ManagedLayout::open(name)?;
            }
            Layout::Path(ref path) => paths.push(path),
        }

        for application in &config.applications {
            if let Some(ref p) = application.working_directory {
                paths.push(p);
            }
        }

        // Verify that all paths exist
        for path in paths {
            if !path.exists() {
                return Err(
                    ErrorKind::PathDoesntExist(path.to_string_lossy().into_owned()).into(),
                );
            }
        }

        Ok(())
    }

    fn list() -> Vec<OsString> {
        configfiles::list(&*PROJECTS_PREFIX)
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    fn prefix() -> &'static OsStr {
        &*PROJECTS_PREFIX
    }
}

/// Get a list of all project names.
///
/// This will check the current users XDG base directories for `i3nator` project configurations,
/// and return a list of their names for use with e.g. [`Project::open`][fn-Project-open].
///
/// [fn-Project-open]: struct.Project.html#method.open
pub fn list() -> Vec<OsString> {
    configfiles::list(&*PROJECTS_PREFIX)
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

fn exec_keys<S: AsRef<OsStr>>(
    base_parameters: &[&str],
    keys: &[S],
    timeout: Duration,
) -> Result<()> {
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
    let base_parameters = &[
        "search",
        "--sync",
        "--onlyvisible",
        "--any",
        "--pid",
        &pid,
        "ignorepattern",
        "windowfocus",
        "--sync",
        "%1",
    ];

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
