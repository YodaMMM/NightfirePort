# WORKLOG

Branch cut from origin/master 0ebfee9a (2026-11-02 06:15 UTC). WORKLOG started fresh (master tracks no docs/WORKLOG.md; recreated per prior-task convention).

## Task
bot-push-per-bot-gates: Bond is pushed off guards every tick like the game, so he can't be shoved through the 03 gatehouse wall. `crates/nf-view/tests/flank_fall_repro.rs` `the_old_courtyard_flank_stays_on_the_floor` is #[ignore]d: on 07000003 Bond rides the gatehouse wall's south end-edge, goes airborne, and on that first airborne tick the port's bot-capsule push discharges the whole overlap built up while grounded (two guards, ~0.74 u in one tick), carrying him across the wall plane x 10.915 into a box with no floor; he falls out of the world. The game pushes off bots EVERY tick [M Collide_Jointy 0x1ec180:229-281, no ground gate], gated per bot by `NDrone2_DoCollision` 0x149ec8 and the drone's +0x4f8 0x20/0x200 bits. Port the per-bot +0x4f8 gates, make the push run every tick as the game's does, un-ignore the test. Run flank_fall_repro, walkreal_07000003_live, walkreal_07000001_live, exit-aim 01 and nf-game.

## bot-push-per-bot-gates (2026-11-02, sessions 1-2)

The Jointy push runs every tick [M Collide_Jointy 0x1ec180:229-281 — no ground gate] and the
bot_capsules fill ports NDrone2_Collision 0x140e90's per-bot arming (alive: DoCollision 0x149ec8 —
bit 0x20 from DefaultInit 0x14b300's 0xe2 fold, kind 0x13 out, kind 0xc in, else +0x538 /
near-2.0 / obj+0xFC / +0x570&2; dead: +0x4f8 & 0x200 arms the bone capsule 0x1c3120 bones 5/0x33
r 0.4 unconditionally). flank_fall_repro un-ignored, passing; the lip press's ungrounded==0 pin
relaxed to the hang signature (the decompile-backed push lifts Bond brushing a guard). The walker
stuck-ladder gained an orbit-around-a-lane-bot fallback. flank 2/2, walkreal_07000003 4/4,
exit_aim_01 1/1, walkreal_07000001 2/6 (the escort routes need re-tuning for the faithful push),
nf-game lib 654/654. The full suite's build could not finish on the 10 GB sandbox disk (three
starts, each resumed after pruning; killed at the deadline). Report:
docs/reports/bot-push-per-bot-gates.md.

## bot-push-01-escort-routes

- Branch `s/bot-push-01-escort-routes` cut from origin/master b62a8907; merged origin/s/bot-push-per-bot-gates 5392f79d. Next: re-tune the four failing 07000001 walks (following_mayhew_down, the_exit_fade_starts, the_plate_refuses_bond_alone, walk_from_the_spawn_to_the_exit_with_the_contact) for the faithful push, real input only.
- Merged origin/s/bot-push-per-bot-gates 5392f79d. The 01 walks re-tuned for the faithful push: the follow steps out of the PlayerInWay band (and kites melee huggers), the plate sister shoots lane blockers with lenient self-healing legs + proactive resupply + a reach-the-plate assert, the hall fights chase the hunter's live position, the final plate approach walks around the parked contact. walkreal_07000001 5/6 (real_damage dies at 2.2 hp — the west-corridor attrition + unseen snipers, NF_RF_TRACE-pinned), exit_aim_01 1/1, flank_fall 2/2, walkreal_07000003 3/4 (the captor-fight ammo regression pre-exists on the merged master tree). Report: docs/reports/bot-push-01-escort-routes.md.
