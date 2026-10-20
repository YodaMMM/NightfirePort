# The bathhouse girls' arms — why no 02 civilian may draw a gun

Task `bathhouse-armed-girl-02` (playtest 1017-1139: "one of the bathhouse girls had a gun and
started shooting at me" — 07000002, F12 at `--bond -43.11,6.46,10.37`, frame 4206).

## The placement [D]

The bathhouse girls are 07000002's three skin-0x05000034 drone placements (kind 8; the census
dump `crates/nf-view/tests/census_07000002_drones.rs`, run 2026-10-19):

- #464 (-37.5, 6.5, 4.4) — dtype 0xE (param 15), mode block 3 (param 16) → mode 7
  (EnemyRunToPoint) [M `NDrone2_GetDTYPENEW` 0x14c938:111-117]
- #465 (-45.5, 5.8, 12.2) — the F12 girl, 2.9 units from the playtest camera; mode 5 (Hostage,
  `GetDTYPENEW` dtype 0xE + prop 0x4E → 5 [M 0x14c938:80-85])
- #466 (-12.0, 6.6, -39.1) — mode 5

All three carry **weapon parameter 9 = 1** (the fists) and **weapon-swap parameter 10 = 1**
[D]. The start-area kind-9 civilian (#463, skin 0x0500004f, team 2) carries 1/1 as well. The
guards beside them (#442-#462, skins 0x05000036..0x3a) carry real guns (param 9 =
0x12/0xe/0x1c/0x1e) and swap id 0xE.

Both girls' teams are 3 — the civilian team of `NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8's
team switch (dtypes 10/0xB/0xD/0xE → 3; `NDrone2_IsCivilian` 0x14aae0 tests the same set).

## What the game gives her [M]

- The weapon byte: `NDrone2_DefaultInit` 0x14b300 takes parameter 9 (`piVar19[0x16]`), default
  1, 0 only for skin 0x0500001E [M 0x14b300:94-99,264], stored to the anim object at +0x62
  [M 0x14b300:668].
- The weapon-swap id: `*(char *)(iVar3 + 0x45) = (char)piVar19[0x17]` = parameter 10
  [M 0x14b300:266]. The girls' is 1 — no drawable gun.

## The arming chain and the game's two gates [M]

An attacking drone without a gun (`anim+0x62` ≤ 1) goes through `DroneFunc_AttackNoWeapon`
0x14af80, called from the Attack state's ENTER (`NDrone2_DSTATE_Attack` 0x164210:32-38, after
`DroneFunc_FirstAttack` returns 0) and from `DroneFunc_FirstAttack` 0x148010:101. It RETURNS
the next state:

- armed (`cVar2 > 1`): team 3 flips to 1 and the armed attack continues (return 0);
- unarmed: **0x12 (CivilianScared)** — unless the swap id `+0x45 > 1`, then **0x66 (DrawWeapon)**.

`NDrone2_DSTATE_DrawWeapon` 0x166000 re-gates the same way: case 1 (enter, msg 1) sends
`+0x45 < 2` drones straight to SetState(0x12) without starting the draw anim; case 2 (the draw
event, msg 2) is the only writer of the sidearm — `anim+0x62 < 2 → DroneWeap_ChangeWeapon(6)`
[M 0x166000:41-43] — and can only fire for a drone that entered with a swap id.

So the game's answer to the mode-7 girl attacking Bond mid-course is CivilianScared: she never
plays the draw anim, never takes the sidearm, never fires. (The mode-5 girls are Hostages and
never reach Attack at all in the traced play.)

## Why the port armed her

The port's unarmed Attack ENTER sent EVERY attacker to DrawWeapon (the `+0x45` branch existed
only as a comment), and the DrawWeapon EXIT hardcoded case 2's sidearm (weapon 6, anim set 4,
16 rounds) without any swap-id gate — the girl drew a gun in combat. The port also never parsed
`+0x45` (the field existed, documented "not parsed from the setup word yet", always 0).

## The fix (this branch)

`crates/nf-game/src/drone.rs`:

1. `Drone::new` loads `weapon_swap` from parameter 10 [M 0x14b300:266].
2. The unarmed ATTACK ENTER arm implements `AttackNoWeapon`'s return: swap id > 1 →
   DrawWeapon, else CivilianScared [M 0x14af80].
3. The DRAW_WEAPON ENTER bails to CivilianScared when `weapon_swap < 2` [M 0x166000 case 1].
4. The DRAW_WEAPON EXIT arms only for a drone that entered with a swap id (the port's EXIT is
   the game's msg-2 event; a bailed state leaves by EXIT and must not arm).

The armed Attack-ENTER's team-3→1 flip [M 0x14af80's first branch] is NOT ported: no 02 drone
reaches it (every 02 civilian is unarmed), and no test pins a team-3 armed attacker yet.

## Tests

`crates/nf-view/tests/bathhouse_armed_girl_live.rs` (opt-in NF_ISO):
- `every_02_civilian_spawns_unarmed_with_no_drawable_swap_id` — every 02 civilian placement
  spawns weapon 1 / swap id 1 / civilian team; every guard-skin drone spawns armed with a
  drawable swap id; #448 (the F12-scene guard) holds 0x1c [D].
- `the_woken_girl_never_draws_or_fires_and_the_guard_still_fires` — the pads' channels
  (22/23/25 [D params 3]) wake the bathhouse block with Bond at the playtest spot; the mode-7
  girl's state log answers her unarmed attack with CivilianScared, never DrawWeapon; no
  civilian ever owns a bullet; the armed guard on the same channels opens fire (the sister that
  fails pre-fix — the girl carried weapon 6 in combat ~tick 700 before the fix).

## Live checks for the main agent

None required — no visual change is claimed (the girls' towel model/pose is untouched; the fix
only removes a state path the game never takes). If wanted: PCSX2 at the playtest spot after
the terrace pads, watch `Drone+0x62` (anim weapon byte) of the skin-0x05000034 drones —
expected 1 (fists) throughout; the port matches.
