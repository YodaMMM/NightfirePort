
## retune-nb-02
- Sandbox reset; re-cloned, branch s/retune-nb-02 created from origin/master (6fe4339a1).
- Task: re-tune walk_the_castle_all_objectives_real_health_00agent for the 3412a89a3 guard changes.
- Reproduced the nb-scan-all death on 2026-11-18 (t10472, the plate): the spacing scan turned the
  geisha-1 give-ground into a siege (the captains hold step-aim camps and volley; the stand-off
  hold bled 87->39 and the walk-back died at the 0.8-u plate mouth). Lever 1 (the plate vantage +
  the lean-hold duel) is in: geisha-1 green; the walk now dies t27653 at the lane crossing at 7.8
  health (the sniper's -48.0). Next: cheapen the crossing pool (the post-loop pursuer sweep spent
  87->33; the armour detour 33->7.8). Sandbox disk is 9.9 GB: the full nf-view test build (1111
  binaries) does not fit; the suite could not run here — see the report.
