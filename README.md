# Ext Cardano Ogmios

This project allow demeter to run and expose ogmios

## Environment

| Key  | Value        |
| ---- | ------------ |
| ADDR | 0.0.0.0:5000 |


## Commands

To generate the CRD will need to execute crdgen

```bash
cargo run --bin=crdgen
```

and execute the controller

```bash
cargo run
```

## Metrics

to collect metrics for Prometheus, an http api will enable with the route /metrics.

```
/metrics
```
