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
        "#;

    let actual: Config = toml::from_str(fragment).unwrap();
    let expected = Config {
        general: General {
            working_directory: Some("/path/to/my/working/directory".to_owned()),
            workspace: Some("0".to_owned()),
            layout: None,
            layout_path: Some("/path/to/my/layout.json".to_owned()),
        },
        applications: vec![Application {
                               command: ApplicationCommand {
                                   program: "mycommand".to_owned(),
                                   args: Some(vec!["--with".to_owned(),
                                                   "multiple args".to_owned()]),
                               },
                               working_directory: Some("/path/to/a/different/working/directory"
                                                           .to_owned()),
                               text: None,
                               text_return: true,
                               keys: None,
                           }],
    };

    assert_eq!(actual, expected);
}
