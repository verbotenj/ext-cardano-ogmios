#!/bin/bash
cd ../operator
cargo run --bin crdgen | tfk8s > ../bootstrap/crds/main.tf

