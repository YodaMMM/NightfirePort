# WORKLOG

## retune-nb-16 (2026-11-18, s/retune-nb-16)

Task: commit 3412a89a3 made every guard run the game spacing scan and the close-range alt attack from the fire states; the level 16 walks `walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent` and `_00agent` now die. Read both ignores and the 16 walk changes in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000016_live binary.

### retune-nb-16 progress

retune-nb-16 progress (2026-11-18): the Agent walk (d2) is GREEN — un-ignored. The lever is one
gate change, file-local: the squad-phase close stands (the 00 Agent law, 120-tick bounded stands on
any live hostile inside 10 u between the member slices) extended to `difficulty >= 2 && !w.tops_up()`.
The measured death it fixes (t25168, reproduced byte-identical first): with the spacing scan live the
post-chaser pit-shooter clear picks NOTHING (no guard exposed from the fight's end spot), so the
pit-room AR guard #15 (0x50000b2, 0x14, hp 20) enters the squad phase alive, hunts Bond through the
10800-tick far-member stand (invisible #13), closes to 2.0 u (inside the 3.0 always-hit band
[M 0x179740:113-115]) and pours 48.7 in 10 hits; the dealer answer fired once (4 of 20) before Bond
died INSIDE it. Green run: squad audit clean (21 hits, every delta within the game's chain), ch
0xbf/0xc0/0x82, the button, MissionState 6. Sisters/punch/d1 untouched (gates verified).

The 00 Agent walk (d3) is still RED at the furthest measured point; the full ledger + the next lever
are in the test's ignore message. Landed d3 levers (all `difficulty >= 3 && !tops_up()`, file-local):
the east-high ring guards join the peek cycles' fight list; the close pair fights from the peek; the
exit stays fought. Measured-worse levers tried and REVERTED (each on its own real run): level_fights
at d3 (t3912 ramp death), the after-exit resupply (t3608), the ramp-foot perch stand (t4230, the gun
is at clip 1 spare 6), east-high x>8.5 + peek r 0.9 (t6008 fist convergence). The ammo chain (the
exit-fight drops 2-6 u off the path; the resupply trail corridor misses them) is the next lever.

Binaries at the final tree: walkreal_07000016_live 18 passed / 1 ignored (the d3 walk, ledgered);
the three smaller crates' full binary sets run in the background at push time (the nf-view binary
set ENOSPCs this sandbox — the report documents it).
