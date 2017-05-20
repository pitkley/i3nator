// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use errors::*;
use i3ipc::I3Connection;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;
use toml;
use types::*;
use xdg;

lazy_static! {
    static ref PROJECTS_PREFIX: OsString = OsString::from("projects");
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix("i3nator").expect("couldn't get XDG base directory");
}

pub struct Project {
    pub name: String,
    pub path: PathBuf,
    config: Option<Config>,
}

impl Project {
    pub fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let mut path = OsString::new();
        path.push(PROJECTS_PREFIX.as_os_str());
        path.push("/");
        path.push(name);
        path.push(".toml");

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

    pub fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let mut path = OsString::new();
        path.push(PROJECTS_PREFIX.as_os_str());
        path.push("/");
        path.push(name);
        path.push(".toml");

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

    pub fn config(&mut self) -> Result<&Config> {
        if self.config.is_none() {
            self.config = Some(self.load()?);
        }

        Ok(self.config.as_ref().unwrap())
    }

    pub fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let new_project = Project::create(new_name)?;
        fs::copy(&self.path, &new_project.path)?;
        Ok(new_project)
    }

    pub fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)?;
        Ok(())
    }

    pub fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        // To avoid having to duplicate the path-handling in `create` et al, just copying and
        // deleting is the easiest way to rename.
        //
        // TODO: this is obviously not the cleanest way, `std::fs::rename` would be. Maybe we
        // should extract the path-handling code from `create` into a separate method, like
        // `projects::project_path`.
        let new_project = self.copy(new_name)?;
        self.delete()?;

        Ok(new_project)
    }

    pub fn start(&mut self,
                 i3: &mut I3Connection,
                 working_directory: Option<&OsStr>,
                 workspace: Option<&str>)
                 -> Result<()> {
        let config = self.config()?;
        let general = &config.general;

        // Create temporary file if required
        let mut tempfile = if general.layout.is_some() {
            Some(NamedTempFile::new()?)
        } else {
            None
        };

        // Get the provided layout-path or the path of the temporary file
        let path: &Path = if let Some(ref path) = general.layout_path {
            path
        } else if let Some(ref layout) = general.layout {
            // The layout has been provided directly, save into the temporary file.
            let mut tempfile = tempfile.as_mut().unwrap();
            tempfile.write_all(layout.as_bytes())?;
            tempfile.flush()?;
            tempfile.path()
        } else {
            // Neither `layout` nor `layout_path` has been specified
            bail!(ErrorKind::LayoutNotSpecified)
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
            // Set args if available
            if let Some(ref args) = application.command.args {
                cmd.args(args);
            }

            // Get working directory. Precedence is as follows:
            // 1. `--working-directory` command-line parameter
            // 2. `working_directory` option in config for application
            // 3. `working_directory` option in the general section of the config
            let working_directory =
                working_directory
                    .map(OsStr::to_os_string)
                    .or_else(|| {
                                 application
                                     .working_directory
                                     .as_ref()
                                     .map(OsString::from)
                             })
                    .or_else(|| general.working_directory.as_ref().map(OsString::from));

            if let Some(working_directory) = working_directory {
                cmd.current_dir(working_directory);
            }

            cmd.stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
        }

        Ok(())
    }

    pub fn verify(&self) -> Result<()> {
        self.load().map(|_| ())
    }
}

pub fn list() -> Vec<String> {
    let mut files = XDG_DIRS.list_config_files_once(PROJECTS_PREFIX.to_string_lossy().into_owned());
    files.sort();
    files
        .iter()
        .map(|file| file.file_stem())
        .filter(Option::is_some)
        .map(Option::unwrap)
        .map(|stem| stem.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
}
