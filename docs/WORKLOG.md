# WORKLOG

(Started fresh from the rewritten master a67f1a27e; older entries live on the pre-rewrite branches.)

## breakable-damage-unit — start

- Branch `s/breakable-damage-unit` created from origin/master a67f1a27e.
- Task: how much damage a player bullet does to a 0x20 breakable. `Break_Create` stores param 1 as int->float [M 0x12c230:22-27] so 0700000a vending glass #379 has health 9. `Break_Update` subtracts each class-5 contact's HITDATA+8 [M 0x12bfd8:37]; the port passes the bullet's raw weapon `damage` (+0x0C = 0.1 for gun 0x43), so ~90 contacts. Find what the collide pass writes to HITDATA+8 for a bullet contact on scenery, and the one-contact-per-bullet-per-object rule. Un-ignore `interactables_0700000a_live::vending_glass_breaks_under_fire` with the proven count as pin; sister: far glass #378 stays alive.
- Plan: read the decompile for the HITDATA fill on class-5 hitters; check the lobby-roof-glass-09 branch for prior art; fix the port; prove count; un-ignore; suite; push.

## breakable-damage-unit — research settled (round findings)

- The collide pass's HITDATA+8 for a bullet contact = the tested object's HITTEST+0x90 [M 0x1ebaa8:320-322 writes it into the pick node; 0x1ea398:191 disasm 0x1ea690-98 copies the pick node to the victim's obj+0xD0 list with node+8 = the TESTED object's HITTEST+0x90 and node+0x58 = the tested object]. The bullet IS the tested object: Bullet_update re-arms its embedded HITTEST (obj+0xD4 = BUdata+0x50) every flying tick — kind 0x101 point / 0x201 ray, and +0x90 = weapon_data+0x0C, overridden to 1.0 for base ids 0x2A/0x2C-0x2F and 0.0 for 0x2B/0x34-0x37/0x3A-0x40 when wdef+0x08 > 0 [M disasm 0x12f030-0x12f0b8]. The port's `bullet_contact_damage` (nf-formats weapon.rs) already implements this exact rule (the searchlight feed uses it).
- The count rule: Collide_PickObj finds dynamic objects (pick node +0x58 = victim, +0x54 = 0 [M 0x1eae80:124-125]); the static paths set +0x54 (victim) and +0x58 = 0 — only dynamic victims receive copies. The copy walk is sorted nearest-first and STOPS at the first non-pass node [M 0x1ea398:216-229]; Bullet_CollisionHandler kills the round on that same hit (state 2 [M 0x12ce80 disasm 0x12cff0-4]). One contact per bullet per object.
- Breakables are class 0x20, bullets class 5 [M Bullet_init 0x12c5a0 disasm 0x12c624-0x12c62c]; Break_Update's class-5 arm subtracts node+8 [M 0x12bfd8:36-37]. Health = (float)(int)param 1 [M 0x12c2b4-0x12c2bc cvt.s.w + the negative reconstruct 0x12c2c8-0x12c2dc] — the PORT parsed param 1's BITS as f32 (breakable.rs `f32::from_bits`): #379 got 1.26e-44 instead of 9.0. That is the pass-at-any-damage bug.
- The count for 0700000a's vending glass #379 (health 9.0 [D param 1 = 9], K5 dart 0x43 contact 0.1f [L ee_snapshot weapon_data 0x2bf150 + 0x43·0x10C + 0x0C], pellets u32 +0x18 = 1, one contact per dart): the f32 accumulation kills on contact 91 (after 90: +2.0414591e-6). Decompile-settled; no live read required.
- Port fixes queued: (1) the health parse, (2) the merged nearest-solid contact for breaks+destroys gated by the first non-pass world hit (`segment_bullet`), the round dies on it, the damage = bullet_contact_damage.

## breakable-damage-unit — fixes landed (merged origin/master 326db1530)

- breakable.rs health: (param(1) as i32) as f32 — origin/master's batch-1022ae landed the same cvt.s.w fix independently (the gondola glass); the merge kept master's wording.
- drone.rs update_player_bullets: the breaks/destroys contact is now the game's ONE sorted collide list rule — the NEAREST solid dynamic victim takes the contact and the round dies on it [M Collide_Update 0x1ea398:216-229 + Bullet_CollisionHandler 0x12cff0-4]; gated by the first non-pass world node (`segment_bullet`) and the drone candidate; damage = bullet_contact_damage (the per-tick HITTEST+0x90 [M 0x12f05c-60]); the FilterBullets reject word (master's weapon_word, [M 0x1ece30]) applied per candidate; the script-pane kill deletion (master's) kept. The 0x54 camera-subject detector split into Props::camera_shot_hit (a marker, not a contact) and runs beside it; the monitors block unchanged (its node+8 question noted in the report).
- The vending live test: UN-IGNORED (master's batch-1022ae had re-ignored it pending this count) with the pins: load health 9.0; one K5 dart = exactly one 0.1f contact (9.0→8.9f); the kill count through the real weapon wheel to the P2K (0x02, 3.5f contact): 8.9f − 3.5f × 3 → the f32-mirrored count 3, the kill value −1.6f; sister #378 at 9.0. The K5 alone cannot kill (5+10 darts [D] vs 91 needed — the 91 figure is the f32 arithmetic of 9.0 − 0.1f, noted in the report; the exact f32 sequence is asserted via the in-test mirror of [M 0x12bfd8:37,49]).
- Suite started on the merged tree.
