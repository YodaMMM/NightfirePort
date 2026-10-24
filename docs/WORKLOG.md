# WORKLOG

## level-15-ladder-3 (branch s/level-15-ladder-3, started 2026-10-24 05:06 UTC)

Task: 07000015 ladder #382 — settle whether the game's pick (Collide_Pick 0x1eb320) ever admits the unlinked ladder. Master's interactables_07000015_live.rs ladder doc claims a crouch-walk-off-the-lip + strafe route reaches it; branch s/level-15-ladder-2 (unmerged) says the ladder links to no cel, is never on the pick's candidates, and can never be grabbed. Read Collide_Pick in full (disasm where unclear), decide, then either make master's route test pass with real input or replace it with the branch's pin + sister.

Work Log:
- 05:06 UTC: sandbox alive (repo, work/, ISO, ../nf-queue all present; suite is not started yet at this point — started right after branch push). Branch s/level-15-ladder-3 created from origin/master 5215090c. Baseline suite launched in background.
