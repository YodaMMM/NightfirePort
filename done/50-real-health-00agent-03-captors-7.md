## Your task: `real-health-00agent-03-captors-7`, 07000003's 00 Agent full-route walk completes every objective

Branch from current origin/master.

Master: death t74135 at the east lawn saddle (23.18, 5.50, 7.20): Bond arrives with the P2K 3+13 (the only rounds left; every 0x12/0xe/0x1c drop near the route expires on its lifetime before a loot pass), the saddle wake sweep spends them, and #7 (0x5000039, 0x12, hp 4) at (11.4, 10.1, -16.1) lands 1.8 a hit from 26.5 u. Measured: a wider loot radius, a dry-wheel cycle skip, the saddle armour (behind the door (22.3, 10.9)), stands from (14..20, 0) - no gain; the captors-5/6 branch kills #7 but dies earlier (t67488). Goal: rounds that survive to the saddle (or the saddle crossed without the sweep) and the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-03-captors-7`.

---
claimed-by: sandbox1 at 2026-11-15T05:34Z

---
## Report (done, 2026-11-15T06:51Z)


---
## Report from the worker (done, 2026-11-15T06:51Z)

## Report (real-health-00agent-03-captors-7, 2026-11-15)

Task: continue the 07000003 00 Agent full-route real-damage walk (still ignored). The walk is
NOT green; the ignore message on the branch carries this round's full ledger and the next levers.
The frontier moved **67488 -> 151214** (master's tree dies 74135 dry at the saddle; captors-6
died 67488 to #14), and the si-1 servant rescue SAVED for the first time past the trio wake.

### What was done

Branch base: origin/master 4f3802517 with origin/s/real-health-00agent-03-captors-6 merged (its
walkreal file carries the -6 ledger; the WORKLOG conflict was resolved to the union superset).
All runs are real-input d3 walks of
`walk_the_full_route_and_complete_every_objective_real_damage_00agent`, each lever judged on its
own run, no top-ups, damage/health/ammo untouched:

- **Run 50 (baseline)** — the merged tree reproduces the -6 ledger: death 67870 mid mouth nav.
  The new fact: the mouth nav's stuck handler E-opened the (39.0,12.9) door MID-NAV and the hits
  began during that leg (frame 67499) — the door OPEN CHANNEL is the #14 wake [L d3run50].
- **Run 51 (the route probe, cheap, no walking)** — `probe_mouth_approach_routes` prints every
  nav route and the nav's own move_test verdicts: EVERY route to the mouth crosses the
  (39.0,12.9) doorway band (the only descent is nodes (39.93,13.55) -> (39.90,12.23) through the
  leaf); the east strip x 40.0-41.0 and the direct south-lawn chain are ALL blocked. The doorway
  is the only way down — the #14 fight is forced.
- **Run 52 (the funnel v1, tried)** — the trio-funnel shape at the north door (standoff, the 0x2
  heavy gun cycled up, pre-raised aim, E-open, the aimed sweep): one ~8-dmg tap landed (12 -> 4)
  but #14 flanked east to point-blank d 1.5 and burst 41.2 in 2 — death 67715. The aimed stand's
  STILL gate cannot tap a charger.
- **Run 53 (THE #14 FUNNEL v2 = the fix)** — the captor-chase shape: after the E-open, a plain
  `w.shoot` loop (no still gate) taps him THROUGH the charge. #14 died for ONE 17.8 hit (Bond
  23.4 -> 5.6, survived) and the walk PASSED the mouth for the first time.
- **Run 54 (the new blocker, measured)** — the mouth nav ARRIVES without touching door 8
  ((35.91,4.34)); the settle loop's `at_pocket` reads 0 (the sleeping cluster never walked), the
  d3 sweep finds nobody exposed, and the servant nav's own door-8 E-opens wake the cluster
  MID-WALK — #18's point-blank shotgun volley (23.4 in 12 hits at d 1.0) — death 107736.
- **Run 55 (the cluster funnel v1, tried)** — E-open door 8 at the mouth + a 600-tick poll found
  NOBODY: the pops do NOT ride the door-8 open channel alone. The walk still SAVED SERVANT 1
  (post-save state 0x13, ch 0x50 on — the rescue asserts passed for the first time past the
  wake) before dying 106170 to the awake #19 point-blank during the fight chase.
- **Run 56 (THE CLUSTER FUNNEL v2 = the current shape)** — E-open door 8, walk INTO the pocket
  (38.7,0.2) (the pops ride the interior entry: #2 was found the moment Bond stood inside), the
  patient shoot loop (10 rounds, 600-tick polls, r 16): #2 dropped 24 -> 4, servant 1 SAVED
  again, death 151214 mid the post-rescue chase at Bond hp 9.0.
- **The load probe** — the whole map's guards load `enabled=false, present=true, state 0x1`
  [L the probe dump]; door 8: state 0x2, pos01 0.50, ch_lock 0x0.

All new/changed behaviour is gated to `difficulty >= 3` inside
`crates/nf-view/tests/walkreal_07000003_live.rs` — no shared helper
(`crates/nf-view/tests/common/`) was touched, no `crates/*/src/` change at all. Two cheap
`#[ignore]`d probes were added (routes + the load dump) for the next round.

### Verification (what ran, with pass counts)

- walkreal_07000003_live WHOLE on the final tree: **6 passed / 0 failed / 3 ignored**
  (316.95 s) — the byte-identity proof for every other stream in the binary (the d1/d2 walks
  and the replays stayed green; the ignored = the d3 walk + the 2 probes).
- interactables_07000003_live WHOLE: **9 passed / 0 failed** (44.75 s).
- The FULL suite (tools/test/suite.py) did NOT run: the four-crate build does not fit this
  sandbox's 9.9 G disk (the build was killed at 4.8 G target / 693 M free; the earlier attempt
  also failed on `-lasound` until the ALSA runtime lib landed in a user prefix). The two
  binaries above cover every test that uses the touched code (the diff is confined to
  walkreal_07000003_live.rs + docs). No test FAILED in any run.

### What's left (the next round's levers, in order)

1. FINISH THE CLUSTER AT THE POCKET: the shoot loop needs the kill-loot cycle INSIDE it (each
   drop carries the guard's own weapon + clip — loot_the_drops_03 + scoop_drops_03 between
   rounds; 3 guards = 48 hp vs Bond's ~11 spare rounds is not enough without the drops), and the
   loop must keep shooting while ANY cluster member lives (the break on `!w.shoot` lost #2 at
   hp 4).
2. THE HP LEDGER: Bond enters the endgame at 23.4 and dies at 9.0. Trace the phase-boundary
   health prints (the 41.2 -> 23.4 gap before the funnel needs an owner; #14's one 17.8 funnel
   hit is the measured floor — a clean funnel = 41.2 at the cluster) and cut the pre-saddle 58.8
   (the pre-crossing stands' 36.0 #11 + the crossing's 24.0 #11 are the two big lines; the
   driver's measured refutations stand: wider loot radius / dry-wheel skip / saddle armour /
   (14..20,0) stands all no gain).
3. THE POST-RESCUE CHASES: the servants'-ground fight's flank navs walk Bond into the surviving
   cluster members' point-blank (both the 106170 and 151214 deaths); with the cluster dead at
   the pocket (lever 1) the chase walks empty ground.
4. The older levers stand: the 87000-tick (35.6,8.7) door stall on the post-rescue west-lawn
   leg, the exit-lawn #16 post-kill volley, #7's upper-floor drop (11.41,9.09,-15.55).

### Live checks for the main agent

- What EXACTLY pops the door-8 cluster: stand Bond outside door 8, E-open it, watch #18/#2/#19's
  enable flip for 600 ticks (the funnel poll says NOT the open channel); then step 1 u inside
  and watch again (run 56 says the interior entry pops them). Addresses: the drones' enable at
  Drone+0xF4 (the port's `enabled` field), the door at (35.62,6.51,2.71).
- The 41.2 -> 23.4 pre-funnel hp gap: a gdbwatch on the health address across the saddle
  re-take + the standoff walk would name the damager (the damage ledger credits #14 with
  exactly that 17.8, but at a frame AFTER the [03-ammo] print that already shows 23.4).

## Usage

not visible to me — this sandbox exposes no token, cost or context-window meter; no figures were
readable anywhere in the environment.
