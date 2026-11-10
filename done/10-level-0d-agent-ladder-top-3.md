## Your task: `level-0d-agent-ladder-top-3`, 0700000d can be walked to the end on Agent with real health

Branch from origin/s/level-0d-agent-ladder-top-2, merge current origin/master, and read its docs/reports/level-0d-agent-ladder-top-2.md plus the Agent ignore message. The ladder-top phase now survives, and the death moved from 4140 to 8110, on the wire ride. Take its next levers in order: (a) the wire-ride shooter #19, with a pre-ride box act on (172, -292.5), or accept the ride's one 10.0 hit as the ledger allows; then the levers after it in the ignore message. Judge each over several real runs. Do not repeat its measured dead ends. Helpers stay gated to 0700000d and the difficulty. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-agent-ladder-top-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-10T05:31Z

---
## Report (done, 2026-11-10T07:11Z)


---
## Report from the worker (done, 2026-11-10T07:12Z)

# level-0d-agent-ladder-top-3 — the death moved 8110 → 13291 (the ride, the deck and the descent now pass on Agent real health); the act-proof law and the lever-(a) blocked-ray measurement are the session's settled facts; still red with the ledger and the next levers in the ignore message

Branch `s/level-0d-agent-ladder-top-3`, based on origin/master c7d5b9dfe merged with the
current origin/master 6a4fcf4ef (the merge commit 3e741c16a; the prior task's branch
`s/level-0d-agent-ladder-top-2` is deleted from the remote — its changes were merged to
master as c7d5b9dfe "0d Agent ladder leg", so the branch base already carried them; the
walk-2 report was read from the queue `done/10-level-0d-agent-ladder-top-2.md`).

## 1. What was measured first: the trio is ACT-PROOF from the ladder top (the run-4380 law)

The driver's lever (c) (the in-pass retarget) exposed the real cause of the 4034/4380-class
phase deaths. The retarget pick (pre-verified `clear_box_aim`, nearest-first) chose g33 —
whose ray audit reads full CLEAR / static clear / bullet_scoped CLEAR every tick — and the
pass landed **zero** damage in 450 ticks with 8+ aimed rounds at d 63 (run 4380). The second
pass on g32 (also head-only-clear) landed zero again (run 5239). The damage summary puts the
same pair's cadence at 94 of the 100 health. Meanwhile the SAME box act one-shot the deck
#20 at d 31 (one head hit, +209). The settled law, measured across five rolls / five code
variants:

- **The west-roof pair (g33 (151.5, 22.7, −247) and g32 (151.4, 22.7, −248.5)) is
  unkillable from the ladder top.** Their torso lines die on the mid-band obstruction (the
  z −249..−252 band + the -4 slab edge, the walk-2 ledger's own probe law); only the part-5
  HEAD boxes clear, and the head line GRAZES at d 63 (the 0.12 u tap box is the head's own
  angular size there). Close (d 31) the head line lands; at d 63 it does not — in any of the
  rolls.
- **The pair's opening cadence (3292/3532 = 56) is the phase floor** — it fires before any
  act can (the acts need 372+ ticks; the trio opens 240 after the climb ends, the walk-2
  ledger's number, confirmed every roll).
- **The aim bit FLAPS with the fire script between shots** (the run-4380 pass fired 5 HIP
  rounds — the full 100-unit cone [M Bullet_init 0x12c5a0:139-170] — at d 63 while
  `sim.aiming` read false). The d2 trigger now holds fire until the bit is back up (the
  aim-bit trigger gate, `agent_fast` only; the plain/Operative drained paths keep their
  byte-identical behaviour — the plain replay stayed green).

## 2. What landed (all inside `difficulty >= 2` + `real_damage` gates, local to
walkreal_0700000d_live.rs — no `tests/common/` change, no `src/` change)

1. **The retarget pick** (`pick_trio_target`): pre-verified `clear_box_aim`, torso-first,
   zero-damage blacklist, **the head-only skip** (a candidate whose only clear box is the
   head is never attempted — the measured graze law), and `band_all_stalled` (every live
   band guard is blacklisted / blocked / head-only) → the loop BREAKS instead of waiting.
2. **The act-proof skips of the re-burn legs**: the head sweep (4 × 400), the deck #16 leg
   (600), the all-difficulty west-roof sweep (3 × 900) and the never-landing g21 leg
   (2 × 600) all re-burn their caps on the same act-proof pair — skipped while act-proof
   survivors live (`act_proof` / `act_proof_live`).
3. **Lever (a), the pre-ride #19 box act** (the driver's first lever): ran and is
   **measured dead as a kill line** — g17 (#19) sits at (170.66, 19.58, −293.37) (the
   ledger's (172, −292.5) ±8 caught him), the ray from the ladder top is **BLOCKED at
   0.592** (full 0.592 / static 0.593@y0.0 / bullet_scoped 0.593 — the line dies mid-flight
   on the roof structure), the act held fire (zero rounds — the world-ray gate worked as
   designed), 800 cap-ticks stood. **The ride survived with #19 landing ZERO this roll** —
   the walk-2 8110 ride hit was dice, not a law. Next session may trim the leg to one
   250-tick probe (the blocked-ray audit is already on the log).

Net: the trio window collapsed to ~0, the phase exit health rose to ~38-40, and the run
went **from 4380 to 13291** — the wire ride (the walk-2 death), the deck, the E on #450,
the walk-off and the descent all pass on Agent real health now.

## 3. The new death and the next levers (pushed in the test's ignore message)

The death is at **13291**, (197.95, 10.56, −278.88) — the descent/bay approach. The roll's
drains: #33 2 × 28 (the unavoidable opening cadence), #20 6.0 (the climb graze), #32 0.0,
**#21 28.0** (the ground-band sniper, (157.8, 1.0, −228.4), hp 10 at the death). In order:

1. **The band-sweep #21/#36 kill audit** — the band sweep (the chest act, 600 × 3, the
   PROVEN clear lines over the low x ~160 wall) did not land #21 this roll; audit the
   sweep's `[0d:ray]`/convergence lines in the run log (the same d-63-class graze is the
   suspicion). His 28.0 is the new decisive drain.
2. **The descent/bay approach exposure** at (197.95, 10.56, −278.88) — the Operative twin's
   yard legs (the crouch-dodge law [M Player_GetHeadPos 0x195aa0 + M 0x176a28]) are the
   shape reference.
3. **The pre-wake census widening** — the pre-wake snipe legs (the platform stand, the head
   lines CLEAR [probe_0d_roof_rays]) found NOBODY at (145.6, −256.7)/(145.7, −247.3) in the
   walked timeline, every run. Widening the census to find where the west-roof placements
   actually are pre-wake is the path that deletes the act-proof pair entirely and collapses
   the phase floor to #20's 6.0.
4. The strip-mouth west rays to the ROOF remain unmeasured (the walk-2 ledger's lever (b)).

## 4. What was run (all on the final tree, merged with origin/master)

- **`walkreal_0700000d_live` whole binary, `--include-ignored --test-threads=1`**:
  **1 passed** (the plain master replay — byte-identical, green), **2 failed** — the
  Operative twin on its OWN inherited ledger (the slot-hop jump-drift assert at 13800,
  untouched by the d2 gates, the level-0d-operative-walk-2 task's) and the Agent twin at
  its measured ledger death (13291, above).
- **`interactables_0700000d_live`** (`--test-threads=1`): **11 passed, 0 failed**.
- **`nf_game` lib** (`--test-threads=1`): **759 passed, 0 failed, 5 ignored**.
- **nf-game integrations**: `fix_ai_remainder_live` 2 passed; `npc_footsteps_live` 2
  passed; `fix_portonly_review_live` 3 passed; `fix_sound_remainder_live` 4 passed.
- **`nf_formats` lib**: **92 passed, 0 failed**; `disc_golden`: **11 passed, 3 failed —
  exactly the 3 documented `disc_golden` tests the driver allows on the slim ISO**;
  `effect_mesh_flags_census`: 1 passed.
- **`nf_audio`**: **27 passed, 0 failed**.
- **The full four-crate suite via `tools/test/suite.py` did NOT fit this box**: the build
  phase needs ~7 GB of `target/release` on a 9.9 GB rootfs that already carries the ISO
  (1.0 GB), the repo and the toolchain — 2.3 GB free at build time; this is the same
  `No space left on device` death the walk-2 session documented. Per the driver's
  precedent ("The full suite did not fit" is fine): the diff touches **only
  `crates/nf-view/tests/walkreal_0700000d_live.rs`** (+ the WORKLOG and this report) — no
  `src/`, nothing under `tests/common/` — so no other binary's behaviour can change; the
  binaries that use the changed file ran whole (above), and the three crates whose libs
  the merge touched ran their binaries whole too.

## Live checks for the main agent

1. **The act-proof graze, one probe** (settles the head-line law on the real game): on
   0700000d at Agent, from the ladder top (213.1, 18.1, −234.5) aim the P2K at the west-roof
   guard at (151.5, 22.7, −247) — the head box is the only exposed part (the torso line dies
   on the mid-band obstruction): gdbwatch the bullet spawn and the impact — expected: the
   game's own rounds graze the head box at d 63 the same way (if the GAME lands them, the
   port's ray construction deviates at steep-up long range and `probe_0d_bullet` needs a
   re-calibration; if the game also grazes, the act-proof law is the game's own geometry).
2. **The #19 blocked ray**: at the ladder top, the line to (170.7, 19.9, −293.4) reads
   blocked at 0.592 in the port — a fieldprobe render from `--bond 213.1,18.1,-234.5` yaw
   toward the south-west roof should show the roof structure that eats the line.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.
