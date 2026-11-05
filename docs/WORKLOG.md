
## gadgets-across-part-changes
Started 2026-11-05 12:23 UTC. Branch s/gadgets-across-part-changes (cut from origin/master f68a72216).
Task: a carry sweep found a few gadgets lost on three of the 14 part changes tried, while weapons
and ammo carried. List which items drop on which change through the real level exit, read the
game's carry rule [M RamSave and the level-change path], and fix anything the game keeps. If a
drop matches the game, pin it with the address.

Progress 2026-11-05 12:23 UTC: branch cut, WORKLOG started. Sandbox survived (repo+work+ISO
present, target/ warm).

Progress 2026-11-05 12:25-13:10 UTC: the sweep + the game's carry rule, all [M]-cited.
- The sweep (crates/nf-view/tests/gadget_part_carry_live.rs): the 14 within-mission part changes
  (mission::LEVELS parts per mission: 3+3+2+1+3+2), each hop = cold-load src → grant the 18
  tracked gadget ids (+ the P2K so the p2k-fallback never fires) → `warm_exit_save`
  (Player_RamSave 0x195670) → `load_with_carry` (Player_RamLoad 0x1957a8) → read owned/clip.
- The game's carry rule [M]: RamSave copies the whole inventory (the 0x42 spare-ammo words
  +0x038, the 114 12-byte weapon entries +0x07C) and RamLoad restores it — weapons AND ammo AND
  gadgets ride the save. The drops come from the load itself: `ResetMap_Load` 0x1d6360's
  LoadState-3 arm runs `Mem_Init` + `hashtable_initialise` on EVERY level load (:130-134) and
  the chain loads only the shared 07000500 pack + the level's own archive (:28-36; 07000500.bin
  holds no objects/skins [D script-census.md]) — the object table the gate sees is per-load,
  NOT the accumulated session. Then `Player_InitWeapon` 0x1a7ef8's tail runs
  `Player_CheckWeaponsLoaded` 0x1a97c8: records with weapon_data+4 == 0 are skipped (kept),
  `+0xDC == 0` clears owned, a `hashtable_getitem(+0xDC)` miss clears it; `Player_EquipWeapon`
  0x1a8a58:20 refuses the same equips at kit time. The measured warm-15 Grapple loss
  [L 2026-10-14, fix_start_loadouts_live] is the empirical proof of the per-load reading:
  05's own archive carries the Grapple fp skin (its boot equip owns it [L ee_start_05 owned
  0x50]), yet the game's warm 15 drops it — the accumulated-session reading in the §2/g-021
  comments is wrong for the OWNERSHIP gate (the conflict is written up in the report; the
  §2 mesh merges are a render-side divergence the main agent should review).
- 07000007 is the party disguise: case 0x7000007 runs `Player_InitAmmoWeapons` + the kit EVEN
  warm [M 0x1a7ef8 case 0x7000007] — the game wipes the carried gadgets at the party by design.
- RESULT: every one of the port's drops on the 14 hops MATCHES the game's rule — the sweep
  asserts port == game per hop and passes. Kept = the fp skin (+0xDC) is in the destination's
  own archive (or the record is not selectable); dropped = otherwise; 07 keeps only its kit.
  Nothing to fix — the drops are pinned with 0x1a97c8 + 0x1d6360:130-134 + 0x1a8a58:20.
- An accumulation-model fix was tried first (session fp skins merged into the gate from earlier
  story archives) and reverted: it contradicts the [L] measurement and OOMs the multi-load
  tests in its merge form (baseline proof: master passes, the merge SIGKILLs).
- Tests: the sweep 1/1; fix_start_loadouts_live 8/8 (all pass individually; the full binary
  SIGKILLs at test 6 in this sandbox today — a borderline OOM on master's own code, it passed
  8/8 earlier today); qworm_not_on_0c_live 3/0; gadget_carry_live 6/0; level_carry_live 8/0;
  mission_stats_carry_live 2/0; loadout_disc_rows_live 2/0. The final tree is master + the new
  test file only (the lib.rs fix attempt fully reverted); nf-game's full suite ran green on
  this master this morning (1077/0/8) and the 12 walks 89/0 — same code.
