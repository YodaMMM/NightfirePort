# WORKLOG

## burst-fire-held-trigger (2026-11-03)

Task: burst-mode weapons fire their full burst from one trigger pull. The ignored leg in
crates/nf-view/tests/fire_modes3_live.rs (~line 182) measures 1 shot per pull on burst mode,
not 3. Port the game's held-trigger path, Player_WeaponFiring 0x1a5460:601-655 [M]: rounds
per press, interval between them, and what releasing mid-burst does. Drive it through the
real input path (Sim::step_bond, trigger held then released). Check the other ignored legs
(~248 the 0x2D press after 0x2C camera, ~281 the 0x2B trigger edge, ~321 the 0x32 draw stall)
for the same cause; fix if shared, otherwise name each separate cause. Un-ignore what passes.
Run fire_modes3_live, fix_weapon_fire_modes_live, weapon_matrix_live, nf-view lib tests.

Branch cut from origin/master 6f16146e. Sandbox carried (repo, work/, ISO present at
/home/z/nf/Nightfire-onfoot.iso); no setup redo needed.
