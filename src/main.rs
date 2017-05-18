// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate xdg;

mod cli;
mod errors {
    error_chain! {
        foreign_links {
            IoError(::std::io::Error);
        }

        errors {
            EditorNotFound {
                description("cannot find an editor")
                display("cannot find an editor. Please specify $VISUAL or $EDITOR")
            }

            NoProjectExist {
                description("no projects exist")
                display("no projects exist. Feel free to create one")
            }

            ProjectExists(t: String) {
                description("project already exists")
                display("project already exists: '{}'", t)
            }

            UnknownProject(t: String) {
                description("project is unknown")
                display("project is unknown: '{}'", t)
            }
        }
    }
}

use clap::ArgMatches;
use errors::*;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

static PROJECT_TEMPLATE: &'static [u8] = include_bytes!("../resources/project_template.toml");
//static PROJECTS_PREFIX: &'static str = "projects";

lazy_static! {
    static ref PROJECTS_PREFIX: OsString = OsString::from("projects");
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix(crate_name!()).expect("couldn't get XDG base directory");
}

struct Project {
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

fn command_copy(matches: &ArgMatches<'static>) -> Result<()> {
    // `EXISTING` and `NEW` should not be empty, clap ensures this.
    let existing_project_name = matches.value_of_os("EXISTING").unwrap();
    let new_project_name = matches.value_of_os("NEW").unwrap();

    let existing_project = Project::open(existing_project_name)?;
    let new_project = Project::create(new_project_name)?;

    fs::copy(existing_project.path, new_project.path)?;
    println!("Copied existing project '{}' to new project '{}'",
             existing_project_name.to_string_lossy(),
             new_project_name.to_string_lossy());
    Ok(())
}

fn command_delete(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();

    Project::open(project_name)?.delete()
}

fn command_edit(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::open(project_name)?;

    println!("opening your editor to edit project {}",
             project_name.to_string_lossy());
    Command::new(get_editor()?)
        .arg(project.path)
        .status()
        .map(|_| ())
        .map_err(|e| e.into())
}

fn command_list(_matches: &ArgMatches<'static>) -> Result<()> {
    let mut files = XDG_DIRS.list_config_files_once(PROJECTS_PREFIX.to_string_lossy().into_owned());

    if files.is_empty() {
        Err(ErrorKind::NoProjectExist.into())
    } else {
        // Sort projects
        files.sort();

        println!("i3nator projects:");
        for file in files {
            // Map file to it's stem name (no path, no extension)
            if let Some(file_stem) = file.file_stem()
                   .and_then(|stem| stem.to_str())
                   .map(|stem| stem.to_owned()) {
                println!("  {}", file_stem);
            }
        }

        Ok(())
    }
}

fn command_local(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_new(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("PROJECT").unwrap();
    let project = Project::create(project_name)?;

    // Copy template into config file
    let mut file = File::create(&project.path)?;
    file.write_all(PROJECT_TEMPLATE)?;
    file.flush()?;
    drop(file);

    // Open config file for editing
    println!("opening your editor to edit project {}",
             project_name.to_string_lossy());
    Command::new(get_editor()?)
        .arg(project.path)
        .status()
        .map(|_| ())
        .map_err(|e| e.into())
}

fn command_start(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn get_editor() -> Result<OsString> {
    env::var_os("VISUAL")
        .or_else(|| env::var_os("EDITOR"))
        .ok_or_else(|| ErrorKind::EditorNotFound.into())
}

fn run() -> Result<()> {
    let matches = cli::cli().get_matches();

    match matches.subcommand() {
        ("copy", Some(matches)) => command_copy(matches),
        ("delete", Some(matches)) => command_delete(matches),
        ("edit", Some(matches)) => command_edit(matches),
        ("list", Some(matches)) => command_list(matches),
        ("local", Some(matches)) => command_local(matches),
        ("new", Some(matches)) => command_new(matches),
        ("start", Some(matches)) => command_start(matches),
        ("", None) =>
            // No subcommand given. The clap `AppSettings` should be set to output the help by
            // default, so this is unreachable.
            unreachable!(),
        _ =>
            // All subcommands are defined above, this is unreachable.
            unreachable!(),
    }
}

quick_main!(run);
