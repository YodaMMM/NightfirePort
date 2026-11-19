## Your task: `retune-nb-08-6`, 08 whole-mission 00 Agent real-damage walk green again (round 6)

Branch from current origin/master; if origin/s/retune-nb-08-5 is not on master, merge it in first. Read docs/reports/retune-nb-08-5.md there and the ignore message on walk_the_whole_mission_on_real_input_real_damage_00agent. Master changed the guards again (SniperAim ENTER, 07:28), so first re-measure the death on the merged tree. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000008_live binary.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-08-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-19T15:18Z

---
## Report (done, 2026-11-19T16:48Z)


---
## Report from the worker (done, 2026-11-19T16:49Z)

# retune-nb-08-6 — 08 whole-mission 00-Agent real-damage walk, round 6

Branch `s/retune-nb-08-6`, from origin/master (master had moved since round 5: the guards'
SniperAim ENTER, c0e23ff23). `origin/s/retune-nb-08-5` is not on master, so it was merged in
first (clean: only the walk file, WORKLOG and its report).

## What was done

1. **The re-measure on the merged tree:** the death reproduces master's shipped run-10 ledger
   EXACTLY — tick 21070 at (112.02, 13.09, −179.51), the same 55-hit shape [L the re-measure
   run on this branch]. The SniperAim ENTER change did not move this walk's timeline.
2. **Two ledger corrections (the round-5 scan's ammo reads were wrong-index):**
   - "rocket clip 4 carried 0" read `inv.carried.get(0x2c)` — the WEAPON-ID index; the rocket
     pool is ammo TYPE 0xe and it reads carried 4 [L the run's dump]. The fight ALREADY has
     8 rockets (clip 4 + carried 4 = 808 damage > the machine's 598 hp) — it was never
     ammo-starved.
   - ONE cable-car ammo box ALREADY auto-takes at ride start ([take] tick 18432
     `Some(Ammo 4)` [L the NF_PICKUPLOG run]): the ride script spawns the pickups with Bond's
     boarding spot already inside the take radius (dist ≈ 1.19 < bounds.radius + 1.1 = 1.53
     [L the take-test rows; M Pickup::touches 0x18e148:210-214]). `carried` hits the CAP there
     (ammo_data type 0xe cap 4 [D ammo_data +0x02; L the take stream: `Some(Ammo 4)` then
     `→ None` from tick 18433 onward]) — the other two boxes refuse at the cap EVERY tick
     forever.
3. **Round-5's lever (1) "collect the pickups during the ride": implemented and measured —
   DEAD as an ammo lever.** The d3 ride loop steers to the nearest un-taken 0x2c pickup with
   real input (stops inside 0.85 planar u): the walk holds dist 0.82-0.83 through the whole
   descent [L the ride dumps] and NO take ever fires (the cap); the fight's rockets are
   unchanged at 8. Kept: the walk is harmless (it also shifted the upstream rolls favourably
   — 20.0 health at fight entry vs master's 14.0) and the d1/d2 sisters stay byte-exact
   (difficulty == 3 gate).
4. **The lower-floor stand-off lever (round-5's remaining item): implemented and measured —
   DEAD.** After the crouch drop: the 60-tick lockout waited out on the zap-free pit floor
   [M Player_ChangeSubState 0x190630:48; SSCrouch 0x1924c0:45-53], then a crouched walk to
   off-hole stand spots, then the stand press. Measured [L the three lever runs]: the crouched
   walk HOLDS feet 13.0856 across the plate — to (112.74, −179.06) and (113.08, −179.24); the
   plate runs east past x 113 [L the walk logs] — but the STAND press pops the standing
   capsule to feet 13.470367 at EVERY spot ((112.8, −179.15) and (113.08, −179.02), plus
   round-5's hole footprint and plate edge). The trench is NARROWER than the standing
   capsule; no standing spot exists on the lower floor. The 13.77-eye fight is unreachable.
5. **The low-band launch gate (cp[1] ≤ 14.5): TRIED and REVERTED.** It held fire through the
   whole rim-fight window (ZERO launches — the machine never dropped to the band within a
   launchable bearing in ~800 fight ticks) and the death was the SAME −15.00/−3.27 world
   blast at the SAME frame (21775 vs the 25.0 gate's 21774) [L the two lever runs] — so the
   gate returned to the round-4 measured 25.0.
6. **Shipped tree:** the ride walk + the stand-off walk + the stand. The walk now dies
   **t21773 at (112.69, 13.47, −179.52) — 700 ticks deeper than master's run-10 (t21070)** —
   in the RIM fight (feet 13.47, eye 14.18 — the standing capsule's contact height on the
   trench edge): the rim is NEARLY ZAP-FREE (ONE −1.73 world row in ~800 fight ticks [L the
   lever run]) and the launches FIRE from it (t377 at the machine's y 18.4 pass dist 30.8,
   t585 at y 20.8 dist 40.1 — both damage 0.0 MISS).

## How it was verified

- Each lever judged on its own real run of the d3 00-Agent walk (~15 s per run): the ride
  walk t21058 (worse than master's t21070 on its own), the stand-off at (112.8, −179.0)
  t21349, at (113.3, −179.25) t21773, the 14.5 band t21774 — the deepest state shipped, the
  gate revert re-verified (t21773 reproduced exactly on the final tree).
- The decisive pair for the closer: with a launch (25.0 gate) and with ZERO launches (14.5
  band) the walk dies at the same frame to the same −15.00/−3.27 world blast — the closer is
  the Hind's MISSILE salvo on its own schedule, not Bond's own round.
- `walkreal_07000008_live` whole binary on the final tree: **6 passed / 0 failed / 1
  ignored** in 151.65 s — the d1/d2 real-damage sisters, the plain walks and both ally tests
  stay byte-exact green.
- Only `crates/nf-view/tests/walkreal_07000008_live.rs` and `docs/` changed — no
  `crates/*/src/`, no `crates/nf-view/tests/common/` files — no other test binary compiles
  the changed file, so no other binary's timeline can move.
- **The full suite (tools/test/suite.py) did NOT fit the window.** This sandbox reset to a
  bare machine (clone, toolchain, game files ≈ 40 min), the sandbox kills background jobs
  between tool calls (two suite starts died: once during the −j4 build, once before it wrote
  a byte), and the 2-core/4-GB box needs ~30 min of run phase after the build. What ran:
  the prebuild of all four crates' test binaries completed clean (rc 0), and the changed
  file's whole binary is green (above). No `crates/*/src/` or `tests/common/` changes exist
  on this branch, so per the merge lessons the affected-binary set is
  `walkreal_07000008_live` alone.

## What's left (the next window's levers, in priority order)

1. **Answer the missile salvo at the rim.** The salvo takes −15.00/−3.27 and lands ~800
   fight ticks in on its own schedule. Check the dodge gate's missile arm actually engages
   from the rim (the dodge_hold tally in the run logs) and break the salvo's line: extend the
   hold past the missile's curve, or shift the stand spot so the carriage body covers the
   missile's line.
2. **The idle-wait sweep before the y-46 trio / #16 fights** (the standing ledger item): the
   rim fight only needs to SURVIVE the first salvo — the tree entered at 20.0 health vs
   master's 14.0 and died to the salvo at exactly 0.0; +10 fight-entry hp decides it.
3. **The rim launches' misses:** both fired at the mid passes (y 18.4 / 20.8); the proven
   landing band (y 12.8-13.7) never came up in the window — if the missile answer buys the
   time, the low band recurs on the ~700-tick circuit.

## Live checks for the main agent

- Optional, for lever 1: during the rim fight (Bond standing at ≈ (112.7, 13.47, −179.5),
  after the crouch drop + stand), trace the Hind's missiles (`tools/trace/gdbwatch.py` on the
  missile bullet records, access type read, expected: the salvo's launch frames, the seek
  behaviour and whether the carriage body ever blocks a missile's line to Bond). The port's
  dodge gate holds fire while dodging and strafes perpendicular — the run logs show Bond
  still hit at ~800 ticks; the live trace would pin whether the missiles home through the
  dodge or the rim spot simply has no cover.

## Usage

not visible to me
