## Your task: `retune-nb-15-2`, 15 spawn to the Omega sector exit 00 Agent walk green again (round 2)

Branch from current origin/master; if origin/s/retune-nb-15 is not on master, merge it in first. Read docs/reports/retune-nb-15.md there and the ignore message on walk_spawn_to_the_omega_sector_exit_real_damage_00agent. The report found the airborne grenade cannot be beaten by the kill; avoid the throw instead (route, timing or stand). Take the next lever from that ignore message and win the fight that now kills Bond. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000015_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-15-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-19T00:21Z

---
## Report (done, 2026-11-19T02:30Z)

# retune-nb-15-2 — 15 spawn→Omega sector exit 00 Agent real-health walk green again (round 2)

## What the task was

`walk_spawn_to_the_omega_sector_exit_real_damage_00agent` still red after retune-nb-15: the plank
and the west room were fixed there (58.6 kept) but the death had moved to the GREAT HALL — the
baseline died t20233 at (0.02,41.41,45.30), −34.58 from the placement-733 grenade guard (port
#5, weapon 0x1a) while the fights' stand held Bond in the pop zone. Branch from origin/master
73822d3a0 with origin/s/retune-nb-15 merged (it was not on master). All changes are in
`crates/nf-view/tests/walkreal_07000015_live.rs`, every one gated
`difficulty >= 3 && !w.tops_up()`; real input only, no top-ups, no damage/health changes, no
guard-code changes.

## The levers (each judged on its own real run of the real input path from the level start)

1. **The trace.** A crossing trace ([15hall]) was added (print-only, same legs): at t19840 the
   leaf is open, NO grenades airborne, #5 alive hp 20 at (7.93,41.37,36.54); the killing throw
   came at ~20030 MID-FIGHT and its rest (6.1,41.4,39.6) popped at 20233, 8.35 u off Bond's
   throw-time spot. The rest sits on the Bond↔#5 line; the flight is ~200 ticks.
2. **Run 1 — the fightless cross** (the shooters_window opt-in set empty for the hall legs): the
   fight hold was the exposure, but Bond then walked INTO #5 — he stands ON the bridge lane
   (live (7.93,36.54) vs the east-bridge leg (6.5,34.0), closest approach ~1.7 u) and his
   point-blank launcher killed at t20058 (−14.00×2 −13.58 at d 1.5, frames 20050-20058).
3. **Run 2 — the through-leg lob**: the frag (0x34) lobbed at #5 from (0,44) — past the leaf
   plane, the throw line crosses no leaf and crosses the pit's x −4..4 band at x 6.3 over the
   bridge strip — flushed #5 into DODGE MODE (he fled to (−0.15,27.84) and dealt ZERO hits on
   the whole run ledger). Bond crossed the bridge clean. The new death: #19 (also arcing 0x1a)
   sat on the hall-south funnel (his spot 0.7 u off the waypoint) and his point-blank round
   exploded on both (−38.08 at d 1.1, frame 21358; it killed #19 too, hp −49).
4. **Run 3 — the fights restored before the hall-south leg**: no fight ever engaged; the
   NF_RF_TRACE run showed fight#19 tapping #13 (hp 10 at (−3.1,30.9), d 10.2) for 290 ticks
   with NO damage (clip 7→1, tgt hp pinned — the free-aim cursor offset rides the view
   [M scene.rs free-aim]) while #19 (hp 2!) and #5 (resumed from his dodge, −7.00×2 at 21345)
   finished Bond at the bridge end.
5. **Run 4 — the AIMED stands at the bridge end** (head-first, the aim button, spread-0 head
   taps — the [15solo] #11 shape): **#5 and #19 killed CLEAN, health 34.6**. The death moved to
   (0.39,41.46,29.14): **−34.58 from the world** at frame 21436.
6. **Run 5 — the airborne wait + re-cross** (the runs' working theory: a dying grenade): the
   SAME pop fired again at Bond's re-approach (t22733) — position-triggered, not timed.
7. **The identification** (the NF_DMG_TRACE + dump runs): the pops are TWO EXPLOSION records
   (radius 5, damage 100, kind 0, owner −1) at **(1.3,40.9,28.2)** and **(−2.0,40.9,29.0)**.
   NOT mines (the mines list owns no placement there — the [15minelist] dump), NOT hurt
   volumes (the hurts list is the pit kill floors, y 38-39 — the [15hurts] dump), NOT triggers.
   They are the **DESTROY objects (the explosive barrels)**: the smash queues the barrel's own
   blast (radius 5, damage 100, weapon_type 0, owner the smasher) [M drone.rs's Destroy_Smash →
   pending_blasts push]. Bond's own body contact smashed them — the hall-south lane passes
   through both; a probe spot 5.39/7.82 u off both stayed cold (the contact reaches ~2 u). The
   old green streams never met them live: #5's grenade spray smashed them long before the lane
   (the lob's dodge suppression kept #5 quiet on this stream).

## THE FIX

In `cross_hall_pass` (the file-local helper walking the four hall legs, d3-gated):

- the lob block (before the cross): the frag lobbed at #5 from the door[50] through leg, the
  retreat north to (0,55) (out of his sight: the sight line to (0,55) crosses the door plane at
  x 3.8 — outside the aperture; the pops aimed at the lob spot reach (0,55) at 15.03 u > radius
  15 → zero [M Explode_Propagate's truncation]), 360 ticks, then the P2K drawn for the fights;
- the fight stands suppressed (shooters_window Some((0,0))) for the three bridge legs;
- the bridge-end cluster stands: head-first aimed stands on the nearest live hostiles within
  14 u (up to 3) — #5 and #19 die clean at 34.6;
- **the barrels shot from the bridge end** (smash_prop_idx, one round at the first — 8.52 u,
  +14 ticks — the twin chained by the case-0x21 smash chain [M 0x17c518 → Destroy_Smash], 3.4 u
  apart): the lane is blown clean before Bond walks it;
- the fights restored before the hall-south leg, the corridor legs and the tail unchanged.

**Measured: 34.6 health kept through the whole walk to the exit.** The pop probe and the
mines/hurts dumps stay in the file as the ledger (print-only).

## How it was verified

- Every lever was judged on its own real run (the run logs are the [L] evidence above).
- `walkreal_07000015_live` whole, `--test-threads=1`, on the final tree: **9 passed, 0 failed,
  0 ignored** (twice — once with the ignore still set, once after its removal; 95.16 s). The
  other eight tests (the d1 real-damage, the d2 Agent real-damage, both top-up walks, the d2/d3
  objective-chain sisters, the range-shot fence test and the body-contact fence test) pass
  unchanged — their timelines are untouched (all changes are inside the
  `difficulty >= 3 && !w.tops_up()` gates).
- No code under `crates/*/src/` and no shared helper under `crates/nf-view/tests/common/` was
  touched (`git status` clean apart from the one test file), so no other binary's inputs changed.

## What's left

- The full four-crate suite on the final tree did not fit this session: the sandbox reset cost
  the whole setup (a cold release build of the nf-view test binaries on a 9.9 GB disk), and the
  background suite runner is killed between tool calls here (the prior session recorded the
  same). The only changed file is `crates/nf-view/tests/walkreal_07000015_live.rs` and its
  whole binary passed 9/0/0 on the final tree; the next session should run the chunked suite
  (`python3 tools/test/chunk_suite.py` to ALL DONE) before merging.

## Usage

not visible to me

## ntfy failed

(to be filled after the finish steps run)
