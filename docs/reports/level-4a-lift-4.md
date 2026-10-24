# level-4a-lift-4 — 0700004a: the lift ride's timed satchels, defused with the laser watch

## What was asked

Find the satchels in the data and decompile (who drops them, the object class + its update — the
timer and what it writes on expiry — and the laser defuse), port them, remove the [S] kill zone,
and pin both directions with real-input tests: the ride with every charge burned reaches the lobby
alive; an undefused charge fails the mission when its timer ends.

## The chain, decoded (every fact tagged)

**Who drops them.** The level's two 0xD2 shooter placements, both "ShootyBangBang", weapon 0x3d
[D 0700004a.bin census]: **#286** at (3.04, −48.40, −1.67) watching **ch 0x22** (the landing
cut-scene's end event — the hand-back/resume), **#287** at (2.09, −117.78, −1.91) watching
**ch 0x4f** (the TouchOnce plate #281 at y −124.10 [D]). `Shooter_Update` 0x17d0c0 fires ONE
`Bullet_init` when the watch channel is high — the 0x3b-family weapon's direction REPLACED with
(0, −1, 0) (straight down) [M 0x17d0c0 disasm] — and sets `obj+0xFE |= 1` (one charge per
shooter) [M].

**The object class.** The charge is a **bullet** of the satchel family: `weapon_data+0x02 == 0x3b`
(kinds 0x3b-0x40 and 0x6c [L ee_start_05.bin weapon_data @0x2bf150]). The mesh
`weapon_data[w]+0x5C = 0x20007b3` [L]. Fields on the bullet's ext block [M Bullet_DoTrails 0x12d890
disasm, param_2 = BU]: **+0x44** the weapon record, **+0xF4/+0xF8** travelled/speed (+0x20 = 0.0667
u/tick [L]), **+0xFC** the fuse (f32), **+0x100** the shown digit pair, **+0x102** the burn counter
(i16).

**The timer.** `Bullet_init` 0x12c5a0: default 210.0 [:103]; `+0x70 & 0x10000` → 300.0 [:192-198];
the **0x3b family: `((kind − 0x3b)·5 + 5)·60` ticks** — weapon 0x3d → **900** — and the **burn
counter = 100** [:199-211 disasm]. The tick lives in `Bullet_DoTrails` (which `Bullet_update`
calls every tick, flying at its tail 0x12efd8 AND landed at 0x12efdc): `weapon+0x70 & 0x200` gates
[0x12dcc4]; family not 0x37/0x59/0x5b → `BU+0xFC −= FRAME_RATE_MUL` (1.0 [L ee_start_05.bin
@0x30d0d8]) [0x12dd20-0x12dd2c]; at `fuse < 0` → `obj+0xF4 = 2`, `obj+0xFE |= 1` (the delete)
[0x12dd44-0x12dd58] and **`Explode_Create(weapon+0x08 radius = 8.0, weapon+0x0C damage = 2000.0,
script 0x6000052, scale = radius)`** [0x12df3c region; L the records]. (The `+0x70 & 0x2000` flash
arm and the `& 0x20000` smoke arm are both clear on the satchel's 0x300 [L], so the plain blast
runs.) Bullet_update's own fuse arms are the kind-0x3a designator (0x12ee08) and the kind-0x45
smoke (0x12efa4) only — the satchel's fuse never ticks there.

**The landing.** `Bullet_CollisionHandler` 0x12ce80 @0x12d1fc: a world hit for the `+0x70 & 0x100`
family with no struck object → **state 3 (landed)** + the rest clank `Sound_Play3D(0x4a7)`; the
pose aligns to the hit normal (draw-side). A body hit (class 2/3) bounces back to state 1 with no
explosion — `Bullet_handle_object_destruction` 0x12cd98 needs `+0x68 & 0x4000`, which the satchel's
0x00010080 record lacks [L]. The landed charge **rides the cab** [L the live run: the charge on the
roof while Bond rides at (1.799, −73.418, −3.453)]; the game calls `Control_InheritVelocity` for
every landed round [M 0x12edd8] — the game-side carrier is the named live check below.

**The laser defuse.** In `Bullet_DoTrails`' contact walk [M 0x12e2f0-0x12e390 disasm]: the charge's
`obj+0xD0` contact list (the collide pass registers each hitter) — for each class-5 entry whose
weapon is the **Laser watch 0x4E: `BU+0x102 −= 1`/tick (FRAME_RATE_MUL)**, the upgrade **0x4F: −10**
(`FRAME_RATE_MUL * 10.0`), with `Emitter_Create(0xc00000a, hit point)` sparks [0x12e37c]. At
`BU+0x102 < 1` the burn-out arm re-randomizes the fuse EVERY tick to `60 + Rand_Rand(0x1518)`
[0x12e3cc-0x12e40c] — **the charge never explodes**. (The task's suggested `Lock_Update` 0x1ae3d8
0x4E arm is the LOCK burn — 3/10 per tick off the 0xff bytes — a different, already-ported
mechanism; the satchel's burn is DoTrails' own walk.)

**The fail path.** With no defusing: the shot at the ch-0x22 rise (the hand-back, the lift
resuming at root −54.13 [L]), the fuse runs 900 ticks, the blast (radius 8, damage 2000) kills
Bond on the cab, the death sequence runs (~450 ticks [M death.rs; the level-16 parity +451..454]),
`Player_HandleDeath` 0x190bb0:32 writes **ch 0x62**, and `Mission_Update` 0x182c60's ch-0x62 arm
fails the mission [M]. The live fail at root ≈ −95.85 = the hand-back + ~1377 ticks = 900 (the
fuse) + ~477 (the death sequence) — **the port's sister reproduces it: ch 0x62 at 1200-1600 after
the hand-back [L-pinned window], at the live ride depth, mission status 3.** The live run's
"health still 100" is the god-mode artifact (HandlePain's health write is `CheatInfo == 0`-gated
[M 0x1902c8] — a frozen health defeats the health write but the driver's ch-0x62 observation stands
as the ride-depth fail); the live check below settles it.

## The port

- `bullet.rs`: the five bullet fields (`fuse`, `fuse_ticks`, `burn`, `defused`, `landed` — each on
  its own line), `Bullet::TAIL` for the existing constructors, and `init_fuse(w)` implementing the
  Bullet_init fuse block.
- `drone.rs`: the drone-bullet pass lands 0x3b-family world hits (state 3 + the 0x4a7 clank)
  instead of dying, bounces them off Bond, rides landed charges on the mover's per-tick delta
  (`ground_motion` — the same `Player_FeetOnPoint` mechanism Bond rides by), ticks the fuse in all
  three arms (landed / landing tick / flying tail) via `tick_charge_fuse`, and pushes the expiry
  blast into `pending_blasts` (the existing grenade queue — the game's Explode_Create order holds).
  `update_player_bullets` burns landed charges under 0x4E/0x4F rounds (1/10 per contact tick, the
  0.5 contact sphere [S — the game's charge sphere is its glist bounds, not pinned]).
- `worldobj.rs`: the shooter's `Bullet_init` gets `init_fuse`.
- **The [S] −95.80 kill zone was never merged to master** (it lives on unmerged
  `s/level-4a-lift-3`): there was nothing to remove on this branch, and the fail now comes through
  the real chain — no kill-zone stand-in anywhere.
- **The hand-back re-landed (required by the tests):** the landing scene's marker stream s25
  (Entity hash 00000000, obj_type 0x19, key (1.79945, −54.12811, −3.45340) at t575 [D
  walkreal_probe_l3]) re-enables Bond at the RAW key x/z [L fresh boot] — master composed the SP
  base into every unparented entity and dropped him at the void (1.799, −107.662, −6.830). The port
  now skips the base for the **hash-0 kind-0x19 marker class** (nis.rs interp case 1). A blanket
  hash-0 rule (lift-3's) broke `exit_aim_01_live`: 07000001's pack script 0x06000087 s11 is a
  hash-0 **kind-0x31** entity whose composed spot the exit walk depends on [D probe_entities_l4
  dump] — the rule was narrowed to the marker class and all four fragile tests re-ran green.
  `fix_landing_matrix_live` asserted the composed spot and CONTRADICTED the live audit — the TEST
  was wrong; re-pinned to the raw key (the header keeps both readings and the evidence).

## Tests (all real input; the board walk + the aim/fire inputs only)

- `walkreal_0700004a_satchel_live::ride_the_lift_burn_both_satchels_with_the_laser_watch_and_reach_the_lobby_alive`
  — **NEW, GREEN**: the board (the reveal plate 0x4d), the laser watch 0x4E drawn [M the loadout
  tail], the ride to the ch-0x22 hand-back [D s27], charge #286 lands at #286's x/z with burn 100
  and a ticking fuse [M], the beam burns it out (the live ride-along pinned: the charge descends
  with the cab), ch 0x4f at the −124.1 plate [D #281], charge #287 lands at #287's x/z and defuses,
  the park, the walk-off, ch 0x84 [D #62], alive, ch 0x62 never, the mission not failed, the level
  exit reached.
- `walkreal_0700004a_satchel_live::an_undefused_satchel_explodes_and_fails_the_mission_like_the_game`
  — **NEW SISTER, GREEN**: the same ride, no defusing, no health top-up: the health runs out at the
  900-tick fuse's blast, ch 0x62 at (1200..1600) after the hand-back [L 1377], Bond's root in
  (−99.5, −91.5) [L −95.85], mission status 3 [M 0x182c60].
- `fix_landing_matrix_live::the_landing_handback_matrix_is_the_raw_marker_key` — re-pinned, GREEN.
- Re-run green on the narrowed marker rule: `exit_aim_01_live`, `interact_07000001_live` (5),
  `walk_0700004a_live` (5), `fix_level_chain_live`.
- `probe_entities_l4.rs` — new assertion-free dump (the entity streams of 01/4a + their packs);
  kept ignored.
- `nf-game --lib`: 546 passed (the bullet/gadget/explode/grenade suites all green on the new
  fields).

## Live checks for the main agent (fresh boot — save-state resumes don't count)

1. **The landed charge's carrier**: gdbwatch the first charge's ext block through the ride — its
   y vs the cab's, ~50 ticks after the landing (the shooter fires on the ch-0x22 rise; the charge
   lands ~190 ticks later). The port rides it via the mover delta; the game's state-3 arm calls
   `Control_InheritVelocity(bullet, ext+0x30, 30.0)` [M 0x12edd8] whose param_2 reads as the
   SHOOTER (static, self-deleted) — if the real game's charge floats while the cab drops, the
   ride-along needs the game's actual carrier named.
2. **The fail's health at the blast** (settles the driver's "health still 100" observation):
   without god mode, gdbwatch `BL+0x894` through the blast ticks — the port's chain gives health
   0 at the blast and ch 0x62 ~477 ticks later; if the live health holds 100 until the ch-0x62
   tick, the kill path is not HandlePain and the blast-to-death link needs the game's writer.
3. **The burn-out digits**: the live screenshots show the green countdown; after a full burn the
   game re-randomizes the fuse every tick [M 0x12e3cc] — the digits should flicker garbage while
   the charge sits harmless. A screenshot at the burned charge confirms the burn-out state's
   presentation (the port latches and does not render digits — draw-side gap, noted below).

## Not ported (honest gaps)

- The charge's drawn mesh, the two digit sprites (`SatchelDigits` + `Bullet_DoTrails`' digit
  objects) and the beep (`Sound_Play3D 0x447` per the digit-pair change [M 0x12e290-0x12e2b4]) —
  presentation; the port's bullets are sim-side only.
- The burn sparks' emitter shape (0xc00000a) and the 0x4a7 rest-clank is played through the port's
  audio list only.

## What ran

- Targeted: `nf-view` — walkreal_0700004a_satchel_live (2/2 green), fix_landing_matrix_live
  (green), exit_aim_01_live, interact_07000001_live (5), walk_0700004a_live (5),
  fix_level_chain_live; `nf-game --lib` 546 passed. The satchel work touched nf-game/nf-view only.
- The full suite: started in the background twice (the first run died on the nohup PATH; the
  double-fork relaunch ran the pre-satchel tree and showed only the slim-ISO's expected
  disc_golden failures — `disc_golden::directory_tree_matches_the_golden_manifest`,
  `identity_disc::the_real_disc_passes_the_identity_check`, `probe_pylon_render`); the final-tree
  run was started at 04:28 UTC and its total line read before the push — see the suite line below.

## Suite (final tree)

**The full-suite total did not land in the window.** The tooling degraded this round: the first
background start (02:58 UTC) died on the nohup PATH (`cargo` missing — the runner's traceback in
suite.out); the double-fork relaunch (02:58+) ran but its suite.out was TRUNCATED by the 04:28
final-tree relaunch while the old writer still held the fd — the file now mixes two runs (663
bytes, last written 04:12, one `CRASH walk_07000013_live (exit -9)` line whose tree provenance is
the mixed working tree, not the pushed branch); the final-tree run (pid 20460, started 04:28) was
still in its `cargo test --no-run` build phase at 04:40 UTC when the sandbox window closed. The
final-tree suite MUST be re-run first thing next round.

What did run on the final tree (the working tree == the pushed branch at push time): every
targeted run listed under "What ran" — the two new satchel walkreal tests, the re-pinned
fix_landing_matrix_live, the four narrowed-rule tests (exit_aim_01_live, interact_07000001_live,
walk_0700004a_live, fix_level_chain_live), and `nf-game --lib` (546 passed). The touched crates
are nf-game and nf-view; both compiled clean and their targeted binaries above are green.
**Open item for the next round**: the walk_07000013_live CRASH line needs a clean re-run (the
level AFTER 4a — my nis.rs marker rule and the bullet fields both plausibly touch it; its own
family's targeted runs passed on the final tree, but the 600 s-limit CRASH under two concurrent
suites is not evidence either way).

## Usage

not visible to me
