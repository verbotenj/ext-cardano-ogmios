# Ext Cardano Ogmios

The approach of this project is to allow a CRD to Ogmios on the K8S cluster and an operator will enable the required resources to expose an Ogmios port.

## Folder structure

* bootstrap: contains terraform resources
* operator: rust application integrated with the cluster
* scripts: useful scripts