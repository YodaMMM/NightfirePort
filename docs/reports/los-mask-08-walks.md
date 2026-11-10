Task: los-mask-08-walks — the 07000008, 11, 12 and 14 walks with the game's line-of-sight masks.
Branch s/los-mask-08-walks = s/los-mask-01-walk-2 (73a33ac5a) + merge origin/master (6a4fcf4ef), continued from the branch's first checkpoint (b1487cde6).

## What's done

**The 08 Operative real-damage walk is GREEN and un-ignored** (`walk_the_whole_mission_on_real_input_real_damage`, 4 passed / 0 failed / 2 ignored on the whole binary). The walk-2 ledger's three levers were applied, with one material finding:

1. **The terminal walk-out lever is retired by the geometry.** The arm was implemented as the ledger described (a 16-ray azimuth sweep at eye height, the farthest clear ray preferring the machine's bearing, real forward input, grounded only) and it did not move Bond one unit in 46 forward ticks: the ride's end (111.08, 13.52, −179.72) has NO floor outside (the file's own probe: `floor_below` None at every candidate 2-8 u out — the terminal hangs over the valley, kill floor y −24 [D #42/#43/#44]) and the sweep's "clear 30 u" rays (az 0.00/0.39/2.36/2.75/3.14/5.89 all t1.00) are the SHOT-OUT PANE holes over the valley, not a doorway [L the walk-out run on this branch]. There is no open pad to walk to; the game's player fights from the carriage.
2. **The 05 pre-engage stands** (fight_pkg-gated = real-damage/d2+/topped-up walks untouched): the station deck after "the station hall", the secure room before the safe, the deck pair after the walkway-head crossing, and the platform-mouth-west 18 u visible stand widened from Agent-only to every real-damage walk (the masked-tree Operative rerun had #19/#27 still on the platform during the Hind fight, draining 7.2+7.2 from ~60 u).
3. **The terminal strafe dodge — the lever that greens the walk.** While any live hostile round is in the air and closing within 60 u, Bond sidesteps out of its lane with real strafe input (`Input.strafe`), the side picked by the eye-height ray with more open air. Evidence [L this branch's runs]: the death ledger's −20.09/−30.99/−24.15 world hits were the machine's own rockets detonating on the wall band (the −24.15 landed on the death tick, 11 ticks after rocket 1's impact — the re-aim gap); with the dodge the world's share fell 39.4 → 14.8 and Bond finishes at **24.6 health** (38 hits for 75.4, 20 return-fire fights, no falls). A 14-u dodge window was tried first and never moved him: the machine's rocket crosses it in ~4 ticks.

**The Agent and 00 Agent sisters stay ignored, re-ledgered with this session's runs.** The Agent walk dies at the same bottom terminal (tick 29625): d2's DiffMod·299 = 448.5-hp machine needs ~5 rocket hits (4 launched by t491, 199.5 done) so the fight is 2-3× the Operative's length under continuous fire, plus the masked station-deck snipers #14 (11.2), #3 (5.6), #11 (5.6) — alive through the whole walk, draining from ~100 u during the Hind fight. The deck-pair stand found NO visible guard at the walkway head (t1771) — the pair's enable plate needs pinning before a pre-engage can meet them. 00 Agent still dies (tick 26314, 23 hits for 100.0 — the ledger's upstream-bleed class). Next levers are in their ignore messages.

**11, 12 and 14 were not reached this session.** The 08 levers took the window (the walk-out lever had to be tried, disproven and replaced before the dodge could be found). Their ignores and re-time levers (the 03-way mid-floor waypoint + pinned fight sites) are unchanged on the branch; the next session continues there. Nothing under their route files was touched.

## How it was verified

- Every change is in `crates/nf-view/tests/walkreal_07000008_live.rs` (test file only — no src changes on this branch). All new tactics are gated `fight_pkg = difficulty >= 2 || !tops_up()`, so the topped-up walks are byte-exact: the binary's topped-up whole-mission walk passed unchanged.
- Whole binary `walkreal_07000008_live`: **4 passed / 0 failed / 2 ignored** (the un-ignored Operative real-damage walk, the topped-up walk, the ally-noise test, the 0x59↔0x5c thrash test; the Agent and 00 Agent sisters are the 2 ignored).
- `nf-game --lib`: **760 passed / 0 failed / 4 ignored**.
- Runs were `cargo test --offline --release -j2 -p nf-view --test walkreal_07000008_live` with NF_ISO on the slim on-foot ISO, `--test-threads=1` per binary, real input from the level start, no teleports, no damage/health/ammo writes; every tactic cites its run evidence ([L]) or decompile addresses already pinned in the file.
- The fast suite (`tools/test/suite.py`) was started per instructions but could not run in this sandbox (fresh clone: it failed at once on `no matching package named anyhow` before `cargo fetch`; the fetch was run afterwards). It was not re-run on the final tree — the session's remaining window went to the named binaries. Not run: nf-formats/nf-audio/nf-view crates beyond the 08 binary, the other walkreal binaries (their code paths are untouched by this branch), `npc_doors_live`/`fix_guard_misc_live` (no src changes, so not triggered by rule 10).

## What's left

1. The Agent sister: pin the deck pair's enable plate, place the pre-engage where they are visible, then the descent re-tune (the ignore message names the levers in order).
2. The 00 Agent sister: the upstream bleed (the walkway pair 36.0, the stairwell/descent fights) — the ledger's descent re-tune.
3. The 11/12/14 re-time levers in their ignore messages (mid-floor waypoints + pinned fight sites, gated per level and difficulty), then their walkreal binaries.
4. The fast suite on the final tree.

## Live checks for the main agent

1. On 07000008, at the bottom terminal (111.08, 13.5, −179.72): which hurt volume or live wire ticks −1.2 every 20-25 frames while Bond stands at the ride's end (fieldprobe on the ride-end spot) — the dodge reduced the zaps' exposure but the source volume is still unnamed; it decides whether a 1-2 u stand shift inside the carriage is the game's own answer.
2. On 07000008, the Hind fight as the game plays it: does the real player fight from INSIDE the carriage (the panes shot out) — gdbwatch on the machine's rockets' Explode_Propagate contact points during a playthrough would confirm the wall-band detonation distances (this branch measured 2.2-5.8 u) and whether the real blast falloff at those distances matches the port's.
3. On 07000012/14, the ledgered wedge spots ((16.66-17.15, 1.39, 3.19) target (17.48, 0.90, 1.51); the tilted sill at (−38.06, 0.71, 33.98)) — whether the game's own walk/step-up crosses them (the walker's step-up gate on the scoped floor's slope band is the 14 lever).

## Usage

not visible to me — this session's driver shows no token, cost or context-window figures to the worker (the IM gateway metadata carries only session/chat ids; no usage counters appeared in any tool result).
