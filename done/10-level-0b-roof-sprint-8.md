## Your task: `level-0b-roof-sprint-8`, the 0700000b roof sprint can be walked on 00 Agent with real health

Branch from origin/s/level-0b-roof-sprint-7; if origin/s/los-mask-repairs exists, merge it too (it repairs the guard sight masks this walk runs under); then merge current origin/master. Read docs/reports/level-0b-roof-sprint-7.md and the 00 Agent ignore message in walkreal_0700000b_live. Sprint-7 pinned the rocket: 0x2e homes (+0x6C = 0x32200, no gravity), refires every 60 ticks, and spawns on the crosshair ray [M 0x1a4bf0:226-245]; one traced round killed two guards. Take its next levers: aim the round through the crosshair (turn the view to the target before firing, since the walk's turn input does not move the spawn ray) so the lock lands on the #6/#7/#8/#9 cluster, then finish the deck. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000b and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000b_live and interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0b-roof-sprint-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T09:25Z

---
## Report (done, 2026-11-10T10:39Z)

# level-0b-roof-sprint-8 — report

Branch `s/level-0b-roof-sprint-8` = `origin/s/level-0b-roof-sprint-7` (16a89b92e) merged with
`origin/master` (db5905728; clean). `origin/s/los-mask-repairs` was NOT on origin when the
branch was cut (checked twice), so the task's fallback base applies. **Status: the 00 Agent
walk is still red, but the driver's lever is now MEASURED SCIENCE: the crosshair-aimed round
is reproduced (the fire-and-run lane rounds lock the drift #5 and the 8.0/100.0 blast pays
−70 on #5 and −55 on #7 — two kills one round, three runs), the refire law is pinned to the
port's own fold, and the traps that kept the full squad clear out of the health budget are
mapped with addresses and run ledgers.** The whole `walkreal_0700000b_live` binary is
**7 passed / 0 failed / 1 ignored** on the final tree; `interactables_0700000b_live`
**14 passed / 0 failed**. No `crates/*/src/` changes — everything is in the 0b test file,
gated to difficulty >= 3 (the d1/d2/warm streams pass the whole binary unchanged). No damage
or health lowered, no top-ups, real input only.

## What's done

1. **The crosshair lever (the driver's ask) is implemented and reproduced.** The sprint-7
   lane-aim rounds flew the lane's OWN bearing (the traced yaw 1.284) because the lane holds
   the spawn ray. Sprint-8 turns the VIEW on each victim while the lane holds — the turn rides
   the same 0.08 stick clamp, the forward gate stops Bond for the ~8-20 turn ticks — and the
   round is LEFT to its homing while Bond resumes the run (fire-and-run, no stands). Round 1
   (the view on the band anchor #6 from the P1 radius) locks the near drift #5 — the scan's
   nearest seekable in the 0.66-dot cone [M Bullet_homing 0x12f358] — and the 8.0/100.0 blast
   paid **−70 on #5 and −55 on #7** (two kills one round) in three separate runs [L runs
   8-14]. The first (standing) version detonated at (3.50-3.66, 22.28) both times; the
   fire-and-run version reproduced the same pair kill ON THE RUN.
2. **The refire law is pinned** (the sprint-7 lever 2): the pull's pending re-arms on the
   FRESH edge only — a held trigger fires ONCE (the first run's clip froze at 3 through t+121
   [L]); the +0x40 = 60 [D] interval runs down on released ticks too; the re-press fires on
   its own tick (+0x44 = 0 → the press-tick spawn [M Player_WeaponFiring 0x1a5460:601-629,
   :688-690; the port fold gun.rs update_with]). The record: +0x68 = 0x14000 (no 0x8000
   state-9 block), +0x44 = 0, clip 4, interval 60 [L ee_start_05.bin 0x2c2178]. The presses
   also wait for the previous round to detonate — a launch past a mid-flight round's
   blocked death re-scan flew straight and paid nobody [L run 2].
3. **The traps are mapped** (all in the ignore's ledger):
   (a) the stand bleeds — a static P1 stand sat in every guard's full-accuracy roll: 59.5 →
   3.5 hp across the 95-tick pre-blast window [L run 1]; the strafe wiggle (the moving latch
   [M 0x179740:71-73]) cut it ~2× but the three-blast stand needs ~300 ticks against the
   59.5 budget — it does not fit;
   (b) **#4 is a lock thief** — alive or ASLEEP he is seekable [M Bullet_heatseekable
   0x12f248: health > 0, state ≠ WAIT_SWITCH — the dart's sleep does NOT exit the scan; only
   the corpse (health ≤ 0) does] and the nearest-in-cone rule hands him every lock whose ray
   passes near his gap column: an r2 from the P1→P2 stretch (#4 0.79 rad off the band ray)
   chased him INTO the column, the blast −82 on the running Bond [L run 11]; an r3 from the
   gap mouth met the closing #4 (0.13 u/tick) at 5.6 u for −16.31 [L run 12]; the
   P2-crossing ray (Δ 1.41 rad, outside the ±0.847 cone) excluded his LOCK but the round
   still died on the gap-mouth structures 40 ticks out [L run 14];
   (c) the r1 chase-blast self-hurts −7.5..−11.7 (the drift lands the blast 5.9-8.9 u from
   the lane [L runs 12/14]);
   (d) **the ring turret is the real 16.0 toller** — the 4 × 4.0 world hits are GT turret
   #291 (placement 291, (−1.14, 49.06, −0.19) [L the 0b-diag turret list]) on the n14→n15
   return lane. The sprint-7 ledger's fusebox attribution was WRONG — 0b builds NO fuseboxes
   [L the same diag's fusebox loop printed none]; the test was attributing from memory.
   The turret shreds any stand (25 × 4.0 in 100 ticks [L the turret-stand run — that
   experiment cost Bond 100.0]); the laser kill is real on paper (the 0x4E contact 3.0/tick
   into ext+0xCC health 100.0, dead at < 0 [M GT_Update 0x180498:236-244; the
   turret_hits_live warrant]) but needs an out-of-arc switch choreography that costs more
   than the 16.0 it saves — measured, not shipped.
4. **The death ledger (run 14, the closest pass)**: 44.7 world (16.0 turret + the r1/r2
   self-blasts) + #4 34.3 (0x18, closing to d 6.3 through the whole P1→P2 leg) + #9 7.0.
   The band (#6/#8/#9) and the drifters (#5/#7) die to the rounds; the budget dies to the
   #4 chase-blasts and the self-hurts.
5. **The next levers** (in the ignore): (1) put #4 into the CORPSE state before any band
   round fires — the dart close gated by DISTANCE not phase (he sat at d 6.3 for 80 ticks
   before the phase-2 close could start [L run 14]); the sleep alone is not enough (still
   seekable [M 0x12f248]); (2) a band ray with #4 outside the 0.66 cone AND the gap
   structures cleared — the 16-ray probe at the P2 crossing (the run-14 round died at 10 u
   on an unmapped gap-mouth piece); (3) the r1 self-hurt vs the #7 17.5 saving; (4) the
   turret 16.0 stays the fusebox task's.

## How it was verified

- `walkreal_0700000b_live` (the whole binary, the final tree): **7 passed; 0 failed;
  1 ignored** (`--test-threads=1`, 47 s) — the green set holds with the d3-gated additions.
- `interactables_0700000b_live`: **14 passed; 0 failed** (27 s).
- The shot work ran with NF_DMG_TRACE captures in /tmp (not committed); every run real
  input only — no teleports after the load, no pokes; the death guards only stop ticking at
  the fail. No crates/*/src changes (the test file + docs only).
- The full `tools/test/suite.py` could not run in this sandbox: the background jobs are
  reaped between tool calls (the first two starts died — one on a missing cargo PATH, one
  on the reap), and the disk is 9.9 GB with ~0.5 GB free (the sprint-7 report's ENOSPC at
  ~90% of the four-crate build). The two required binaries above were built and run
  individually on the final tree.

## What's left

1. The 00 Agent walk (the ignore's ledger): the #4 corpse-state order, the clean band ray,
   the r1 self-hurt trade, the turret 16.0.
2. The live-wire/turret 16.0 — the fusebox task's (the attribution is now correct: turret
   #291, not a fusebox).

## Live checks for the main agent

1. On 0700000b (00 Agent): stand at the deck key, fire one 0x2e round with the crosshair on
   the band anchor #6 — expected: the round's lock (the nearest seekable in the cone) and
   the detonation spot. If the disc's round locks #5 and pays the −70/−55 pair like the
   port, the port's homing fold is exact; a different lock order is the divergence to chase.
2. The ring turret #291: walk the n14→n15 return lane on 00 Agent and count the world hits
   — expected four 4.0 hurts (frames ~13380-13423 in the port's run). If the disc charges
   fewer, the port's turret fire cadence is the divergence.
3. The refire: fire two 0x2e rounds ~60-130 ticks apart with a release between — expected:
   the second launch on the fresh press (the port's law above).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures to the
worker (the IM gateway metadata carries only session/chat ids).
