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
use std::path::PathBuf;
use xdg;

lazy_static! {
    static ref PROJECTS_PREFIX: OsString = OsString::from("projects");
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix("i3nator").expect("couldn't get XDG base directory");
}

pub struct Project {
    pub path: PathBuf,
}

impl Project {
    pub fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let mut path = OsString::new();
        path.push(PROJECTS_PREFIX.as_os_str());
        path.push("/");
        path.push(name);
        path.push(".toml");

        if let Some(_) = XDG_DIRS.find_config_file(&path) {
            Err(ErrorKind::ProjectExists(name.as_ref().to_string_lossy().into_owned()).into())
        } else {
            XDG_DIRS
                .place_config_file(path)
                .map(|path| Project { path: path })
                .map_err(|e| e.into())
        }
    }

    pub fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let mut path = OsString::new();
        path.push(PROJECTS_PREFIX.as_os_str());
        path.push("/");
        path.push(name);
        path.push(".toml");

        XDG_DIRS
            .find_config_file(&path)
            .map(|path| Project { path: path })
            .ok_or_else(|| {
                            ErrorKind::UnknownProject(name.as_ref().to_string_lossy().into_owned())
                                .into()
                        })
    }

    pub fn delete(&mut self) -> Result<()> {
        fs::remove_file(&self.path)?;
        drop(self);
        Ok(())
    }
}
