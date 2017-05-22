// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

extern crate i3nator;
extern crate toml;

use i3nator::types::*;
use std::time::Duration;

#[test]
fn full_config() {
    let fragment = r#"
        [general]
        working_directory = "/path/to/my/working/directory"
        workspace = "0"
        layout_path = "/path/to/my/layout.json"

        [[applications]]
        command = "mycommand --with 'multiple args'"
        working_directory = "/path/to/a/different/working/directory"
        exec = { commands = ["command one", "command two"], exec_type = "text_no_return" }
        "#;

    let actual: Config = toml::from_str(fragment).unwrap();
    let expected = Config {
        general: General {
            working_directory: Some("/path/to/my/working/directory".to_owned().into()),
            workspace: Some("0".to_owned()),
            layout: None,
            layout_path: Some("/path/to/my/layout.json".to_owned().into()),
        },
        applications: vec![Application {
                               command: ApplicationCommand {
                                   program: "mycommand".to_owned(),
                                   args: Some(vec!["--with".to_owned(),
                                                   "multiple args".to_owned()]),
                               },
                               working_directory: Some("/path/to/a/different/working/directory"
                                                           .to_owned()
                                                           .into()),
                               exec: Some(Exec {
                                              commands: vec!["command one".to_owned(),
                                                             "command two".to_owned()],
                                              exec_type: ExecType::TextNoReturn,
                                              timeout: Duration::from_secs(5),
                                          }),
                           }],
    };

    assert_eq!(actual, expected);
}

#[test]
fn exec_commands_only() {
    let fragment = r#"
        commands = ["command one", "command two"]"#;

    let expected = Exec {
        commands: vec!["command one".to_owned(), "command two".to_owned()],
        exec_type: ExecType::Text,
        timeout: Duration::from_secs(5),
    };

    let actual: Exec = toml::from_str(&fragment).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn exec_commands_and_type() {
    let fragment = r#"
        commands = ["command one", "command two"]
        exec_type = "text_no_return""#;

    let expected = Exec {
        commands: vec!["command one".to_owned(), "command two".to_owned()],
        exec_type: ExecType::TextNoReturn,
        timeout: Duration::from_secs(5),
    };

    let actual: Exec = toml::from_str(&fragment).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn exec_commands_type_and_timeout() {
    let fragment = r#"
        commands = ["command one", "command two"]
        exec_type = "text_no_return"
        timeout = 10"#;

    let expected = Exec {
        commands: vec!["command one".to_owned(), "command two".to_owned()],
        exec_type: ExecType::TextNoReturn,
        timeout: Duration::from_secs(10),
    };

    let actual: Exec = toml::from_str(&fragment).unwrap();

    assert_eq!(actual, expected);
}
