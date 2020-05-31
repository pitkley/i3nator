// Copyright Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

//! # i3nator
//!
//! i3nator is [Tmuxinator][gh-tmuxinator] for the [i3 window manager][i3wm].
//!
//! It allows you to manage what are called "projects", which are used to easily restore saved i3
//! layouts (see [Layout saving in i3][i3wm-layout-saving]) and extending i3's base functionality
//! by allowing you to automatically start applications too.
//!
//! For detailed introductions, see the [README][github-readme].
//!
//! [github-readme]: https://github.com/pitkley/i3nator#readme
//!
//! ## License
//!
//! DFW is licensed under either of
//!
//! * Apache License, Version 2.0, (<http://www.apache.org/licenses/LICENSE-2.0>)
//! * MIT license (<https://opensource.org/licenses/MIT>)
//!
//! at your option.

#![recursion_limit = "1024"] // `error_chain!` can recurse deeply
#![deny(missing_docs)]

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
extern crate wait_timeout;
extern crate xdg;

pub mod configfiles;
pub mod errors;
pub mod layouts;
pub mod projects;
mod shlex;
pub mod types;
