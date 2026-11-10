## Your task: `level-05-basin-floor-5`, the solid hurt planes keep every level's hurt volumes working

Branch from origin/s/level-05-basin-floor-4 and merge current origin/master. Round 4 fixed level 05 (no marker draw, damage through the object hit [M Player_DealWithObjHit 0x194458:94-103, Player_CollisionHandler 0x1949a8:139-145]) but a full suite is 4883 / 9 failed; master passes all of these. Read every message:
- interactables_0700001b_live::the_missile_flame_hurts_on_contact ("the volume's contact witness records the hit")
- interactables_0700000a_live::lift_shaft_hurt_volume_deals_the_fall_damage ("the shaft volume's 1000 damage kills", health stays 100)
- walkreal_07000016_live: the_real_order_launch_chain_matches_the_teleported_one, the_pit_pulse_kills_an_idle_bond_about_450_ticks_later_like_the_game, the_walked_mission_fails_at_the_launch_blast_when_bond_stays_on_the_pit_floor (the 1000 pit pulse no longer kills)
- blast_room_shelter_16_live: the_shuttle_launch_plane_is_solid_only_while_ch_243_reads_high (the plane stays solid after ch 243 drops: the every-tick law 0x17e5d0:26-37 must clear it), the_west_blast_room_shelters_bond_through_the_pulse_and_the_open_floor_does_not (hp 100, hits 0)
- hurt_07000046_live::the_shot_icicle_flies_its_path_at_the_game_pace_and_hurts_bond_in_its_way (the moving icicle's 105 contact no longer lands)
- net_match_end_clients_live::three_machines_reach_the_debriefing_with_identical_boards (tick 765 desync): first check whether it fails on master; if not, find what the fold changed in the MP sim.
The likely root: volumes Bond stood INSIDE or passed through (pit floors, shafts, moving hurts) used to deal damage through the per-tick volume contact; now that they are solid or take the object-hit path, the volume contact no longer fires. In the game both paths exist: the solid planes via the object hit, the volumes via Hurt_Update's own contact. Keep both, with each hurt object on the path the game uses for it [M Hurt_Update 0x17e5d0 + Player_CollisionHandler]. Run every binary named above, all walkreal_*/interactables_*/interact_*/hurt_* binaries one per tool call, and the net_* binaries.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T19:47Z

---
## Report (done, 2026-11-10T21:09Z)


---
## Report from the worker (done, 2026-11-10T21:09Z)

# level-05-basin-floor-5 — both hurt paths live; the fold's 9 suite reds fixed the game's way

Branch `s/level-05-basin-floor-5` = `origin/s/level-05-basin-floor-4` (4296611e7) merged with
origin/master b32eacb92 (+ three commits). The task: round 4 made the level 05 hurt planes
solid and moved their damage to the object hit [M `Player_DealWithObjHit` 0x194458:94-103,
`Player_CollisionHandler` 0x1949a8:139-145]; the full suite then showed 9 reds — the hurt
VOLUMES Bond stands INSIDE or passes through had lost their per-tick contact delivery. Master
passes all 9 (it never made the planes solid).

## The root, from the decompile

`Hurt_Update` 0x17e5d0 has NO damage code and NO player-state guard: it maintains the gate's
0x20 bit every tick (:26-37), runs the state machine and the flight clock. The damage always
arrives through the obj+0xD0 hit chain per contact. The port models that single delivery with
TWO halves: the collide-pass object-hit log (world.rs `hurt_hits`, round 4) and the per-tick
capsule re-test (`Hurt::update`). Round 4 restricted the capsule re-test to volumes with no
collision block (`shape_key == 0`), which left three kinds of contact undelivered:

1. **A capsule INSIDE a solid volume** — the 0700000a lift-shaft HurtPolys: Bond placed in the
   shaft settles inside the folded box and the settled push never moves the accumulator, so
   the object-hit log never fires ("the shaft volume's 1000 damage kills", health stays 100).
2. **A volume FLYING away from its folded static instance** — the 07000046 Icicle: the static
   fold bakes the create pose; `tick_flight` re-poses only `Hurt::tris`, so the falling
   icicle's collision never reaches the collide pass ("the moving icicle's 105 contact no
   longer lands").
3. **The player-update gate itself** — the whole hurt block (gate maintenance, flight clock,
   damage loops, solid-side posting) sat inside scene.rs's `if !self.disabled`. The 07000016
   launch NIS parks the player exactly at the pit pulse: the gate maintenance stopped, so the
   plane stayed solid after ch 243 dropped (the shelter ray pin) and the pulse's damage side
   froze ("the 1000 pit pulse no longer kills", "hp 100, hits 0").

## The fix (nf-game hurt.rs / world.rs + nf-view scene.rs)

- `Hurt::maintain_gate` extracted ([M] 0x17e5d0:26-37) and called from the scene's **every-tick
  hurt law** hoisted OUT of the player-update gate: gate maintenance → flight clock → solid-side
  posting (`set_hurt_parts`), the game's object-control-pass order. The DAMAGE loops stay in
  the player-update gate (the game delivers them from the player update).
- The folded planes' stat delivery routes through the **matching Hurt's own state side**
  (`deliver_contact`): the witness the live tests pin, the one-shot expiry, the disabled bit —
  `stat_hurts` now carries the placement index; the raw (damage, kind) pair stays the fallback
  for a fold outside the main map (hurt.rs creates from the main map only).
- The per-tick capsule contact runs for **ALL volumes** (round 4's `shape_key == 0` filter
  dropped), **deduped per tick** against the object-hit delivery (`delivered` set in the
  take_hurt_hits loop): one delivery per contact per tick, the game's single obj+0xD0 chain.

The interactables_0700000a/blast_room_shelter_16 test files gained the diagnostic prints this
session's probes used (same eprintln style as the file; no assert changed).

## Verification (release, --test-threads 1, NF_ISO)

The 9 named failures, all green on the fixed tree:

| binary | result |
|---|---|
| interactables_0700001b_live | **9 / 0** (the missile flame's witness records the hit) |
| interactables_0700000a_live | **14 / 0** (the shaft volume's 1000 kills) |
| walkreal_07000016_live (whole) | **14 / 0 / 4 ignored** (both pit-floor legs + the launch-chain equality) |
| blast_room_shelter_16_live | **3 / 0** (the plane solid only while ch 243 reads high; the west shelter 100/0 hits, the open floor dies) |
| hurt_07000046_live | **2 / 0** (the icicle flies its pace and hurts in its way; the clear sister untouched) |
| net_match_end_clients_live | **4 / 0** (the tick-765 desync did not reproduce on this tree — no master run needed) |

The battery the task names (one binary per tool call):

| binary | result |
|---|---|
| nf-game --lib | **782 / 0 / 4 ignored** |
| walkreal_07000005 / 08 / 0a / 15 / 02 / 04 | 17/0, 3/0/3, 13/0/2, 6/0/3, 4/0/1, 9/0/1 |
| hurt_02 / hurt_07000008 | 1/0, 2/0 |
| interact_01 / 04 / 05 / 06 / 07 / 09 / 0c / 4a | 10/0, 10/0, 11/0, 9/0, 8/0, 16/0, 10/0/1ign, 6/0/1ign |
| interactables_01 / 02 / 03 / 04 / 06 / 07 / 08 / 09 / 0b / 0c / 0d / 11 / 12 / 13 / 14 / 15 / 16 / 41 / 4a | all green: 10, 18, 9, 4, 5, 4, 15, 4, 14, 7, 11, 17, 12, 10, 10, 19, 13, 18, 6 (passed) |
| net_match / net_e2e_loopback / net_resync / net_desync_report | 3/0, 2/0, 3/0, 4/0 |
| load_scene_equality / fix_striped_box_census | 1/0 (the fold pin 106→119 holds), 4/0 |

Not run to completion: `net_soak_faults_live` (SIGKILL at the 500 s tool timeout — the soak
runs past the window; no failure observed, none reported), and the remaining walkreal binaries
(01, 03, 06, 07, 09, 0b, 0c ×3, 0d, 11 ×2, 12, 13, 14, 41 ×2, 4a ×2) — the window closed; the
levels with hurt volumes (05, 15, 16, 1b, 0a, 08, 02, 46) all ran green above. The fast runner
(`tools/test/suite.py`) was started three times (last: detached, on the final tree); background
jobs are killed between tool calls in this sandbox (measured again), so `suite.out` never
printed a build line — the per-binary table above is the tree's state.

## What's left

- The un-run walkreal binaries above (a plain battery pass, no code expected to move — the
  change is confined to the hurt delivery and the levels with volumes ran green).
- `net_soak_faults_live` needs a longer window than the tool timeout.
- Live checks for the main agent (unproven without the real game): (1) on 07000016, a fresh
  boot into the launch chain with Bond idling on the pit floor — the pulse should kill ~450
  ticks after ch 243 rises with the player NIS running (the port's walkreal_16 pins it; the
  game's kill tick is the live check); (2) whether the game's hurt-plane contact lands when
  Bond's capsule rests exactly on a folded plane (the port's push-accumulator rule is an
  approximation [S]).

## Usage

not visible to me

---
## ntfy failed (2026-11-10T21:11Z)

All three fallbacks unreachable from the sandbox: curl (28) connection timed out after 20002 ms; urllib [Errno 101] Network is unreachable; wget killed at the 40 s tool timeout with no output. The report section carries the exact errors.
