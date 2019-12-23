# URL Proxy

Simple HTTP proxy that supports injecting basic auth credentials into backend requests.

## Build

`cargo build`

## Run

For help and all supported options: `urlproxy --help`

E.g.,

`RUST_LOG=debug urlproxy --username my@user.name --password myapitoken https://mycompany.atlassian.net`
