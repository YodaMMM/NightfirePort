## hud-ammo-under-label (2026-11-01, s/hud-ammo-under-label)

Branch from origin/master (6042ee53). Task: at a level start the PS2 shows the ammo bar while the
weapon-name label is up (07000011 f42 capture); the port hides it while the label timer runs, from
an older reading of HUD_UpdateAmmoPane 0x19a7f0. Re-read the pane's label/ammo branches
[M 0x19a7f0:200-260, the 60-tick label timer path] and port what the game draws during the label
(both? which elements?), pinning it with the HUD tests (sister: after the timer the label is gone
and the bar remains). Run hud_level_start_live, ammo_pane_live, fix_hud_audit_live and nf-game;
quote the totals.
