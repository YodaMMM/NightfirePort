# WORKLOG

(Started fresh from the rewritten master a67f1a27e; older entries live on the pre-rewrite branches.)

## lobby-roof-glass-09 — start

- Branch `s/lobby-roof-glass-09` created from origin/master a67f1a27e.
- Task: 07000009 lobby roof glass (round 2). `crates/nf-view/tests/interact_07000009_live.rs::shooting_the_lobby_roof_glass_breaks_it` — on master the test is NOT ignored but the driver says its fire harness cannot land a gun round on pane #123 since the bullet filter `filter | 0x106` [M 0x12bfd8:15 + 0x1ece30:24-33] landed. The full ignore analysis from the previous round lives on `s/995-gondola-windows-08` (1ade0bc73): the muzzle side offset puts every bullet outside the pane's 0.2-wide x-slab.
- Plan: reproduce on master, trace the game's shot route from a standable spot (pane filter param, weapon word, muzzle origin), port/fix, un-ignore-equivalent (make it pass), sister stays.

## lobby-roof-glass-09 — the filter, the health cvt and the legal-stance re-pin

- Read the previous round's analysis (`s/995-gondola-windows-08` 1ade0bc73's ignore message) and re-derived it from the decompile:
  - `Collide_FilterBullets` 0x1ece30 [M decomp + disasm 0x1ece94-0x1ecea8]: a class-5 contact whose weapon-definition word (`lhu 0x10($a0)`, +0x10) ANDs the mask non-zero is marked t = 1e8 (0x4cbebc20) and the rebuild (disasm 0x1ecee4-0x1ecef8) hands every t == 1e8 node to `LList_Add` — OUT of the list. Only the zero-AND contacts reach `Break_Update`'s damage walk. `Break_Update` 0x12bfd8:15 passes mask = tag+0x08 | 0x106 [M disasm 0x12c028-0x12c030].
  - `Break_Create` 0x12c230 [M disasm 0x12c2a8-0x12c2e0]: health = (f32)(i32) param 1 (the cvt.s.w), not f32::from_bits; datum+0x08 = param2 == 0 ? 0 : ~param2 [M disasm 0x12c2e4-0x12c2f8].
  - Weapon words [L ee_start_05.bin weapon_data @0x2bf150 + w·0x10c]: guns 0x0008 (P2K 0x2, K5 0x43), laser 0x0002, watch/Decryptor 0x0100 → the 0x106 base mask passes guns, rejects laser/watch.
- Ported: `Breaks::bullet_hit` takes the weapon WORD and skips non-zero-AND hits; the drone.rs caller reads `weapons.records[w][0x10..0x12]`; `create` decodes health as (f32)(i32). Unit tests re-pinned to GUN words + two new filter tests. Master's merge (batch-1022ae) had landed the same filter/cvt via the gondola branch — the merge kept the fuller citations and fixed master's 0x1e→0x0008 mangling of the evidence addresses (0x0008ce30→0x1ece30, 0x0008d548→0x1ed548).
- Reproduced the stray kill: a shot from the x-8.02 line kills BOTH #123 and the perpendicular pane #116 — the game's own muzzle offset (the bullet starts at the midpoint of the eye and the view datum 0 [M 0x1a4bf0:226-245], ~0.15 right = west) clips #116's grown box (z 35.96, x 6.59..7.86). The final harness stands 0.5 east ([8.5, −369.0, 34.5]), where the offset cannot reach #116, and one P2K round kills #123 alone (health 6.0 → −1.0, kill (123, 2, 0x2E)).
- The glass test is UN-IGNORED (it passes); new sister `the_security_room_glass_refuses_the_same_gun`: securityroom_glass #87 (param 2 = 0x30 → mask 0xffef) keeps health 1.000 under 57 P2K trigger pulls — the filter pinned through the real fire path on this level.
- Suites on the merged tree: nf-game lib 544/0; interact_07000009_live 14/0 (+1 pre-existing ignore); break_debris_live 2/0; glass_breach_live 1/0; interact_07000001_live 10/0; breakables_live 2/0.
