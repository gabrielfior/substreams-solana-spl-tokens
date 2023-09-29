# Substreams Solana SPL Token

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)


## Requirements

Follow [Installation Requirements](https://substreams.streamingfast.io/developer-guide/installation-requirements#local-installation) instructions on official Substreams documentation website.

## Building the spkg

* `make build` will compile the substreams rust code
* `make package` will compile the substreams rust code and package it into an spkg file


## Running the Substreams


```
make stream
```


## Running other output modules

* `db_out` will output the blockmeta data in tables with columns, to be saved in a database. See some integrations in https://github.com/streamingfast/substreams-sink-postgres and https://github.com/streamingfast/substreams-sink-mongodb
* `kv_out` will output the blockmeta data in a format to be saved in a key/value store.  See its integration in https://github.com/streamingfast/substreams-sink-kv (note: this module outputs one entry per blockHash instead of per day/month).
