## Your task: `real-health-00agent-01-hall-4`, 07000001's 00 Agent real-damage walk gets past the hall door phase

Branch from origin/s/real-health-00agent-01-hall-3 (its ledger is not on master yet) and read docs/reports/real-health-00agent-01-hall-3.md there, "What's left" and the next lever it names: the advance dance with the hard exchange budget. Land that lever, measure it on its own real run, then carry the walk through the AND gate and the rest of its route (the post-AND route is untouched and was green through the AND on the older streams). The report's live notes say the door-phase crowd reacted to a standing Bond through the aperture but not to a crouched one; use that only as a lead and confirm it in the port's own runs. Keep the ledger in the test's ignore message. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the ledger and the next lever. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-13T11:00Z

---
## Report (done, 2026-11-14T01:38Z)


---
## Report from the worker (done, 2026-11-14T01:38Z)

# real-health-00agent-01-hall-4

Branch `s/real-health-00agent-01-hall-4` (cut from `origin/s/real-health-00agent-01-hall-3`
0a05db489). The task: land hall-3's named next lever — the advance dance with the hard
exchange budget — measure it on its own real run, then carry the 00 Agent walk through the
AND gate and the rest of its route.

## What's done

The lever landed and moved the wall ~6300 ticks east: the door phase is now WON (the AND
fires) and the walk reaches the escort at 21.4 hp, where hall-3's tree died in the door
lane at 55.0. The 00 Agent walk is still red; the ledger + the next lever are pushed in the
d3 test's ignore message, and every change is d3-gated (d1/d2 byte-identical — their tests
pass unchanged).

The lever set (each piece earns its place by a measured death it removes):

1. **Kill-ends-leg** — the opt-in `Walker::leg_abort_on_kill: Option<Vec<usize>>` makes a
   nav leg return early (u32::MAX−1, not the wedge marker) the first loop head after a
   listed guard dies. Baseline: the guards ALIVE at the leg's start (the first version
   aborted every later leg on g6's corpse and the dance stalled in place [L the run2
   trace]). This removes the measured 7217→7301 window: the free kill no longer rides into
   the captain's 6-pellet 40.6 frame that lands 84 ticks later while the leg keeps walking.
2. **The hard exchange budget** — `d3_advance_dance` never presses into the lane from
   below 45.0 = the captain's worst measured frame (40.6 in ONE tick — the 6-pellet 7301
   burst lands inside the tick before any loop-head abort can run [L the dance5 run]) +
   the advancer's volley (3.6 [L the stance-fight run]). The hall-3 gate (≤15) rode exactly
   the lethal-frame death: pressed at 40.6, the frame landed 40.6, dead.
3. **The determined pass OFF inside the legs** — the pass's `shoot_bounded` NAV walks Bond
   to the guard and ignores `flank_hold_z`; the run3 death (11644) was the pass flanking
   Bond deep into the hall (z 19.4) for the g11 kill, cost 19.2 there, then the retreat
   from that depth ate the captain's frames at the aperture.
4. **The crouched retreat** — the opt-in `Walker::leg_crouch` runs the dance's home legs
   crouched. Two measured facts: the peek runs' law (crouched Bond in the lane took 0 fire
   across 24 windows), and the run1 death (the home walk's own return fire HOLDS Bond in
   the lane while the shooter shows no exposed part — he died at the leg-end position,
   never having moved; the captain's frame landed 55 ticks after the kill).
5. **The d3 detour-only dry resupply** — the press legs put the hall's INTERIOR on the
   walked-path trail, so `resupply()`'s nearest-on-trail rule started admitting the hall's
   own deep box (3.6,17.8); the run8/run10 deaths were the dry walks into it (the #5/#14
   west-lane volleys own that box's lane). d3 now walks the off-path detour boxes only;
   d1/d2 keep the plain resupply (their streams never put the hall on the trail).
6. **The escort loop breaks on a posted exit** — the run13 stream posted the exit mid-walk;
   every follow leg then no-ops at +0 ticks (`stop_on_exit`), the contact froze mid-route,
   and the 24 stall rounds expired into the panic. A posted exit IS the walk's completion
   signal; the loop now breaks to the asserts.

Measured on this tree ([L] runs 1–15, all through the real input path):

- g6 free kill at 7217 (55.0 → 55.0), the captain #7 free kill at 9525 (47.8), g11 down to
  the press finish. The press itself needed two fixes: the first version targeted the
  guard's own position (z 18.3) and ate his 12.0 melee + #5's west-lane pellets deep (the
  run7 death); the landed version presses a FIXED shallow spot (6.5,13.2 — just inside the
  door plane) and fires ONE wound-aborted `shoot` attempt from there.
- **THE HALL AND FIRED at 13831** (ch 153, all three named guards dead through the real
  fire path), and the walk reached the escort phase at **21.4 hp** (12 hits for 78.6 across
  the whole walk — the ledger's 55.0-at-the-door shape never appears because the exchanges
  are budgeted).

## The new wall (the next lever, in the ignore message)

The escorted contact (drone #161, skin 0x0500004F, death channel 0x63) died to a PLAYER
round in the post-AND resupply's west-corridor crossfire: `DroneFunc_OnInitDeath` with a
non-team-1 victim and a by-player kill → reason 1 → FailLabel 0x4000034 "Killed Civilians"
+ ch 0x60 → the monitor turns it into 0x63 → Mission_Update fails → the fail-restart exit
posted at 14551 [M ally.rs `on_init_death`/`death_channel_fail`; the exit line: `exit
Some(117440513) cold false restart true mission state 2 internal 5 fail_label 0x4000034`].
The return-fire stands already hold the `fire_shield` cone (`Some((ci, true))` since
level-06-rook-door-3), so the kill rides an unshielded fire path (the test's own direct
`step_bond(input, trigger, …)` tap loops have no cone check) or sits outside the modelled
cone (reach 0.9 + 0.05·ds, bloom-capped 0.2464·ds). Next lever: instrument the contact's
death (`NF_DRONE_WATCH ci` + `NF_DMG_TRACE` — the watch prints his health per tick, the
dmg trace prints every round's owner/frame), find the firing path, widen the shield; then
the escort's west route, the slab ambush and the plate AND-touch.

## How it was verified

- `walkreal_07000001_live` (whole binary, `--test-threads=1`): **7 passed, 0 failed,
  1 ignored** — the ignore is the d3 `walk_..._real_damage_00agent` ledger; the d1
  real-damage and d2 Agent sisters pass (every change is difficulty-gated; their streams
  are byte-identical through the None-defaulted opt-ins) [L 84.6 s].
- `interactables_07000001_live`: **10 passed, 0 failed** [L 28.9 s].
- The d3 run traces cited above ([L], /tmp/d3_run*.txt on this sandbox: runs 1–15 with the
  per-leg bond/guard state lines, the abort prints, the dry-gate dump and the exit line).
- **The fast suite did not complete**: `tools/test/suite.py` was started in the background
  in the first minutes (the wrapper at `/home/z/my-project/scripts/start_suite.sh`, PID
  alive and building), then killed at the lever round — the cargo file lock serializes
  against the lever builds (each `cargo test` waits for the suite's lock), and this 9.9 G
  disk hit ENOSPC twice on prior rounds with the full 4-crate build (the 1b-5 note: "the
  full 4-crate suite cannot fit this 9.9 G sandbox"). `suite.out` is empty (the runner
  buffers). The two whole binaries above are the coverage; re-run the suite on the final
  tree before merging.

## What's left

- The contact-kill lever (above), then the escort's west route → the slab ambush → the
  plate AND-touch → the exit.
- The suite re-run on the final tree (merged with origin/master).

## Live checks for the main agent (PCSX2, fresh boot)

1. The contact's death attribution: with Bond fighting the west-corridor pair (#14/#15) and
   the captain #5 from the hall's south, watch which round kills the contact (#161) —
   expect a player-origin bullet (the game's fail is "Killed Civilians" only on a
   by-player kill [M ally.rs on_init_death]), and note the weapon/position.
2. The dance's crouched retreat under fire: with the captain #7 firing through the open
   aperture, confirm a crouched Bond crossing the lane takes no pellet damage while a
   standing one does (the peek runs' law, now load-bearing for the retreat).

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token,
cost or context-window readout to the worker. What was visible: the task message's clock
(sandbox expiry ~02:49 UTC; this report written ~02:16 UTC), `git` hashes, test outputs
and process exit codes. No input/output token counters, no credit balance, no dollar cost.

## ntfy

Sent at the finish step (see the queue record); the fallbacks fire in order if the first
curl times out — this sandbox's network blocks ntfy.sh the way hall-2/hall-3 documented
(curl 28 / errno 101 / wget timeout); the exact error output lands below only if all three
fail.
