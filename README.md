# hibana-mgmt

`hibana-mgmt` owns the ordinary management choreography prefixes and payload
owners that were moved out of `hibana` core.

It exports:

- `request_reply::PROGRAM`
- `observe_stream::PROGRAM`
- management request / reply payload owners

The crate depends on the public `hibana` and `hibana-epf` GitHub repositories
at immutable revisions rather than filesystem path dependencies. Downstreams
should consume the same immutable GitHub revision boundary or coordinated
release tags, not restore path-based manifests.
