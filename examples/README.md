# Proxy Run Example

This example shows how to run the proxy and all dependencies including the operator, Prometheus, and Grafana.

## Setup

The setup file will execute some commands to prepare the environment. This script needs `kind`, `docker`, and `kubectl`. This example uses a mock API to simulate the Ogmios instance and only to Mainnet.

```sh
./setup
```

## Using

To use the proxy, it's necessary to create a port forward and to use Grafana as well. In Grafana, to connect Prometheus it's necessary to use this endpoint `http://prometheus`

For example, with a port forward created and using port 8080 it's possible to request the endpoint below.

**HTTP**
`http://dmtr_ogmios1faekjknsx3gnwarjt9e5c3t3xa2hgcmdxdmste4k6h.mainnet.ogmios-1.localhost:8080/headers`

**ws**
`http://dmtr_ogmios1faekjknsx3gnwarjt9e5c3t3xa2hgcmdxdmste4k6h.mainnet.ogmios-1.localhost:8080`

the proxy will call a mock API.

Port Keys available
- dmtr_ogmios1faekjknsx3gnwarjt9e5c3t3xa2hgcmdxdmste4k6h
- dmtr_ogmios124u5ga63wqc5u6je2f297azpvfvyxdtewpgslrdk79

