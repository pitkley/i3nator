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
extern crate glob;
extern crate xdg;

mod cli;
mod errors {
    error_chain! {
        foreign_links {
            GlobError(::glob::PatternError);
            IoError(::std::io::Error);
        }

        errors {
            EditorNotFound {
                description("cannot find an editor")
                display("cannot find an editor. Please specify $VISUAL or $EDITOR")
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
use glob::glob;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;

static PROJECT_TEMPLATE: &'static [u8] = include_bytes!("../resources/project_template.toml");
static PROJECTS_PREFIX: &'static str = "projects";

lazy_static! {
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix(crate_name!()).expect("couldn't get XDG base directory");
    static ref PROJECTS_PATH: PathBuf =
        XDG_DIRS.create_config_directory(PROJECTS_PREFIX)
                .expect("couldn't create projects directory");
}

fn command_copy(matches: &ArgMatches<'static>) -> Result<()> {
    // `EXISTING` and `NEW` should not be empty, clap ensures this.
    let existing_project_name = matches.value_of("EXISTING").unwrap();
    let new_project_name = matches.value_of("NEW").unwrap();

    let existing_project_path = PROJECTS_PATH.join(format!("{}.toml", existing_project_name));
    let new_project_path = PROJECTS_PATH.join(format!("{}.toml", new_project_name));

    if existing_project_path.exists() && existing_project_path.is_file() {
        if new_project_path.exists() {
            Err(ErrorKind::ProjectExists(new_project_name.to_owned()).into())
        } else {
            fs::copy(existing_project_path, new_project_path)?;
            println!("Copied existing project '{}' to new project '{}'",
                     existing_project_name,
                     new_project_name);
            Ok(())
        }
    } else {
        Err(ErrorKind::UnknownProject(existing_project_name.to_owned()).into())
    }
}

fn command_delete(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of("PROJECT").unwrap();
    let project_path = PROJECTS_PATH.join(format!("{}.toml", project_name));

    if project_path.exists() && project_path.is_file() {
        fs::remove_file(project_path)?;
        println!("Deleted project '{}'", project_name);
        Ok(())
    } else {
        Err(ErrorKind::UnknownProject(project_name.to_owned()).into())
    }
}

fn command_edit(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of("PROJECT").unwrap();
    let project_path = PROJECTS_PATH.join(format!("{}.toml", project_name));

    if project_path.exists() && project_path.is_file() {
        println!("opening your editor to edit project {}", project_name);
        Command::new(get_editor()?)
            .arg(project_path)
            .status()
            .map(|_| ())
            .map_err(|e| e.into())
    } else {
        Err(ErrorKind::UnknownProject(project_name.to_owned()).into())
    }
}

fn command_list(_matches: &ArgMatches<'static>) -> Result<()> {
    let project_path = PROJECTS_PATH.join("*.toml");
    let project_path = project_path
        .to_str()
        .ok_or_else(|| Error::from("couldn't get projects"))?;

    println!("i3nator projects:");

    for entry in glob(project_path)? {
        if let Some(project) = entry
               .ok()
               .and_then(|pathbuf| {
                             let stem = pathbuf.file_stem();
                             stem.and_then(|filestem| filestem.to_str().map(|s| s.to_owned()))
                         }) {
            println!("  {}", project);
        }
    }

    Ok(())
}

fn command_local(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_new(matches: &ArgMatches<'static>) -> Result<()> {
    // `PROJECT` should not be empty, clap ensures this.
    let project_name = matches.value_of("PROJECT").unwrap();
    let project_path = PROJECTS_PATH.join(format!("{}.toml", project_name));

    if project_path.exists() {
        Err(ErrorKind::ProjectExists(project_name.to_owned()).into())
    } else {
        let path = XDG_DIRS
            .place_config_file(&format!("{}/{}.toml", PROJECTS_PREFIX, project_name))?;
        let mut file = File::create(path)?;
        file.write_all(PROJECT_TEMPLATE)?;
        file.flush()?;
        drop(file);

        println!("opening your editor to edit project {}", project_name);
        Command::new(get_editor()?)
            .arg(project_path)
            .status()
            .map(|_| ())
            .map_err(|e| e.into())
    }
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
