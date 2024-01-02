# Ext Cardano Ogmios Operator

This operator allow demeter to run and expose ogmios

## Environment

| Key           | Value         |
| ------------- | ------------- |
| ADDR          | 0.0.0.0:5000  |
| DNS_ZONE      | demeter.run   |
| NAMESPACE     | ftr-ogmios-v1 |
| INGRESS_CLASS | ogmios-v1     |
| API_KEY_SALT  | ogmios-salt   |

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
