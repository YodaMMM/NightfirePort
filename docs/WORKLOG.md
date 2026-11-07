## mp-enviro-mods-apply

Branch `s/mp-enviro-mods-apply` from `origin/master` (10956badb). Task: the Enviro-Mods page
(`P_MPENVIROMODS` 0x210030) writes `mp_envmods` (crates/nf-view/src/menu.rs) and the rows' save
and summary use it, but the match settings carry no such word (comment at nf-view/src/lib.rs:1748)
so `gunimps` is cleared for every match; `mp_enviro_value_label` has no caller. Find what each row
does in the game and whether it reaches a match on master: +0x1C0 view distance, +0x1B8
(`GunImp_Create` 0x131f20:52-58 mounted guns), +0x1D4 (locked row), +0x1D0 (grapple, see
mp/equip.rs), +0x1CC vehicles. Read every reader of those five words in the decompile; add the
words to `MpSettings`, carry them from the menu into the match on host and clients, apply each
reader. Mounted guns must stay in step on every machine — prove that before turning them on.
Vehicles: report what the row gates only, leave car creation alone. Use `mp_enviro_value_label`
for the page's rows if the page's text does not already match. Pin through real matches, row by
row; run every mp_*_live binary, mp_world_sync_live, the menu binaries that walk Enviro-Mods and
nf-game. Entry started.

Work (same entry): the reader census landed — every reader of the five words in the decompile:
+0x1C0 (`_448_4_`) is the SPAWN-SELECTION POLICY, not a view distance — the only gameplay reader is
`MP_GetSpawnPoint` 0x185098:165-183 (0 Near / 1 Far / 2 Random over the team's spawn range;
`BOT_fellOutMap` 0x1259d0:12-19 forces 1 around one respawn); "View Distance" is only the menu-def
row title 0x1af [L mp-menus.md]. The port's `spawn_policy` mapping is correct and its
`MpMatch::get_spawn_point` implements the game's policy (pinned by mp_core_live). +0x1B8
(`_440_4_`) Gun Emplacements — sole gameplay reader `GunImp_Create` 0x131f20:56-58 (no mount when
0), turret = weapon set 0..9, set 10 = `Rand_Rand(10)` [M :59-63]. +0x1D4 (`_468_4_`) Explosive
Scenery — sole reader `Destroy_Create` 0x12c318:15; masked &1 at confirm [M P_MPCONFIRM
0x20ec00:148]; locked until the type-3 reward 0x3e [M Menu_UnlockMPSettings 0x1ff698:103].
+0x1D0 (`_464_4_`) grapple — sole reader `MP_EquipPlayer` 0x1855f8:27-34 (the port's equip.rs
already applies it). +0x1CC (`_460_4_`) vehicles — sole reader `Car_Create` 0x1305b0:13,29-43
(report-only; the port gates 0xDA placements off it with the documented private-stream heli roll,
no car creation). Boot defaults [M bootup 0x1c7e78:28,38,42,43,44]: +0x1C0=2, others 0.
Code: the mounts now reach the match — nf-game::gunimp::apply_mp_turrets (the gate + turret
assignment; Random draws a seed-keyed private stream, not the shared Rand stream [S]), applied at
build (scene.rs) and at match start (lib.rs — the unconditional clear replaced), and the mounts
ride the world hash fold (turret/aim/heat; the machine-local audio handle and the draw quad stay
out). The Enviro-Mods page's rows draw mp_enviro_value_label (the locked row binds by the unlock
flag [M 0x210030:35-44] — a fresh locked profile showed "Off" where the game shows Locked).
Carry: the words were already in MpSettings and on the wire (nf-net mods[4], mpnet.rs) from
mp-lobby-settings; unchanged. LEFT: the row-by-row real-match pins (mounts on/off, destroy on/off,
grapple equip, the two-machine in-step proof), the suite rerun, the menu binaries' rerun.
