// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

use errors::*;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
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

        if let Some(_) = XDG_DIRS.find_config_file(&path) {
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

    fn load(&mut self) -> Result<()> {
        let mut file = BufReader::new(File::open(&self.path)?);
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.config = Some(toml::from_str::<Config>(&contents)?.clone());

        Ok(())
    }

    pub fn config(&mut self) -> Result<&Config> {
        if self.config.is_none() {
            self.load()?;
        }

        Ok(self.config.as_ref().unwrap())
    }

    pub fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Project> {
        let new_project = Project::create(new_name)?;
        fs::copy(&self.path, &new_project.path)?;
        Ok(new_project)
    }

    pub fn delete(&mut self) -> Result<()> {
        fs::remove_file(&self.path)?;
        drop(self);
        Ok(())
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
