# hibana-mgmt

`hibana-mgmt` owns the ordinary management choreography prefixes and payload
owners that were moved out of `hibana` core.

It exports:

- `request_reply::attach_controller(...)` / `attach_cluster(...)`
- `observe_stream::attach_controller(...)` / `attach_cluster(...)`
- management request / reply payload owners

The crate's default manifest lane depends on immutable `hibana` and
`hibana-epf` GitHub revs. Coordinated local-worktree validation belongs to the
dedicated `hibana-cross-repo` workspace smoke runner, which applies explicit CLI
patch overlays for the sibling checkouts.
