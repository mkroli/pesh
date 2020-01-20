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

use std::collections::BTreeMap;

use nom::character::complete::*;
use nom::number::complete::*;

use crate::errors::*;
use crate::registry::REGISTRY;

pub struct Metric {
    pub name: String,
    pub tags: BTreeMap<String, String>,
}

pub trait Command {
    fn execute(&self) -> Result<Option<String>>;
}

struct SetMetricCommand {
    metric: Metric,
    value: f64,
}

impl Command for SetMetricCommand {
    fn execute(&self) -> Result<Option<String>> {
        let mut registry = REGISTRY
            .lock()
            .map_err(|_| Error::from("failed to lock registry"))?;
        registry.add(&self.metric, self.value)?;
        Ok(None)
    }
}

struct DelMetricCommand {
    metric: Metric,
}

impl Command for DelMetricCommand {
    fn execute(&self) -> Result<Option<String>> {
        let mut registry = REGISTRY
            .lock()
            .map_err(|_| Error::from("failed to lock registry"))?;
        registry.remove(&self.metric)?;
        Ok(None)
    }
}

struct GetMetricCommand {
    metric: Metric,
}

impl Command for GetMetricCommand {
    fn execute(&self) -> Result<Option<String>> {
        let registry = REGISTRY
            .lock()
            .map_err(|_| Error::from("failed to lock registry"))?;
        let value = registry.get(&self.metric)?;
        Ok(Some(format!("{}", value)))
    }
}

struct ExitCommand;

impl Command for ExitCommand {
    fn execute(&self) -> Result<Option<String>> {
        std::process::exit(0);
    }
}

struct HelpCommand;

impl Command for HelpCommand {
    fn execute(&self) -> Result<Option<String>> {
        Ok(Some(include_str!("help.txt").to_string()))
    }
}

named!(
    string<&str, String>,
    map!(
        delimited!(
            char!('"'),
            escaped!(
                none_of!("\"\\"),
                '\\',
                one_of!("\"\\")
            ),
            char!('"')
        ),
        |s| s.to_string()
    )
);

named!(
    tags<&str, BTreeMap<String, String>>,
    map!(
        separated_list!(
            tag!(","),
            do_parse!(
                key: map!(alpha1, String::from) >>
                multispace0 >>
                tag!("=") >>
                multispace0 >>
                value: string
                >>
                ((key.to_string(), value.to_string()))
            )
        ),
        |tag_list| {
            let mut tags = BTreeMap::<String, String>::new();
            for (key, value) in tag_list {
                tags.insert(key, value);
            }
            tags
        }
    )
);

named!(
    metric<&str, Metric>,
    do_parse!(
        name: map!(alpha1, String::from) >>
        tags: map!(
            opt!(
                complete!(
                    delimited!(
                        char!('['),
                        tags,
                        char!(']')
                    )
                )
            ),
            |o| {
                o.unwrap_or(BTreeMap::<String, String>::new())
            }
        )
        >>
        ({
            Metric {
                name,
                tags,
            }
        })
    )
);

named!(
    command_set_metric<&str, Box<dyn Command>>,
    do_parse!(
        tag!("set") >>
        multispace1 >>
        metric: metric >>
        multispace0 >>
        tag!("=") >>
        multispace0 >>
        value: double
        >>
        (
            Box::new(SetMetricCommand {
                metric,
                value,
            })
        )
    )
);

named!(
    command_del_metric<&str, Box<dyn Command>>,
    do_parse!(
        tag!("del") >>
        multispace1 >>
        metric: metric
        >>
        (
            Box::new(DelMetricCommand {
                metric,
            })
        )
    )
);

named!(
    command_get_metric<&str, Box<dyn Command>>,
    do_parse!(
        tag!("get") >>
        multispace1 >>
        metric: metric
        >>
        (
            Box::new(GetMetricCommand {
                metric,
            })
        )
    )
);

named!(
    command_exit<&str, Box<dyn Command>>,
    do_parse!(
        alt!(tag!("exit") | tag!("quit"))
        >>
        (
            Box::new(ExitCommand {})
        )
    )
);

named!(
    command_help<&str, Box<dyn Command>>,
    do_parse!(
        tag!("help")
        >>
        (
            Box::new(HelpCommand {})
        )
    )
);

named!(
    command<&str, Box<dyn Command>>,
    alt!(
        complete!(command_set_metric) |
        complete!(command_del_metric) |
        complete!(command_get_metric) |
        complete!(command_exit) |
        complete!(command_help)
    )
);

named!(
    empty<&str, ()>,
    map!(multispace0, |_| ())
);

named!(
    line<&str, Option<Box<dyn Command>>>,
    exact!(
        alt!(
            map!(command, |c| Some(c)) |
            map!(empty, |_| None)
        )
    )
);

pub fn parse_command(s: &str) -> Result<Option<Box<dyn Command>>> {
    let (_, cmd) = line(s).map_err(|_| Error::from("failed to parse"))?;
    Ok(cmd)
}
