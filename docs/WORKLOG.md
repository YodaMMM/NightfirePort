
## retune-nb-14 (2026-11-19, s/retune-nb-14)

Task: commit 3412a89a3 made every guard run the game spacing scan and the close-range alt attack from the fire states; the level 14 walks `walk_the_infiltration_to_the_delta_exit_real_damage` and its `_00agent` sister now die. Read both ignores and the 14 walk changes in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.

### retune-nb-14 progress

- Branch cut from origin/master 73822d3a0. Both ignore messages read:
  - `_00agent` (d3): Bond dies t47542 at (-48.91,1.13,34.27), drone #33 at 9.6 u (leg 4 east, the Kiko corridor / the east lane above the pocket line).
  - `real_damage` (d1): Bond dies t18133 at (-26.09,0.55,-10.56), a 29.0 world blast (the leg-5 wake-legs nav route).
- The same commit's 14 walk changes: `shoot_the_drop_band_guard` extracted and now called on every walk (the close-range alt attack holds the drop band), `drop_aim_z` corpse band widened, the tripbomb route calls it too.
