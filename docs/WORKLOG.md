## level-13-interactables-2 (2026-11-01, s/level-13-interactables-2)

Branch from origin/master (e79794da). Task: level-13 — every player-touchable KIND on 07000013
behaves like the game. interactables_07000013_live.rs has 6 tests; using interactables_0700000b_live.rs
as the model, run the census of 07000013 [D every placement kind], and for every kind a player can
touch add the live test that drives it through the real path (walk/E/shoot) and asserts the game's
result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result.
Fix any port gap the tests expose, with evidence. Run interactables_07000013_live,
walkreal_07000013_live and nf-game; quote the totals. Full suite with the fast runner on the
final tree (merged with origin/master).

## Progress (2026-11-01, ~01:10 UTC)

- Census [D]: re-walked the 07000013 archive placements with an independent parse of the same
  disc bytes (ISO9660 → ACTION.ELF FileList → FILES.BIN → archive dir → map block 0x1A per
  docs/formats/placement.md): 439 placements across 42 maps, main map last in archive order —
  every kind/param in docs/levels/07000013.md §7 confirmed (doors #274/#355, glass #286 filter
  0x30, pickups kind-6 ×2 + tripbomb ×3, hints #423/#424, SS #149-equivalent, SP #427).
- interactables_07000013_live.rs: 6 → 10 tests, all green. NEW: the_security_door_walks_open_
  and_the_stairwell_door_refuses; the_tripbombs_arm_and_the_bond_bonus_takes_with_the_medal
  (BaseMap 0x07000011 [L ee_start_05 @0x2a45a8]); the_securityroom_glass_refuses_the_p2k_word
  (word 0x0008 [L] vs filter ~0x30); the_hint_plates_carry_the_disc_params (parse-model pin).
- CORRECTION (doc + prose, with addresses): Door_Update 0x1363c8's sphere scan only holds or
  closes an OPEN door — there is NO closed-door proximity open; the §7 "straddle param 4 = 1 →
  proximity open" row and the pickups test's prose misread it. The E opens #355 [M Door_Activate
  0x135be0]; the approach alone opens nothing; #274 refuses the E while ch 0xc8 is off.
- Totals: interactables_07000013_live 10/10; walkreal_07000013_live 1/1; nf-game 959 passed,
  0 failed, 9 ignored (98 binaries). Final fast suite running on the final tree.
