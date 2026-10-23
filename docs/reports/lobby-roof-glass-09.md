# lobby-roof-glass-09 — 07000009: shooting the lobby roof glass (round 2)

## What the blocker actually was

The previous round's `#[ignore = …]` (on `s/995-gondola-windows-08`, 1ade0bc73) blamed the muzzle
offset; this round re-derived the whole shot route from the decompile and found the port missing
the game's own bullet filter — the gondola branch had since landed it on master (batch-1022ae),
so the final tree carries the same rule with sharper citations, the health decode, a re-pinned
harness shot from a spot the game actually allows, and a new level-real filter sister.

Evidence chain ([M] = decompile + disassembly, checked in `work/elf/ACTION.ELF`):

- `Break_Update` 0x12bfd8:15 filters the pane's contact list with
  `Collide_FilterBullets(list, tag+0x08 | 0x106)` (disasm 0x12c028-0x12c030: `lhu 8($s1)` →
  `ori 0x106`).
- `Collide_FilterBullets` 0x1ece30 (disasm 0x1ece94-0x1ecea8) marks every class-5 contact whose
  weapon-definition word (`lhu 0x10($a0)`) ANDs the mask non-zero with t = 1e8 (0x4cbebc20), and
  the rebuild (disasm 0x1ecee4-0x1ecef8: `c.2.s f0,f20` / `bc1f` keep / `jal LList_Add` drop)
  hands every t == 1e8 node to `LList_Add` — OUT of the contact list. Only the zero-AND contacts
  reach `Break_Update`'s damage walk (0x12bfd8:44-48).
- `Break_Create` 0x12c230 (disasm 0x12c2a8-0x12c2e0) decodes the health as (f32)(i32) param 1
  (`cvt.s.w`) — the old `f32::from_bits` read the bits raw (authored 6 became 8.4e-44, not 6.0);
  datum+0x08 = param 2 == 0 ? 0 : ~param2 (disasm 0x12c2e4-0x12c2f8).
- Weapon words [L ee_start_05.bin weapon_data @0x2bf150 + w·0x10c]: the guns carry 0x0008
  (the P2K 0x2, the K5 0x43), the Laser 0x0002, the laser watch 0x0100. So on a no-filter pane
  the base mask 0x106 passes gun rounds and pulls laser/watch contacts — "lasers never break
  glass, guns do".

## The pane, the stance and the shot

Placement #123 (`pillarglass1`, census row "m39 … @(8.02,−361.69,36.74)", params [0:2, 1:6], no
param 2 [D]) is a 0.00006-thick slab at x 8.0229 spanning z 36.10..37.38 and y (up)
−369.25..−354.12 [D the port's Breaks row, probed]. The pane next to it (#116) is perpendicular
(z 35.962, x 6.59..7.86) — a corner junction.

The harness settles Bond on the lobby floor 0.5 east of the slab ([8.5, −369.0, 34.5]), cycles to
the P2K (word 0x0008 & 0x106 == 0, asserted), aims up the pillar and fires real trigger ticks.
One P2K round kills #123 (health 6.0 → −1.0, kill effect (123, kind 2, sound 0x2E) [M Break_Kill
0x12b970]). A shot from the x-8.02 line instead killed BOTH #123 and #116: the game's own muzzle
offset (the bullet starts at the midpoint of the eye and the view model's datum 0 [M
Player_WeaponInitBullet 0x1a4bf0:226-245], ~0.15 right = west when facing +z) clips #116's grown
box — the previous round's "muzzle's side offset" observation, now with the exact mechanism. The
0.5-east stance puts the offset out of #116's reach, so the test pins the single-pane kill.

## What changed

- `crates/nf-game/src/breakable.rs`: `bullet_hit` takes the weapon WORD and skips non-zero-AND
  hits (the filter, with the full disasm citation chain); `create` decodes health as (f32)(i32);
  the unit tests ride a `GUN` word const, plus two new unit tests (laser/watch never reach a
  no-filter pane; the 0x30-filter pane rejects gunfire but takes a zero word).
- `crates/nf-game/src/drone.rs`: the breaks pass reads `weapons.records[w][0x10..0x12]` (the same
  read the fusebox pass already makes).
- `crates/nf-game/tests/break_debris_live.rs`: the gun word + the mangled evidence addresses
  fixed (0x0008ce30 → 0x1ece30 `Collide_FilterBullets`, 0x0008d548 → 0x1ed548
  `Debris_Create_Loop` — a bad 0x1e→0x0008 replace on master had corrupted them).
- `crates/nf-view/tests/interact_07000009_live.rs`: the glass test UN-IGNORED with the
  legal-stance harness; the aim-alone sister re-pinned to the same stance; NEW
  `the_security_room_glass_refuses_the_same_gun` — securityroom_glass #87 (param 2 = 0x30 →
  mask 0xffef [M 0x12c2e4-0x12c2f8 + 0x12c028-0x12c030]) keeps health 1.000 under 57 P2K trigger
  pulls: the filter pinned through the real fire path on this level.

## Verification

- nf-game lib: 544 passed / 0 failed (incl. the 12 breakable unit tests).
- interact_07000009_live: 14 passed / 0 failed / 1 ignored (the lift-door E-route test — its own
  pick-origin session, untouched here).
- break_debris_live 2/0; glass_breach_live 1/0; interact_07000001_live 10/0; breakables_live 2/0.
- Baseline full suite on the start tree (a67f1a27e, slim ISO): 726 passed / 4 failed / 3 ignored
  — the 3 `disc_golden` failures are the documented slim-ISO set; `identity_disc` also needs the
  full ISO (518,435 sectors vs the documented 1,191,072 — the slim on-foot image).

## Live checks for the main agent (optional confirmation, PCSX2 fresh boot of 07000009)

1. Stand on the lobby floor beside the pillar glass at approximately (8.5, lobby floor, z 34.5)
   facing the pane, fire the P2K — expect the pane to shatter (one round, glass sound 0x2E).
2. gdbwatch the pane obj's tag+0x04 (health) through the shot: expect 6.0 → −1.0 on the hit tick
   (the cvt.s.w decode), not ~0.
3. gdbwatch `Collide_FilterBullets` 0x1ece94's `lhu 0x10($a0)` word during a P2K shot — expect
   0x0008.

## Final suite

(to be filled from suite.out at cutoff — see the pushed suite.out snapshot note in the report
footer if empty)

## Usage

not visible to me
