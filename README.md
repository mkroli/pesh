# pesh

**p**rometheus **e**xporter **sh**ell starts a prometheus exporter along with an interactive shell to manually export metrics to be scraped by prometheus for testing purposes (e.g. alerts).

## Installation
```sh
cargo install --git https://github.com/mkroli/pesh.git
```

## Usage
```
pesh> help
Available commands:
 - quit|exit                - quit shell
 - help                     - display this help message
 - set <metric> = <value>   - sets a metrics value
 - del <metric>             - deletes a metric
 - get <metric>             - prints the current value of the given metric

<metric> is defined as follows:
name | name[key="value", ...]
```

## Example
```
$ pesh
pesh> set test[node="test1.example.com"] = 0.95
pesh> set test[node="test2.example.com"] = 0.85
pesh> set nodes = 2

$ curl localhost:9000/metrics
# HELP nodes help
# TYPE nodes gauge
nodes 2
# HELP test help
# TYPE test gauge
test{node="test1.example.com"} 0.95
test{node="test2.example.com"} 0.85
```
