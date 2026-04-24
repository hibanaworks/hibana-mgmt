# hibana-mgmt

`hibana-mgmt` owns the ordinary management choreography prefixes and payload
owners that were moved out of `hibana` core.

It exports:

- `request_reply::attach_controller(...)` / `attach_cluster(...)`
- `observe_stream::attach_controller(...)` / `attach_cluster(...)`
- management request / reply payload owners

The crate depends on the sibling `hibana` and `hibana-epf` checkouts through
explicit local path dependencies. Coordinated development runs against the
current worktrees with no separate git-rev lane and no repo-local patch shim.
