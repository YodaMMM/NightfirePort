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

Progress 02:45 UTC: read Player_WeaponFiring 0x1a5460's whole flow [M]. The held path: the
fresh press (Input_Action mask 4, :601) arms pending = +0x28 and the timer at 1.0 (:617-624);
each shot re-arms max(+0x40,1) (:657-660); the held non-fresh ticks only return when the timer
has already run out (:613-615) — so a held trigger runs the whole pull. Releasing mid-burst
cancels it: the mask-1 release read zeroes +0x936 (:443-445) before any further fire. The port's
gun.rs already implements exactly this (update_with: the fresh arm, the run-out return, the
release clear at !pressed); the burst leg's harness (fire_once) released the trigger after one
tick, so the port measured 1 round — the harness was wrong, not the port. Rewrote the leg with a
fire_held helper (trigger held, then released): pins shots at ticks [0,4,8] (spp 3 [L +0x28],
interval 4 [L +0x40]) and the release-cancel sister (1 round, clip 29). 0x0B record bytes [L
ee_start_05.bin @ 0x2bf150+0xB·0x10C]: spp 3/3, interval 4, fire_anim_len 0, flags68 0, fire
script 0x0600076f. Diagnosed leg 281 (MGL 0x2B): the port's grenade family spawns a Grenade
object, not a bullet, and the grenade path never pulses shot_tick — the leg's `shots >= 1`
assert measures the wrong counter; the grenade-count and fuse-death asserts pin the fire.
Suite/env: sandbox disk went 100% full (rust-lld bus errors); cargo clean freed 6.8 GB, rebuild
underway. ALSA link needs RUSTFLAGS="-L native=/home/z/alsa/usr/lib/x86_64-linux-gnu"
(untracked /home/z/env.sh, never committed). Next: run the four ignored legs, diagnose 248/321.

Progress 03:46 UTC: all four ignored legs now pass (fire_modes3_live 9/9). The port fix: the
direct switch request's owned read is the BASE's slot [M Player_EquipWeapon 0x1a8a58:39-41, the
twins share BL+0x1BA+base·0xC; the port's own Inventory::equip_weapon writes the same base slot]
— the Samurai 0x32 give owns base 0x33 and the old gate refused the draw on owned[0x32]. Harness
fixes, each cause named: (248 launcher) the guided round flies 200 units at 0.25/tick [L +0x1C/
+0x20] ≈ 800+15 ticks and the substate-10 ride swallows every press [M Player_Update 0x193600
case 10] — the old 400-tick wait pressed into the ride; and the 4-rocket pull needs the held path
(spp 4 [L +0x28], interval 30 [L +0x40], shots [0,30,60,90]). (281 MGL) the grenade family spawns
a Grenade object with no shot_tick pulse (the port's bullet-path presentation) — the `shots >= 1`
assert measured nothing. (321 samurai) the base-owned draw above; plus the blast safety: the
spawn's own floor is 0.36 under the eye [L ee_start_05 rest 24.9633 vs eye 25.32], so any
down-tilt detonates the 256-damage bolt / 100-damage rockets at Bond's feet — level aim crosses
the courtyard. The burst leg's arm() helper now writes the SHARED base clip slot [M
Player_ReloadAmmoType 0x1a1988:18-22]. Kill switches: none; no health writes; all real input.

Final 04:03 UTC: the report is docs/reports/burst-fire-held-trigger.md. The full-suite re-run
did not fit the window (the OOM-killed nf_view lib-test link forced a full nf-view relink at
-j4); the next window runs `NF_ISO=... nohup python3 tools/test/suite.py > suite.out 2>&1 &`
on this branch — the build cache makes the rerun the only cost. Scratch probes deleted;
env is the untracked /home/z/env.sh (PATH, PKG_CONFIG_PATH, RUSTFLAGS -L native for the user
ALSA prefix, NF_ISO).
