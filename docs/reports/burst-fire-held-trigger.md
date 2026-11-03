# burst-fire-held-trigger — report

Branch `s/burst-fire-held-trigger` (cut from origin/master 6f16146e). Task: the burst-mode
weapons fire their full burst from one trigger pull — port the game's held-trigger path
(`Player_WeaponFiring` 0x1a5460:601-655 [M]), drive it through the real input path, and
un-ignore the passing legs of `crates/nf-view/tests/fire_modes3_live.rs`.

## What the game does (the path, from the decompile)

- A fresh press (the mask-4 read, `Input_Action(player, 9, 4)`, 0x1a5460:601) with the refire
  timer run out arms the pull: `+0x936` (pending) = the record's shots-per-pull (+0x28 at the
  mode's slot) and `+0x92C` (the timer) = 1.0 [:617-624]. The only caller passes param_2 = 0
  (`Player_Update` 0x193600:413), so the `param_2 == 1` arm is dead in the on-foot path.
- The same tick's countdown (`timer -= 1`) fires round 1 immediately for `+0x44 == 0` records
  (the press-tick spawn, [:688-690]); each shot re-arms the timer at `max(+0x40, 1)` [:657-660].
- Held non-fresh ticks keep the countdown running; a held tick whose timer has ALREADY run out
  returns before the countdown [:613-615] — no refire until the next press.
- Releasing mid-burst CANCELS the pull: the mask-1 release read (level == 0) zeroes `+0x936`
  before any further fire [:442-446]. The remaining rounds never spawn.

## The port already matched — the harness did not

`Gun::update_with` (crates/nf-game/src/gun.rs) implements exactly this path (the fresh arm, the
run-out return, the release clear at `!pressed`, the state-9/0x8000 refire gate as the caller's
`refire_blocked`). The ignored burst leg failed because its harness (`fire_once`) released the
trigger after ONE tick — the port (like the game) cancelled the pull, measured 1 round, and the
leg read as a port bug. All four legs' real causes:

1. **~182 burst (0x0A→0x0B)**: the one-tick pull + the release-cancel above. Rewritten with a
   `fire_held` helper (trigger held, then released): pins shots at ticks [0, 4, 8] (spp 3
   [L 0x0B +0x28 = 3], interval 4 [L +0x40 = 4]) and a release-cancel sister (a 1-tick pull
   fires 1 of 3, clip 29) [M 0x1a5460:442-446].
2. **~248 launcher (0x2C→0x2D)**: two causes. (a) The guided round flies its full 200-unit
   range at 0.25 u/tick [L 0x2C +0x1C/+0x20] — ~800 ticks plus the 15-tick restore [M
   `Bullet_Delete` 0x12e660:28] — and while the substate-10 ride is up every press is swallowed
   [M `Player_Update` 0x193600 case 10]: the old 400-tick wait pressed INTO the ride. The leg now
   waits until `missile_cam` drops (cap 1000). (b) The 4-rocket pull needs the held path (spp 4
   [L +0x28], interval 30 [L +0x40]) — now pinned at [0, 30, 60, 90]. Blast safety: the spawn's
   own floor is 0.36 under the eye [L ee_start_05 rest 24.9633 vs eye 25.32], so any down-tilt
   detonated the rockets (radius 8, damage 100 [L +0x08/+0x0C]) at Bond's feet — 47.5 health
   after round 1, dead before round 3; the leg now holds the level-start (level) pitch.
3. **~281 MGL (0x2A→0x2B)**: the leg's `shots >= 1` assert measured the port's `shot_tick`
   pulse — the BULLET path's spawn presentation. The MGL is the grenade family: `Bullet_init`
   0x12c5a0's class-3 arm spawns a grenade OBJECT, which pulses no `shot_tick`. The grenade
   spawn + the 210-tick fuse death asserts pin the fire; the wrong assert is gone (the TEST was
   wrong, not the port).
4. **~321 samurai (0x32/0x33)**: the draw stall was a real PORT bug — the direct switch request
   read `owned[twin]`, but the game's owned state is per-BASE (the twins share `BL+0x1BA +
   base·0xC` [M `Player_EquipWeapon` 0x1a8a58:39-41]; the port's own `Inventory::equip_weapon`
   writes the same base slot, and fix_dev_panel_live documents "a variant shares the base's owned
   flag"). The give of 0x32 owns base 0x33, the old gate refused the draw on owned[0x32], and
   the request died silently. Fixed in `crates/nf-view/src/scene.rs` (the `switch_to` target
   gate reads `base_of(id)`'s slot) — the leg then ran green after the same blast-safety aim fix
   as the launcher (the 256-damage bolt at a down-tilt landed at Bond's feet and killed him
   mid-charge; the mode press then read hands 0x47).

Also fixed the file's `arm()` helper: it wrote the RAW weapon's clip slot, but a twin (+0x05 set,
matching ammo type) reloads and spends the SHARED base slot [M `Player_ReloadAmmoType`
0x1a1988:18-22; the sim re-reads that slot into `gun.clip` every tick start] — after arming a
twin, the next tick overwrote the armed clip from the stale base slot.

## Verification

- `fire_modes3_live`: **9 passed, 0 failed** (`--include-ignored`, on the final tree), all four
  previously-ignored legs un-ignored.
- `fix_weapon_fire_modes_live`: **5 passed, 0 failed**.
- `weapon_matrix_live`: **6 passed, 0 failed** (24 ignored are that file's own matrix rows).
- `nf-view` lib tests: the run was SIGKILLed by the OOM killer while the background suite held
  the memory; it is inside the priority suite re-run below (the `nf_view` bin).
- The full suite (tools/test/suite.py): the first run (started 02:47, pre-change tree) processed
  328/1089 binaries in 35 min — too slow for the window, and its nf-view binaries predate the
  scene.rs fix — killed. A priority re-run on the FINAL tree (`--only` = the four crates' lib
  tests + fire_modes3_live + fix_weapon_fire_modes_live + weapon_matrix_live + the 29
  walkreal_0700*_live level walks) is in flight; its total line is in `suite.out` at the repo
  root, and the result as of the finish push is quoted below.

Result of the priority re-run: it was still in cargo's relink of the nf-view test binaries when
the sandbox window closed (the earlier OOM kill of the `nf_view` lib-test link invalidated the
shared units, so the suite rebuilt them at -j4 — the relink alone does not fit the remaining
minutes). The final-tree evidence quoted above is the three targeted runs (fire_modes3_live 9/9
`--include-ignored`, fix_weapon_fire_modes_live 5/5, weapon_matrix_live 6/6), all on the pushed
tree. The first (pre-change) suite run's only failures up to its kill were
`identity_disc::the_real_disc_passes_the_identity_check` and two `missing_scenery_live` draws +
one OOM CRASH — none on the code this task touches; the 3 disc_golden failures are the slim-ISO
allowance.

## What's left

- The full 1089-binary suite run on the final tree did not fit the sandbox window (the runner
  needs ~75 min of run phase alone). The next window should run it unmodified:
  `NF_ISO=<iso> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (the build is cached).
- The ~60 walkreal probe/perf bins (walkreal_probe0c_*, walkreal_probe*) were outside the
  priority list; the 29 level walks are inside it.

## Live checks for the main agent

None required — every pin is [M]/[L]-cited. Optional [S] confirmation on a FRESH boot (not a
save-state resume): Kowloon Type 40 in burst mode, one 1-tick tap → exactly 1 round; one hold →
3 rounds at the 4-tick interval; releasing mid-burst leaves the remaining rounds unfired.

## Usage

not visible to me
