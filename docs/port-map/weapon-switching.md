# Port map: weapon switching

Every ACTION.ELF function the weapon-switching subsystem involves. Status: **ported** / **partial** / **stub** / **not started**.
Addresses are this build's (ACTION.ELF `e7ffdcb3…`). Tags: [M] decompile, [D] disc, [L] snapshot, [S] suggested.

## The switch cycle (which weapon to switch to)

| Address | Name | Status | Notes |
|---|---|---|---|
| 0x1a24c8 | `Player_Weapon` | **partial** | The per-tick weapon manager. Reads the d-pad/gadget actions and calls `Player_WeaponChange`. Ported: the trigger, reload, aim-mode toggle, zoom. **Not ported**: the weapon-cycle (actions 0xA/0xB → mode=1 gadget cycle), the gadget-cycle (actions 0xF/0x10 → mode=0 weapon cycle), the `bVar4` gate (switch channel 0x48 on level 07000007), the fire-mode cycle (weapon+0x05, `BLData+0x1BB`), action 0x13 (aim mode toggle). STANDIN: nf-view's `Sim::step_bond` handles switching directly instead of calling this. |
| 0x1a6f80 | `Player_WeaponChange` | **ported** | `Gun::cycle` (the walk from the request's base, the twin step, the wrap) + `Gun::toggle_back` (+0x930/+0x932) + the entry refusals in `Sim::step_bond`'s `cycle_ok` (substates 2/10/12/13/16, held 0x47/0x5D/0x5E, the aim bit, the view states). A press during a holster or draw moves the request; a walk that finds nothing leaves it. |
| 0x1a6eb0 | `Player_IsBetterWeapon` | **ported** | `Inventory::is_better_weapon`. |
| 0x1a16c0 | `Player_WeaponHasAmmo` | **ported** | `Gun::has_ammo` + `Gun::placed_keeps_ammo` (the +0x68 & 0x20000 object walk: a placed mine keeps its detonator). |
| 0x1a9680 | `Player_GetBestWeapon` | **ported** | `Gun::get_best_weapon`: base + the family's twin marker, the AIMS-20 on its rifle. |
| 0x1a2ba8 | `Player_HandleHasNoAmmo` | **ported** | `Gun::no_ammo_request`, from `update_with`'s `dry` flag (the release arm) and the case-0xc dry arm; the 'R' arm stays in `Sim::run`. |
| 0x1a9858 | `Player_WeaponNone` | **ported** | `Gun::go_to_hands`, from the ladder, wire, creep, gun-mount and death paths only. |

## The holster/draw state machine (the animation)

| Address | Name | Status | Notes |
|---|---|---|---|
| 0x1a2cc0 | `Player_WeaponSelect` | **partial** | Starts the switch. Ported: checks +0xC4 (holster) to decide state 1 vs 3. **Not ported**: the variant-swap path (same base weapon → plays +0xC8 at 1.3x, state 8), the `weapon_data[w]+0xDC` first-person model check, the `BLData+0x1BC` variant-slot update. STANDIN: nf-view's `Sim::run` does the state 0→1 transition inline. |
| 0x1a2e38 | `Player_SetWeaponAnimObj` | **partial** | The view-script state machine. Ported: state 0 (idle → holster), state 1 (holstering → plays +0xC4), state 3 (draw → plays +0xC0), state 4 (draw done → idle). **Bug fixed**: was using +0xCC (idle-alert flag) for holster and +0xD0 (idle-alert raise) for draw; corrected to +0xC4 (holster) and +0xC0 (draw). **Not ported**: state 2 (holster done → weapon byte swap + MP team message), state 8 (variant swap → +0xC8), state 0xD/0xE (MP variant), the idle-alert sub-states (BL+0x959 = 0/1/3, the target scan, +0xD0 raise, +0xD4 lower), the view-offset position update (+0xE0..+0xEA + bob), the zoom reset. |
| 0x1a5460 | `Player_WeaponFiring` | **ported** | The trigger. `Gun::update_with`. |
| 0x1a8a58 | `Player_EquipWeapon` | **ported** | `Inventory::equip_weapon`. |
| 0x1a8d18 | `Player_EquipAmmo` | **ported** | `Inventory::equip_ammo`. |

## weapon_data fields used

| Offset | Type | Meaning | Evidence |
|---|---|---|---|
| +0x04 | u8 | selectable flag (1 = in the switch cycle) | [M] `Player_WeaponChange` 0x1a6f80 line 179: `weapon_data[w*0x10c + 4] == 1` |
| +0x05 | u8 | shares base clip | [M] `Player_EquipWeapon` |
| +0x14 | u16 | auto-aim accuracy % | [M] `Check_AutoAim` 0x1a73f8 |
| +0x38 | u32 | name label hashcode | [M] `Pickup_Handler` |
| +0x48 | u32 | first-person muzzle flash mesh | [M] `Draw_MuzzleFlash` |
| +0x60 | u32 | drone fire sound id | [M] `docs/systems/sound.md` §7 |
| +0x68 | u32 | flags (0x10 fill-empty-clip, 0x40 scoped, 0x100 one-at-a-time, 0x1000 idle-not-readded, 0x8000 aim-mode, 0x100000 fire-on-trigger) | [M] |
| +0x88 | f32 | zoom factor (1 = no zoom) | [M] `Player_SetWeaponAnimObj` state 4 |
| +0xA0 | u32 | idle script | [M] case 4, case 7 |
| +0xA4 | u32 | reload script | [M] case 5/6 |
| +0xA8 | u32 | reload-start script | [M] case 5 |
| +0xB0 | u32 | fire script | [M] case 9 |
| +0xB4 | u32 | fire-on-last-round script | [M] case 9/0xB |

| **+0xC0** | **u32** | **draw script** (case 3) | **[M] disassembly 0x1a3828: `lw a1,192(s1)`** |
| **+0xC4** | **u32** | **holster script** (case 1, played at 1.25x) | **[M] disassembly 0x1a36c0: `lw v1,196(s1)`** |
| **+0xC8** | **u32** | **variant-swap script** (same-base swap, case 8, 1.3x) | **[M] `Player_WeaponSelect` 0x1a2cc0 line 24** |
| +0xCC | u32 | idle-alert flag (non-zero = has idle-alert behavior) | [M] case 0: `weapon_data[iVar15 + 0xcc] == 0` |
| +0xD0 | u32 | idle-alert raise script (played when a target appears) | [M] case 0: `weapon_data[iVar15 + 0xd0]` |
| +0xD4 | u32 | idle-alert lower script | [M] case 0 sub-state 3 |
| +0xD8 | u32 | holster variant flag (case 1: if non-zero, `AnimScriptAdd` instead of `AnimScriptAddSpeed`) | [M] disassembly 0x1a36d4: `lw a1,216(s1)` |
`Player_SetFiringAnim` (0x1a3c70) picks between +0xB0 and +0xB4 per shot: flag-0x1000/base-0x11 weapons on
their last round (`clip < 1`, post-decrement) play +0xB4; flag-0x80000 weapons (the fists, 0x01 [L]) draw
`Rand(100)` and play +0xB4 on > 50 — the left/right punch alternation (one shared-RNG draw per punch);
flag-0x40000 weapons flip the `BLData+0x958` toggle and alternate strictly. Port: `Gun::firing_anim_script`
(`gun.rs`), called from the view-script block in `scene.rs` (fix-player4).

| +0xDC | u32 | first-person model hashcode | [M] `Player_EquipWeapon` |
| +0xE0..+0xE8 | f32×3 | first-person gun offset (x, y, z) | [M] `Player_PositionGun` |

## BLData (player `obj+0xE0`) fields used

| Offset | Type | Meaning | Evidence |
|---|---|---|---|
| +0x62 | u8 | current weapon id (`anim+0x62`) | [M] |
| +0x63 | u8 | switch-request weapon id (`anim+0x63`) | [M] `Player_WeaponChange` sets this |
| +0x64 | u8 | previous weapon id (for toggle-back) | [M] `Player_WeaponSelect` line: `anim+0x64 = anim+0x62` |
| +0x930 | s16 | last gadget (for gadget toggle) | [M] `Player_WeaponChange` param_3=0 |
| +0x932 | s16 | last weapon (for weapon toggle) | [M] `Player_WeaponChange` param_3=1 |
| +0x959 | u8 | idle sub-state (0 = idle, 1 = recovering, 3 = alert-idle) | [M] case 0 |
| +0x1BA | u8 | owned flag (per weapon, 0xC-stride) | [M] |
| +0x1BC | u8 | variant slot (for same-base variants) | [M] `Player_WeaponSelect` |

## Counts

- **Ported**: 9 (`Player_WeaponFiring`, `Player_EquipWeapon`, `Player_EquipAmmo`, `Player_WeaponHasAmmo`,
  `Player_GetBestWeapon`, `Player_WeaponNone`, `Player_WeaponChange`, `Player_IsBetterWeapon`, `Player_HandleHasNoAmmo`)
- **Partial**: 3 (`Player_Weapon`, `Player_WeaponSelect`, `Player_SetWeaponAnimObj`)
- **Stub**: 0
- **Not started**: 0

## STANDINs (audit-confirmed gaps)

| Item | Reason | Audit |
|---|---|---|
| `0x1DB` change-click sound | The mode-2 "no weapon to switch to" click is not played. STANDIN. | audit-batch2 finding 5 |
| Case 0 idle-alert sub-state machine | The `+0xCC`/`+0xD0`/`+0xD4` idle-alert scripts, the drone-targeting scans, the `0x959`/`0x93A`/`0x93E` counters are not ported. STANDIN. | audit-batch1 case 0 |
| `+0xD8` holster variant script | The `holster_variant_flag` accessor is mislabeled (it's a script hashcode, not a flag) and the path is not ported. STANDIN. | audit-batch1 case 1 |
| `BLData+0x8D4` zoom reset | Zoom is not modelled in the switch state machine. STANDIN. | audit-batch1 case 1/4 |
| Case 0xa (reverse draw), 0xb/0xc (put-away with fire-last), 0xf/0x10 | Not ported. STANDIN. | audit-batch1 |
| Case 9 firing state's gadget auto-swap | The state 9→8 transition (gadgets that switch off after firing) is not ported. STANDIN. | audit-batch1 case 9 |
| Post-switch tail (`+0x30`/`+0x34`/`+0x38` position blend) | Not ported. STANDIN. | audit-batch1 |
