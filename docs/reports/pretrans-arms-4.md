# pretrans-arms-4 — the six interaction reds, one level per tool call

Branch s/pretrans-arms-4 = origin/s/pretrans-arms-3 525153962 (merge origin/master 0a9d39435 was
already up to date). Task: repair the six walkreal reds the PreTransCallback root zero left on the
rebased tree (the -3 report's list), the -2 round's way — per-level walk repairs with the trace, no
sim changes. Result: **4 of the 7 red tests fixed and green; 3 ledgered ignores with measured
mechanisms and next levers.** Real input only; no damage, health, pin or top-up touched; every fix
cites its run.

## Fixed (green, verified on this tree)

1. **01 escort real-damage pair** (`walk_from_the_spawn_to_the_exit_with_the_contact_real_damage`
   + `_agent`) — the -3 report's spelled-out lever plus one measured addition:
   - `DamageLog::volley_carrier_first` (OPT-IN, set by the 01 real-damage walks only): the
     damage-weighted pick + the stand's 30-tick retarget, without the cooldown drop. The NF_DMG_TRACE
     run pinned the mechanism: the 24-hp shotgun captain #7's 8-pellet volley is a SINGLE-FRAME
     blast (8 × 6.0 at d 2.3, weapon 0x1c at fight time — the ledger's 0x0 is the post-death read),
     and Bond was walked to 2.3-2.5 u of the planted guards by the legs.
   - `Walker::legs_keep_off` (OPT-IN, the 01 real-damage walks only): the nav-leg walk keeps the
     walker's own `keep_off` (6.5) to live ARMED hostiles ahead toward the waypoint — the legs
     stepped back instead of walking into the shotguns. Released before the stairwell descent: the
     walled pocket sniper #2 (y 10.2 over the mouth, never fired in any ledger [L]) sits 3.2 u over
     the final legs and every green tree walked that stretch; with the release, #2 opens fire on the
     plate leg, the determined pass drops him for 0, the coast clears and the contact finishes the
     AND. d1: passed, lowest health 46.0 (was death at tick 10682, 46 hits for 100.0). d2: passed,
     lowest 37.7.
2. **05** (`the_door_band_exit_chains_to_07000006_walked_real_damage_agent`): two walk-file fixes —
   the fight-broken-leg re-walk un-gated from d3 (master's solid bodies moved the d2 fights into the
   descent legs: the fall cost 56.7 "other" [L the run]); the shelf drop-line re-approach band is
   now 28.0 < y < 35.0 (the old p[1] < 35.0 accepted the tower base y 6.9 and fired the crawl prompt
   against the face — the 6000-tick wedge). Passed, health 38.4, other 0.0.
3. **09** (`the_alarm_switch_fails_objective_1`): the press-reach check runs BEFORE the approach
   beat in the hop loop, and the final nav node's post-arrival beat is skipped when Bond is already
   in the switch's press reach — the beat's 35-u re-dart detour was dragging him 13 u off the switch
   (nav 18 left him at dxz 1.21 IN reach) and the alarm fired behind the walk [L the run]. Passed in
   3.4 s.

## Ledgered ignores (the driver's still-red rule; each carries the mechanism + next lever)

4. **01 `following_mayhew_down_completes_the_level`**: the -3 report's "the AND-touch plate never
   fires" is the ESCORT'S DEATH — the follow ends with the contact in state 0x47 (DSTATE_Dead
   [M drone.rs:7704]), level_exit already Some (the mission-fail restart), ch 0x10 None; killed by
   GUARD crossfire during the stairwell-head clear ([M NDrone2_DealWithObjHit 0x13fff0:181-232] —
   the walk's shield models only Bond's own rounds). The walk is top-up, so this round's real-damage
   levers do not apply and the run is identical to the -3 tree's. Next lever: clear the gallery
   guards before the contact crosses under them (a forward band on the contact's route ahead).
5. **0c handoff + 0c_eastroute ground route** (both wire #282 B-end reach miss): the split-leg probe
   pins the fall EXACTLY — the diagonal leg to (-7.4,-217.2) arrives ON the strip (-8.02,11.87,-216.19)
   and the next 1.2-u step lands Bond at (-7.20,6.42,-217.38); the strip's band ends there ([D probe
   blk: x −12..−8, z −216..−218]) and the grab sphere (y 9.06..19.16) is unreachable from the yard
   floor. The -2 tree walked the same line green; master 0a9d39435 made corpses solid — the yard
   fights' drops on the strip's east run are the working theory [S unverified]. Next lever: a drone
   dump at (-7.6,-217), then route around the body or a per-walker corpse-capsule opt-in for this
   crossing. (The walkreal0c.rs diagonal split stays — it is the probe that names the fall spot and
   is behaviour-preserving: barrelrow stays green with it.)
6. **11 `walk_spawn_to_the_lower_floor_load_point_real_damage`**: the south-band doorway leg wedges
   6000 ticks at the (21.5,15.9) desk corner AFTER the leg's mid-leg dry resupply displaced Bond
   15 u east to the box (24.85,22.21) — the resumed straight line clips the desk. TRIED AND
   REVERTED: desk-lane waypoints (wrong side — wedged on the stairwell door) and a pre-leg top-up
   (the resupply's own nav wedged on the same door) [L the 11a/11b/11c runs]. Next lever: gate the
   walker_b2 leg-level dry resupply to in-place spots (per-walker opt-in), or walk the doorway leg
   from the box's side.

## What ran (this tree, release, --test-threads=1)

- walkreal_07000001_live: **6 passed, 0 failed, 2 ignored** (85.6 s) — the two real-damage escorts
  green inside the full binary; the mayhew + pre-existing 00agent ledgers are the ignores.
- walkreal_07000014_live: **7 passed, 0 failed, 3 ignored** (100.7 s) — the -2 round's other repair
  level, re-run green.
- walkreal_07000005_live: the d2 red **1 passed** (6.3 s) after the fix; walkreal_07000009_live:
  the red **1 passed**, binary **8 passed, 1 → 0 failed** across the runs (the other 8 were already
  green and stayed green in the 09a full run).
- walkreal_0700000c_barrelrow_live: **1 passed** (7.1 s) with the walkreal0c.rs split.
- walkreal_0700000c_eastroute_live: **0 passed, 1 ignored** (the ledger registers).
- ai_live **37 passed**, ai_missing_states_live **10 passed**, ai_points_live **4 passed**.
- nf-game lib: **756 passed, 0 failed, 5 ignored**.
- The fast suite (tools/test/suite.py) did NOT run on the final tree: background jobs are killed
  between tool calls in this sandbox (measured: the baseline suite died with an empty suite.out), and
  the foreground per-binary runs above filled the window. The full four-crate suite is the first
  thing the next round should start.

## Live checks for the main agent

1. [S] gdbwatch on the escort contact (skin 0x4F) during the 01 stairwell-head clear in the real
   game: which guard's round kills him (the kill attribution +0x470 last hit), and whether the
   game's own AllyLeadHide run keeps him alive through the same crossfire — settles whether the
   walk's clear order (Bond-radius band) is the right lever or the contact's route timing is.
2. [S] A drone dump at (-7.6, 12.4, -217.0) on the 0c west ledge after the yard fights: the corpse
   positions vs the strip's east run — confirms/refutes the corpse-solid theory behind the diagonal
   fall (the -2 tree walked the same line with non-solid corpses).
3. [S] In the real game at 07000009, watch the ch-124 switch's press reach: the walk now presses the
   moment Bond is in reach (dxz < 1.4) — confirm the game's own switch press fires ch 0x7C from the
   same spot (the fail objective then follows [M 0x1827d8:128-131]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy failed

All three fallbacks failed — ntfy.sh is unreachable from this sandbox (the branch push in the same
minute succeeded):
1. `curl -sS -m 20 ...` → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 urllib` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data ...` → no output, killed by the 45 s tool timeout
