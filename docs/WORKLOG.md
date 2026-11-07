# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## unread-params-switches

2026-11-07 09:18 UTC, branch `s/unread-params-switches` (from origin/master 10956badb).

Task: for every switch/trigger/sensor/touch/channel-logic class created by
`parsemap_create_dynamic_objects` (crates/nf-game/src/useobj.rs), read its `*_Create` in the
decomp, list every placement param/word it stores, and check what the port does with each. Where
the port ignores one, census non-zero values on EVERY level (`Level::parse` + `nf_view::dev::LEVELS`);
zero everywhere = closed with the census as proof; non-zero on any on-foot level = a real gap:
trace the reader, port it, test on the real level, red on the old code. Timing from fresh-load
readings, not symbol names (the Destroy p6 scale is 30 per p6, not FRAME_RATE). Census table in
docs/reports/unread-params-switches.md.
