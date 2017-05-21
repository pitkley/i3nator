// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! i3nator is [Tmuxinator][gh-tmuxinator] for the [i3 window manager][i3wm].
//!
//! It allows you to manage and restore saved i3 layouts (see [Layout saving in
//! i3][i3wm-layout-saving]) easily, and extends i3's base functionality by allowing you to start
//! applications too.
//!
//! * [Documentation][i3nator-docs]
//! * [GitHub source repository][i3nator-gh]
//!
//! # Installation:
//!
//! You have multiple options to install i3nator:
//!
//! 1. If you have a recent Rust with Cargo installed, you can install `i3nator` directly from
//!    crates.io:
//!
//!    ```console
//!    $ cargo install i3nator
//!   ```
//!
//! 2. Alternatively, you can download the supplied static binary from the [release
//!    page][i3nator-releases], this should work without any additional dependencies.
//!
//! 3. Another option is to install from directly from source (this again requires a recent Rust
//!    installation):
//!
//!    ```console
//!    $ git clone https://github.com/pitkley/i3nator.git
//!    $ cd i3nator
//!    $ cargo install
//!    ```
//!
//! # Usage:
//!
//! Following the usage of i3nator as of 0.1.0.
//!
//! ```text
//! USAGE:
//!     i3nator <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     copy [FLAGS] <EXISTING> <NEW>
//!               Copy an existing project to a new project
//!
//!     delete <PROJECT>...
//!               Delete existing projects
//!
//!     edit [FLAGS] <PROJECT>
//!               Open an existing project in your editor
//!
//!     help
//!               Prints this message or the help of the given subcommand(s)
//!
//!     info <PROJECT>
//!               !!! TODO !!!
//!
//!     list [FLAGS]
//!               List all projects
//!
//!     local [OPTIONS]
//!               Run a project from a local TOML-file
//!
//!     new [FLAGS] <PROJECT>
//!               Create a new project and open it in your editor
//!
//!     rename [FLAGS] <CURRENT> <NEW>
//!               Rename a project
//!
//!     start [OPTIONS] <PROJECT>
//!               Start a project according to it's configuration
//!
//!     verify [PROJECT]...
//!               verify the configuration of the existing projects
//! ```
//!
//! # Example
//!
//! ## Full workflow
//!
//! 1. Open all applications you want and lay them out on a workspace as desired.
//!
//! 2. Save the workspace's layout using [`i3-save-tree`][i3wm-save-tree]:
//!
//!    ```console
//!    $ i3-save-tree --workspace 1 > mylayout.json
//!    ```
//!
//!    You can also skip the file-redirection and copy the layout into your clipboard.
//!
//! 3. Modify the saved layout to accurately match created applications. See [Editing layout
//!    files][i3wm-modify-layout] on how to do this.
//!
//!    If you copied the layout to your clipboard, you will be able to do this in step 5.
//!
//! 4. Create a new project:
//!
//!    ```console
//!    $ i3nator new myproject
//!    Created project 'myproject'
//!    Opening your editor to edit project myproject
//!    ```
//!
//!    This will open your default editor with a configuration template. If it doesn't, you have to
//!    specify either the `$VISUAL` or the `$EDITOR` environment variable.
//!
//!    You can also simply edit the configuration file directly. Use `i3nator info <PROJECT>` to
//!    retreive its path.
//!
//! 5. Modify the template to fit your needs. This includes:
//!
//!    1. Setting the main working directory.
//!    2. Setting the destination workspace (this is optional, if not specified, the active one
//!       will be used).
//!    3. Specifying which layout to use. You can either supply a path to the file containing the
//!       layout created in step 2, or you can use the `layout` variable to paste the layout from
//!       step 2 as a multi-line string. At this point you will also be able to modify the layout
//!       to match applications correctly.
//!    4. Configuring which applications to start and how to start them. This is done by setting
//!       the `command` to the full command to be used to start the application and optionally
//!       configuring a different working directory if desired.
//!
//! 6. Save and close your editor. This will automatically verify the created configuration. If
//!    there is an issue it will tell you what failed and allow you to reedit the file directly or
//!    ignore the error and exit.
//!
//! With these prerequisites fulfilled, you are now able to start a configuration which appends
//! your layout to a specified workspace and starts the configured applications:
//!
//! ```console
//! $ i3nator start myproject
//! ```
//!

//! [gh-tmuxinator]: https://github.com/tmuxinator/tmuxinator
//! [i3nator-docs]: https://docs.rs/i3nator
//! [i3nator-gh]: https://github.com/pitkley/i3nator
//! [i3nator-releases]: https://github.com/pitkley/i3nator/releases
//! [i3wm]: https://i3wm.org/
//! [i3wm-modify-layout]: https://i3wm.org/docs/layout-saving.html#_editing_layout_files
//! [i3wm-layout-saving]: https://i3wm.org/docs/layout-saving.html
//! [i3wm-save-tree]: https://i3wm.org/docs/layout-saving.html#_saving_the_layout

#![recursion_limit = "1024"] // `error_chain!` can recurse deeply
#![warn(missing_docs)] // TODO: increase from `warn` to `deny`

#[macro_use]
extern crate error_chain;
extern crate i3ipc;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate tempfile;
extern crate toml;
extern crate xdg;

mod deserializers;
pub mod errors;
pub mod projects;
mod shlex;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
