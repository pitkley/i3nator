// Copyright 2017 Pit Kleyersburg <pitkley@googlemail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified or distributed
// except according to those terms.

#![recursion_limit = "1024"] // `error_chain!` can recurse deeply
#![warn(missing_docs)] // TODO: increase from `warn` to `deny`

#[macro_use]
extern crate error_chain;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod deserializers;
pub mod errors;
mod shlex;
pub mod types;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
