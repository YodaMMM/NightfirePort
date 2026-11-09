# 07000007: party guests with voice 0x10 turn hostile near an armed Bond

CivilianPatrol 0x15d610:53-55 sets the team byte to 1 for voice 0x10, so 07000007's two voice-0x10 guests turn hostile when an armed Bond is near; the port now does this but no test reaches it through play (the guest never sees Bond on the current walk). Find where these two guests patrol, walk an armed Bond into their sight with real input, and pin what the game does (alert, attack, team), fixing the port if it differs.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-07-guest-hostile`, report in docs/reports/level-07-guest-hostile.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-09T06:17Z
