# MP spawn equip: the fists-then-draw (mp-respawn-fists-draw)

Every MP spawn — the match load, each respawn, the late join — leaves the player HOLDING THE
FISTS for one tick, with the weapon set's spawn weapon pending as a switch request. The spawn
weapon arrives through the normal weapon switch (the draw), never instantly.

## The equip's tail: the anim trio

`MP_EquipPlayer` 0x1855f8 grants the kit (the wipe, the spawn weapon with `clip_size·2` rounds
split clip/carried, the grapple, the scenario bonus weapon [M 0x1855f8:14-48]) and then writes
the anim object's (`*(obj+0xdc)`) weapon trio [M 0x1855f8:50-53]:

- `+0x64 = 0x47` — the saved id (the anim's held model) is the fists;
- `+0x62 = 0x47` — the held id is the fists;
- `+0x63 = sVar4` — the spawn weapon id goes into the pending/next slot (the switch request);
- `view state +0xf4 = 0` — idle, so the switch machinery sees the request on the next tick.

`Player_WeaponNone` 0x1a9858:12-14 writes the same `+0x62/+0x63 = 0x47` pair when Bond goes to
the hands [M], which is what makes 0x47 the fists/no-weapon id across the trio.

The load path does the same: `Player_InitWeapon` 0x1a7ef8:137-139 writes `+0x64 = 0x47` and
`+0x62 = 0x47` before its MP arm calls `MP_EquipPlayer` [M 0x1a7ef8:143-145] (the SP arms write
the level's starting weapon into `+0x63` instead, same fists tail). So there is NO spawn path —
load, respawn (`MP_ReSpawn` 0x185368:74 → `MP_EquipPlayer`), late join — where the player starts
holding a gun.

## The consumer: the normal weapon switch

`Player_SetWeaponAnimObj` 0x1a2e38's default view state notices `+0x63 != +0x62`, saves the old
held id (`+0x64 = +0x62`) and calls `Player_WeaponSelect` 0x1a2cc0 [M 0x1a2e38 LAB_001a31e8].
`Player_WeaponSelect` compares weapon classes: from the fists the classes differ, so it reads
`weapon_data[+0x64's weapon]+0xc4` (the holster script of the weapon the anim shows — the fists
row, whose `+0xc4` is 0) and takes the direct-draw arm: `+0x62 = +0x63` (the held id flips to
the spawn weapon) and view state `+0xf4 = 3` [M 0x1a2cc0:33-35]. Case 3 plays the draw script
(`weapon_data[the new weapon]+0xc0`) and moves to state 4; the switch settles back to idle a few
ticks later [M 0x1a2e38 case 3]. Only then can the trigger fire (the game's state 1/3 checks
refuse the switch/trigger paths [M 0x1a6f80:33-47]).

## What the port does

- `Sim::mp_equip_local` (nf-view scene.rs) mirrors the trio: `gun.weapon = 0x47`,
  `gun.saved_weapon = 0x47`, `gun.switch_request = the spawn weapon`, `gun.switch_state = 0`,
  and `drones.bond_weapon = 0x47` (the AI's class byte reads `obj+0xdc+0x62` [M 0x178768]);
  it returns the equipped id so the load can seed the shot gates with the real weapon
  (`gate_setup`/`gate_equip` — the gate is the port's shared anti-cheat kit record, re-seeded at
  the respawn tick as before). The port's own switch machinery (the `Gun::switch_state` machine
  in `Sim::run`, `Player_SetWeaponAnimObj` 0x1a2e38 + `Player_WeaponSelect` 0x1a2cc0) then runs
  the draw exactly as for any in-game switch.
- The wire byte (nf-net input byte 21, `held`) is built from `sim.gun.weapon` (lib.rs) — it
  carries 0x47 at the equip tick and the spawn weapon from the draw tick on, so every machine's
  remote view follows the same fists-then-weapon sequence.
- The shot gates re-seed with the spawn weapon at the equip tick (not at the draw's end) — the
  equip's grant is the gate's law [M 0x1855f8:22-26]; no shot can be validated mid-draw because
  the local trigger is blocked while the switch plays.

## The RNG note (why old bot pins moved)

The port's instant equip used to start the held weapon's IDLE view script at the spawn tick, and
its frame-1 op-1 sound (weapon 6's idle 0x060004b9, sound 0x1, looped every ~37 ticks) fed
`Sound_Play3D` → the bots' hearing → shared-RNG alert draws at tick 0. With the draw first, the
idle script starts only after the draw settles, so those tick-0 draws are gone — the stream is
now the one the game's own spawn order produces. Anything pinned to the old stream (a specific
bot firing inside a 60 s window) was pinning the artifact, not a decompile law.

## Tests

- `crates/nf-view/tests/mp_death_hands_online.rs` — the fresh load's trio pins, the respawn
  tick's fists + request, the draw-tick flip (held = the spawn weapon, view state 3), the
  settled kit (`assert_kit`), the 3-machine hash agreement.
- `crates/nf-view/tests/mp_respawn_ammo_live.rs` — offline + online respawn re-arms: the fists +
  request at the respawn tick, the draw settle, the `clip_size·2` kit.
- `crates/nf-view/tests/mp_bots_live.rs` — the bots' match-level fire law (the stream note above).
