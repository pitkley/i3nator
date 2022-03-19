// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

mod cli;
mod errors {
    use error_chain::error_chain;

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

use crate::errors::*;
use clap::Parser;
use error_chain::quick_main;
use getch::Getch;
use i3ipc::I3Connection;
use i3nator::{configfiles::ConfigFile, layouts::Layout, projects::Project};
use lazy_static::lazy_static;
use std::{
    convert::Into,
    env,
    ffi::{OsStr, OsString},
    fs::File,
    io::{stdin, BufReader, Read},
    process::{Command, ExitStatus},
};

static PROJECT_TEMPLATE: &[u8] = include_bytes!("../resources/project_template.toml");

lazy_static! {
    static ref GETCH: Getch = Getch::new();
}

fn command_copy<C: ConfigFile>(
    existing_configfile_name: &OsStr,
    new_configfile_name: &OsStr,
    no_edit: bool,
    no_verify: bool,
) -> Result<()> {
    let existing_configfile = C::open(existing_configfile_name)?;
    let new_configfile = existing_configfile.copy(new_configfile_name)?;

    println!(
        "Copied existing configfile '{}' to new configfile '{}'",
        existing_configfile.name(),
        new_configfile.name()
    );

    // Open config file for editing
    if !no_edit {
        open_editor(&new_configfile)?;
        if !no_verify {
            verify_configfile(&new_configfile)?;
        }
    }

    Ok(())
}

fn command_delete<C: ConfigFile, S: AsRef<OsStr>>(configfiles: &[S]) -> Result<()> {
    for configfile_name in configfiles {
        C::open(configfile_name)?.delete()?;
        println!(
            "Deleted configfile '{}'",
            configfile_name.as_ref().to_string_lossy()
        );
    }

    Ok(())
}

fn command_edit<C: ConfigFile>(configfile_name: &OsStr, no_verify: bool) -> Result<()> {
    let configfile = C::open(configfile_name)?;

    open_editor(&configfile)?;

    // Verify configfile contents
    if !no_verify {
        verify_configfile(&configfile)?;
    }

    Ok(())
}

fn command_info<C: ConfigFile>(configfile_name: &OsStr) -> Result<()> {
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

fn command_list<C: ConfigFile>(quiet: bool) -> Result<()> {
    let configfiles = C::list();

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

fn command_rename<C: ConfigFile>(
    existing_configfile_name: &OsStr,
    new_configfile_name: &OsStr,
    edit: bool,
    no_verify: bool,
) -> Result<()> {
    let existing_configfile = C::open(existing_configfile_name)?;
    println!(
        "Renaming configfile from '{}' to '{}'",
        existing_configfile_name.to_string_lossy(),
        new_configfile_name.to_string_lossy()
    );
    let new_configfile = existing_configfile.rename(new_configfile_name)?;

    // Open editor for new configfile if desired
    if edit {
        open_editor(&new_configfile)?;
        if !no_verify {
            verify_configfile(&new_configfile)?;
        }
    }

    Ok(())
}

fn project_local(
    project_path: &OsStr,
    working_directory: Option<&OsStr>,
    workspace: Option<&str>,
) -> Result<()> {
    let mut project = Project::from_path(project_path)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project.start(&mut i3, working_directory, workspace)?;

    Ok(())
}

fn project_new(project_name: &OsStr, no_edit: bool, no_verify: bool) -> Result<()> {
    let project = Project::create_from_template(project_name, PROJECT_TEMPLATE)?;
    println!("Created project '{}'", project.name);

    // Open config file for editing
    if !no_edit {
        open_editor(&project)?;
        if !no_verify {
            verify_configfile(&project)?;
        }
    }

    Ok(())
}

fn project_start(
    project_name: &OsStr,
    working_directory: Option<&OsStr>,
    workspace: Option<&str>,
) -> Result<()> {
    let mut project = Project::open(project_name)?;
    let mut i3 = I3Connection::connect()?;

    println!("Starting project '{}'", project.name);
    project.start(&mut i3, working_directory, workspace)?;

    Ok(())
}

fn project_verify<S: AsRef<OsStr>>(configfiles: &[S]) -> Result<()> {
    // The list of config-fiels can be empty. If so, use the entire configfile list.
    let mut configfiles: Vec<OsString> = configfiles
        .iter()
        .map(|v| v.as_ref().to_os_string())
        .collect::<Vec<_>>();
    if configfiles.is_empty() {
        configfiles = Project::list();
    }

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

fn layout_new(layout_name: &OsStr, template: Option<&OsStr>, no_edit: bool) -> Result<()> {
    let layout = if let Some(template) = template {
        // Open appropriate reader
        let stdin_;
        let reader: Box<dyn Read> = if template == "-" {
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
    } else {
        Layout::create(layout_name)?
    };
    println!("Created layout '{}'", layout.name);

    // Open config file for editing
    if !no_edit {
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
                !matches!(ch, Some('a') | Some('r'))
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
    let clap = cli::Cli::parse();
    match &clap.command {
        cli::Commands::Copy {
            existing,
            new,
            no_edit,
            no_verify,
        } => command_copy::<Project>(existing, new, *no_edit, *no_verify),
        cli::Commands::Delete { names } => command_delete::<Project, _>(&names[..]),
        cli::Commands::Edit { name, no_verify } => command_edit::<Project>(name, *no_verify),
        cli::Commands::Info { name } => command_info::<Project>(name),
        cli::Commands::List { quiet } => command_list::<Project>(*quiet),
        cli::Commands::Local {
            file,
            working_directory,
            workspace,
        } => project_local(file, working_directory.as_deref(), workspace.as_deref()),
        cli::Commands::New {
            name,
            no_edit,
            no_verify,
        } => project_new(name, *no_edit, *no_verify),
        cli::Commands::Rename {
            existing,
            new,
            edit,
            no_verify,
        } => command_rename::<Project>(existing, new, *edit, *no_verify),
        cli::Commands::Start {
            name,
            working_directory,
            workspace,
        } => project_start(name, working_directory.as_deref(), workspace.as_deref()),
        cli::Commands::Verify { names } => project_verify(&names[..]),
        cli::Commands::Layout(layout_commands) => match layout_commands {
            cli::LayoutCommands::Copy {
                existing,
                new,
                no_edit,
            } => command_copy::<Layout>(existing, new, *no_edit, false),
            cli::LayoutCommands::Delete { names } => command_delete::<Layout, _>(&names[..]),
            cli::LayoutCommands::Edit { name } => command_edit::<Layout>(name, false),
            cli::LayoutCommands::Info { name } => command_info::<Layout>(name),
            cli::LayoutCommands::List { quiet } => command_list::<Layout>(*quiet),
            cli::LayoutCommands::New {
                name,
                no_edit,
                template,
            } => layout_new(name, template.as_deref(), *no_edit),
            cli::LayoutCommands::Rename {
                existing,
                new,
                edit,
            } => command_rename::<Layout>(existing, new, *edit, false),
        },
        cli::Commands::GenerateShellCompletions {
            generator,
            output_path,
        } => cli::generate_completions(*generator, output_path.as_deref()).map_err(|e| e.into()),
    }
}

#[cfg(unix)]
quick_main!(run);
