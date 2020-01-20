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

use std::thread::spawn;
use std::time::Duration;

use prometheus_exporter::{FinishedUpdate, PrometheusExporter};

use crate::errors::*;

pub fn setup_prometheus(addr: &str) -> Result<()> {
    let prometheus_addr = addr.parse().chain_err(|| "failed to parse address")?;
    let (rcv, snd) = PrometheusExporter::run_and_repeat(prometheus_addr, Duration::from_secs(1));
    spawn(move || {
        for _ in rcv {
            snd.send(FinishedUpdate {}).unwrap()
        }
    });
    Ok(())
}
