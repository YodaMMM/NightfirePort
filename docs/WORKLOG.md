# WORKLOG

## level-02-skip-handback (2026-10-28, branch s/level-02-skip-handback)

Task: where the game puts Bond when a cut-scene is SKIPPED. Master hands Bond
back from an animated (kind-2) cut-scene at the anim object's matrix
[M Script_EventHandler 0x1d94c0:73-84,:125-134], matching the game when the
scene PLAYS (07000002 intro: (-27.504, 1.131, 32.348) [L]). When the player
SKIPS, the port fast-forwards and lands at the anim's END pose (-27.30, 1.23,
28.90) — unverified. Read the game's skip path [M Script_FFwd 0x1dac78,
SP_Update 0x1dbda8's skip arm, how event 6 and the anim's interp run (or not)
during the fast-forward] and make the port's skip hand-back exactly the
game's, for every SP with a kind-1/kind-2 hand-back. Test both paths on
07000002/03/14 through the real skip input. Full suite; report; finish steps.

Plan:
- Fast runner started in the background in the first minutes.
- [M] read Script_EventHandler 0x1d94c0, Script_FFwd 0x1dac78, SP_Update
  0x1dbda8's skip arm; event 6 and the anim's interp during fast-forward.
- Find the port's skip hand-back; compare against the game's.
- Tests through the real skip input on 07000002/03/14, both paths (play + skip).
- Full suite on the final tree; push; report; finish steps.
