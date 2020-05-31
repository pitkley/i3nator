// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! Module for layout handling.

use configfiles::{self, ConfigFile, ConfigFileImpl};
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
    configfile: ConfigFileImpl,

    /// The name of the layout.
    ///
    /// As represented by the stem of the filename on disk.
    pub name: String,

    /// The path to the layout configuration.
    pub path: PathBuf,
}

impl Deref for Layout {
    type Target = ConfigFileImpl;

    fn deref(&self) -> &ConfigFileImpl {
        &self.configfile
    }
}

impl Layout {
    fn from_configfile(configfile: ConfigFileImpl) -> Self {
        let name = configfile.name.to_owned();
        let path = configfile.path.clone();

        Layout {
            configfile,
            name,
            path,
        }
    }
}

impl ConfigFile for Layout {
    fn create<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let configfile = ConfigFileImpl::create(LAYOUTS_PREFIX.as_os_str(), name.as_ref())?;
        Ok(Layout::from_configfile(configfile))
    }

    fn create_from_template<S: AsRef<OsStr> + ?Sized>(name: &S, template: &[u8]) -> Result<Self> {
        let configfile = ConfigFileImpl::create_from_template(
            LAYOUTS_PREFIX.as_os_str(),
            name.as_ref(),
            template,
        )?;
        Ok(Layout::from_configfile(configfile))
    }

    fn from_path<P: AsRef<Path> + ?Sized>(path: &P) -> Result<Self> {
        let configfile = ConfigFileImpl::from_path(path)?;
        Ok(Layout::from_configfile(configfile))
    }

    fn open<S: AsRef<OsStr> + ?Sized>(name: &S) -> Result<Self> {
        let configfile = ConfigFileImpl::open(LAYOUTS_PREFIX.as_os_str(), name.as_ref())?;
        Ok(Layout::from_configfile(configfile))
    }

    fn copy<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let configfile = self.configfile.copy(new_name)?;
        Ok(Layout::from_configfile(configfile))
    }

    fn delete(&self) -> Result<()> {
        self.configfile.delete()?;
        Ok(())
    }

    fn rename<S: AsRef<OsStr> + ?Sized>(&self, new_name: &S) -> Result<Self> {
        let configfile = self.configfile.rename(new_name)?;
        Ok(Layout::from_configfile(configfile))
    }

    fn verify(&self) -> Result<()> {
        Ok(())
    }

    fn list() -> Vec<OsString> {
        configfiles::list(&*LAYOUTS_PREFIX)
    }

    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn path(&self) -> PathBuf {
        self.path.to_owned()
    }

    fn prefix() -> &'static OsStr {
        &*LAYOUTS_PREFIX
    }
}

/// Get a list of all layout names.
///
/// This will check the current users XDG base directories for `i3nator` layout configurations,
/// and return a list of their names for use with e.g. [`Layout::open`][fn-Layout-open].
///
/// [fn-Layout-open]: struct.Layout.html#method.open
pub fn list() -> Vec<OsString> {
    configfiles::list(&*LAYOUTS_PREFIX)
}
