# hibana-mgmt

`hibana-mgmt` owns the ordinary management choreography prefixes and payload
owners that were moved out of `hibana` core.

It exports:

- `request_reply::PROGRAM`
- `observe_stream::PROGRAM`
- management request / reply payload owners

The crate depends on the public `hibana` and `hibana-epf` GitHub repositories
rather than filesystem path dependencies. Until crates.io releases exist for
the split repos, downstreams should use the same GitHub origin or a local Cargo
patch override instead of restoring path-based manifests.
