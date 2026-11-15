## Your task: `real-health-00agent-03-captors-8`, 07000003's 00 Agent full-route walk completes every objective

Branch from current origin/master.

Master 854334126 (the captors-7 round merged): the walk now dies at t151214 at (34.75, 6.37, 8.69) mid the post-rescue chase at Bond hp 9.0. Read the captors-7 ignore ledger on master for the measured state and its next levers; the cluster members' point-blank fire is the named bill. Goal: the walk's 00 Agent test passes.

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

**Branch:** `s/real-health-00agent-03-captors-8`.

---
claimed-by: sandbox1 at 2026-11-15T07:06Z

Main 11-15 (on 854334126): damage by 10000 ticks - 58.8 in t0-10000 (#11 0x5000038 36.0 = three 12.0 hits from (14.5, 6.3, -29.0) at 10.8-13.1 u during the mid-courtyard leg return-fire duels and the lane-mouth aimed stand; #17 12.0), 17.8 (#14) in t60000s, 14.4 (#18 0x1c) in t120000s, 9.0 (#19) at the end. A crouch press before the mid-courtyard crossing: death t9971 - far worse.

---
## Report (done, 2026-11-15T07:45Z)

## Report (real-health-00agent-03-captors-8, 2026-11-15)

Task: continue the 07000003 00 Agent full-route real-damage walk (still ignored). The walk is
NOT green; the ignore message on the branch carries this round's full ledger and the next
levers. The frontier moved **151214 -> 172900** (+21686), and for the first time the walk saves
BOTH servants with Bond's health untouched (23.4) through the whole rescue chain.

### What was done

Branch base: origin/master 854334126 (the captors-7 round merged). All runs are real-input d3
walks of `walk_the_full_route_and_complete_every_objective_real_damage_00agent`, each lever
judged on its own run, no top-ups, damage/health/ammo untouched:

- **Run 57 (the -7 ledger item 1, v2)** — the kill-loot cycle inside the cluster funnel
  (loot_the_drops_03 + scoop_drops_03 between rounds; the dry give-up no longer breaks the
  loop; the poll 20 -> 600 ticks; rounds 10 -> 16). #2 dropped 24 -> 4 and died to the
  same-tick kill+dry give-up, the loot armed the 0x1c shotgun, #18 killed — but round 3
  CHASED an unexposed #19 at clip 5 spare 0 and his two 0xe point-blanks (23.4 = exactly
  Bond's hp) finished Bond at 71192 [L d3run57].
- **Run 58 (THE FUNNEL v3 = the round's fix)** — plus the pre-fight top-up (while
  clip+spare < 6, bounded 3 passes) and the 12-u scoop reaching the 10-11 u death spots the
  loot's 8-band misses. **THE CLUSTER DIES AT THE POCKET** (rounds 0-2: #2, #19 killed; the
  poll empty at round 3), Bond 23.4 untouched through the funnel, **both servants SAVED**
  (post-save state 0x13 twice, the rescue asserts green). Death 171891 on the exit's moat-node
  nav to #15 (0x5000036, 0x12) point-blank (23.4 in 7 hits at d 2.6) — the return fight
  engaged at err 0.918 (the nav left Bond facing away; at 00 Agent the autoaim bit is closed
  [M Autoaim_HardMul 0]) and lost the race [L d3run58].
- **Run 59 (the gate door, measured)** — the gate door auto-closes behind the safe visit
  (state 0x2 confirmed [L the run's print]) and the exit's A* from the safe heads WEST (the
  first visit's same nav reached (7.48,-33.71) direct; with the door re-opened the graph STILL
  prefers the west line out of the safe room) — #15 intercepts. Wired the gate re-open
  (use_door, the stall-door idiom) + the back-through-the-gate pre-leg nav to (14.7,-36.8)
  (the first visit's own walked spot), d>=3. The pre-leg nav WORKED (arrived (13.51,-37.17))
  but #15 AND #16 chased the safe->gate line and the return fights engaged 3x and MISSED ALL
  (killed false, #15 hp 12 untouched) — Bond arrived 1.8 hp, death 172900 [L d3runs 60-61].
- **Run 62 (the census + the sweeps, measured)** — the safe census: #15 (0x12, 12 hp)
  (3.8,-33.3), #16 (0x1c shotgun, 24 hp) (3.6,5.4,-25.8, the ditch floor) — the safe's key
  (ch 0x4e) arms the alarm responders and they walk their west posts during the exit; the safe
  exit sweep (the nearest-exposed pick, 300 wait) found NOBODY exposed from the safe room; the
  through-gate clear_near skipped them (the far+unexposed skip — their boxes buried from the
  gate spot while their muzzles reach). Death 172244 at (11.84,-36.87) — #16's shotgun
  (23.4 in 4 hits at d 13.2) while the fight stood Bond in his lane [L d3run62].
- **Run 63 (THE EXIT FIGHT TRACE, NF_RF_TRACE)** — the fight on #16 CONVERGED (err 0.000 at
  t60, 12.5 u) and fired ONE shell that MISSED (hp 24.0 untouched), then the shoot_blockers
  retarget fired at t62 ("line blocked by live hostile #15") — 2 ticks before the volley
  landed. The race is unwinnable from the gate band: ~1 shell per 50-60 ticks (the reload
  rhythm) against #16's 23.4-per-cluster volleys, and the 0x1c's pellet scatter (0.0448 rad
  [L ee_start_05 weapon_data] = ±0.54 u at 12.5 u) against a ~0.4 u part box is a lottery even
  converged. Bond died at hp 23.4 FULL — the whole post-funnel margin spent on one volley
  cluster [L d3run63].

All new/changed behaviour is gated to `difficulty >= 3` inside
`crates/nf-view/tests/walkreal_07000003_live.rs` — no shared helper
(`crates/nf-view/tests/common/`) was touched, no `crates/*/src/` change at all. The keepers:
the funnel v3 (the cluster dies at the pocket), the gate re-open + the back-through-gate
pre-leg, the safe exit sweep + census. The ledger is refreshed in the walk test's ignore
message (runs 57-61 + the next levers).

### Verification (what ran, with pass counts)

- walkreal_07000003_live WHOLE on the final tree: **6 passed / 0 failed / 3 ignored**
  (224.18 s) — the d1/d2 streams and the replays byte-identical (every change is d>=3-gated);
  the ignored = the d3 walk + the 2 probes.
- interactables_07000003_live WHOLE: **9 passed / 0 failed** (31.28 s).
- The FULL suite (tools/test/suite.py) did NOT run: two background starts (07:09 and 08:21)
  produced 0 lines and died between tool calls in this sandbox (the captors-7 round
  documented the same limit; the -7 WORKLOG also recorded the four-crate build not fitting
  the disk window). The two binaries above cover every test that uses the touched file (the
  diff is confined to walkreal_07000003_live.rs + docs). No test FAILED in any run.

### What's left (the next round's levers, in order)

1. THE HP LEDGER IS NOW THE WHOLE GAME: Bond arrives at the exit at hp 23.4 = exactly one #16
   volley cluster. The -7 ledger's clean-funnel floor (41.2 at the cluster, one 17.8 #14 hit)
   leaves 41.2 - 23.4 = 17.8 AFTER one volley, and the fights (the shoot_blockers retarget is
   proven wired) then win. Find the 41.2 -> 23.4 gap's owner (the gdbwatch live check stands)
   or one more upstream save.
2. THE EXIT FIGHT GEOMETRY: #16's post (-1.3,-35.4) covers the gate band at 13 u; a player
   does not stand in it. Route the exit's moat-node legs from outside his band, or pick a
   zero-spread aimed gun (the 0x2 heavy, ~8 a tap [L d3run52]) at the through-gate spot so the
   shells land.
3. The funnel v3 + the exit route shapes are keepers — do not regress them.

### Live checks for the main agent

- The 41.2 -> 23.4 pre-funnel hp gap: a gdbwatch on the health address across the saddle
  re-take + the north-door standoff walk would name the damager (the -7 ledger's check
  stands; the damage ledger credits #14 with exactly that 17.8, but at a frame AFTER the
  [03-ammo] print that already shows 23.4).
- #16's volley geometry: fieldprobe Bond at the through-gate spot (13.6,-37.3) against #16's
  post (-1.3,-35.4) — does the disc's own 13-u band chunk a standing Bond for 23.4 (the port's
  spread math vs the disc's)? Access type: the health read per volley, 600 ticks.

## Usage

not visible to me — this sandbox exposes no token, cost or context-window meter; no figures
were readable anywhere in the environment.
