# ping-exporter
[![Build Status](https://travis-ci.org/knsd/ping-exporter.svg?branch=master)](https://travis-ci.org/knsd/ping-exporter)

## Building and running

### Local Build

    make
    ./ping-exporter

Visiting [http://localhost:9346/ping?target=google.com](http://localhost:9346/ping?target=google.com) will return metrics for an ICMP ping against google.com.

### Building with Docker

    docker build -t ping-exporter .
    docker run -d -p 9346:9346 --name=ping-exporter ping-exporter

## Settings

| Environment variable                   | Default Value |
| -------------------------------------- | ------------- |
| PING_EXPORTER_LISTEN                   | [::]:9346     |
| PING_EXPORTER_DEFAULT_PROTOCOL         | v4            |
| PING_EXPORTER_RESOLVER                 | system        |
| PING_EXPORTER_DEFAULT_COUNT            | 5             |
| PING_EXPORTER_MAX_COUNT                | 30            |
| PING_EXPORTER_DEFAULT_PING_TIMEOUT     | 1000          |
| PING_EXPORTER_MAX_PING_TIMEOUT         | 10000         |
| PING_EXPORTER_DEFAULT_RESOLVE_TIMEOUT  | 1000          |
| PING_EXPORTER_MAX_RESOLVE_TIMEOUT      | 10000         |

## Available metrics

### `/ping` endpoint

| Metric name          | Type      | Description                                                                                    |
| -------------------- | --------- | ---------------------------------------------------------------------------------------------- |
| ping_resolve_error   | gauge     | Boolean metric if there's an error during the resolve (error message will be in "error" label) |
| ping_resolve_time    | gauge     | Time it take to resolve domain to an IP address                                                |
| ping_packets_total   | gauge     | Total number of sent pings                                                                     |
| ping_packets_success | gauge     | Total number of success pings                                                                  |
| ping_packets_failed  | gauge     | Total number of failed pings                                                                   |
| ping_packets_loss    | gauge     | A percentage of failed pings from the total pings                                              |
| ping_times           | histogram | A histogram of round-trip times                                                                |

### `/metrics` endpoint

| Metric name | Type    | Description                          |
| ----------- | ------- | ------------------------------------ |
| http_ping   | counter | Number of requests to /ping endpoint |

## Prometheus Configuration

This exporter needs to be passed the target as a parameter, this can be done with relabelling.

Example config:
```yml
scrape_configs:
  - job_name: 'ping'
    metrics_path: /ping
    static_configs:
      - targets:
        - google.com # Target to ping
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        replacement: 127.0.0.1:9346 # This exporter's real hostname:port
```

In addition to that you can scrape the /metrics endpoint to be able to monitor exporter's own statistics.

Example config:
```yml
scrape_configs:
  - job_name: 'ping_exporter'
    static_configs:
      - targets:
        - 127.0.0.1:9346 # This exporter's real hostname:port
```
