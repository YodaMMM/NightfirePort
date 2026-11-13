## Your task: `real-health-00agent-15-6`, 07000015 can be walked on 00 Agent with real health past the grenadier

Branch from origin/s/real-health-00agent-15-5, merge current origin/master, and read its docs/reports/real-health-00agent-15-5.md, "What's left", plus the ignore message. Take its levers in order: answer grenadier #8 before his first throw (widen the pre_throw_stands idiom to difficulty 3, or a d3 stand at the plank's cover edges); re-take the #9 give-up at node 2; then the meet's pool chain; then the coded levers that never ran. Judge each over several real runs. Helpers stay gated to 07000015 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: main agent at 2026-11-12T12:05:16Z

---
## Main agent (2026-11-12, released back)
On s/real-health-00agent-15-5 + master 729f9694e the d3 walk dies at t5527 (13 hits for 100, 10 fights). Lever 1 measured: pre_throw_stands ALONE at d3 (no switch_dry) dies at t1048 at the aperture (81.53, 41.41, -0.49), 11 hits - identical to the earlier both-flags run, so the stand itself is the cost at 00, not switch_dry. Rule it out; go to the plank cover-edge stand or the fight pick reaching #8 before the arc lands.

---
claimed-by: sandbox7 at 2026-11-13T04:16Z

---
## Report (done, 2026-11-13T05:06Z)


---
## Report from the worker (done, 2026-11-13T05:06Z)

# real-health-00agent-15-6 — 07000015 on 00 Agent with real health, past the grenadier

**Result: the walk passes the grenadier and the whole plank/meet phase clean, and dies at
t5790 at (34.74, 41.48, 63.88) — door[47]'s north side — with 12 hits for 100.0** (was t5527 at
(37.42, 41.46, 66.03) with the grenadier #8's 39.5 in 3 blasts). Still red: the desk-corner AR
guard #2's volley rode the pre-clear stand's unarmed window. The ledger and the next levers are
in the refreshed ignore message on `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`.

## What's done (each lever measured on its own run, ~5 s per walk)

1. **The fence chain's −16.52 is dodged.** The mark stand (z 7.5, 10.4 u off the fence's
   centre) stood inside the sphere-edge reach [M Explode_Propagate 0x17c518] by ~0.4 u — the
   measured hits −6.90/−3.49/−6.13 at ticks 3638-39 [L the run15 trace]. The d3 shot line steps
   to z 6.5 (11.46 u at the pop): zero fence hits in every run since (the run15d-m ledgers).
   The sister test's mark and the d1/d2 lines are untouched.
2. **The plank/meet grenadier pair dies with zero volleys** (was the walk's biggest dealer,
   39.5 + the fight's 41.3-59.3 when poked wrong):
   - the AR draw at door[38] north (d3, real-damage only) shifts the stream so the fights' own
     law kills #7 mid-leg (the 4228 line; the run15f ledger) — without the draw the fights
     never reach the grenadiers and their volleys cost 41.3 + 21.0 (the run15e/g ledgers);
   - the door[39] stand resolves by SKIN 0x50000b2 within 14 u and answers with the zero-spread
     aimed stand: #8 VANISHED at +74 (corpse, no throw), the chaser #17 at +44 (hp −10). The
     skin filter matters: #17 (0x5000087) stands 2.2 u off #7's [D] spot and an unskinned
     nearest answers with him (the run15c poison: 59.3 taken at the 21 u mark).
3. **The chaser #17's 48.0 in 16 hits is gone.** The fights engaged him three times
   ("killed false", the 300-tick law) across the plank and door[46]; the aimed stand at the
   door[39] mark (7.0 u, a part exposed [L the run15f ledger]) kills him where the plain hip
   cone scattered 8 rounds for 0.
4. **The gates.** Every d3 lever carries `!w.tops_up()` — the top-up 00 Agent walk keeps its
   pinned timeline (the first whole-binary run caught the miss: the top-up walk failed until
   the gates were fixed; 8/0/1 after).
5. **The desk-corner pre-clear is coded and measured** (the run15i lever): both guards are
   EXPOSED from door[47]'s north side; the aimed stand converged on #2 and dropped him to
   **hp 2** — but his volley (59.3 in 5) rides the stand's ~330-tick settle+raise window.

## Measured dead ends (do not repeat)

- The door[38] 24.0 u shoot_bounded stand: the grenadiers are SPAWNER-RELEASED and not present
  at ~t3900 (the run15f ledger: no stand printed), and at 21 u the aim gate is a torso width.
- The d3 chaser resupply detour at the door[39] stand: the plain shoot burned the AR dry at
  +61 (scatter), the resupply nav walked Bond east onto the plank grid and route_b1's crossing
  fell him −56.28 at (34.02, 31.82) (the run15f ledger). The chaser answer is the aimed stand.
- The desk-corner resupply: its only corridor pile is the plank drops 29.2 u SOUTH — the
  backtrack nav ate 56.3 under #2's fire (the run15l ledger). The ammo law is the P2K swap
  (clip 5 spare 34 from the fence dance; the fights never touch the P2K — the AR was in hand
  from door[38] north on).
- The global `pre_throw_stands`/`switch_dry` d3 widening (15-5's own measured WORSE, confirmed
  not repeated).

## What's left (the next session's levers, in the ignore message too)

1. **Answer the desk-corner AR guard #2 from a defended window.** The exposure is the aimed
   stand's settle+raise (~330 ticks, unarmed) at 10.7 u from a live AR guard. Options: crouch
   first ([S] unverified — no accuracy law pinned), kill #3 FIRST from the same mark (the
   fights killed #3 at 5567 in the run15j ledger — his removal cuts ~20.3) then take #2 with a
   shorter convergence, or pre-damage #2 from the door[46] through leg.
2. **The #462 beam's own blast −14.72** at the 13.56 u shot stand (the pop at +473 ≈ tick
   2151): the door[42] leaf blocks the far line (the max_d 18.5 gate rode to the same 13.56
   stop), so a north-side line (the beam spans z −2..3.2) is the untried dodge.
3. Then the meet's pool chain (the −15-4 stub's named lever) and the coded levers downstream:
   the #526 lob from door[32]'s threshold (implemented, never reached), the west-room stands,
   the great hall tail.

## How it was verified

- `walkreal_07000015_live` WHOLE: **8 passed / 0 failed / 1 ignored** (the ignored is the d3
  real-damage walk this session works on; the d1/d2/top-up streams stayed byte-identical).
- `interactables_07000015_live`: **19 passed / 0 failed**.
- Ten ledger runs of the ignored d3 walk (the run15a-m series in /tmp, the key ones cited
  above), each with NF_DMG_TRACE for the per-hit/explosion trace.
- The fast runner (`tools/test/suite.py`) was started three times in this sandbox and died
  silently each time before printing anything (an empty suite.out, no process — the earlier
  sessions' nohup pattern); the whole-binary + interactables runs above are the reported
  coverage. The suite was NOT re-run on the final tree — the next session should start it
  first thing.

## Live checks for the main agent

None required — all levers settled in-sim. If wanted: a PCSX2 watch on the desk corner
(x 26-30.5 at z ≈ 70) during the door[47]-north phases would pin whether the game's own guard
#2 (placement roster d3) exposes a head part from the door[47] north mark that the port's
exposure laws rate visible (the aimed stand converged to hp 2 there — the disc may answer
faster).

## Usage

Not visible to me: this environment exposes no token, credit or context-window figures for the
session.


## ntfy failed (from the branch report, d52518cad)

All three fallbacks failed: curl (28) Connection timed out after 20002 ms; urllib URLError [Errno 101] Network is unreachable; wget silent network failure. The same egress artifact as the 15-5 session.
