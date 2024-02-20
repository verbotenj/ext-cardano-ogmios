# Ext Cardano Ogmios Proxy

The proxy will manage the connection to the Ogmios, when a user makes a request, the proxy will decide which Ogmios instance will be requested using the hostname.

An example about how the proxy will decide which instance will be requested.

| Host                         | Instance         |
| ---------------------------- | ---------------- |
| mainnet.ogmios-1.demeter.run | ogmios-mainnet-1 |


The proxy exposes metrics about HTTP requests and WebSocket frames.

## Environment

| Key             | Value          |
| --------------- | -------------- |
| PROXY_ADDR      | "0.0.0.0:8100" |
| PROMETHEUS_ADDR | "0.0.0.0:5000" |
| OGMIOS_PORT     | -              |


## Commands

Execute the proxy

```bash
cargo run
```

## Metrics

to collect metrics for Prometheus, an HTTP API will enable the route /metrics.

```
/metrics
```
