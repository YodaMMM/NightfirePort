# mp-respawn-ammo — every MP respawn re-arms the game's spawn loadout

The user (an MP match from the menus, 2026-10-19): "you respawn with no ammo".

## The game's respawn loadout [M]

A human's respawn is `MP_ReSpawn` 0x185368 (ACTION.ELF). Its human arm (obj+0xff ∈ {3, 0x12} — 3 is
the LIVE state too, so a round-restart respawn re-arms survivors as well):

- `Player_SetHealth(MPSettings[player·0x30+0x2c] + 100.0)` — the slot's armor_bonus + 100
  [M 0x185368:65-73];
- **`MP_EquipPlayer(obj)` 0x1855f8** [M 0x185368:74] — the spawn loadout, identically to the match
  load: `Player_InitAmmoWeapons` WIPES the carried rounds, the clips and the owned bytes first, then
  the weapon set's spawn weapon `Upgrade_MPWeapon(base(PickupMatrix[set][0]))` with `clip_size·2`
  rounds (the equip splits `clip_size` into the clip, the rest carried), the fists, the AR's grenade
  ammo, the grapple when MPSettings+0x1D0, the scenario bonus weapon 0x3B (Protection team 0 /
  Demolition team 1) [M 0x1855f8:14-53; the matrix bytes [D] the ELF's 0x2b9100];
- the death sequence had left the fists (`Player_WeaponNone` 0x47 [M 0x190810:61]) and the spent
  rounds — the wipe is what makes the fresh kit ALL the player has.

A bot's respawn is `BOT_respawn` 0x124e38 → `BOTWEAP_InitWeapon` 0x129e48:44-48: the set's spawn
weapon with `clip_size·2` rounds [M; nf-game::mp::equip::bot_start_weapon].

The rounds are the shared, hashed `mp_combat` gate state (docs/systems/mp-shot-gate.md) — every
machine re-seeds the respawned slot's gate the same way, or the hashes diverge.

## The port's divergence

The port's `mp_equip_local` (`Sim::mp_equip_local` — `nf_game::mp::equip::mp_equip_player` into the
local inventory + the gun) and the shared gate's `MpCombat::gate_equip` ran at the match load and in
the ONLINE respawn (`Sim::step_net_combat`'s respawn loop), but NOT in:

1. the OFFLINE death→respawn (`Sim::step_mp_death_respawn`) — a player who died after spending his
   rounds respawned on the fists with the spent inventory: "you respawn with no ammo";
2. the round restart (`Sim::step_mp_scenario_sync`'s `restart_pending` arm) — only a dead local
   re-armed; survivors kept their pickups into the next round, and no gate re-seeded (the shared
   rounds stayed the previous round's).

The bots already re-armed at every respawn (`MpMatch::arm_bot` from `step_mp_bots`' respawn [M
BOT_respawn → BOTWEAP_InitWeapon]).

## The fix

- `step_mp_death_respawn` (offline): after the respawn apply, `mp_equip_local()` +
  `gate_equip(0, …)` — the same pair the online path already runs.
- `step_mp_scenario_sync`'s restart: every respawned human slot's gate re-seeds
  (`gate_equip(slot, …)`), and the local re-arms unconditionally (the game's full arm runs on the
  live obj+0xff==3 too [M 0x185368:74, 0x183fc0:55-75 → MP_ReSpawn per slot]).
- `nf_game::pickup::Inventory::clip_slot` is `pub` now (a pure getter the draw code and the tests
  read the clip index through).

`MP_ReSpawn`'s gates and the SP arms are untouched; a fresh match start runs the same
`mp_equip_local`/`gate_setup` pair as before (the load path).

## Tests (crates/nf-view/tests/mp_respawn_ammo_live.rs, NF_ISO-gated)

Each drives the real paths and pins numbers only, each with a can-fail sister:

- `offline_respawn_rearms_set_0_normal` / `_set_1_pistols`: load (set 0 Normal: spawn weapon 6; set
  1 Pistols: spawn weapon 2 [D the PickupMatrix bytes]) → the fresh kit (sister) → spend the spawn
  weapon dry through the real trigger taps → health 0 → the death sequence → the 5 s respawn → the
  kit is back exactly: the spawn weapon drawn and owned, `clip[clip_slot]` = `clip_size`, the ammo
  type carrying the other `clip_size`, health = armor_bonus(0)+100 [M 0x185368:65-73]. These FAILED
  on the pre-fix tree (clip 0 after the respawn — the user's symptom).
- `online_respawn_rearms_and_the_machines_agree`: two machines in lockstep (`nf_view::net_tick`,
  every tick's hash asserted); the fresh kits and the shared gate's split (sister) → a real P2K
  kill of slot 1 → the 5 s wait → the respawn: machine B's local kit restored, both machines' gate
  copies for slot 1 identical with the fresh split, the respawned health 100.0 on both.

Verified additionally: `mp_scenarios_live` all 12 `*_online` tests (every scenario's restart and
death→respawn path), `mp_body_anim_live` 2/2, `mp_combat_weapons_live::hitscan…` (the kill → the
idle respawn window), and the new file's 3 tests — all green on the fix tree.
