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
