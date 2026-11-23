# retune-nb-16-8 — 16 chasers-defeat 00 Agent real-damage walk, round 8

Branch `s/retune-nb-16-8`, from origin/master 7562f1e70, with `origin/s/retune-nb-16-7`
merged in first per the task (16-7 was not on master; the merge was clean — master's side for
README/gadget-census, the 16-7 side for the walk file's `#[ignore]` ledger and the docs).

Task: take the next untried lever from the ignore message (the fights' SPEED at the ring), judge
each lever on its own real run. Real input only, no top-ups, never lower damage or health, never
change guard code. Helpers stay gated to the level and the difficulty (every new gate evaluates
`difficulty >= 3 && !w.tops_up()` — exactly the d3 real-damage defeat walk; the d1 walk, the d2
walk, the punch walk, the tops-up sisters and the door tests are missed by construction). The
walk is still red; the refreshed ledger and the next levers are in the ignore message.

## Done — the SMG-speed lever landed, the walk reached the chaser stage (the furthest measured point)

- **run 1 — the ledger's lever as named (prefer the owned 0x1a in the clears' loot)**: the
  stream came back BYTE-IDENTICAL to the 16-7 run 10 (death t1620 at (10.70,6.75,−77.94), 22
  hits for 100.0). Measured why: the ring-phase clears BAIL at their own >15 u close gate —
  the ring east stretch's guards stand 20–33 u out (#3 24.4–26.0 u, #4 20.5–21.5 u, #5 32.5 u
  [L the run's NF_DMG_TRACE: no `[clear]` line ever printed, every clear returned at the
  distance arm]) — so the clear's kill-loot never fires before the ring-1 pour and the SMG
  preference never arms. The premise needed the loot to EXIST on this stream first.
- **run 2 — the ring-1 loot detour, nav-only**: the walk claimed arrived 4.3 u SHORT of the
  drop (ended (12.12,6.75,−77.19), the drop at (7.8,5.7,−77.0) — the NE guard #0's reactive
  kill at t1345 dropped his 0x1a, 12 rounds, 7.3 u off the ring-1 line), the take never fired,
  owns unchanged, the swap never armed. Death t1705 (the pour during the detour + the same
  fatal fight).
- **run 3 — the take finished with return fire ON**: the second reactive fight opened ON the
  take hop itself (~t1591), the stand's no-displacement law froze Bond 2.7 u short of the
  drop, the take never completed. Death t1705, same spot class.
- **runs 4 — the detour completed (return fire OFF, the pocket-dash idiom + the direct
  walk_to finisher at 0.35)**: the take fired (in hand 0x1a clip 30 spare 0 at health 37.0),
  the swap armed, and the SMG fights killed #4 at t1660 (60 ticks), #3 at t1804, #5 at t2062
  — the ring phase's WHOLE pourer set. The walk crossed the ring, the station, the #596
  armour, the returns and the descent — then died t4996 at (14.24,0.91,−90.38) to the ramp
  foot's kind-27 world blast (50.0, radius 4.0) while a DRY SMG (clip 0 spare 0 at the lane
  head [L the trace]) stood fighting drone 16 — the P2K's 34 spare sat unused in the carry.
- **run 5 — switch_dry added (d3-gated, the fight's own opt-in)**: the dry fights now cycle to
  the next firearm with rounds (the P2K) instead of standing on an R that can never load. The
  walk killed drone #1 (t4877) and drone #16 (t5257) on the lane, crossed the doors, the
  hand-back stand answered the pit soldiers at +0, and died **t9383 AT the shelter entry**
  (+322 into the finale, (−14.50,−22.00,−170.18), health 9.0): the shelter-door kind-27
  barrel at (−12.4,−21.4,−169.4), d 2.4, radius 4.0, dealt the last 9.0 — its fuse lit during
  the hand-back stand (ch 0x27/0x28 true at the blast [L the trace]). This is the FURTHEST
  the d3 walk has ever measured (the prior best was the 12872-ledger chaser-stage arrival at
  9.5–19.5; the 16-7 tree died at t1620/t1705 in the ring phase).
- **run 6 — the ignore message's lever (c), the lane batch** (skip the not-exposed pick,
  widen the band z −84..−132 to catch #3/#4's strafe spots, 6 iterations): MEASURED WORSE —
  death **t1383 AT the stand**. The batch's own DURATION is the exposure: the stand stood the
  lane east ~230 ticks while the NE ring guard #0 (9.0,6.7,−78.7 — OUT of any gallery band,
  d 6.4) poured 3.5 a hit the whole time and the hunter #1's line joined from 29.7 u — 24
  hits for 96.5. Reverted with the ledger note (the batch arms do not retry).
- **run 7 — the determinism check**: the reverted tree reproduced run 5 BYTE-IDENTICALLY
  (t9383, the same spot, the same hit list) — the final composition is runs 5/7.

## How it was verified

- Every lever was judged on its own real run through the real input path (NF_DMG_TRACE), the
  traces kept in the session (not committed). The determinism of the final shape was verified
  by the identical repeat (run 7 = run 5).
- The diff touches ONLY `crates/nf-view/tests/walkreal_07000016_live.rs` and the docs — no
  `crates/*/src/` and no `crates/nf-view/tests/common/` files (the new helpers
  `prefer_the_dropped_smg_16` / `loot_the_ring_drops_16` are file-local; `switch_dry` is an
  EXISTING shared opt-in set by this test's d3 gate only). The sisters' streams are gated
  untouched by construction, and the whole binary's pass confirms it.
- **The whole `walkreal_07000016_live` binary at the final tree: 18 passed / 0 failed / 1
  ignored** (139.8 s, `--test-threads=1`, NF_ISO the onfoot ISO) — the d1 real-damage walk,
  the d2 real-damage walk, both tops-up sisters, the punch pin, the door/interactable tests
  all green; the 1 ignored is the d3 walk (the ledgered one).
- **nf-game + nf-formats + nf-audio at `--test-threads=1`: 260 passed / 2 failed / 0
  ignored.** The 2 failures are the `crp_mesh_disc` disc-content pair
  (`every_track_and_car_crp_yields_meshes`, `every_track_crp_yields_collision_under_its_road_network`)
  reading `/DRIVING/MIS01.VIV: too small for a BIGF header` — the driving content is absent
  from the onfoot ISO (the documented slim-ISO limitation; nf-formats code is untouched by
  this branch).
- **The fast suite runner**: `tools/test/suite.py` was started in the background per the task
  (`setsid nohup`, the first minutes of the session) — the sandbox killed it between tool
  calls before a single line (suite.out empty, no process left; the same wall the −5/−6/−7
  reports documented). The foreground four-crate run did not fit the remaining window after
  the lever session; what ran is listed above. Since the diff touches only the 016 walk test
  file and docs, no other crate's or level's test binary can move (the affected set is the
  one binary reported green).

## What's left

- The d3 walk (above): one session of the shelter-barrel lever (the ignore message names the
  two candidates in order: the hand-back stand's line/volume probe with a live check for the
  main agent naming what ignites the barrel; then the abort_on_wound/finish_fights
  composition as a shared-helper opt-in), then the pit-squad stage unchanged from the 16-5
  green shape, then un-ignore.

## Live checks for the main agent

- The shelter barrel at (−12.4,−21.4,−169.4) (kind 27, radius 4.0, damage 50.0): watch its
  state during the hand-back stand (Bond at (2.85,−23.18,−177.83), the soldiers answered
  there) — gdbwatch on the barrel's health/state field from the hand-back to the shelter
  entry (the blast went off at +322 into the finale) names WHAT ignites it (a soldier's
  stray round vs Bond's own muzzle line vs a scripted channel). Expected: the ignition tick
  and the owner.
- The ring-1 drop take: fieldprobe the pickup at (7.8,5.7,−77.0) during the loot detour —
  confirm the walk-over take (Pickup_Handler 0x18e650) consumes it at the 0.35 arrival.

## Usage

not visible to me

## ntfy
