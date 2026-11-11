# level-14-skybridge-retune-2 — report

Branch `s/level-14-skybridge-retune-2`, base `origin/s/level-14-skybridge-retune` (dc0555112,
already containing origin/master 784ba2d99). Task: 07000014's Operative and Agent real-damage
walks to the delta exit — the slab-pocket camper's accumulated fire by the ledger's levers,
then the crossing re-route it feeds; the _agent sister re-measured and re-routed by the
inherited ledger; the 00 Agent walk left alone (real-health-14-00agent-10's). One walk per
tool call, helpers gated to 07000014 and the difficulty / `!tops_up`, real input only, no
top-ups, never lower damage or health. All changes in
`crates/nf-view/tests/walkreal_07000014_live.rs` and docs; no `common/` helper touched.

## Done

**The camper lever LANDED.** The base round's diagnosis was right and the missing half was
found this round: the drop loop's raw-input ticks BYPASS the walker's leg machinery, so the
return-fire fights never ran on the whole descent — the 5130 death carried 48 hits for 100.0,
0 return-fire fights, the camper's box still full (hp 28) at d 1.4 [L the same trace]. With
`w.return_fire` wired into the loop every 20 ticks (`!tops_up`-gated; the loop cap 2400 → 3600
for the fights' windows; the tripbomb's verbatim duplicate untouched) the fights KILLED the
camper at tick 4742 (health 94.0 → 88.0) and the descent completes — the base ledger's lever
(1), "the fights must engage at that instant", is exactly what the loop was swallowing [L the
r2/r8 traces on this tree].

**The d2 green set on d1.** The path-clear stands (`with_agent_path_clear` — thread-local,
this test only) plus `lane_fights` held the route fights to 1.0 damage total across 18 fights —
Bond reaches leg 3 at ~100 [L the r11 trace]. Two shapes measured and rejected with evidence:
`back_off 4.0` alone (its backpedal displaced Bond AIRBORNE off the south loop's lane — the
hover class, (−29.11, 0.65, 30.95) grounded=false at health 100.0, the 6000-tick cap [L r9]),
and a `clear_hostiles` pre-pass (byte-identical failed stands — the hp-5 survivor #9 finished
Bond point-blank at 9111 — and a regression vs the fights-in-the-loop shape; reverted [L
r5/r6]).

**The `_agent` sister is UN-IGNORED and GREEN.** The base report said "re-measure on this tree
before re-routing" — the re-measure shows the base ledger's d2 death (tick 4297, the same
slab-pocket camper #6) does not reproduce: the fights-in-the-loop fix is shared through the
same body (`!tops_up`-gated) and answers the camper for the d2 walk too. The walk completes:
`walked out in 22633 ticks; stuck 0; falls 0`, the exit hold drained at (26.997, 1.134,
−24.847) and 07000015 loaded warm, arrival (94.50, 41.37, 0.002) on the silo gantry [L two
consecutive runs, byte-identical]. No re-route was needed — the inherited lever WAS the fix.
The d2 set is unchanged from the base branch.

**The fusebox stance work (the d1 round's deepest blocker).** The walk now reaches leg 3's
fusebox #383 stance with ~100 health and dies there to a zap that takes 99.0 from FULL health
~80 ticks after the stance arrival (the earlier −24 prints were the same blast clipped to the
remaining health [L r2/r8/r11]). Established this round: it is NOT the fusebox's own cut damage
(param 2 non-zero → the cut never hurts [M 0x1adfe8:99-101; D the placements census]) and it is
NOT in `sim.hurts` (the standing-capsule probe passed at the 2.2-3.2 u ring and the pulse still
landed). Wired, all `!tops_up`-gated: the stance candidate ring at 2.2-3.2 u with the
hurt-probe (the plain walk keeps its measured ±0.6 candidates byte-identical); the 3.6 u ring
measured WORSE (the spots sit 0.32 under the hall plane — the bp-anchored ±3.0 band passes
them — and the stance walk_to wedges closing the last 0.87 [L r12/r13]), so the code carries
the 3.2 ring with the r12/r13 evidence in the comment.

**d1 stays `#[ignore]`d with the full ledger and the next levers in its message** (the task's
provision): (1) a LIVE CHECK on the zap's source (gdbwatch the hall's hurt volumes / the
fusebox wire's own zap radius — is the zap the port's over-deal? the game's Laser cut stands
inside 4.0 [L weapon_data 0x4E +0x1C]); (2) a stance spot walked from the ramp head that keeps
≥4 u from the wire aim point; (3) the SkyBridge crossing re-route (the base ledger's lever 2 —
the trench-floor claw-up to #658 (45.29, 5.10, 62.94) [D icon census], pinned [M
Player_WeaponFiring 0x1a5460:515-528]) stands as pinned — it needs the store-room→trench leg
whose crawl lintel stop (x 40.98) is the base report's own live check, so neither half of it
was landable this window without the main agent's render/live answers.

**The 00 Agent walk is untouched** (its ignore is real-health-14-00agent-10's), the tripbomb
cleanup untouched (optional, green), `hop_into_the_trench` and the route's shape unchanged.

## How it was verified

- `walkreal_07000014_live` whole: **8 passed, 0 failed, 2 ignored** (the d1 walk and the 00
  Agent walk, both with their ledgers) — release, `--test-threads=1`, `NF_ISO` the slim ISO,
  final tree. The 8 include the plain infiltration walk and the tripbomb route — the shared
  body edits are `!tops_up`-gated and their timelines are unchanged.
- `interactables_07000014_live` whole: **10 passed, 0 failed**.
- Determinism: the `_agent` green run reproduced byte-identically (22633 ticks, stuck 0, the
  same arrival) on two consecutive runs.
- No `crates/*/src/` code changed, so the every-walkreal rule is not triggered. The fast
  runner (`tools/test/suite.py`) cannot run in this sandbox: background processes die between
  tool calls (measured again this session — nohup+PATH and setsid attempts, 0 survivors), the
  same constraint the base report and the previous rounds recorded. The per-binary coverage
  above is the substitute.

## What's left

- **d1** (`walk_the_infiltration_to_the_delta_exit_real_damage`): the fusebox zap — the
  ledger, the tried levers and the next levers are in its ignore message on the branch. The
  walk is healthy to leg 3 (~100 health, the camper dead, the descent and the route fights
  solved); the zap's source (a live check) or a ≥4 u stance spot is the next move.
- **The SkyBridge crossing re-route** (the trench-floor claw-up): pinned but not landed — it
  needs the store-room→trench leg (the crawl lintel live check) and the claw-up's landing
  answer (the base report's live check 2). Both are main-agent items; the ignore carries them.
- The 00 Agent walk (real-health-14-00agent-10's) and the tripbomb cleanup (optional) are
  untouched.

## Live checks for the main agent

1. **The fusebox zap's source (the new blocker):** on a fresh boot, stand at the fusebox #383
   stance area — the port's stance ring lands at (−88.4…−86.5, −1.1, 31.3) — and watch what
   fires ~80 ticks after arrival: gdbwatch the hall's hurt volumes (address, access type:
   the Hurt update/damage apply; expected: the ~99-damage apply's source volume or script) —
   and whether the GAME's own cut stands inside 4.0 u of the wire without dying.
2. The base report's checks stand: (a) does the game's crawl pass the east duct's lintel at
   x ≈ 41.0 (the port stops at 40.98)? (b) at the trench-floor icon (45.27, 0.41, 62.94),
   fire the Q-Claw at #658 (45.29, 5.10, 62.94) — latch, reel, hang < 2.0, and where the
   release lands (the east walkway 2.40 or back into the trench) decides the crossing's
   shape. (c) whether the machine-room guard's pocket ever exposes his box to Bond before the
   drop — moot now that the fights answer him from the loop, but it settles the fight's cost.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the
session (no /usage, no billing panel).
