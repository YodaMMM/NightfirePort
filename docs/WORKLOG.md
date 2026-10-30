# WORKLOG

### level-09-live-guards-walk (start, 04:47 UTC, 2026-10-30)

Task: level 09 (07000009) — `walkreal_07000009_live::walk_tower_a_start_to_finish` sets
`drones.hold_npcs = true` (line 76) so the guards never act. The level-09-every-objective
report left: a challenge-state guard took zero damage from the player's rounds. Root-cause
that ([M] the damage path; [D] the guard's records), fix the port side that is wrong, then
make the walk pass WITHOUT hold_npcs on real input (clear each patrol beat, resupply when
dry). Every test in the file green; run the touched binaries and quote their totals; full
suite via the fast runner, quote the total line. Branch from origin/master 814c8976.

## level-09-live-guards-walk progress (05:45 UTC)

Root cause of the challenge-state zero-damage open item, three layers:
1. The 15 rounds were the K5 SLEEP DARTS — 09's stealth loadout opens on weapon 0x43 (clip 5 +
   spare 10 = 15 [L scratch run: bond weapon 0x43 clip 5 spare 10 at load]); darts route msg 0x19
   (NDrone2_DealWithObjHit 0x13fff0:269-306) which never reaches HitDamage — zero damage is the
   game's own design, not a pick gap.
2. REAL port gap (fixed): `Drone::on_dart_impact`'s whitelist lacked CIVILIAN_CHALLENGE — the
   decompile's group {7,9,0x17,0x18,0x19} in NDrone2_DSTATE_CivilianChallenge 0x162558:40-46
   forwards to DroneFunc_HandleImpact(…,0x89,1) → case 0x19 → STUN_DART_IMPACT 0x50. Before the
   fix the dart was unhandled → GLOBAL → dropped: no sleep, no flinch (the observed symptom).
   After the fix the challenge-state guard falls asleep to a dart, health untouched [L scratch].
3. The "mid-stair geometry shadow" hypothesis is dead: with the P2K (0x2) the challenge-state
   guard dies through the real pick at 2.5u mid-stair [L scratch]. The one-round kill was the
   dart sleep + the stun-family msg-8 arm (a sleeping guard dies to any bullet [M 0x163430/570/
   700 case 8/9 → 0x44 Death_Anim, health irrelevant]).

Walk work continues: cycle to the P2K, clear each live guard (ever_seen/challenge/attack/run/
press) per nav node, resupply on dry; walkreal_07000009_live loses hold_npcs.

## level-09-live-guards-walk result (06:20 UTC)

The walk PASSES without hold_npcs on real input: 4/4 green on the merged tree (master
0c90eba8 merged in before the final push) — the exit fired 28 ticks into the cab walk,
216,228 ticks start-to-finish, 0 stuck, 0 falls, no alarm channel ever. The dart whitelist
fix (CIVILIAN_CHALLENGE) verified by the new file pin (challenge → dart → sleep 0x51, hp
unchanged). The takedown shape that survived: silent K5 dart sleeps via nav-graph approaches
(the stair geometry eats from-below lines), the heard-noise investigators prioritized, the
loose point-blank aim gate, the P2K only as the dry fallback. Slice (touched binaries,
final tree): 625 passed, 0 failed, 7 ignored (4 min) — zero FAILED/TIMEOUT/CRASH. The full
suite restarted on the final tree (build ok 915 in 1109 s); its tail is the next session's
first read. Report: docs/reports/level-09-live-guards-walk.md.
