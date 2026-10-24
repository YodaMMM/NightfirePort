# WORKLOG

Entries appended at the end, one per task. Do not rewrite earlier entries.

---
Task ID: level-4a-lift-walk-5
Agent: sandbox2
Task: 0700004a: re-run the ignored walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby on master. If it passes, un-ignore it and drop its fall exemption if no longer needed (no vacuous pass). If it fails, find where and why with real input and fix [M]; the satchels now drop on this ride (shooters at ch 0x22 / ch 0x4f), so the walk must burn them with the laser watch or survive them the way the satchel test does.

Work Log:
- (started) Branch s/level-4a-lift-walk-5 created from origin/master at 26543436 (batch-1023j); setup inherited (work/, ISO, nf-queue worktree present).

Stage Summary:
- Branch created; re-run of the ignored walk starting.

---
Task ID: level-4a-lift-walk-5 (final)
Agent: sandbox2
Task: 0700004a: re-run the ignored spawn-to-lobby lift walk on master; un-ignore + drop the fall exemption if it passes; otherwise fix with real input and burn the satchels.

Work Log:
- Re-ran the ignored walk on master 26543436: FAILED at t2167 (y −54.34 want −71.01, grounded) — the want-line does not model the landing scene's 575-tick hold [D 0x060006cf s25 marker at t575; D script event 3; L lift-4 audit]. The hand-back itself is correct (raw marker key, grounded at the re-enable) — the TEST was wrong.
- Fixed the test: the want-line holds while disabled (hold_ticks shifts the keyed descent's phase); the laser watch is drawn and both satchels burned with held fire [M Bullet_DoTrails 0x12e2f0-0x12e390] (run: burned 2); the fall exemption REMOVED (a void fall trips the scrape check on its first enabled tick — gap 53.5 ≫ 5); new pins: bottom plate ch 0x84 [D #62] and the mission's LevelEnd [M 0x182c60:73].
- Un-ignored and green (~5 s). Suite on the final tree, foreground chunks (background killed between tool calls): 1320 passed, 4 failed (the slim-ISO disc-golden set), 62 ignored, no TIMEOUT/CRASH.

Stage Summary:
- Task complete: walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby un-ignored, green, non-vacuous. Report: docs/reports/level-4a-lift-walk-5.md. No src changes; one test file + docs.
