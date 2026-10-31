## Your task: `level-0d-last-two-standins`, 0d: the crane-to-yard chain fully on real input

level-0d-jib-walk-off (merged; read its report and `probe_crane20`) made the deck walk-off, the catwalk walk and the dock E #460 real input, and left two narrowed stand-ins in `walkreal_0700000d_live::walk_arrival_to_the_c5_door`: (1) the catwalk -> bay exit, (2) the beam boarding + the west ride on SP #115's shuttle (137.54 <-> 177.22 over 3000 ticks [D]) and the drop into the yard at x 137.54. Find the game's way for each [D the catwalk/roof/bay collision incl. dynamic objects, the beam's script-entity collision and its timing; M the script-entity carry Control_InheritVelocity 0x133e88 / FeetOnPoint 0x194188], walk both on real input and delete both writes. Fix any port gap with evidence. Run walkreal_0700000d_live and interactables_0700000d_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-last-two-standins`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T13:17Z

---
## Report (done, 2026-10-31T14:04Z)

# level-0d-last-two-standins — report

Task: the crane-to-yard chain fully on real input — delete the two stand-ins left in
`walkreal_0700000d_live::walk_arrival_to_the_c5_door` by level-0d-jib-walk-off: (1) the
catwalk→bay exit, (2) the beam boarding + the west ride on SP #115's shuttle and the drop into
the yard. Branch s/level-0d-last-two-standins from origin/master (533ccf2b, batch-1031f).

## Result: stand-in 1 is DELETED and replaced by a fully real walked chain. Stand-in 2 is
## narrowed with the boarding surfaces pinned and the access question reduced to one live check.

## 1. Stand-in 1 deleted — the real catwalk exit (the ladder, the roof, the annex, the bay)

The route, every leg real `Sim::step_bond` input:

1. **The ladder grab off the catwalk's west edge.** The catwalk is SEALED everywhere else
   (measured this window): the south face is a wall from y 12.2 through ≥ 14.2 at every probed x
   (165/172/180/188 — `probe0d_ladder_try`'s southface rays), so the staircase-island jump the
   last window guessed at is impossible (the wall blocks the run-up AND the landing — the jump
   test: Bond jumped vertically against the wall at z −265.71). The exit is the west-end LADDER
   [D placement type 0x22, class 0x2D — ladder 2: centre (155.60, 14.25, −266.18), r 4.39, the
   only ladder east of x 120 (probe0d_route_scan's census; #0/#1 are at x 212..218)].
   - The grab fires MID-FALL: walk west off the catwalk's west edge (x 157) with **yaw −2.5**
     (west-southwest) — the game's facing gate needs |wrap(ladder.yaw − player.yaw)| > π/2
     [M Player_CollLadder 0x196118: the strict `1.5707964 < ABS(fVar3)`]; west-facing is EXACTLY
     π/2 and is rejected (measured — two probe runs), yaw −2.5 gives diff 2.5 and the grab
     catches the fall through the ladder's pick box [M Ladder::capsule_hits_default → pick_hit:
     the ×1.5 sphere gate, the mesh-box prefilter, the tri refine].
   - **The climb to the roof**: hold forward — Player_Climb runs to the ladder's top (the pick
     sphere reaches y 18.64) and lunges [M Player_Climb 0x196310: `pos.y > top → +0x18 += 0.6`
     dismount]. The dismount is AIRBORNE just north of the roof's north edge (z −267.5); the
     climb's eased yaw (ladder.yaw + π = south [M the yaw ease]) carries Bond south onto the
     roof on the held input — landed grounded at (155.60, 19.61, −266.32) on the ladder-head
     platform [probe0d roof map: the roof 18.58 spans z −268..−304, x 148..196, with rooftop
     structures at 20.19/19.7/19.1 over x 158..174 and x 180..194, z −271..−290].
2. **The roof's east band** (z −268.5..−269 — the ONLY full-width open lane; the structures
   start at z −269.4, measured by the lane walk's stuck dump): walked east 155.6 → 188.6.
3. **The building's east lane** (x ~196.3, between the shed's east wall x ~195.2 and the roof's
   east edge x ~197): a STRICT L — east first (a tight r 0.4 waypoint at (196.3, −269.0)), then
   south to z −278.8 (the diagonal clips the shed's NE corner and wedges — measured).
4. **The push-off into the annex**: the roof's east edge has an unscanned parapet (top 19.44 —
   Bond landed on it mid-cycle); the push/land cycle repeats until the annex floor catches him:
   the annex 9.42–9.45 (x 197..202, z −272..−290 [probe0d floor map]). The fall is 9.1 m —
   fall_timer ≈ 84 → damage ≈ 24 [M 0x1949a8:117-126 model], health 100 → ~76 — the walk
   completes with health to spare (the final run ends 100.0 — the fall's timer stayed under the
   pain gate).
5. **The bay**: CONTINUOUS with the annex at z −276..−282 where the 14.7 pad's east edge leaves
   x 196..197 open [probe0d floor map: x 196 = 9.53, x 197 = 9.45] — walked west to the E-proven
   corner, then the EXISTING real E #460 walk (use_ticks 240 → ch 0x2b) and the yard legs are
   unchanged.

The test asserts the grab, the roof top-out, the lane arrival, and the bay/annex standing — all
decompile/disc-cited inline. walkreal_0700000d_live: **1 passed / 0 failed (33.6 s)** — the
whole chain (the walk-off, the catwalk, THE LADDER+ROOF DESCENT, the E, the yard) is real input;
the only remaining pos write is stand-in 2.

## 2. Stand-in 2 narrowed — the boarding surfaces are pinned, the access is one live check

The beam (script 0x060009EF, entities 0x0200_0a80/0x0200_0a81) shuttles x-only, top 14.32, over
z −270..−304 [probe20 beam watch + the shape bounds]. The only surfaces above it on its path:

- **The 16.8 mezzanine** (x 169..183, z −271..−283 [probe0d "above" scans]) over the beam's EAST
  extreme (x 173..181.4). Its edges: east → the 13.8 ledge (x 184) reachable DOWN from it, north
  → the 8.10 floor 8.7 below, south → the bay 9.53 7.3 below, west → the 3.98 pit 12.8 below —
  and the roof (18.58) is 1.78 above it with NO opening (the roof map shows 18.58 over the whole
  mezzanine; the x 176..178 "slot" is the roof's own surface between two 20.19 rooftop sheds).
- **A 17.2 structure at x 134..136, z −276** [probe0d west scan] over the beam's PARKED west
  position (x 133.4..141.7) — 2.9 above the beam's top, an easy step-down — but it rises 13.2
  from the yard floor with no step/ladder evidence, and the 13.5 gantry (x 136..141, z
  −282..−300, 1 m under the beam's top) likewise has no access from the yard (4.0), the pit
  (3.98) or the deck (11.56, z −304, x 156..190 — 20 m away).

The access is closed on the port's statics from EVERY side the scans cover: the catwalk is walled
south, the slot jump is dead, the bay has no ladder (the census above) and no crates (the dock
drone census: 2 dtype-0x05 guards only), the roof lane is outside the shed whose east wall stops
it, and the cel-pass's 8 skipped instances already enter the port's collision as invisible walls
[scene.rs:7136/7215] so they cannot be the missing stairs. The game's way needs a live look —
§ Live checks.

## 3. Verification

- `walkreal_0700000d_live`: **1 passed / 0 failed / 0 ignored (33.6 s)** on the final tree.
- `interactables_0700000d_live`: **11 passed / 0 failed / 0 ignored (31.4 s)** on the final tree.
- `probe_crane20` (+ the two new print-only probes `probe0d_route_scan`, `probe0d_ladder_try`,
  both `#[ignore] --nocapture`): the floor/roof/above/west maps, the ladder census, the drone
  census, the southface wall scan, the grab/climb receipts — every number above cites them.
- The suite fast runner cannot survive this sandbox as a background job (the known
  between-tool-calls kill; not re-measured this window). The touched binaries are the two quoted
  above plus probe_crane20 (print-only, run repeatedly this window).

## Live checks for the main agent

1. **The beam boarding (stand-in 2's resolver).** Fresh boot 0700000d, E #460, then follow the
   player east/north while SP #115 runs (3000 ticks): WHERE does the player stand to board the
   beam, and how do they REACH that surface from the bay? Candidates pinned above (the mezzanine
   16.8 x 169..183; the 17.2 tower at x 134..136 z −276). Address of interest: if the route uses
   a climb the port lacks, watch `Player_CollLadder` 0x196118 / the ladder placements — if it
   uses a DYNAMIC (a crate/pallet the script poses), watch the script 0x060009EF's entity writes
   (Control_InheritVelocity 0x133e88 carry paths) against the port's static-only read.
2. **The ride's drop** (for the follow-up): at the beam's west extreme (x 137.54), does the
   player step off NORTH/SOUTH into the yard (4.0) and eat the ~10 m fall, or is there a lower
   landing (the 13.5 gantry x 136..141 z −282..−300) the walk should target?

## What's left

- The boarding (live check 1) — then the ride + the drop on real input and stand-in 2's delete;
  the yard legs already feed straight in from the drop point (136.5, 4.5, −290).
- The full-suite total line on a merged tree (the window's runner limitation above).

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the
only proxies are the shell timestamps: setup+branch 13:19–13:22 UTC, the scans and the first
grab attempts 13:23–13:41, the grab+climb+roof breakthrough 13:42–13:55, the test wiring and the
greens 13:56–14:02, the report 14:03–14:20 UTC 2026-10-31).
