# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## unread-params-props

2026-11-07 10:36 UTC, branch `s/unread-params-props` (from origin/master 10956badb).

Task: sibling sweep to unread-params-switches over the prop classes — pickups, breakables,
gun mounts, hazards, placed effects and lights (crates/nf-game/src/pickup.rs, breakable.rs,
gunimp.rs, effect.rs, light.rs). Read each class's `*_Create` in the decomp, list every
placement param/word it stores, and check what the port does with each. Port-ignored params
get a census of non-zero values on EVERY level (`Level::parse` + `nf_view::dev::LEVELS`);
zero everywhere = closed with the census as proof; non-zero on any on-foot level = a real
gap: trace the reader in the game, port it, test on the real level, red on the old code.
Timing from fresh-load readings, not symbol names. Census table in
docs/reports/unread-params-props.md.
