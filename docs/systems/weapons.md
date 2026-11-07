# Weapon fire modes

Every weapon's modes: the mode button, the twin-variant swap, the per-record bullet flags that make
them behave differently, and the guided-missile camera. Facts are tagged [M] decompile
(`work/decomp/ACTION.ELF/file:line`), [L] measured bytes (`work/ee_start_05.bin`, weapon_data @
0x2bf150, record size 0x10C), [D] disc, [S] suggested. Ground truth: docs/live/2026-10-13-weapon-modes-pcsx2.md.

## The button

`Player_Weapon` 0x1a24c8:204-260 [M]: the mode press (`Input_Action(slot, 0xc, 4)`; the watch pair
0x54/0x55 ORs in the fire press) with the view idle and not (aiming ∧ `+0x68 & 0x40`):

- `weapon_data[w]+0x05 == 0` → the mode byte `BL + w*0xC + 0x1BB` cycles up to the record's mode
  count `+0x2E`. Only the grenade records 0x60/0x61 have count 2 [L]; every gun has 1, so guns
  never move the byte — the real modes are the TWIN records below.
- `+0x05 != 0` (s8 twin) → `new = w + twin`, gated by `Player_WeaponHasAmmo` 0x1a16c0 (carried
  rounds / clip / ammo type 0; NOT the owned byte), written to `obj+0x63` (the switch request).

The pad's "Classic Bond" layout binds **SQUARE** ("Alt. Fire") to the mode action — circle does
nothing [D PCSX2 pass]. The port keeps its B key on the game's mode action.

## The mode table (32 weapons, the PCSX2 labels + the record bytes that differ)

Mode 1/Mode 2 are the HUD's fire-mode lane (`Txt_BindLabel(+0x30 + mode·4)`); "→" is the twin the
press swaps to. All record bytes [L ee_start_05.bin].

| Weapon (id ↔ twin) | Mode 1 / Mode 2 | What the records change | [L record bytes] |
|---|---|---|---|
| Wolfram P2K 0x02↔0x03 | Semi / Silenced | +0x68 0x1004→0x1884 (0x800 muzzle-flash mesh off) | |
| P2K family 0x04↔0x05, 0x06↔0x07, 0x08↔0x09 | Semi / Silenced | same bit | |
| Kowloon Type 40 0x0A↔0x0B | Semi / Burst (3) | +0x28 shots-per-pull 1→3 | |
| Kowloon Type 80 0x0C | none | no twin, count 1 | +0x05 = 0 |
| Raptor Magnum 0x0E↔0x0F | Semi / Semi | +0x40 rate 15→30 (slower), +0x68 0x1000→0x1008 | |
| Delta Repeater 0x11 | Single | no twin | |
| Storm M32 0x12↔0x13 | Auto / Semi | +0x28 999→1 | |
| Deutsche M9K 0x14↔0x15 | Silenced / Burst (3) | +0x68 0x880→0 | |
| SG5 Commando 0x16↔0x17 | Burst / Semi | +0x28 3→1, +0x40 4→15 | |
| AIMS-20 0x1A↔0x1B | Burst / **Grenade** | twin = the underbarrel launcher: +0x70 = 0x12→0x4 — the grenade explodes ON IMPACT | 0x1B +0x70 = 0x4 |
| Frinesi Auto 12 0x1C↔0x1D | Pump / Auto | +0x68 0x8100→0x100, +0x40 60→30 | |
| Winter snipers 0x1E↔0x1F, 0x24↔0x25 | Single / Armor Piercing | +0x0C damage 40→60 (45 on 0x25) | |
| **Militek MGL 0x2A↔0x2B** | **Grenade / Delay** | **0x2A +0x70 = 0x4 (impact explode); 0x2B +0x70 = 0x280 (0x100 bounce + 0x200 fuse 210 ticks = 3.5 s)** — the "always delayed" mode | both +0x6C = 0x828 (gravity) |
| **AT-420 Sentinel 0x2C↔0x2D** | **Guided / Unguided** | **0x2C +0x6C = 0x12204 (0x4 = the rocket camera + look steering); 0x2D = 0x1a200 (no 0x4)** — 0x2C range 200/speed 0.25, 0x2D range 500/speed 0.33 | |
| AT-600 Scorpion 0x2E↔0x2F | Heatseeker / Unguided | 0x2E +0x6C = 0x32200 (0x20000 homing); 0x2F = 0x1a200 | |
| Torpedo Launcher 0x30 | Guided | +0x6C = 0x20200 — homing, but NO 0x4: no rocket camera [D PCSX2] | |
| Phoenix Samurai 0x32↔0x33 | Overcharge / Beam | charge-and-release vs drain (+0x70 0x100000 vs 0x2) | [D PCSX2 behaviour] |
| Frag/Stun/Smoke 0x34/0x35/0x36 | none | +0x70 = 0x280 / 0x2280 (flash) / 0x20280 (smoke) — fuse 210 | |
| Remote Mine 0x37↔(detonator) | mine / detonator | +0x70 = 0x18300, fuse 420 (7777 for the player — trigger-detonated [M 0x12c5a0:219-229]) | |
| Laser Tripbomb 0x3A | none | +0x70 = 0x80100 | |
| Satchel 0x3B..0x40 | none | fuse `((id−0x3B)·5+5)·60` [M 0x12c5a0:199-208] | |

Grenade-mode-count records 0x60/0x61 [L count 2]: the only mode-BYTE cyclers.

## The bullet flags that make the modes behave ([M], readers)

- `+0x6C & 0x8` gravity → the grenade path (`Bullet_update` case 1 applies `WldGravity`).
- **`+0x70 & 0x4` explode on impact** — `Bullet_CollisionHandler` 0x12ce80:194-206 runs
  `Explode_Create(wd+0x8, wd+0x8, wd+0xC, …)` at the contact. Carriers [L]: 0x1B, 0x2A, 0x2C, 0x2D,
  0x2E, 0x2F.
- **`+0x70 & 0x100` bounce, `+0x70 & 0x200` tick fuse** — the fuse (BU+0xFC from `Bullet_init`
  0x12c5a0:103,196-229: 210 default, 300 with `+0x70 & 0x10000`, `((id−base)·5+5)·60` for the
  satchel family, 420/7777 for the mine) is decremented per tick and exploded at < 0 in
  `Bullet_DoTrails` 0x12d890:262-395 (0x2000 → the flash branch, 0x20000 → the smoke branch).
  Carriers [L]: 0x2B, 0x34-0x37, 0x3A, 0x3B-0x40.
- `+0x6C & 0x20000` homing (`Bullet_homing` 0x12f358 via `Bullet_update`:483), `& 0x10000` the
  Swoosh smoke trail (0x12c5a0:116-133), `& 0x2000` the per-tick flight light (0x12e7b0:271-277).
- **`+0x6C & 0x4` the GUIDED-MISSILE CAMERA** — see below.

## The guided missile camera (AT-420 Sentinel 'Guided' 0x2C)

- **Handover on fire** [M `Bullet_init` 0x12c5a0:234-254]: a `+0x6C & 4` shot whose owner is the
  player runs `Player_ChangeSubState(owner, 10)` (0x190630 — clears the anim set), stores
  `BLData+0x878 = the bullet obj`, `BLData+0x942 = the old substate`, `Player_SetCamMode(BLData, 0xc)`
  (0x195928), `BLData+0x944 = 0`.
- **The view** [M `Player_PositionCamera` 0x18f5d8 case 0xc]: while `BLData+0x878 != 0` the view
  matrix IS the missile's `obj+0x90` (nose cam, no offset); the rocket body is drawn only in other
  views; Bond draws in all views; the first-person gun is hidden.
- **The steering** [M `Bullet_update` 0x12e7b0:177-244, gated on owner class 3 ∧ `owner+0xF6 == 10`]:
  the look analogs (`Input_Actionf(slot, 0, 1)` yaw, `(slot, 5, 1)` pitch) scale by 0.024543693
  (π/128) per tick, yaw about the world up, pitch clamped inside the ±0.995 pole; a roll value
  `BU+0x58` accumulates and decays ×0.965 (it feeds only the HUD ring). A 0.1-radius sphere test at
  the bullet destroys it on contact.
- **The end** [M `Bullet_Delete` 0x12e660:26-30 + `Player_Update` 0x193600:242-256]: on death
  `BLData+0x878 = 0`, `BLData+0x944 = 0xf`; substate 10 counts it down, then cam mode 0 and the
  saved substate return — the PCSX2's ~15-tick red-static freeze. The impact blast is the
  `+0x70 & 0x4` Explode (all four rocket records carry it [L]).
- **The HUD** [M `HUD_CreateRedeemer` 0x19dec0 / `HUD_UpdateRedeemerPane` 0x19e178]: the pane's
  ring/bracket sprites (texture 0x200045e) rotate with `BU+0x58`; a bracket locks onto copter bodies
  projected to screen (bright inside 32 px of centre); the static overlay (0x2000465) alpha climbs
  once `elapsed/lifetime > 0.85` (`+ (ratio−0.85)·3400`); scanlines jitter every other frame;
  health/crosshair hidden. NOT yet ported (the sim half is; the pane needs the renderer).

## The port

- `crates/nf-game/src/grenade.rs`: `Grenade.explode_on_impact` (+0x70 & 0x4) — impact explode, and
  the range-out death of a no-fuse record is SILENT (state 4 has no Explode [M 0x12e7b0:158-162]).
- `crates/nf-game/src/drone.rs`: `PlayerBullet.guided_cam`, `steer_missile` (the steering above),
  `blast_at` (the +0x70 & 4 impact blast through `explode::propagate`).
- `crates/nf-view/src/scene.rs`: `Sim.missile_cam` (+ `view_override()`), the substate-10 input
  zeroing + the restore countdown, the grenade spawn's flag feed and the fuse gate (`+0x70 & 0x200`).
- Tests: `crates/nf-view/tests/fix_weapon_fire_modes_live.rs`.

## Handling (refire, spread, zoom, reload)

- **Refire** [M `Player_WeaponFiring` 0x1a5460:601-663]: a fresh press with the timer run out arms
  `spp[mode]` (+0x28 + 2·mode) shots and sets the timer to 1; the timer falls by `FRAME_RATE_MUL`;
  a shot needs the timer at ≤ 0 after the fall and re-arms it to max(+0x40, 1). Without a fresh
  press a run-out timer returns before the fall (:627-629). View state 9 with `+0x68 & 0x8000`
  takes no shot (:631-636) — the pump Frinesi 0x1c, the snipers and the Delta Repeater 0x11 refire
  at their fire script's end (80 / 130 / 80 frames [D 07000005]), not at +0x40.
- **Spread, player owner** [M `Bullet_init` 0x12c5a0:139-172]: s = (+0x24 + bloom) · mul, bloom =
  trunc(min(spp[mode] − `BL+0x936`, +0x92) · +0x98) (the shot's place in the pull), mul = 0 only for
  `+0x68 & 0x400000` with the aim bit set; the cone radius is (FRand(2s) − s) · 0.0014. No movement or
  crouch term: `Player_WeaponInitBullet` 0x1a4bf0 aims head → `Player_GetAimingPoint` 0x1a4a98.
  Port: `nf_game::bullet::player_spread` / `cone`.
- **Zoom** [M `Player_Zoom` 0x1a1a70]: the zoom axis (action 6, aim bit only; the pad's d-pad up −1 /
  down +1) scales target and current ×1.025/tick (÷ for +), the current then becomes the target and
  the base record's remembered zoom (`BL+0x1B4 + base·0xC`); the idle axis eases the current ×1.15/tick
  onto the target; the target is clamped to [1, min(+0x88, 50)]. The aim rise restores the remembered
  zoom (sqrt(+0x88) at level start [M 0x1a17f8:51]; PlayerSetting+6 = 1 [M 0x17edd8:22]), the fall
  sets 1.0 [M `Player_Weapon` 0x1a24c8:156-187]. No scope-flag gate in the game (the mouse reaches the
  aim bit on +0x68 & 0x40 records only — the port's design). Port: `nf_game::gun::zoom_tick`.
- **View-state gate** [M 0x1a5460:631-648]: after the countdown a state outside {0, 9, 0xB} refuses the shot: the
  holster/draw, the reload, the aim raise/lower. The press edge is spent there, so a trigger held through a draw does
  not fire when it ends, and the refire timer keeps falling through a reload. Port: `Gun::refire_blocked`.
- **Reload key** [M 0x1a5460]: read only with the trigger's level at 0, and refused with the aim bit set on a
  `+0x68 & 0x40` record. The auto-reload is the release arm; with nothing carried the same arm is
  `Player_HandleHasNoAmmo` 0x1a2ba8: fists, 0x32/0x33 and 0x4A..0x51 stay in hand, `+0x68 & 0x10` offers the twin,
  the rest take `Player_GetBestWeapon`. The grenade family's case 0xc runs the same request at the +0xB4 stop.
- **Aim bit clears** [M 0x1a24c8:66-99]: every record in states 1-4; `+0x68 & 0x40` records also in 6, 7, 8, and in 9
  with `+0x8000` when the bit was not already set. A scope drops for its reload; with the button held the raise
  re-queues from idle [M :36-47, :117-152].
- **Gadget use gates** [M 0x1a5460, on the trigger-held arm, before `Player_RoundToFire`]: the Micro-Camera needs the
  aim bit; the Q-Claw needs `BL+0x95F == 2` (a type-2 third icon's arm) or MP, and refuses while reeling; the Q-Worm
  needs an idle PC as the aim target within `+0x1C`. A refused press runs no script.
- **Guided ride** [M 0x1a5460 → `Bullet_handle_object_destruction` 0x12cd98]: a fire press with `obj+0xF6 == 10` blows
  the rocket where it is. Port: `Drones::detonate_guided`.
- **Reload time**: the key (branch A, disasm 0x1a5fb0-0x1a5fdc) plays +0xA4 at 1.35× from frame 0; the
  auto-reload (branch B, 0x1a62e4-0x1a631c) starts flag-0x1000 records at frame 6.
- **Under water** [M 0x1a5460:503-506]: `surfaced == 0` refuses the held trigger unless `+0x68 & 2`
  (fists, Torpedo Launcher, laser, stunner twins).
- **Q-Claw aim assist** [M `Player_AutoAim` 0x1a7c98:63-78]: scans class 'Q' grapple points only.
- Tests: `crates/nf-view/tests/weapon_rules_live.rs` (the rules above through the real input path),
  `crates/nf-view/tests/weapon_handling_live.rs` (every weapon's cadence/clip/reload/zoom from
  the disc, the 0x8000 gate through the Sim), unit tests in `gun.rs` / `bullet.rs`.
