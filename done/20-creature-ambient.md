# creature-ambient — the game's Creature objects

Creature_Update, Creature_Move, Creature_Hurt and Creature_Delete are never mentioned in crates/*/src. Find from the decompile and the disc which on-foot levels place creatures [D], what they do and how they die, and port them with addresses (movement, hurt, delete; their draw stays a stub tracked for the merge side if a mesh path is missing). If no on-foot level places one, prove that from the placements and cite it in the code. Done = a real-input test on a placing level, or the citation; every walkreal_*_live green; the full suite total line.

---
claimed-by: sandbox1 at 2026-11-04T04:33Z

---
## Report (done, 2026-11-04T10:05Z)

Branch `s/creature-ambient` (pushed through ca716381): the full report lives at
`docs/reports/creature-ambient.md` on the branch. In short: placement kind 0xE0 ships on
no level [D — the census test, green on the slim ISO], so the citation path applies; the
8-function Creature_* port (create 0x134ab8, calcpoint 0x134cf0, calcwaypoints 0x134e58,
init 0x135208, delete 0x135230, update 0x135268, hurt 0x1352d0, move 0x135340) is in
`crates/nf-game/src/creature.rs` with the kind-0xE0 attach + the class-0x23 control
dispatch wired in drone.rs; an independent decompile audit confirmed the load-bearing
claims and 6 divergences were fixed; 7/7 creature unit tests; full suite 1060 binaries —
4102 passed / 3 failed (the allowed slim-ISO disc_golden) / 326 ignored; all 81
walkreal_*_live + npc_doors_live + fix_npc_doors_live + fix_guard_misc_live green
post-fix (233/0/13). Draw stays a stub (no retail creature to render). Creatures cannot
die: nothing in ACTION.ELF writes state 3 to CREATURE+0xDE and class 0x23 has no collide
handler.
