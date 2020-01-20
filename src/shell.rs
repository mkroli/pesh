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

use std::path::PathBuf;

use app_dirs::AppDataType;
use app_dirs::AppInfo;
use cedarwood::Cedar;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::error::ReadlineError::*;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};

use crate::errors::*;

const PROMPT: &'static str = "pesh> ";

const APP_INFO: AppInfo = AppInfo {
    name: env!("CARGO_PKG_NAME"),
    author: env!("CARGO_PKG_AUTHORS"),
};

pub struct Shell {
    rl: Editor<ShellHelper>,
}

struct ShellHelper;

impl Helper for ShellHelper {}

impl Highlighter for ShellHelper {}

impl Validator for ShellHelper {}

impl Hinter for ShellHelper {}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> std::result::Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let commands: Vec<&str> = vec!["set ", "del ", "get ", "help", "quit", "exit"];
        let key = &line[0..pos];
        if key.is_empty() {
            let res = commands
                .iter()
                .map(|c| Pair {
                    display: c.to_string(),
                    replacement: c.to_string(),
                })
                .collect();
            Ok((pos, res))
        } else {
            let mut cedar = Cedar::new();
            {
                let c: Vec<(&str, i32)> = commands
                    .iter()
                    .enumerate()
                    .map(|(k, s)| (*s, k as i32))
                    .collect();
                cedar.build(&c);
            }
            let result: Vec<Pair> = cedar
                .common_prefix_predict(key)
                .unwrap_or(vec![])
                .iter()
                .map(|(idx, remaining)| {
                    let command = commands[*idx as usize];
                    let completion = &command[(command.len() - *remaining)..];
                    Pair {
                        display: completion.to_string(),
                        replacement: completion.to_string(),
                    }
                })
                .collect();
            Ok((pos, result))
        }
    }
}

impl Shell {
    pub fn new() -> Result<Shell> {
        let config = Config::builder()
            .auto_add_history(true)
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .build();
        let mut rl = Editor::<ShellHelper>::with_config(config);
        rl.set_helper(Some(ShellHelper));
        let mut shell = Shell { rl };
        shell.load_history().unwrap_or(());
        Ok(shell)
    }

    fn load_history(&mut self) -> Result<()> {
        self.rl
            .load_history(&Shell::histfile()?)
            .chain_err(|| "failed to load history")
    }

    fn save_history(&mut self) -> Result<()> {
        self.rl
            .save_history(&Shell::histfile()?)
            .chain_err(|| "failed to save history")
    }

    fn histfile() -> Result<PathBuf> {
        let mut history_file = app_dirs::app_root(AppDataType::UserCache, &APP_INFO)
            .chain_err(|| "cannot determine application directory")?;
        history_file.push("history");
        Ok(history_file)
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        self.save_history().unwrap_or_else(|_| {
            warn!("failed to save history");
        });
    }
}

impl Iterator for Shell {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.rl.readline(PROMPT) {
            Err(Interrupted) => self.next(),
            Err(Eof) => None,
            x => Some(x.chain_err(|| "failed to readline")),
        }
    }
}
