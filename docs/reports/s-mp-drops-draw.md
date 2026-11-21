# s/mp-drops-draw: the dead player's dropped gun is drawn

Rebased on 0590815f1 (s/mp-drops' two commits, then this one). No change here moves the shared
random stream. Two pinned lockstep digests moved because the world hash folds every pickup and
the pickup list gained inert entries; the runs themselves did not change (details below).

## What a player notices

- In a match, the gun a player drops on death now shows on the floor, turning, until someone takes
  it or its 30 s run out. Before, the pickup existed (it could be taken) but nothing was drawn.

## Change

- scene.rs `mp_drop_reservations`: at the match load, after the placed and script pickups, inert
  (`gone`) pickup slots for every weapon a human can hold in that match: the weapon-set row, its
  spawn column through `Upgrade_MPWeapon` (as `MP_EquipPlayer` 0x1855f8 equips it), the arena's
  weapon pickups, the grapple and the scenario bonus 0x3B, each with the match take's remaps
  (`Upgrade_MPWeapon`, 10 / 0xD) [M 0x18e650:141-160]. Each candidate goes through
  `Pickup::player_death_drops` itself, so a slot's id, model and bounds are exactly what a death
  creates. 2 slots per human per weapon. The draw build and the sim build call it with the same
  inputs, so draw slot k is the sim's `pickups[k]`; the draw copies are built live so each gets an
  instance, and `placed_pickup_pose` collapses it while the sim slot is inert, taken or expired.
- `Sim::mp_death_drop` fills a free slot of the drop's id (a never-used one first, else the lowest
  free one; shared state only, so every machine picks the same slot). No free slot: the old
  undrawn push, with an `[mp-drop] no reserved draw slot` log line. The AR's hidden launcher rounds
  still push (hidden pickups are never drawn).
- The bots' pickup rows list the reserved slots in first-activation order (`Drones::mp_drop_used`),
  as the appended drops were, and skip inert slots.
- Load log: `[mp-drop] N reserved drop slots from pickup K: weapons [...]`. Arena 07000023, Normal
  set: weapons 6, 0x19, 0x1C, 0x1E, 0x2C, 0x3B; 12 slots with 1 human, 24 with 2 (from pickup 20).

## Proof

- `mp_death_drops_live` now asserts, offline and on both online machines: the drop takes a reserved
  slot (nothing appended), that slot has drawn vertices in the scene's draw list
  (`Scene::char_verts` with `pickup == Some(k)`), the same mesh vertex for vertex as the arena's own
  P99 pickup (slot 8, 976 vertices), drawn at the pickup's matrix while it lies there; and the pose
  is gone once taken or expired. Fail-first: with the slot fill forced off (the old push), 3 of the
  6 tests fail (local drop, expiry, online drop: "has drawn vertices" / "takes a reserved slot");
  all 6 pass with it.
- Two windows (host + joiner, the copied exe), the same setup as the earlier run where nothing
  showed: the gun lies on the floor in front of the host and turns between captures (ticks 230,
  300, 420, 540, 660), until the host takes it on tick 740; both windows' drop logs agree tick for
  tick. Shots and logs: `NightfirePort-work/audits/mp-drop-shots/` (`before/`, `after/`, and the
  floor crops `before-host-crops.png` / `after-host-crops.png`).
- `mp_two_instance_live` (opt-in): passes on this build (drop on tick 179 with 16 rounds, gone on
  tick 1979 on both windows, the same seven headings).

## Tests

- All 57 of `mp_*_live` / `mp_*` binaries, `load_scene_equality`, `local_match_bot_pickups_live`,
  `pickups_g029_live`, `pickups_g029_upgrades_live`, `fix_player2_live`, `fix_objects_live`,
  `fix_objects2_live`, `doublecross_items_live`: green after the two re-pins below.
- Re-pinned (the world hash folds every pickup; the reserved slots add entries):
  `mp_perf_16_live` 0xa730_a7ac_a608_f75a -> 0x25c0_6d1e_41bf_5788 (still 36 kills);
  `mp_net_slots_post_move_live` 4-human tick 0 0x25f7_edeb_ca9a_12e0 -> 0x2e61_5359_e4ed_c683,
  final 0xaebc_4cc2_1a19_ca59 -> 0xfa4f_3eb1_fbf6_ddda. With the reservation switched off the old
  values come back exactly on this tree.
- nf-game lib `pickup::` `player::` `mp::` `death::` `progress::` (130).

## Rebase notes

- Master's `Upgrades` row (`Upgrade_Weapon(id, &upgrades, game_complete)`) met s/mp-drops' take
  id: `Inventory::take_id`'s single-player arm reads `self.upgrades`; the kit keeps master's
  `upgrades` body inside `init_level_loadout_ex(.., game_complete)`.

## Unproven / not done

- Not compared with the real game live.
- A player who dies more than twice with the same gun within 30 s (per human count) runs out of
  free slots; that drop is still created and takeable but not drawn (logged).
- Read only, not checked in a render: the script pickup reservations (fix-safe08) are built `gone`
  and the draw loop skips `gone` pickups, so they may get no instance at all. Same pattern this
  change avoids by building its draw copies live.
