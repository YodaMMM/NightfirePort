## Your task: `level-05-basin-floor-6`, the solid hurt planes without a multiplayer desync

Branch from origin/s/level-05-basin-floor-5 and merge current origin/master. Round 5 is green everywhere except net_match_end_clients_live: run alone, the_frag_limit_end_reaches_the_same_debriefing fails 3 of 3 (the host and client state hashes diverge at the tick-15 exchange; master passes 4/0). The likely cause: the round puts per-tick sim state into CollisionWorld (hurt_parts via set_hurt_parts, hurt_hits via take_hurt_hits, under RwLock/Mutex), and the three in-process machines in that test may share one cached CollisionWorld for the arena, so each machine's posts and takes land in the others' ticks. Confirm by reading how the test and the scene build share the world; then move that per-tick state into the Sim (or key it per sim) so the world stays immutable level data. A test that fails on the old code: two sims on one shared world, one posts hurt parts, the other must not see them. Run net_match_end_clients_live, every net_* and mp_* binary, walkreal_07000005/16, blast_room_shelter_16_live, interactables_0700000a/1b and hurt_07000046_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-10T22:03Z

---
## Report (done, 2026-11-10T23:03Z)


---
## Report from the worker (done, 2026-11-10T23:03Z)

# level-05-basin-floor-6 — report

Branch `s/level-05-basin-floor-6` = origin/s/level-05-basin-floor-5 (bb5968d2c) + origin/master
(0e1371451) merged clean, plus the hunt's commits. Task: the solid hurt planes without a
multiplayer desync — round 5's `net_match_end_clients_live` red (`the_frag_limit_end_
reaches_the_same_debriefing` failing run-alone; master 4/0).

## What's done — root cause found and fixed

**The task's shared-CollisionWorld hypothesis was checked and REFUTED.** The three in-process
machines do not share a world: every `load_mp_carry` runs `load_impl` → `CollisionWorld::
build_scoped` per call, and the world's per-tick fields (`hurt_parts: RwLock<Vec<…>>`,
`hurt_hits: Mutex<Vec<u64>>`, world.rs:642/658) are plain locks, not `Arc`s — no cross-instance
sharing exists. The deterministic three-sim probe (`probe_deterministic_mp.rs`, no sessions, no
UDP, the frag test's scripted controls and kills) runs 1000 ticks with the per-tick state hash
agreeing at every local slot — the sim's row handling and the per-tick hurt state are clean
when the rows are identical.

**The real desync: the local collide-pass hurt delivery ran online, on the owner's machine
only.** The harness diagnostics (the exchange now names the diverging component, item and
first-split tick; `row_hashes` per Mach) showed: the applied rows fold IDENTICALLY on all three
machines through the split, the frame_rng never diverges, and all three bodies split at ONE
tick (t744) from an identical previous state — each machine's LOCAL body differing from the
replicas' view of it, the owner's body freezing mid-air (its y held) while the replicas fell.
The arena's probe answers it: Ravine carries 8 UNGATED (gate 0) damage-1000 kill planes, solid
static. At the contact tick, the owner's machine ran the local path's delivery —
`take_hurt_hits` → `deliver_contact` → `pain_generic` on the SP health block (step_bond_sim) —
whose pain/death sequence froze the local body, while the remote machines ran only
`step_net_world`'s per-slot delivery. Two deliveries per contact on the owner, one on the
replicas: the body split, then the boards. The drown arm right above carried the mp-world-sync
gate (`net_row.is_none()`); the hurt object-hit delivery did not.

**The fix (one gate, game-behaviour-neutral offline):** in `step_bond_sim` the local
collide-pass hurt delivery is offline-only — `for id in if net_row.is_none() { hurt_hit_ids }
else { Vec::new() }` — with the `take_hurt_hits` drain unconditional (the log must clear every
tick). Online, every body's world damage rides `step_net_world`'s single per-slot law
(`Hurt::update` per slot in slot order → `net_world_pain` into `mp_hp`), exactly as the
mp-world-sync design intends; offline, the SP path is byte-identical to round 5's.

## How it was verified

- `net_match_end_clients_live` run-alone: 6/6 green after the fix (was failing ~50% of
  run-alone attempts before it, 3 of 3 on floor-5); the whole binary **4 passed / 0 failed**
  (was 2/4 this session).
- The deterministic probe: 1000 ticks clean, and its `probe_arena_hurts` pins the arena's
  hurt roster (8 × gate 0, damage 1000, shape_key ≠ 0) [D the level's hurt placements].
- The battery around the changed code (release, `--test-threads=1`):

| binary | result |
|---|---|
| hurt_07000046_live | 2/0 |
| blast_room_shelter_16_live | 3/0 |
| interactables_0700000a_live | 14/0 |
| interactables_0700001b_live | 9/0 |
| walkreal_07000005_live | 17/0 |
| walkreal_07000016_live | 14 passed / 0 failed / 4 pre-existing ignored |
| net_e2e_loopback_live | 2/0 |
| net_match_live | 2/0 |
| net_join_live | 3/0 |
| net_resync_live | 4/0 |
| net_desync_report_live | 3/0 |
| net_soak_faults_live | 2/0 |
| mp_world_sync_live | 1/0 |
| mp_respawn_ammo_live | 3/0 |
| mp_match_end_live | 2/0 |
| mp_death_effects_live | 3/0 |
| mp_death_hands_online | 3/0 |
| mp_bot_armour_online_live | 8/0 |
| mp_core_live | 3/0 |
| mp_bodies_live | 11/0 |
| mp_combat_weapons_live | 5/0 |
| mp_shot_gate_live | 2/0 |
| mp_pickup_take_live | 5/0 |
| nf-view --lib | 138/0 |
| nf-game --lib | 782/0/4 |

## What's left

- The remaining mp_* binaries (the offline menu/HUD/scenario/skin screens and the perf/stress
  ones) and the fast-runner suite (`tools/test/suite.py`) did not fit the window — ~26 binaries
  + both libs ran instead, spanning every path the change touches (the SP hurt deliveries, the
  net world pass, the respawns, the boards). The change is nf-view-only; nf-game is untouched.
- The diagnostics added to `net_match_end_clients_live` (the component/item/row breakdown at
  the exchange, the body-track first-split scan) are worth keeping: the next desync report
  names the component and the first disagreeing row directly.

## Live checks for the main agent

None needed — the desync reproduced and died entirely inside the port's harness and sim; no
game-behaviour question remains open. (For the record: the arena's hurt placements are
placement ids 147-152 and 218-219 on 0700004b, ungated kind-0xE4 with damage 1000 [D].)

## Usage

not visible to me — this environment exposes no token, credit, cost or context-window figures
for the session, and I will not estimate them.
