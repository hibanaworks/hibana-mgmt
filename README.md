# hibana-mgmt

`hibana-mgmt` owns the ordinary management choreography prefixes and payload
owners that were moved out of `hibana` core.

It exports:

- `request_reply::attach_controller(...)` / `attach_cluster(...)`
- `observe_stream::attach_controller(...)` / `attach_cluster(...)`
- management request / reply payload owners

The crate depends on the public `hibana` and `hibana-epf` GitHub repositories
at immutable revisions rather than filesystem path dependencies. Downstreams
should consume the same immutable GitHub revision boundary or coordinated
release tags, not restore path-based manifests.

For local sibling development, this repository keeps the checkout overlay in
repo-local `.cargo/config.toml` so the published manifest can stay publicly
resolvable while contributors still test against `../hibana` and `../hibana-epf`.
