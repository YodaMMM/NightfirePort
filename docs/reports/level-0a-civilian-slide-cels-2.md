# level-0a-civilian-slide-cels-2 — report

Branch `s/level-0a-civilian-slide-cels-2` = `origin/s/level-0a-civilian-slide-cels` d7e047a08
(origin/master already an ancestor at branch time — nothing to merge). The drawn gate itself
(the game's portal-visible room rule, `nf-game/src/celgraph.rs`, `Drones::tick_with_cel_graph`)
is the base branch's work and is UNCHANGED by this task: no `crates/*/src/` change landed
(two temporary env-gated diagnostics in drone.rs/player.rs used to trace the divergence were
reverted before commit; `git diff` on `crates/*/src/` is empty).

## Done — the drawn-gated arm found, the two 0c walks re-tuned green on real input

**The main agent's check is GREEN**: `walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side`
passes on this branch (whole binary: 1 passed). The whole-mission walk is green too:
`walkreal_0700000c_live` whole: **4 passed, 0 failed, 1 ignored** (the ignore is the
real-damage-agent sister's pre-existing ledger).

**Which drawn-gated arm moved Bond off the ledge — measured, then checked against the
decompile.** The method: temporary env-gated knobs built into the branch's drawn-gate site —
`NF_DIAG_OLD=1` forces the master bullet-ray gate for every drone (this run PASSES, so the
failure is purely in the gate's values), and `NF_DIAG_PORTAL_IDX=<ctl index>` gives the portal
rule to ONE control entry and the ray gate to the rest. The full run's per-tick trace
(NF_DIAG15) found 1243 drawn≠ray lines across 8 drones; the per-drone bisect:

- **plc=405 (ctl 32) alone fails the test; plc=404 (ctl 31) alone fails it.** The other six
  flipped drones are innocent (each alone: pass).
- First drone divergence at **t=1699**: with the portal rule the ring patrollers WALK their
  real danim gait; under the ray gate they were UNDRAWN and
  **`NDrone2_DoAnimation` 0x149f88's not-drawn arm** carried them at the fixed 0.02/frame
  **no-gait slide** (the port comment at drone.rs:3210-3215 cites the game: an undrawn drone's
  danim advances with no gait; the drawn patroller shows the walk's 115-tick gait) [M 0x149f88].
  The patrollers' changed patrol positions shift the fights downstream; the fights shift the
  #282 wire-ride landing to **(4.9831, −223.2948) — EXACTLY the real-damage tree's landing**
  [L the run; the plain run's was (4.9688, −223.3107)] — and the plain single long "the ledge
  to the arc" leg overshoots its arrival circle off the band's east edge into the measured
  fall (y 2.01 at (8.99, −222.09), +114 ticks; the wedge at the arc west end, bond
  (11.36, 1.13, −217.52), guard #15 in 0x5f at (11.36, 12.24, −218.92) above [L the run]).

**The game does the same, so the legs are re-tuned (not the port).** Both halves are the
game's own rules: the drawn set is `View_AddObjects` 0x1e62d8:34-71's VisCelList + world-sphere
`Vision_InView` pair [M] (the base branch's finding, unchanged), and the no-gait danim arm is
`NDrone2_DoAnimation` 0x149f88 [M]. A real player walking this route under the game's own
drawn rule meets the same walking patrollers. The re-tune is the 0c-2 ledger's own lever,
REAL INPUT ONLY, no writes, no teleports: the **existing opt-in `with_strip_rd_post_ride`
post-ride mid-ledge stops** (the reset_sidestep + the mid-ledge stop at (7.1,12.4,−222.42)
r0.5 + the arc west-end mid-stop at (10.5,12.4,−220.25) r0.8 — placed POST-ride so they cannot
re-time the ride), built by los-mask-repins-0c-3 for the real-damage tree's identical shifted
landing and identical fall. `walkreal_0700000c_eastroute_live` and the whole-mission walk's
prologue call now opt in (test-side changes only); `walkreal0c.rs` changed comment-only
(the flag's doc comment names both callers). The real_damage sister already opted in and
passes unchanged.

## What ran (whole binaries, this tree)

| binary | result |
|---|---|
| walkreal_0700000c_eastroute_live | **1 passed** (the fixed check) |
| walkreal_0700000c_live | **4 passed, 1 ignored** (the pre-existing agent ledger) |
| walkreal_0700000c_barrelrow_live | 1 passed |
| comms_chatter_0c_live | 1 passed |
| upward_shot_0700000c_live | 2 passed |
| walkreal_0700000a_live | 13 passed, 2 ignored (the pre-existing ignores) |
| walkreal_07000002_live | 3 passed, 2 ignored |
| walkreal_07000007_live | 6 passed |
| walkreal_07000005_live | 16 passed, **1 FAILED** (`..._real_damage_00agent` sister) |
| walkreal_07000008_live | 4 passed, 1 ignored, **1 FAILED** (`..._real_damage_agent` sister) |
| walkreal_07000013_live | 3 passed, **1 FAILED** (`..._real_damage` sister) |
| walkreal_07000001_live | 3 passed, 1 ignored, **4 FAILED** (`the_plate_refuses_bond_alone`, `the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads`, `walk_from_the_spawn_to_the_exit_with_the_contact` + its `_real_damage` sister) |
| guard1_rounds_0700000c_live | 1 passed, **1 FAILED** (`guard1_rounds_at_the_ring_real_health_fight`: bond (6.95,1.06,−210.35) fallen off the ring west edge, target (4.00,11.10,−207.50), health 32.2, no drone within 4.0 [L the dump]) |
| walkreal_07000011_live, walkreal_07000015_live | not counted — each exceeded my 290 s per-binary foreground cap (sandbox background jobs die between tool calls) |

Not run (window exhausted): 03, 04, 06, 09, 0b, 0d, 11_route, 12, 14, 16, 1b, 41, 41_wall,
4a, 4a_satchel, the probe binaries, `interactables_*`, `nf-game --lib`, and
`tools/test/suite.py` (it never started — the sandbox's full 9.9 GB rootfs had to be cleared
first (5.77 GB of stale same-hash workspace artifacts deleted; the recipes are in
docs/reports/level-14-kiko-corridor-2.md), then the merge's Cargo.lock needed a `cargo fetch`,
then the investigation consumed the window; background jobs die between tool calls here, so
the suite could not run unattended).

## What's left (the ledger for the next message)

The drawn gate's real-gait patrollers shift every level whose walks trade with visible
guards. The failures above are all one class — the fights' RNG draws shift, the walk routes
need the same re-tune treatment the 0c legs got. Next levers, in order:

1. **07000001** (4 failures): the exit/plate walks. Get the wedge dumps (`--exact --nocapture`),
   bisect the responsible patrollers with the same NF_DIAG_PORTAL_IDX method (the knobs are
   removed from this tree — re-add them temporarily, they are 15 lines in drone.rs's drawn
   gate), then re-tune the exit legs on real input.
2. **guard1_rounds_0700000c_live** `..._real_health_fight`: the real-health ring fight fell off
   the ring west edge at health 32.2 with NO drone within 4.0 — the fight ring shifted; the
   same bisect, then re-pin the fight stands.
3. **The agent sisters** (05 d3, 08 d2, 13 rd): each fight economy shifted; re-route or
   re-tune per the sisters' own ledger style.
4. Then the un-run binaries list above, and the fast runner on the final tree.

The drawn gate's three task-level readers are confirmed correct-by-the-game: auto aim
(Check_AutoAim 0x1a73f8 skips drawn==0 [M]), seek targets (Bullet_heatseekable 0x12f248 reads
+0xFC [M]), BondTalk (0x1446b0:8-31 [M]) — the walk shifts come from the patrollers' gait
changing who is where, not from a port bug in those readers.

## Live checks for the main agent

1. **The ring patrollers' drawn state, fresh boot 0700000c at Operative**: stand the player at
   the display eye (≈ (6.0, 7.4, −113.0)) facing south-east and read the two ring patrollers'
   `obj+0xFC` (placements **#405/#404**, the ring guards below the display ledge at ≈
   (13.5, 7.27, −133)) — expected **2** (drawn every frame: their cel is portal-visible and
   in-frustum [M 0x1e62d8:34-71]); then walk the player behind the band structure so the ray
   to them is blocked while the cel stays portal-visible — expected **still 2** (the game's
   pair has no rays; this is the whole drawn-gate change). Also read their `obj+0x20`
   (the linked cel) — expected the display/ring cel group (the port resolves cel 212 [L]).
2. **The gait difference, same fresh boot**: with #405/#404 drawn and walking (their state
   0x7 patrol walk), measure their per-frame displacement over ~60 frames (gdbwatch on
   obj+0x30) — expected the walk anim's real gait speed; then `SP_HideObj`-equivalent (or step
   the camera away until obj+0xFC decays to 0) and measure again — expected the fixed
   ≈ 0.02/frame no-gait slide [M 0x149f88's not-drawn arm]. This pins that the port's
   drawn-gated gait matches the game's.
3. **Guard #15 at the arc west end** (the main agent's original dump): with Bond fallen at
   (11.36, 1.13, −217.52), read #15's obj+0xFC — expected 0 (both gates agree he is undrawn
   there [L the port run: drawn=false old=false]), confirming the wedge is NOT #15's
   drawn-side behaviour (the fall is the shifted ride landing; #15 stands frozen above).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
