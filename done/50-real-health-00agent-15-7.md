## Your task: `real-health-00agent-15-7`, 07000015's 00 Agent real-damage walk reaches the omega sector exit

Branch from origin/master. walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent is ignored; its ignore message holds the current ledger (the previous round got past the fence chain, the grenadier pair and the chaser; it now dies at t5790 at door[47]'s north side (34.74, 41.48, 63.88) with 12 hits for 100.0 - the desk-corner AR guard #2's volley during the pre-clear stand's unarmed window). Carry on from the next levers in that ignore. Real input only, no top-ups, never lower damage or health; every lever gated to 07000015, difficulty 3 and !w.tops_up() so the other walks keep their timelines. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000015_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-15-7`.

---
claimed-by: sandbox7 at 2026-11-13T05:25Z

---
## Report (done, 2026-11-13T06:02Z)


---
## Report from the worker (done, 2026-11-13T06:02Z)

# real-health-00agent-15-7 — 07000015 on 00 Agent with real health, the desk corner cleared

**Result: the death moved t5790 → t7985 and the walk now clears the whole desk corner, door[32]
and the #526 lob for the first time.** Still red: the west-room grenadier pair's throw cycle and
the fights' dry-mag resupply nav. The refreshed ledger and the ordered next levers are in the
ignore message on `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`.

## What's done (each lever measured on its own real run)

1. **The desk stands answer HEAD-first.** The shared exposure pick is torso-nearest [M
   DamageLog::exposed_impl — the head box wins only inside the opt-in 6 u HEAD_HUNT window], so
   the old stand ground 3.5 body rounds while the guard's stream rode the window. The fresh run
   pinned the real killer: #2's death hit was his **0x1a GRENADE blast −31.28 at 10.7 u**, not a
   bullet, and he arrived at hp 14 (the fights' work). The file-local `head_aim` mirrors the
   gun-ray law (the eye + the two low stand-in gun rays, `segment_bullet_scoped` — this walk
   never runs `with_glass_sight`) at the part-5 box; the head round is ×10 [M NDrone2_HitDamage
   0x145558 × DroneDamage_Head] = 35 from the P2K — **#2 VANISHED at +20, #3 at +19, zero return
   fire during the stands**.
2. **The #526 lob lever WORKS.** From door[32]'s threshold the frag chained the mine with Bond
   16.22 u off the boom — **−0.42** total (the feared −44.33 rim shot never paid; the d1 walk
   keeps its rim shot, the check is a read).
3. **The west-room fights went honest.** The pair is SPawner-released during the door[32]
   threshold leg ~28 u behind the west wall (the lane-mark 22 u skin stands resolved nothing —
   measured); the fights' eye-law pick fought #11 "killed false" at hp 3 with the rounds on the
   door jamb (the documented 07000013 mismatch). Scoped `pre_throw_stands` + `lane_fights`
   re-armed for the phase (d3-gated, cleared after the jog sweep) made the pick and the taps ride
   the gun-ray law: **#11 KILLED at 7637** — the first west-room kill; his grenade still landed
   −14.0 (the thrown grenades outlive their throwers [L the aim-last-seven §3 ledger]). #10's
   fight never reached him (−21.1) and **Bond survived both pops** (58.9 → 23.8) — the pop
   budget at this health is two, not one.

## The wall (measured)

The fights' **dry-mag resupply nav**: after the P2K's clip burned, the nav walked Bond back east
31 u to the (41.9, 40.4, 67.8) pile with both throwers alive — the `switch_dry` cycle needs a
LOADED wheel entry (the P2K at clip 0 spare 34 did not qualify) — and the grenades chased the nav
to the death at t7985 at the threshold, 15 hits for 100.0.

## What's left (in the ignore message too)

1. Disarm the fights' resupply nav for this phase, or keep a LOADED gun in the wheel (a mid-phase
   reload press — the reload input exists in `aimed_stand`).
2. Kill #10 from the jog with the head-first aimed stand (the raise ~330 ticks eats 1-2 pops;
   after #11's outlived pop the budget holds 44.9 — one pop survivable, the fights' co-fight
   helps).
3. Then the corridor south, the great-hall #5 (the d3 roster's wander — the same spot-resolution
   miss is likely there), the door[42] fence dance, the exit. The upstream costs stand accepted:
   the sneak 14.0, the #462 beam's own blast −14.72 (the north-side line [S] remains the untried
   dodge — the beam spans z −2..3.2), the crossing 9.0.

## How it was verified

- `walkreal_07000015_live` WHOLE: **8 passed / 0 failed / 1 ignored** (68 s; the d1/d2/top-up
  streams byte-identical — every lever is gated to difficulty 3 + !tops_up; the d3 top-up walk
  passes on its pinned timeline).
- `interactables_07000015_live`: **19 passed / 0 failed** (35 s).
- Seven ledger runs of the ignored d3 walk (`/tmp/run3.out`–`/tmp/run7.out` + the first two
  console runs), each with NF_DMG_TRACE: the baseline reproduced t5790 exactly, then the head
  lever (t7684), the lane-mark stands (no-op — the release timing pinned), the scoped flags
  (#11 killed), the ammo gate (the resupply still fired), the switch_dry scope (the resupply nav
  still walked Bond east — the death's mechanics pinned).
- The fast runner (`tools/test/suite.py`) was started three times this session (twice plain
  nohup, once setsid) and died silently each time with an empty `suite.out` and no process — the
  same artifact the 15-6 session recorded. The whole-binary + interactables runs above are the
  reported coverage; the suite was NOT re-run on the final tree.

## Live checks for the main agent

None required — all levers settled in-sim. If wanted: a PCSX2 watch on the west room (x −6..8,
z 61..77) from t≈7300 would pin whether the game's own guards round the corner into sight before
the port's exposure rays rate them visible (the fights' honest pick already kills #11 through the
open doorway, so the port is if anything stricter).

## Usage

Not visible to me: this environment exposes no token, credit or context-window figures for the
session.

---
Merged (squash, docs dropped): master 679168719; next round queued.
