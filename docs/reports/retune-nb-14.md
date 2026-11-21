# retune-nb-14 — 14 infiltration to the delta exit, real-damage walks (Operative green, 00 Agent ledgered)

Branch `s/retune-nb-14`, cut from origin/master 73822d3a0 (merge with origin/master checked at the end: "Already up to date").

## What the task was

Commit 3412a89a3 (the spacing scan on every level + NDrone2_CanAltAttack from the fire states) made two real-damage walks of 07000014 die; each carried its measured death in the ignore message:

- `walk_the_infiltration_to_the_delta_exit_real_damage` (Operative, top-ups off): death t18133 at (-26.09,0.55,-10.56), "a 29.0 world blast".
- `walk_the_infiltration_to_the_delta_exit_real_damage_00agent` (00 Agent): death t47542 at (-48.91,1.13,34.27), drone #33 at 9.6 u.

## What changed (all in `crates/nf-view/tests/walkreal_07000014_live.rs`; no `crates/*/src/`, no `crates/nf-view/tests/common/`, no guard code, no damage/health values)

1. **Operative real-damage fight law** (`else if !w.tops_up()` beside the existing `difficulty >= 2` block): `w.dmg.back_off = Some(4.0)` + `w.dmg.finish_fights = true`. The nb-scan-all death: the south-loop GL ambush (drone #12 skin 0x5000087 post (-23.1,1.0,-14.5), #13 skin 0x5000090 post (-24.5,1.0,-10.7), both weapon 0x14) held Bond d 0.9–3.0 for 340 ticks taking 2.0/hit rounds plus their explosions (51.0 in 4 world-attributed blast hits, the 29.0 blast finished him) while the plain return-fire fight fired one window ("killed false") and the next stalled "cycling to a gun (0)". `back_off` walks the stand out of the under-3.0 band (every drone round hits there [M DroneWeap_DoBulletAccuracy 0x179740:113-115 — the 07000013 chain's passing law]) and `finish_fights` runs the stand to the kill. `lane_fights` was tried (v1) and measured WORSE: its resupply fetch walks displaced Bond into a 6000-tick wedge at the descent's node-57 leg, so it stays OFF for d1. The v0 fight had already wounded #12 with the plain aim — only the point-blank band was the killer.
2. **Operative wake legs** (new `else if !w.tops_up()` branch beside the d3 `round_dodge` branch): the shipped single `follow_nav_route` walk now walks the SAME route node-by-node (one `nav_route` read from `cur_pos()`, the same `r(1.3)` legs — the d3 branch's byte-exact note) with `delta_approach_scan` after node 21 (the d3 chain's own law: gun-ray gate, 0x1a carriers first, bounded 300) plus the d3 chain's loot pass keyed on the dry gun (`clip+spare < 10`) instead of a fixed node. Measured death answered: the wake route's 0x1a posts #28 (post (9.0,1.0,-22.5)) / #29 (post (8.7,1.0,-21.5)) dealt 65.2 at d 4.0 before the return-fire fights finished them (hp -81/-73), and the fight on #28 at d 15.8 burned clip 15 + spare 12 dry ("out of ammo at +572") so the UNARMED walk died at t36801.
3. **00 Agent switch-band stands**: `w.switch_to_gun` before the three stands, and the gun-ray gate (`DamageLog::visible_gun_rays`, the delta-approach scan's own law) on the stand pick. Measured: the fusebox cuts leave the Laser (0x4e) in hand; `switch_to_armed` reads its 500 beam charge as ammo (the Q-Decryptor note on `switch_to_gun`), so the v1/v2 stands fired 49 real rounds on #20/#32 EAST of the still-closed door plane and both read their start health after 600 ticks — ZERO rounds land through the closed door leaves' plane x≈-52.35 [L v2 run]. The gun-ray gate also stops the Kiko-corridor stand on #20 (eye-visible, muzzle blocked at d 6.6 — ZERO rounds in the 300-tick window while his GL blasted the stand).
4. Both ignore messages updated: the d1 one REMOVED (green), the d3 one now carries the v3 measured ledger and the next levers.

## Verification (each lever judged on its own real run; [L] = this sandbox's real-input runs)

- `walk_the_infiltration_to_the_delta_exit_real_damage` — **GREEN, un-ignored**: walked out in 41051 ticks, stuck 0, falls 0, the exit hold drained at (27.00,1.13,-24.85) and 07000015 loaded warm (arrival (94.50,41.37,0.00)); re-confirmed green after the final ignore edits (39.8 s, `1 passed; 0 failed`).
- Whole `walkreal_07000014_live` binary (10 tests): **9 passed, 0 failed, 1 ignored** (171.5 s) — the d3 sister is the ignored one, with its ledger in the message. Every other test in the file, including `walk_the_infiltration_to_the_delta_exit` (d1, topped-up), `..._real_damage_agent` (d2) and the tripbomb route, passed unchanged.
- `interactables_07000014_live`: **10 passed, 0 failed** (14.4 s).
- `walk_the_infiltration_to_the_delta_exit_real_damage_00agent` — still red, judged on four runs (v1 t47542, v2 same shape with the stands' zero-land measured, v3 death t44004 at (-59.40,0.72,38.69) at 7.5 health). The measured ledger, what the shipped levers fixed (the v3 run killed #33 at the door leg clean and preserved the fights' ammo) and the next levers are in the ignore message + the test's doc comment.

## The fast full suite: what happened

`tools/test/suite.py` was started per the driver's instruction, but this sandbox killed background jobs between tool calls and then spent the window on the build phase: four `cargo test --no-run -j4` windows (10 min each) still had nf-view's ~25 test binaries linking when the clock ran out, with two disk-full events (9.9 GB volume; the linker's debug-info binaries plus regenerated build-script outputs filled it twice; both were cleared). At the hard stop only 3 small binaries of the 52 were linked and run suite-style: **6 passed, 0 failed, 0 ignored** (`bank_live` 1, `fix_sound_remainder_live` 4, `music_live` 1). The full-suite total line is therefore NOT available for this tree — that is a sandbox-throughput gap, not a red result. The task's required binaries were run whole and green (numbers above). Nothing under `crates/*/src/` or `crates/nf-view/tests/common/` was touched, so the `walkreal_*_live` cross-level re-run obligation (lessons 7/10) does not apply beyond the 07000014 file, which was run whole.

## What's left for the next message

1. The d3 sister: implement the three levers in the ignore message (the d3-gated descent-corridor scan at the leg-2 spot; a d3-gated bounded-stand pick in `clear_hostiles` (the loose aim sprays at d 10-12); the door-leg scan AFTER the door's `pos01 > 0.5` assert). The v3 run's bleed concentrates in the descent corridor's four guards (55.0 before their fights finished them).
2. The full suite on the final tree (the build phase fits a normal sandbox; here it did not).
3. The d3 evidence ledger cites: #56 15.0/3, #1 15.0/1, #9 12.5/5, #11 12.5/5, #2 7.5/3 (descent corridor), #33 3 x 2.50 at d 8.7, #34 (0x5000093, 0x14, hp 14) at the door plane.

## Live checks for the main agent

None requested this round; the measured runs are all walker-side and the addresses cited are the chain's existing [M]s (0x179740 accuracy bands, the door leaves' plane from the west-route doc block).

## Usage

Not visible to me — this session exposes no token, cost or context-window figures to the worker; nothing to copy.

## ntfy failed

All three fallbacks failed (2026-11-18 ~19:47 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "retune-nb-14: ..." https://ntfy.sh/nightfire-yoda-8k3x` → `curl: (28) Connection timed out after 20001 milliseconds` (exit 28).
2. python3 urllib fallback → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>` (socket.connect raise, after `curl` also timed out at connect).
3. `wget -q -O- --post-data "retune-nb-14: ..." https://ntfy.sh/nightfire-yoda-8k3x` → killed after 30 s with no output (connection never established).

The git pushes (branch + queue) all succeeded; only the ntfy.sh notification endpoint was unreachable from this sandbox.
