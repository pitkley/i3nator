// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! Module for layout handling.

use configfiles::{self, ConfigFile};
use errors::*;
use std::ffi::{OsStr, OsString};
use std::ops::Deref;
use std::path::{Path, PathBuf};

lazy_static! {
    static ref LAYOUTS_PREFIX: OsString = OsString::from("layouts");
}

/// A structure representing a managed i3-layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    configfile: ConfigFile,

    /// The name of the layout.
    ///
    /// As represented by the stem of the filename on disk.
    pub name: String,

    /// The path to the layout configuration.
    pub path: PathBuf,
}

impl Deref for Layout {
    type Target = ConfigFile;

    fn deref(&self) -> &ConfigFile {
        &self.configfile
    }
}

impl Layout {
    /// Create a layout given a `name`.
    ///
    /// This will not create the configuration file, but it will ensure a legal XDG path with all
    /// directories leading up to the file existing.
    ///
    /// If you want to pre-fill the configuration file with a template, see
    /// [`Layout::create_from_template`][fn-Layout-create_from_template].
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the layout and the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Layout` for the given `name`.
    /// - `Err`: an error, e.g. if the layout already exists or couldn't be created.
    ///
    ///
    /// [fn-Layout-create_from_template]: #method.create_from_template
    pub fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let configfile = ConfigFile::create(LAYOUTS_PREFIX.as_os_str(), name.as_ref())?;
        Ok(Layout::from_configfile(configfile))
    }

    /// Create a layout given a `name`, pre-filling the configuration file with a given `template`.
    ///
    /// See [`Layout::create`][fn-Layout-create] for additional information.
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the layout and the configuration file on disk.
    /// - `template`: A byte-slice which will be written to the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Layout` for the given `name` with the contents of `template`.
    /// - `Err`: an error, e.g. if the layout already exists or couldn't be created.
    ///
    ///
    /// [fn-Layout-create]: #method.create
    pub fn create_from_template<S: AsRef<OsStr> + ?Sized>(name: &S,
                                                          template: &[u8])
                                                          -> Result<Self> {
        let configfile =
            ConfigFile::create_from_template(LAYOUTS_PREFIX.as_os_str(), name.as_ref(), template)?;
        Ok(Layout::from_configfile(configfile))
    }

    fn from_configfile(configfile: ConfigFile) -> Self {
        let name = configfile.name.to_owned();
        let path = configfile.path.clone();

        Layout {
            configfile: configfile,
            name: name,
            path: path,
        }
    }

    /// Opens an existing layout for a given path.
    ///
    /// This will not impose any XDG conventions, but rather allows to open a configuration from
    /// any path.
    ///
    /// See [`Layout::open`][fn-Layout-open] if you want to open a layout by name.
    ///
    /// # Parameters
    ///
    /// - `path`: A `Path` specifiying the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Layout` for the given `path`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-Layout-open]: #method.open
    pub fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self> {
        let configfile = ConfigFile::from_path(LAYOUTS_PREFIX.as_os_str(), path)?;
        Ok(Layout::from_configfile(configfile))
    }

    /// Opens an existing layout using a `name`.
    ///
    /// This will search for a matching layout in the XDG directories.
    ///
    /// See [`Layout::from_path`][fn-Layout-from_path] if you want to open a layout using any
    /// path.
    ///
    /// # Parameters
    ///
    /// - `name`: A `OsStr` naming the layout and the configuration file on disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Layout` for the given `name`.
    /// - `Err`: an error, e.g. if the file does not exist.
    ///
    ///
    /// [fn-Layout-from_path]: #method.from_path
    pub fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let configfile = ConfigFile::open(LAYOUTS_PREFIX.as_os_str(), name.as_ref())?;
        Ok(Layout::from_configfile(configfile))
    }

    /// Create a copy of the current layout, that is a copy of the configuration file on disk,
    /// with a name of `new_name`.
    ///
    /// # Parameters
    ///
    /// - `new_name`: A `OsStr` that is the name of the destination layout.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Layout` for the new layout.
    /// - `Err`: an error, e.g. if a layout with `new_name` already exists or copying the file
    /// failed.
    pub fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let configfile = self.configfile.copy(new_name)?;
        Ok(Layout::from_configfile(configfile))
    }

    /// Delete this layout's configuration from disk.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: nothing (`()`).
    /// - `Err`: an error if deleting the file failed.
    pub fn delete(&self) -> Result<()> {
        self.configfile.delete()?;
        Ok(())
    }

    /// Rename the current layout.
    ///
    /// # Parameters
    ///
    /// - `new_name`: A `OsStr` that is the name of the destination layout.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    ///
    /// - `Ok`: an instance of `Layout` for the renamed layout.
    /// - `Err`: an error, e.g. if a layout with `new_name` already exists or renaming the file
    /// failed.
    pub fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let configfile = self.configfile.rename(new_name)?;
        Ok(Layout::from_configfile(configfile))
    }
}

/// Get a list of all layout names.
///
/// This will check the current users XDG base directories for `i3nator` layout configurations,
/// and return a list of their names for use with e.g. [`Layout::open`][fn-Layout-open].
///
/// [fn-Layout-open]: struct.Layout.html#method.open
pub fn list() -> Vec<OsString> {
    configfile::list(&*LAYOUTS_PREFIX)
}
