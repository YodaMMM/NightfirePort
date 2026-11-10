## Your task: `level-05-valley-hurt`, The Exchange's valley hurt planes hurt Bond when he walks onto them

Branch from origin/master. walkreal_07000005_live::the_valley_plane_hurts_bond_walked is ignored: the walked descent to hurt #3's plane stalls at the y-14.1 lip ((123.3,14.1,122.9), 6 legs identical) and the run+jump drive wedges at (105.6,25.1,114.6). Next lever from the note: the nav-lenient descent from the armour-pickup leg (59.65,33.57,86.11) to the valley floor (42.5,8.9,88.3), then north into hurt #4 (floor y 4.38). If a lip blocks a descent the game allows, that is a port collision bug: find it [M] and fix it. Assert the hurt values from the decompile. Un-ignore it. Run the whole walkreal_07000005_live binary and interactables_07000005_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-valley-hurt`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
claimed-by: valley-hurt worker at 2026-11-10T07:51Z

---
## Report (done, 2026-11-10T07:51Z)

# level-05-valley-hurt — the walked route now reaches the lip; the port collision bug the task anticipated is LOCATED (room 14's collision has no basin floor east of the lip); the hurt damage 1000 is pinned from the decompile; the test stays ignored with the ledger + next levers in its ignore message

Branch `s/level-05-valley-hurt`, cut from origin/master 6a4fcf4ef (head b073a882c).

## 1. What the route now does (the test body, `the_valley_plane_hurts_bond_walked`)

- The decompile-backed hurt pin FIRST: every valley-floor plane (kind 0x27, flat y 4-6)
  must carry `damage 1000.0` — placement param 1 [M `Hurt_Create` 0x17e870: damage = param
  1, gate = param 2, sound id = param 3, class mask = param 4], applied to the player with
  no level/who gate [M `Player_DealWithObjHit` 0x194458:95-103 — obj+0xFF == 0x27 applies
  `Hurt_GetDamage` 0x17ea18]. The load census prints all 13 volumes: **every one reads
  damage 1000** — the assert passed on the real load.
- The gatehouse leg (98.6, 26, 106.5) — proven, asserted.
- The east steering descent (`try_walk_to([120, 4, 115])`): lands the **lower lane
  (118.71, 16.79, 114.44)** — the walk-2 ledger's own lane point, reproduced.
- The raw run+jump drive (no walker edge logic — pure real input) descends the truck slope
  from the lane to **(132.0, 14.17, 125.37)** and hard-blocks.

## 2. The port collision bug, located (the session's main finding)

The task: "If a lip blocks a descent the game allows, that is a port collision bug: find it
[M] and fix it." The block is measured end to end [L hurt10-hurt14, the run logs on the
branch]:

- **No wall exists**: from Bond's stall point, direction rays at dy −4..+2 and d 2-6 all
  read CLEAR (the raw physics drive is blocked by nothing segment-visible).
- **No ground lies ahead**: the down-rays 0.5-8 u past the lip read `none` BOTH scoped to
  Bond's own cel AND unscoped, from start heights 13/20/30.
- **The room split**: the lip stand and every point past it resolve `find_cel` = **Some(14)**
  (Bond's own room); the WEST basin floor — which Bond stood on at (47.20, 8.37, 84.06)
  after the nav-lenient west descent [L hurt6, the note's lever proven en route] — resolves
  **Some(15)**. Room 14's build COVERS the east basin (Some(14) at y 14/8/5) but carries
  **no floor quads there**. The deep pocket the west drive found (56-59.5, 2.0-3.3, 65-68,
  [L hurt6/7]) is room 15's floor too.

So the descent refuses because there is no geometry to descend onto: **the basin floor east
of the lip is absent from the port's room-14 collision build** — exactly hurt #3's own band
(x 80.78..168.38, z 96.66..144.68, y 5.11 [L the census]). The fix is the room-14 build
(the level bin's collision data [D] vs the port's parse; the game's FindCel relink [M
build_FindCel 0x1c8cb0] vs the port's ScopeGraph) — a full session's work, ledgered as the
next lever.

## 3. What was run (all on the final tree)

- **`walkreal_07000005_live` whole binary, `--test-threads=1`**: **16 passed, 0 failed,
  1 ignored** (the valley ledger test stays ignored — red at the death assert, above).
- **`interactables_07000005_live`** (`--test-threads=1`): **6 passed, 0 failed**.
- No `crates/*/src/` changed (the diff is the test file + docs), so no other binaries'
  behaviour can change; both binaries that use the changed file ran whole.

## 4. What's left (the ignore message carries the same ledger)

1. The room-14 basin-floor fix: compare the level bin's collision data for the east basin
   region [D] with the port's parse, and the game's FindCel relink law [M build_FindCel
   0x1c8cb0] against the port's ScopeGraph.
2. Once the floor exists, the walk descends onto hurt #3's plane, the −1000.0 record fires,
   and the test un-ignores.
3. A west-route fallback also exists now (the nav-lenient descent to the west floor + the
   deep pocket) if the east basin floor lands first.

## Live checks for the main agent

1. **The truck's path through the lip** [settles "the game allows this descent"]: on
   07000005, gdbwatch the truck script SP #37's mover while it drives — does its path
   descend the lane slope past (132, 14, 125)? The lane slope's y 14.17 lip is where the
   port's collision has no floor. If the truck's path keys carry it through, the game's
   own geometry has the basin floor and the port's room-14 build is the bug.
2. **Bond walks the lip on the real game** [the direct proof]: from the lower lane
   (118.7, 16.8, 114.4), walk east down the slope on the real game — expected: the game's
   Bond descends past y 14.17 onto the basin floor and the hurt plane kills him with the
   1000 hit (the screen's death). If the game ALSO blocks at y 14.17, the lip is the
   game's own wall and the port is faithful — the pin needs a different access route
   (the west basin floor is the candidate: room 15 exists in the port).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.
