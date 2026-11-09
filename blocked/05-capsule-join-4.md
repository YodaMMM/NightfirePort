# Solid guard bodies and corpses: one join on current master

Merge s/capsule-join-3 (guard standing body within 2.0 of the player: 0x140e90:91-121 radius 0.4 / 0.55 kind 0xc, 0x149ec8:16-31 with 0x14b190; bump contact through Collide_Jointy 0x1ec180) and s/dead-body-collision (a corpse always has a body, radius min(hip, 0.4) [0x140e90:124,142]; the player collides with it [0x194df0:88]) onto origin/master as one branch (both edit scene.rs's bot_capsules block; combine them). The triage (audits notes) found on the merged tree:
- guest_faces_live (no talking line attached), speech_facial_fallback_live (crosshair never reaches the talker): check whether party guests carry the standing body in the game; if they do, re-stage the tests' Bond spot clear of the crowd.
- walkreal_07000001 (Mayhew plate never fires; real damage: the contact dies in crossfire; Agent: Bond dies to drone #17), walkreal_07000005 00 Agent (E at the vent leaves Bond in substate 0 - check the vent's use with a guard body near), walkreal_07000011 Agent (6000-tick cap at the east connector).
Decide port bug vs lost margin for each; fix or re-route as a player would. Then run every walkreal_* binary one per call. Never lower damage or health, no loosened pins, no new ignores.

Branch from origin/master, push to `s/capsule-join-4`, report in docs/reports/capsule-join-4.md.

---
claimed-by: sandbox1 at 2026-11-09T16:16Z

---
## Report (blocked, 2026-11-09T16:27Z)

# capsule-join-4 — report

Branch `s/capsule-join-4` from origin/s/capsule-join-3 (6e4bcbd2a). **Status: blocked — the
session's window closed with the red list unreduced; the 05 red is now root-caused with three
measured repair shapes, and the evidence is on the branch.** The next window continues from
`s/capsule-join-4` (97d15f6fb).

## What's done

1. **The 05 vent red (`the_door_band_exit_chains_to_07000006_walked_real_damage_00agent`) is
   root-caused — it is NOT a body blocker.** capsule-join-3's report guessed a guard standing in
   the vent mouth; this session's sweep found **no live guard within 6 u** of either ledge
   blocking point [L the run's dump]. The measured chain:
   - The d2 Agent sister descends clean and enters the crawl: after the E press (25.51, 31.94,
     71.41) **sub 0x7 ctx 0x6** [L the run].
   - The d3 run carries four d3-only tower-top guards (#0/#1/#14/#15, y 37.0, z 45-50, hp 10.0)
     whose fire covers the descent; their return-fire stands come back **killed false** twice
     (top's south edge, ledge north) and the ledge-north leg ends 1.3 u east of the d2 line
     ((27.05, 34.82, 73.60) vs (25.96, 34.40, 74.76)) [L both runs].
   - From that offset the wall-hug walk_to stalls east of the shelf drop line: after-E
     (26.28, 35.94, 71.43) **sub 0x0 ctx 0xff** — the crawl never engages [L the run].
   - The fights themselves are survivable (the no-sweep run reaches the E press at 89.0
     health); what kills the leg is the POSITION.
2. **Three repair shapes tried and measured (all on the branch, all fail-safe)**:
   - (a) clear-the-top sweep (`shoot_point`, chest aim): kill #0 true, then a second guard
     charged the stand to d 2.3 and killed Bond (83.5 in 16 hits); the nearest-first re-scan
     died in the first stand (19 hits for 100.0). The hp-10 patrollers swarm a standing shooter.
   - (b) custom re-approach targets: z 71.3 stalls dxz-arrived 4.7 u above the shelf; y 32.5
     steers Bond into the **x 26.25 z-74 floor hole** (86.0 fall damage, one death); a two-phase
     north-then-west crossing rides the upper contour (y 35.9) into the hole's east lip —
     Bond oscillates at x 26.1-26.5 and never crosses to the shelf band (x ≤ 25.75).
   - (c) re-walking the original descent chain from the south ledge: Bond fell again
     (other 89.0, death).
   - The working theory the next window starts from: the crossing must happen at the LOWER
     contour (y ~34.5) at z ≥ 74.5 — exactly the d2 line — and the fights must be answered
     DURING the walk (return fire costs 11 hp and resolves; the killed-false stands happen when
     a fight opens mid-leg and the nav resumes from a fight-changed spot). The candidate shape:
     walk the descent legs, and after EVERY return-fire fight re-walk the current leg from its
     start (a fight-aware nav re-walk), or split "fight to silence, then descend".
3. **The touched binary is re-run whole**: `walkreal_07000005_live` **15 passed / 1 failed /
   1 ignored** — identical to the branch point; no other test moved (the session's edits touch
   only that test file).
4. Checkpoints pushed: 91ee62537 (branch + WORKLOG), 97d15f6fb (the 05 root-cause + edits).

## What's left (the next window)

1. The 05 repair per the working theory above (fight-aware re-walk or fight-then-descend), then
   the whole binary again.
2. walkreal_07000001 ×3 (the escort contact dies to crossfire — the contact-protection re-route)
   — untouched this window.
3. walkreal_07000011 Agent margin — untouched.
4. The guest tests' staging surgery (the verdict is settled from capsule-join-3) — untouched.
5. The full walkreal sweep one binary per call — only 05 ran this window.

## Live checks for the main agent

None new this window (the two from capsule-join-3's report stand).

## Usage

not visible to me — this session's environment shows no token, credit or context-window figures
(no usage display was available anywhere in the interface I worked from).
