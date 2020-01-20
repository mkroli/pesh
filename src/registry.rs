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
use std::sync::Mutex;

use prometheus::core::Collector;
use prometheus::{default_registry, GaugeVec};

use crate::commands::Metric;
use crate::errors::*;

lazy_static! {
    pub static ref REGISTRY: Mutex<DynamicRegistry> = { Mutex::new(DynamicRegistry::new()) };
}

pub struct DynamicRegistry {
    metrics: BTreeMap<String, (GaugeVec, Vec<String>)>,
}

impl DynamicRegistry {
    fn new() -> DynamicRegistry {
        let metrics = BTreeMap::new();
        DynamicRegistry { metrics }
    }

    pub fn add(&mut self, metric: &Metric, value: f64) -> Result<()> {
        match self.metrics.get(&metric.name) {
            Some((gauge, label_names)) => {
                let default_value = "".to_string();
                let mut label_values = Vec::<&str>::new();
                for key in label_names {
                    let value = metric.tags.get(key).unwrap_or(&default_value);
                    label_values.push(value);
                }
                gauge.with_label_values(&label_values).set(value);
            }
            None => {
                let opts = opts!(metric.name.clone(), "help".to_string());
                let labels = metric.tags.keys().cloned().collect();
                let label_names = metric
                    .tags
                    .keys()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>();
                let label_values = metric
                    .tags
                    .values()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>();
                let gauge = register_gauge_vec!(opts, &label_names)
                    .chain_err(|| "failed to register metric")?;
                gauge.with_label_values(&label_values).set(value);
                self.metrics.insert(metric.name.clone(), (gauge, labels));
            }
        }
        Ok(())
    }

    pub fn get(&self, metric: &Metric) -> Result<f64> {
        if let Some((gauge, label_names)) = self.metrics.get(&metric.name) {
            let default_value = "".to_string();
            let mut label_values = Vec::<&str>::new();
            for key in label_names {
                let value = metric.tags.get(key).unwrap_or(&default_value);
                label_values.push(value);
            }
            Ok(gauge.with_label_values(&label_values).get())
        } else {
            bail!("metric not found")
        }
    }

    pub fn remove(&mut self, metric: &Metric) -> Result<()> {
        if let Some((gauge, label_names)) = self.metrics.get(&metric.name) {
            let default_value = "".to_string();
            let mut label_values = Vec::<&str>::new();
            for key in label_names {
                let value = metric.tags.get(key).unwrap_or(&default_value);
                label_values.push(value);
            }
            gauge
                .remove_label_values(&label_values)
                .chain_err(|| "failed to register metric")?;

            if let Some(s) = gauge.collect().first() {
                if s.get_metric().is_empty() {
                    if let Some((gauge, _)) = self.metrics.remove(&metric.name) {
                        default_registry()
                            .unregister(Box::new(gauge))
                            .chain_err(|| "failed to unregister metric")?;
                    }
                }
            }
        }
        Ok(())
    }
}
