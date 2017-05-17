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
use std::fs;
use std::path::PathBuf;

lazy_static! {
    static ref XDG_DIRS: xdg::BaseDirectories =
        xdg::BaseDirectories::with_prefix(crate_name!()).expect("couldn't get XDG base directory");
    static ref PROJECTS_PATH: PathBuf =
        XDG_DIRS.create_config_directory("projects").expect("couldn't create projects directory");
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

fn command_edit(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
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

fn command_new(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
}

fn command_start(_matches: &ArgMatches<'static>) -> Result<()> {
    unimplemented!()
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
