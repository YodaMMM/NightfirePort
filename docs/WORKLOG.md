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

2026-11-07 10:35 UTC — the sweep is complete: every *_Create store for the 0x1d04f0 switch /
trigger / sensor / touch / mux / sound / music / inverter / lock classes checked against the
port, and every unread param censused on all 34 levels. One real gap: Switch param 6 — the
SSysItems slot arm (Switch_Create 0x1adcc0:39-41), non-zero on 07000009/0a/046, with the whole
crate::ssys machine dead code (no caller). Wired: the arm at load, SSys_Monitor ticked from the
Game_Run pass under the ScriptCam gate, raw channel writes, the msg sounds; monitor returns the
writes collected. Closed by census: sensor p5, lock p2, the mux p7..p11, switch p3 (the
init_channels rest claim holds on every level). Pin: ssys_switch_arm_live on 07000046 (the 240 s
countdown, the 0x2a2 arm sound, the expiry channel-off) — red on the base. Runs: nf-game lib
688/0/5ign, walkreal_07000009 6/0, walkreal_0700000a 6/0/1ign, npc_doors 8/0, fix_guard_misc
13/0/2ign; the chunked suite's 3 failures reproduce on a clean master worktree (pre-existing
draw tests).
