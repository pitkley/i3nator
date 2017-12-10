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
extern crate getch;
extern crate i3ipc;
extern crate i3nator;
#[macro_use]
extern crate lazy_static;
extern crate tempfile;

mod cli;
mod errors {
    error_chain! {
        foreign_links {
            I3EstablishError(::i3ipc::EstablishError);
            I3MessageError(::i3ipc::MessageError);
            IoError(::std::io::Error);
        }

        links {
            Lib(::i3nator::errors::Error, ::i3nator::errors::ErrorKind);
        }

        errors {
            EditorNotFound {
                description("cannot find an editor")
                display("cannot find an editor. Please specify $VISUAL or $EDITOR")
            }

            NoConfigExist {
                description("no configfiles exist")
                display("no configfiles exist. Feel free to create one")
            }
        }
    }
}

use clap::ArgMatches;
use errors::*;
use getch::Getch;
use i3ipc::I3Connection;
use i3nator::configfiles::ConfigFile;
use i3nator::layouts::Layout;
use i3nator::projects::Project;
use std::ascii::AsciiExt;
use std::convert::Into;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{stdin, BufReader, Read};
use std::process::{Command, ExitStatus};

static PROJECT_TEMPLATE: &'static [u8] = include_bytes!("../resources/project_template.toml");

lazy_static! {
    static ref GETCH: Getch = Getch::new().expect("failed to create getch");
}

fn command_copy<C: ConfigFile>(matches: &ArgMatches<'static>) -> Result<()> {
    // `EXISTING` and `NEW` should not be empty, clap ensures this.
    let existing_configfile_name = matches.value_of_os("EXISTING").unwrap();
    let new_configfile_name = matches.value_of_os("NEW").unwrap();

    let existing_configfile = C::open(existing_configfile_name)?;
    let new_configfile = existing_configfile.copy(new_configfile_name)?;

    println!(
        "Copied existing configfile '{}' to new configfile '{}'",
        existing_configfile.name(),
        new_configfile.name()
    );

    // Open config file for editing
    if !matches.is_present("no-edit") {
        open_editor(&new_configfile)?;
        if !matches.is_present("no-verify") {
            verify_configfile(&new_configfile)?;
        }
    }

    Ok(())
}

fn command_delete<C: ConfigFile>(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME`s should not be empty, clap ensures this.
    let configfiles = matches.values_of_os("NAME").unwrap();

    for configfile_name in configfiles {
        C::open(configfile_name)?.delete()?;
        println!("Deleted configfile '{}'", configfile_name.to_string_lossy());
    }

    Ok(())
}

fn command_edit<C: ConfigFile>(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME` should not be empty, clap ensures this.
    let configfile_name = matches.value_of_os("NAME").unwrap();
    let configfile = C::open(configfile_name)?;

    open_editor(&configfile)?;

    // Verify configfile contents
    if !matches.is_present("no-verify") {
        verify_configfile(&configfile)?;
    }

    Ok(())
}

fn command_info<C: ConfigFile>(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME` should not be empty, clap ensures this.
    let configfile_name = matches.value_of_os("NAME").unwrap();
    let configfile = C::open(configfile_name)?;

    println!("Name: {}", configfile.name());
    println!(
        "Configuration path: {}",
        configfile.path().to_string_lossy()
    );
    println!(
        "Configuration valid: {}",
        if configfile.verify().is_ok() {
            "yes"
        } else {
            "NO"
        }
    );

    Ok(())
}

fn command_layout(matches: &ArgMatches<'static>) -> Result<()> {
    match matches.subcommand() {
        ("copy", Some(sub_matches)) => command_copy::<Layout>(sub_matches),
        ("delete", Some(sub_matches)) => command_delete::<Layout>(sub_matches),
        ("edit", Some(sub_matches)) => command_edit::<Layout>(sub_matches),
        ("info", Some(sub_matches)) => command_info::<Layout>(sub_matches),
        ("list", Some(sub_matches)) => command_list::<Layout>(sub_matches),
        ("new", Some(sub_matches)) => layout_new(sub_matches),
        ("rename", Some(sub_matches)) => command_rename::<Layout>(sub_matches),
        ("", None) =>
            // No subcommand given. The clap `AppSettings` should be set to output the help by
            // default, so this is unreachable.
            unreachable!(),
        _ =>
            // If all subcommands are defined above, this should be unreachable.
            unreachable!(),
    }
}

fn command_list<C: ConfigFile>(matches: &ArgMatches<'static>) -> Result<()> {
    let configfiles = C::list();
    let quiet = matches.is_present("quiet");

    if configfiles.is_empty() {
        Err(ErrorKind::NoConfigExist.into())
    } else {
        if !quiet {
            println!("i3nator {}:", C::prefix().to_string_lossy());
        }
        for configfile in configfiles {
            if quiet {
                println!("{}", configfile.to_string_lossy());
            } else {
                println!("  {}", configfile.to_string_lossy());
            }
        }

        Ok(())
    }
}

fn command_rename<C: ConfigFile>(matches: &ArgMatches<'static>) -> Result<()> {
    // `CURRENT` and `NEW` should not be empty, clap ensures this.
    let current_configfile_name = matches.value_of_os("CURRENT").unwrap();
    let new_configfile_name = matches.value_of_os("NEW").unwrap();

    let current_configfile = C::open(current_configfile_name)?;
    println!(
        "Renaming configfile from '{}' to '{}'",
        current_configfile_name.to_string_lossy(),
        new_configfile_name.to_string_lossy()
    );
    let new_configfile = current_configfile.rename(new_configfile_name)?;

    // Open editor for new configfile if desired
    if matches.is_present("edit") {
        open_editor(&new_configfile)?;
        if !matches.is_present("no-verify") {
            verify_configfile(&new_configfile)?;
        }
    }

    Ok(())
}

fn project_local(matches: &ArgMatches<'static>) -> Result<()> {
    // `FILE` should not be empty, clap ensures this.
    let project_path = matches.value_of_os("file").unwrap();
    let mut project = Project::from_path(project_path)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project.start(
        &mut i3,
        matches.value_of_os("working-directory"),
        matches.value_of("workspace"),
    )?;

    Ok(())
}

fn project_new(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("NAME").unwrap();
    let project = Project::create_from_template(project_name, PROJECT_TEMPLATE)?;
    println!("Created project '{}'", project.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        open_editor(&project)?;
        if !matches.is_present("no-verify") {
            verify_configfile(&project)?;
        }
    }

    Ok(())
}

fn project_start(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME` should not be empty, clap ensures this.
    let project_name = matches.value_of_os("NAME").unwrap();
    let mut project = Project::open(project_name)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project.start(
        &mut i3,
        matches.value_of_os("working-directory"),
        matches.value_of("workspace"),
    )?;

    Ok(())
}

fn project_verify(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME`s can be empty, if so, use the entire configfile list
    let configfiles: Vec<OsString> = matches
        .values_of_os("NAME")
        .map(|v| v.map(OsStr::to_os_string).collect::<Vec<_>>())
        .unwrap_or_else(Project::list);

    for configfile_name in configfiles {
        if let Err(e) = Project::open(&configfile_name)?.verify() {
            println!(
                "Configuration INVALID: '{}'",
                configfile_name.to_string_lossy()
            );
            println!("Error:");
            println!("    {}", e);
            println!();
        } else {
            println!(
                "Configuration   VALID: '{}'",
                configfile_name.to_string_lossy()
            );
        }
    }

    Ok(())
}

fn layout_new(matches: &ArgMatches<'static>) -> Result<()> {
    // `NAME` should not be empty, clap ensures this.
    let layout_name = matches.value_of_os("NAME").unwrap();

    let layout = if !matches.is_present("template") {
        Layout::create(layout_name)?
    } else {
        let template = matches.value_of_os("template").unwrap();

        // Open appropriate reader
        let stdin_;
        let reader: Box<Read> = if template == "-" {
            stdin_ = stdin();
            Box::new(stdin_.lock())
        } else {
            Box::new(File::open(template)?)
        };
        let mut reader = BufReader::new(reader);

        // Load bytes from reader
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes)?;

        // Create layout from template
        Layout::create_from_template(layout_name, &bytes)?
    };
    println!("Created layout '{}'", layout.name);

    // Open config file for editing
    if !matches.is_present("no-edit") {
        open_editor(&layout)?;
    }

    Ok(())
}

fn get_editor() -> Result<OsString> {
    env::var_os("VISUAL")
        .or_else(|| env::var_os("EDITOR"))
        .and_then(|s| if !s.is_empty() { Some(s) } else { None })
        .ok_or_else(|| ErrorKind::EditorNotFound.into())
}

fn open_editor<C: ConfigFile>(configfile: &C) -> Result<ExitStatus> {
    println!("Opening your editor to edit '{}'", configfile.name());
    Command::new(get_editor()?)
        .arg(configfile.path().as_os_str())
        .status()
        .map_err(|e| e.into())
}

fn verify_configfile<C: ConfigFile>(configfile: &C) -> Result<()> {
    while let Err(e) = configfile.verify() {
        println!();
        println!("VERIFICATION FAILED!");
        println!("Error:");
        println!("  {}", e);
        println!();

        let mut ch: Option<char>;
        while {
            println!("What do you want to do?");
            println!("(R)eopen editor, (A)ccept anyway");

            ch = GETCH
                .getch()
                .ok()
                .map(|byte| byte.to_ascii_lowercase())
                .map(|byte| byte as char);

            if ch.is_none() {
                true
            } else {
                match ch {
                    Some('a') | Some('r') => false,
                    _ => true,
                }
            }
        } {
            // Ugly do-while syntax:
            //   https://gist.github.com/huonw/8435502
        }

        match ch {
            Some('a') => break,
            Some('r') => open_editor(configfile)?,
            _ => continue,
        };
    }

    Ok(())
}

fn run() -> Result<()> {
    let matches = cli::cli().get_matches();

    match matches.subcommand() {
        ("copy", Some(sub_matches)) => command_copy::<Project>(sub_matches),
        ("delete", Some(sub_matches)) => command_delete::<Project>(sub_matches),
        ("edit", Some(sub_matches)) => command_edit::<Project>(sub_matches),
        ("info", Some(sub_matches)) => command_info::<Project>(sub_matches),
        ("layout", Some(sub_matches)) => command_layout(sub_matches),
        ("list", Some(sub_matches)) => command_list::<Project>(sub_matches),
        ("local", Some(sub_matches)) => project_local(sub_matches),
        ("new", Some(sub_matches)) => project_new(sub_matches),
        ("rename", Some(sub_matches)) => command_rename::<Project>(sub_matches),
        ("start", Some(sub_matches)) => project_start(sub_matches),
        ("verify", Some(sub_matches)) => project_verify(sub_matches),
        ("", None) =>
            // No subcommand given. The clap `AppSettings` should be set to output the help by
            // default, so this is unreachable.
            unreachable!(),
        _ =>
            // If all subcommands are defined above, this should be unreachable.
            unreachable!(),
    }
}

#[cfg(unix)]
quick_main!(run);
