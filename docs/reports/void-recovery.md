# void-recovery — the game recovers an out-of-cel object, the port now does too

Branch `s/void-recovery` from `origin/master` a6665d115. Code: `crates/nf-game/src/world.rs`,
`crates/nf-game/src/player.rs`, `crates/nf-view/src/scene.rs`; test:
`crates/nf-view/tests/void_recovery_07000003_live.rs` (new). No shared test helper touched, no
`[L]` pin loosened, no ignore added, no test loosened.

## 1. The mechanism, cited

`control_handle_cel_change` 0x133288 [M ACTION.ELF] — the default arm of the type switch (the
type-2 drone case falls through to it in SP; `MPSettings._388_4_` only short-circuits the drone
case, so MP players run it too): for an object that was LINKED, left its cel's box
(`build_point_in_bounding_box` 0x1c9500 miss), crossed no portal (`Cel_ObjectLeftCel` 0x1e0d58
miss) and got no `build_FindCel` 0x1c8cb0 hit at the `build_LinkToRoom` 0x1c9680 retry:

- `pos (obj+0x30) := obj+0x40` — the move segment's FROM, i.e. the previous position
  (the port's `Bond::prev_pos`, whose own doc already names obj+0x40);
- each axis then clamps into the OLD cel's box `cel+0x40..0x58` minus the world-sphere radius
  (`obj+0x8C`): `if v < min + r → min + r` (the else branches PULL IN from the low edge),
  else `min(v, max − r)` [M 0x133288:186-221 — x 0x40/0x50, y 0x44/0x54, z 0x48/0x58, r 0x8C];
- the recovered vector is copied to obj+0x70 and obj+0xC0, the object relinks into the OLD cel's
  list, the channels refresh [M 0x133288:222-261];
- an UNLINKED object gets NO recovery — the obj+0x20==0 path retries `build_LinkToRoom` once and
  returns [M 0x133288:63-73].

The radius is the player's world-sphere radius **1.0576** [L ee_start_05.bin M_CONTROL 0x16cb8a0
+0x8C, scale |obj+0xE8| = 1; M `Control_BuildWorldSph` 0x133070] — now
`Bond::SPHERE_RADIUS`.

This one arm explains BOTH measured spots [D live, the task brief]:

- **(12.0, 6.3, −32)** (the west void strip, no room cel's bbox holds it — the port's probe: 0
  bbox holders, `find_cel` None): the poke writes obj+0x30 only, so obj+0x40 still holds the
  pre-teleport position — the level start. Recovery: pos := the level start (inside the start
  cel's box, clamp no-op), relink the start cel. Measured: "back at the level start (−8.85,
  6.41, −23.19) inside 1 s" — the port now lands at (−8.85, 6.41, −23.19) cel 139 exactly.
- **(16.5, 6.3, −33)** (inside cel 8's bbox x 13.01-17.04 but over the east void strip — no
  floor within 90 u in the whole level data, the port's unscoped down-ray): Bond falls out of
  cel 8's box (y < 5.25) → recovery: pos := prev clamped into cel 8's box → x = 17.04 − 1.0576
  = **15.98 ≈ the measured 15.99**, y pulled to 5.25 + 1.0576 = 6.31; the ground disc (r 0.4)
  reaches back onto the mid strip's floor edge at x 15.59 → stands (measured y 6.23).

## 2. The port change

`CollisionWorld::out_of_cel_recovery(cur, prev, radius)` in `world.rs` — the clamp above,
`None`-preserving for an unlinked object. Wired at all three relink sites in `scene.rs`:

- the SP bond's per-tick relink (the `player_cel_relink` miss arm);
- the MP body relink (the same arm — the game runs it in MP too [M 0x133288:179-184]);
- the placement/teleport arm (>3.0 jump): on a `find_cel` miss the recovery now runs from the
  PRE-jump `prev_pos` — the game's poke semantics (obj+0x40 survives a poke; only
  `Player_StandAtNewPosition` 0x190d88 rewrites it, and that path keeps its own direct relink).

Kept to the out-of-cel case only, per the brief: no collision, ground or cel-find code touched.

## 3. Verification

`crates/nf-view/tests/void_recovery_07000003_live.rs`, 4 tests on the real level, **red proven
on the old code** (`git stash` of the three src files → the two void pokes fail with cel None /
fall, `git stash pop` → green):

| test | old code | new code |
| --- | --- | --- |
| `poke_into_the_west_void_returns_to_the_poke_origin` | FAILED (cel None) | ok — back at (−8.85, 6.41, −23.19), cel 139 |
| `poke_into_the_east_void_recovers_into_cel_8` | FAILED (cel None, y −160) | ok — cel 8, y > 5, x 16.17 |
| `poke_on_the_mid_strip_still_stands` | ok | ok (guard: the game stands here too) |
| `walk_off_the_west_band_edge_recovers` | ok | ok (see §4) |

Binary pass counts (whole binaries, `--test-threads=1`, this branch):

| binary | result |
| --- | --- |
| `void_recovery_07000003_live` | 4 passed / 0 failed |
| `walkreal_07000003_live` | 6 / 0 / 1 ignored (the 00 Agent per its own ledger) — the 03b baseline |
| `walkreal_07000002_live` | 3 / 0 / 1 ignored (same ledger) — the 03b baseline |
| `walkreal_07000004_live` | 10 / 0 |
| `mp_perf_16_live` | 2 / 0 |

## 4. The real-input walk, honestly

At FLOOR LEVEL a walking player cannot enter the strips: the moat wall stops the capsule at the
band's edge (the walk test: Bond blocked at x 10.36, centre + r 0.4 = the wall face 10.75; the
same wall and capsule exist in the game). The strips are entered from the UPPER perches only —
the building's upper-floor row to its garden end and the door top (door-walk-03b §3's
player-reachability answer). From a perch the fall leaves every room cel's bbox (0 bbox holders
at (12.0, 6.3, −32)), so `build_FindCel` misses and **the game's recovery WOULD run there** [M];
the port now runs the same arm — the poke tests prove it at the two measured spots.

## 5. What's left

- The (16.5) end state: the port recovers into cel 8 (x 16.17, y ~6.2, no void fall) but does
  NOT settle into the game's exact stable stand at (15.99, 6.23) — the recovery lands x 15.98
  over no floor and the port's centre-ray ground probe cannot catch the floor edge, while the
  back-facing wall run pushes it east again. The residual is the ground-contact model (centre
  ray vs the game's capsule base), NOT the recovery arm — out of scope here (broad collision
  changes failed the suite on `s/level-14-float-on-master`). The recovery keeps it bounded
  inside cel 8's box forever; nothing falls into the void.
- The full four-crate suite did not fit this window (the three named gates + the sisters ran
  whole instead; the change touches one movement branch that only fires on a FindCel miss —
  every walk's route keeps `falls: []`).

## Live checks for the main agent

1. **The (16.5) settle** — in PCSX2 at (16.5, 6.3, −33): watch whether Bond's rest state is
   exactly STANDING (flags 0x18, y 6.23 stable) or drifts. If the game's ground contact there is
   the capsule base on the floor edge at x 15.59, the port needs the capsule-base ground probe
   (a separate task; the recovery arm stays as built).
2. **The unlinked retry** — poke Bond to a spot out of every room bbox with obj+0x20 pre-zeroed
   (if a debug poke can clear it): the game should NOT recover [M 0x133288:63-73] — confirm,
   since the port's `out_of_cel_recovery` also refuses when `cur` is None.
3. **A drone into the same strip** — the type-2 case runs `Drone_FellOutMap` 0x13a878 (SP: sets
   obj+0xFE bit 0x1, then falls through to the same default arm). The port's drone path was NOT
   touched; a live check whether SP drones visibly re-appear after a strip fall would pin
   whether a follow-up drone task is warranted.

## ntfy failed

All three fallbacks, in order, with their exact output:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "void-recovery: done — ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'void-recovery: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "void-recovery: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 30 s, killed (no output).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the session.
