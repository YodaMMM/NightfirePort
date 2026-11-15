## Your task: `real-health-00agent-1b-9`, 0700001b's 00 Agent walk survives the k1 tail wave and reaches Drake

Branch from current origin/master; if origin/s/real-health-00agent-1b-8 is not on master, merge it in first. Read docs/reports/real-health-00agent-1b-8.md there and the ignore message on walk_arrival_to_drakes_defeat_on_real_health_00agent. Take the next lever the ignore message names for the k1 tail wave fight. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700001b_live binary and interactables_0700001b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-1b-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-14T14:35Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): 21 hits to the t9134 death - the astronaut 5.0 rounds through the console phase (#9/#11 at x 43-48, #9/#13 at x 31-34), one 13.94 world blast (kind 107, damage 20, d 7.3) at (25.2,-6.9) t7541, then #16/#15/#18/#14 5.0 each and #13 3.72 at x 18-25.

Main 11-14: the named round-8 lever, CONSOLE_ORDER_HARD [5,4,3,8,6,7,1,2] (k7 first after k6), measured on 7c721b2ea: k5/k4/k3/k8/k6 latch, the walk dies at t8344 at the x46 nest (46.00,3.74) - earlier than the current order (t9134). Not shipped.

Main 11-14: a crouch press at the console phase start (hard) - Bond does not die, but the console 1 latch assert fails (line 1528, the real shot never latches it: crouched, the station shots miss the screen). Not shipped; a crouch would need to stand up for each console chain.

Main 11-14: crouch-between-shots inside the console screen loop (stand on fire, crouch after 90 quiet ticks) - byte-identical t9134 (the damage lands in the defense fights, not the screen loop). The crouch-at-start run survived only because it never latched console 1 (the walk ran out of console work, not the wave).

Main 11-14: the two world blasts (kind 107, owner -2: t4658 -12.34 at d 3.9, t7541 -13.94 at d 7.3) are the space laser [space_laser.rs state 0->1 locks target = the player position, fires record 107 at it when the script ends; the whine starts at half the script]. Radius 20 - a zero-g float cannot leave it in the charge window; 26 of the 100. The astronauts 5.0 rounds are the bulk.

---
## Report (done, 2026-11-15T01:10Z)


---
## Report from the worker (done, 2026-11-15T01:10Z)

# real-health-00agent-1b-9

Task: take the round-8 ledger's named lever (the k7-first order swap, with the con1/con2
stance-exposure measurement first) on 0700001b's 00 Agent real-health walk
(`walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent`, still
ignored), measure each change on its own real run, and write the ledger back.

## What's done

All work is in `crates/nf-view/tests/walkreal_0700001b_live.rs` (the 00 Agent walk only —
every change is inside the `hard` branch or hard-gated blocks, so the d1/d2 walks are
byte-identical: the whole `walkreal_0700001b_live` binary ran 18 passed / 0 failed / 1
ignored, and the d1/d2 real-health walks and replays take the same paths as master). The
test is still ignored (still red), but the round advanced more than any round since 6:

1. **The [expo] stance-exposure dump the round-8 ignore message asked for** (first item):
   the ring scan now prints the lined/close muzzle count for every shootable+roomed ring
   stance ([expo] lines, read-only queries). Measured: the nest ring's 27/120
   shootable+roomed stances sit at lined 0-3 (rad14 b6 (65.8,-7.4,-9.3) lined 0), while
   the round-8 x20 scan read lined 5-8 — the nest is the better tail shelter.

2. **Lever 1 (the named k7-first order swap, CONSOLE_ORDER_HARD [5,4,3,8,6,7,1,2]) measured
   RED [L /tmp/lever1_r9.log]: death 8344 (round-8: 9134), m1/m7 launched.** The run proved
   WHY: the tail consoles' held targets DO NOT EXIST during the whole approach — zero
   [cands]/[ring] dumps for con7/con1/con2, zero taps 7453..8053 — the UI scripts' first
   hold lands ~1000 ticks later than the modeled armed+359 gate. The round-8 shape's
   exposed idle at the nest then lost the 8053 fight.

3. **The dead window discovered and mapped.** missile_start [9084,9084,4175,2975,1775,
   5375,9084,5375] [L lever5] — the tail missiles' p0s start ~1009 past the modeled 8075
   (the same late-p0 pattern the ledger's m4 outlier showed, now on all three tail
   missiles), so the UI gates fire ~9443 and the first holds ~10120: NO tail console work
   exists ~7119..~10120 (~3000 ticks, not the modeled 1300). The window is pure survival.

4. **The dead-window survival shape measured, three attempts:**
   - static shelter rad 10: hp 47.7 → 0.0 in 1382 ticks [L lever2] — the hysteresis froze
     Bond while the 6-9 chasers converged and lined him through the open arcs;
   - deep shelter rad 15 (reaching the measured lined-0 east pocket): the pick froze IN the
     bank pocket (40.9,-9.0,1.7), death 8502 with clip 100 untouched [L lever3] — a static
     spot cannot beat a chaser wave anywhere in the room;
   - **the KITE (kept in the code): unconditional 30-tick re-picks (the fight cover's own
     no-hysteresis pattern) + a wave-repulsion term 0.05*(40-d) over the live wave + the
     rad-15 reach + the 3.5 u melee fights as the only fights.** The grind slowed ~2.5x:
     hp 47.7 → 27.0 over 1963 ticks at 6-10 shooters [L lever4/lever5]. Two window bugs
     found and fixed on the way: the shelter stopped at the TAIL_RESERVE boundary while
     the target was still not live, and a 0-tick break + continue spun the block forever
     without advancing the sim (the lever-4 run froze at 8973).

5. **Result [L /tmp/lever5_r9.log]: death 9134 → 10816 (+1682), k6's write ~7119, k7
   LATCHED ~10816 and m2 DISARMED** (the fights' stray shots latch the screens BEHIND the
   wave — the taps fire west through the wave onto the x20 screens, k2's UI held from
   ~10120, the stray rounds resumed its holds, ch2 rose ~10800 — the same mechanism that
   rode k6's latch in round-8). m1/m7's reads sit at 12384, 1568 ticks past the death.

## Live checks for the main agent

- Whether the tail consoles' UI scripts really hold ~10120 on a fresh boot (the port's
  missile_start 9084 vs the modeled 8075: decode the p0's first 100 ticks of m1 on the
  disc stream — is the p0's own t0 delayed by a wait event, or does the port's state-0
  exit fire late?). Address: the missile p0 scripts (SP#205-207 area), gate 30+k writes.
- The kite's wave-repulsion weight (0.05) vs the exposure terms: a live check at Agent,
  Bond kiting at the x46 nest with 8+ astronauts — record the hit rate per 300 ticks at
  repulsion 0.05 vs 0.1 (the grind rate is the round's binder).

## How it was verified

- 5 real runs of the walk itself, each a full deterministic in-process sim from the
  shuttle arrival on real input only (no top-ups, no health/ammo/channel writes):
  /tmp/lever1_r9.log .. /tmp/lever5_r9.log. Every lever was measured on its own run; the
  anti-levers (the static shelters) are documented in the ignore message.
- `walkreal_0700001b_live` whole binary: **18 passed, 0 failed, 1 ignored** (the 00 Agent
  walk itself) — includes the d1 and d2 real-health walks, byte-identical paths.
- `interactables_0700001b_live`: **9 passed, 0 failed**.
- The full four-crate suite did not fit again: this sandbox kills background jobs between
  tool calls (the nohup runner died within seconds twice) and the cold release build of
  the other three crates alone exceeds the remaining window; the change is confined to
  this one test file (`grep -l` for the changed symbols across `crates/*/tests/*.rs`
  matches only walkreal_0700001b_live.rs and interactables_0700001b_live.rs — both run
  green above; the `mp_death_hands_online.rs` "dead window" match is an unrelated netcode
  comment).

## What's left

The walk still dies at **10816** with the wave 10 strong and k1 unlatched (k2's channel
rose via the stray shots, m2 disarmed; k7's write landed ~10816 — m7 would disarm at its
12384 read). The next lever is named in the ignore message: (a) the kite's kill arm (the
aim_close pick on the lowest in-cap shooter while kiting — the wave's 4-13 riders have
4-7 aims inside the max_y 7.0 cap, each tap one kill) or (b) thin the wave at the mid
phase (the 5514/5814 zero-kill fights opened the screen window while the f5100 spawn wave
landed 4 → 7 un-thinned — gate the streak window on the wave count across the spawn).
Then the chains k7 → k1/k2 against the 12384 read. The full ledger with all evidence tags
is in the test's ignore message.

## Usage

Context window/cost figures are not visible to me (no token or cost readout is exposed in
this environment; the session ran from ~00:29 to ~02:07 UTC on 2026-11-15).

Main 11-14: the level holds NO live pickups at the walk start (no armour, no ammo) - the 100 health is the whole budget.
