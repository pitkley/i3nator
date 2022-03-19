// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! Module consolidating common functionality between projects and layouts.

use crate::errors::*;
use lazy_static::lazy_static;
use std::{
    ffi::{OsStr, OsString},
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
};

lazy_static! {
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix("i3nator").expect("couldn't get XDG base directory");
}

/// Helping type to consolidate common functionality between projects and layouts.
pub trait ConfigFile: Sized {
    /// Create a copy of the current configfile, that is a copy of the configuration file on disk,
    /// with a name of `new_name`.
    ///
    /// This will keep the same prefix.
    ///
    /// # Parameters
    ///
    /// - `new_name`: A `OsStr` that is the name of the destination configfile.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the new configfile.
    /// - `Err`: an error, e.g. if a configfile with `new_name` already exists or copying the file
    /// failed.
    fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self>;

    /// Create a configfile given a `name`.
    ///
    /// This will not create the configuration file, but it will ensure a legal XDG path with all
    /// directories leading up to the file existing.
    ///
    /// If you want to pre-fill the configuration file with a template, see
    /// [`ConfigFile::create_from_template`][fn-ConfigFile-create_from_template].
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `name`.
    /// - `Err`: an error, e.g. if the configfile already exists or couldn't be created.
    ///
    ///
    /// [fn-ConfigFile-create_from_template]: #method.create_from_template
    fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self>;

    /// Create a configfile given a `name`, pre-filling it with a given `template`.
    ///
    /// See [`ConfigFile::create`][fn-ConfigFile-create] for additional information.
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the the configuration file on disk.
    /// - `template`: A byte-slice which will be written to the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `name` with the contents of `template`.
    /// - `Err`: an error, e.g. if the configfile already exists or couldn't be created.
    ///
    ///
    /// [fn-ConfigFile-create]: #method.create
    fn create_from_template<S: AsRef<OsStr> + ?Sized>(name: &S, template: &[u8]) -> Result<Self>;

    /// Delete this configfile from disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: nothing (`()`).
    /// - `Err`: an error if deleting the file failed.
    fn delete(&self) -> Result<()>;

    /// Opens an existing configfile for a given path.
    ///
    /// This will not impose any XDG conventions, but rather allows to open a configuration from
    /// any path.
    ///
    /// See [`ConfigFile::open`][fn-ConfigFile-open] if you want to open a configfile by name and
    /// prefix.
    ///
    /// # Parameters
    ///
    /// - `path`: A `Path` specifiying the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `path`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-ConfigFile-open]: #method.open
    fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self>;

    /// Get a list of all configfile names.
    ///
    /// This will check the current users XDG base directories for configuration files, and return a
    /// list of their names for use with e.g. [`ConfigFile::open`][fn-ConfigFile-open].
    ///
    /// [fn-ConfigFile-open]: struct.Layout.html#method.open
    fn list() -> Vec<OsString>;

    /// Returns the name of this configfile.
    ///
    /// As represented by the stem of the filename on disk.
    fn name(&self) -> String;

    /// Opens an existing configfile using a `name`.
    ///
    /// This will search for a matching configfile in the XDG directories.
    ///
    /// See [`ConfigFile::from_path`][fn-ConfigFile-from_path] if you want to open a configfile
    /// using any path.
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `name`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-ConfigFile-from_path]: #method.from_path
    fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self>;

    /// Returns the path to the configfile.
    fn path(&self) -> PathBuf;

    /// Return the prefix associated with this type of configfile.
    fn prefix() -> &'static OsStr;

    /// Rename the current configfile.
    ///
    /// # Parameters
    ///
    /// - `new_name`: A `OsStr` that is the name of the destination configfile.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the renamed configfile.
    /// - `Err`: an error, e.g. if a configfile with `new_name` already exists or renaming the file
    /// failed.
    fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self>;

    /// This verifies the project's configuration, without storing it in the current project
    /// instance.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: nothing (`()`) if the verification succeeded.
    /// - `Err`: an error if the configuration could not be parsed with details on what failed.
    fn verify(&self) -> Result<()>;
}

/// Helping type to consolidate common functionality between projects and layouts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigFileImpl {
    prefix: OsString,

    /// The name of the configfile.
    ///
    /// As represented by the stem of the filename on disk.
    pub name: String,

    /// The path to the configfile.
    pub path: PathBuf,
}

impl ConfigFileImpl {
    /// Create a configfile given a `name` and `prefix`.
    ///
    /// This will not create the configuration file, but it will ensure a legal XDG path with all
    /// directories leading up to the file existing.
    ///
    /// If you want to pre-fill the configuration file with a template, see
    /// [`ConfigFile::create_from_template`][fn-ConfigFile-create_from_template].
    ///
    /// # Parameters
    ///
    /// - `prefix`: A `OsStr` defining a prefix which is used as a sub-directory for the configfile.
    /// - `name`: A `OsStr` naming the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `name`.
    /// - `Err`: an error, e.g. if the configfile already exists or couldn't be created.
    ///
    ///
    /// [fn-ConfigFile-create_from_template]: #method.create_from_template
    pub fn create<S: AsRef<OsStr> + ?Sized>(prefix: &S, name: &S) -> Result<Self> {
        let path = config_path(prefix, name);

        if XDG_DIRS.find_config_file(&path).is_some() {
            Err(ErrorKind::ConfigExists(
                prefix.as_ref().to_string_lossy().into_owned(),
                name.as_ref().to_string_lossy().into_owned(),
            )
            .into())
        } else {
            XDG_DIRS
                .place_config_file(path)
                .map(|path| ConfigFileImpl {
                    prefix: prefix.as_ref().to_owned(),
                    name: name.as_ref().to_string_lossy().into_owned(),
                    path,
                })
                .map_err(|e| e.into())
        }
    }

    /// Create a configfile given a `name`, pre-filling it with a given `template`.
    ///
    /// See [`ConfigFile::create`][fn-ConfigFile-create] for additional information.
    ///
    /// # Parameters
    ///
    /// - `prefix`: A `OsStr` defining a prefix which is used as a sub-directory for the configfile.
    /// - `name`: A `OsStr` naming the the configuration file on disk.
    /// - `template`: A byte-slice which will be written to the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `name` with the contents of `template`.
    /// - `Err`: an error, e.g. if the configfile already exists or couldn't be created.
    ///
    ///
    /// [fn-ConfigFile-create]: #method.create
    pub fn create_from_template<S: AsRef<OsStr> + ?Sized>(
        prefix: &S,
        name: &S,
        template: &[u8],
    ) -> Result<Self> {
        let configfile = ConfigFileImpl::create(prefix, name)?;

        // Copy template into config file
        let mut file = File::create(&configfile.path)?;
        file.write_all(template)?;
        file.flush()?;
        drop(file);

        Ok(configfile)
    }

    /// Opens an existing configfile using a `name`.
    ///
    /// This will search for a matching configfile in the XDG directories.
    ///
    /// See [`ConfigFile::from_path`][fn-ConfigFile-from_path] if you want to open a configfile
    /// using any path.
    ///
    /// # Parameters
    ///
    /// - `prefix`: A `OsStr` defining a prefix which is used as a sub-directory for the configfile.
    /// - `name`: A `OsStr` naming the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `ConfigFile` for the given `name`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-ConfigFile-from_path]: #method.from_path
    pub fn open<S: AsRef<OsStr> + ?Sized>(prefix: &S, name: &S) -> Result<Self> {
        let path = config_path(prefix, name);
        let name = name.as_ref().to_string_lossy().into_owned();

        XDG_DIRS
            .find_config_file(&path)
            .map(|path| ConfigFileImpl {
                prefix: prefix.as_ref().to_owned(),
                name: name.to_owned(),
                path,
            })
            .ok_or_else(|| {
                ErrorKind::UnknownConfig(prefix.as_ref().to_string_lossy().into_owned(), name)
                    .into()
            })
    }
}

impl ConfigFile for ConfigFileImpl {
    fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let new_configfile = ConfigFileImpl::create(self.prefix.as_os_str(), new_name.as_ref())?;
        fs::copy(&self.path, &new_configfile.path)?;
        Ok(new_configfile)
    }

    fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        Err(ErrorKind::UnknownConfig(
            "NO PREFIX".to_owned(),
            name.as_ref().to_string_lossy().into_owned(),
        )
        .into())
    }

    fn create_from_template<S: AsRef<OsStr> + ?Sized>(name: &S, _template: &[u8]) -> Result<Self> {
        Err(ErrorKind::UnknownConfig(
            "NO PREFIX".to_owned(),
            name.as_ref().to_string_lossy().into_owned(),
        )
        .into())
    }

    fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)?;
        Ok(())
    }

    fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() || !path.is_file() {
            Err(ErrorKind::PathDoesntExist(path.to_string_lossy().into_owned()).into())
        } else {
            Ok(ConfigFileImpl {
                prefix: "local".to_owned().into(),
                name: "local".to_owned(),
                path: path.to_path_buf(),
            })
        }
    }

    fn list() -> Vec<OsString> {
        // This cannot be implemented.
        vec![]
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        Err(ErrorKind::UnknownConfig(
            "NO PREFIX".to_owned(),
            name.as_ref().to_string_lossy().into_owned(),
        )
        .into())
    }

    fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    fn prefix() -> &'static OsStr {
        OsStr::new("")
    }

    fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        // Create new configfile
        let new_configfile = ConfigFileImpl::create(self.prefix.as_os_str(), new_name.as_ref())?;
        // Rename old configfile
        fs::rename(&self.path, &new_configfile.path)?;

        Ok(new_configfile)
    }

    fn verify(&self) -> Result<()> {
        Ok(())
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

/// Get a list of all configfile names for a given prefix.
///
/// This will check the current users XDG base directories for configuration files, and return a
/// list of their names for use with e.g. [`ConfigFile::open`][fn-ConfigFile-open].
///
/// [fn-ConfigFile-open]: struct.Layout.html#method.open
pub fn list<S: AsRef<OsStr> + ?Sized>(prefix: &S) -> Vec<OsString> {
    let mut files = XDG_DIRS.list_config_files_once(prefix.as_ref().to_string_lossy().into_owned());
    files.sort();
    files
        .iter()
        .filter_map(|file| file.file_stem())
        .map(OsStr::to_os_string)
        .collect::<Vec<_>>()
}
