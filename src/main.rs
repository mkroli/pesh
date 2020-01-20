/*
 * Copyright 2020 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate prometheus;

use clap::{AppSettings, Arg, ArgMatches};

use errors::*;

use crate::exporter::setup_prometheus;
use crate::shell::Shell;

mod commands;
mod errors;
mod exporter;
mod logging;
mod registry;
mod shell;

const DEFAULT_PROMETHEUS_ADDRESS: &str = "127.0.0.1:9000";

fn run() -> Result<()> {
    logging::setup_logger()?;

    let matches: ArgMatches = app_from_crate!()
        .global_setting(AppSettings::ColoredHelp)
        .arg(
            Arg::with_name("prometheus_address")
                .short("a")
                .long("address")
                .value_name("host:port")
                .default_value(DEFAULT_PROMETHEUS_ADDRESS),
        )
        .get_matches();

    let prometheus_address = matches
        .value_of("prometheus_address")
        .unwrap_or(DEFAULT_PROMETHEUS_ADDRESS);

    setup_prometheus(prometheus_address)?;
    let shell = Shell::new()?;
    for line in shell {
        match commands::parse_command(&line?) {
            Err(e) => {
                warn!("{}", e);
            }
            Ok(None) => (),
            Ok(Some(command)) => match command.execute() {
                Ok(result) => {
                    if let Some(out) = result {
                        println!("{}", out);
                    }
                }
                Err(e) => {
                    warn!("{}", e);
                }
            },
        }
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => {
            error!("Execution failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
