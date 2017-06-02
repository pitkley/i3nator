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
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use xdg;

lazy_static! {
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix("i3nator").expect("couldn't get XDG base directory");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFile {
    prefix: OsString,
    pub name: String,
    pub path: PathBuf,
}

impl ConfigFile {
    pub fn create<S: AsRef<OsStr> + ?Sized>(prefix: &S, name: &S) -> Result<Self> {
        let path = config_path(prefix, name);

        if XDG_DIRS.find_config_file(&path).is_some() {
            Err(ErrorKind::ConfigExists(prefix.as_ref().to_string_lossy().into_owned(),
                                        name.as_ref().to_string_lossy().into_owned())
                        .into())
        } else {
            XDG_DIRS
                .place_config_file(path)
                .map(|path| {
                         ConfigFile {
                             prefix: prefix.as_ref().to_owned(),
                             name: name.as_ref().to_string_lossy().into_owned(),
                             path: path,
                         }
                     })
                .map_err(|e| e.into())
        }
    }

    pub fn create_from_template<S: AsRef<OsStr> + ?Sized>(prefix: &S,
                                                          name: &S,
                                                          template: &[u8])
                                                          -> Result<Self> {
        let configfile = ConfigFile::create(prefix, name)?;

        // Copy template into config file
        let mut file = File::create(&configfile.path)?;
        file.write_all(template)?;
        file.flush()?;
        drop(file);

        Ok(configfile)
    }

    pub fn from_path<S: AsRef<OsStr> + ?Sized, P: AsRef<Path> + ?Sized>(prefix: &S,
                                                                        path: &P)
                                                                        -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() || !path.is_file() {
            Err(ErrorKind::PathDoesntExist(path.to_string_lossy().into_owned()).into())
        } else {
            Ok(ConfigFile {
                   prefix: prefix.as_ref().to_owned(),
                   name: "local".to_owned(),
                   path: path.to_path_buf(),
               })
        }
    }

    pub fn open<S: AsRef<OsStr> + ?Sized>(prefix: &S, name: &S) -> Result<Self> {
        let path = config_path(prefix, name);
        let name = name.as_ref().to_string_lossy().into_owned();

        XDG_DIRS
            .find_config_file(&path)
            .map(|path| {
                     ConfigFile {
                         prefix: prefix.as_ref().to_owned(),
                         name: name.to_owned(),
                         path: path,
                     }
                 })
            .ok_or_else(|| {
                            ErrorKind::UnknownConfig(prefix.as_ref().to_string_lossy().into_owned(),
                                                     name)
                                    .into()
                        })
    }

    pub fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let new_configfile = ConfigFile::create(self.prefix.as_os_str(), new_name.as_ref())?;
        fs::copy(&self.path, &new_configfile.path)?;
        Ok(new_configfile)
    }

    pub fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)?;
        Ok(())
    }

    pub fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        // Create new configfile
        let new_configfile = ConfigFile::create(self.prefix.as_os_str(), new_name.as_ref())?;
        // Rename old configfile
        fs::rename(&self.path, &new_configfile.path)?;

        Ok(new_configfile)
    }
}

fn config_path<S: AsRef<OsStr> + ?Sized>(prefix: &S, name: &S) -> PathBuf {
    let mut path = OsString::new();
    path.push(prefix);
    path.push("/");
    path.push(name);
    path.push(".toml");

    path.into()
}
