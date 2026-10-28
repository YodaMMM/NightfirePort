# Report: cutscene-handback-anim-prev-3

Branch `s/cutscene-handback-anim-prev-3` (from `origin/s/cutscene-handback-anim-prev-2` @ 567bc19c; merged `origin/s/cutscene-handback-sp-path` @ e5e9bee7 — its WORKLOG entry kept alongside the -2/-3 entries; merged `origin/master` — already an ancestor, "Already up to date"). The kind-2 hand-back lands with the four -2 reds green.

## The merges

`origin/s/cutscene-handback-sp-path` merged clean except `docs/WORKLOG.md` (both sides had appended entries; both kept). `origin/master` @ fcf6f334 was already an ancestor of the -2 branch. The driver asked to fold master's scoped 07000014 vent-drop fallback for scene 0x06000833 in `nis.rs` into the general kind-2 rule: **master's tree carries no such arm** — `grep -rn "06000833" crates/*/src` hits only test files (`capture_placement_07000014_live.rs`, `walkreal_07000014_live.rs`, `walk_07000014_live.rs`, `countdown_cutscene_cameras_live.rs`); `nis.rs` has no vent-drop arm and the kind-2 hand-back already resolves 07000014's opening scene through the general rule (the sp-path real-path test covers the mechanism). Nothing to fold; noted rather than guessed.

## The four reds, each re-derived on the real input path

1. **walkreal_07000002_live `walk_the_castle_all_objectives_to_the_exit` — GREEN.** Two harness defects, both on the test's side (the decompile-backed assert — ch 0x48 [D pl[011] param 0; M Destroy_CollisionHandler 0x12bd48] — stands untouched):
   - **The shot line.** The [L]-shifted start's nav approach arrives on the corridor's raised mezzanine (y 6.47), never the lower walkway (the -2 report's diagnosis). A segment-bisect probe [L, this sandbox, 2026-10-28] shows the walkway ray from the corridor's west end is clear — the eye→centre segment's first blocked point is t=1.000, the box itself — while the -2 "raised arrival A" eye sits INSIDE geometry (t=0.000). The leg now shoots from the corridor end the route already walks (`bunker corridor` (−55.1, 5.9, −23.1) [D route]).
   - **The fire discipline.** `shoot_point` fired every tick while discarding its own `on_target` computation — at 6.5 m the converging aim's ±0.05-rad wobble is a 0.33 m miss band over the 0.41 m-tall computer, and the clip emptied on the geisha-1 captors before the box. The tap is now gated on the view being on the point (the same discipline `shoot` uses) and `tick_fire` reloads an empty clip with spare ammo (the game's R).
2. **civilian_no_shoot_live `the_no_shoot_crosshair_keys_on_the_held_weapon` — GREEN.** After the skip-path hand-back (the FFwd end pose (−27.30, 1.23, 28.90) [L port NF_SCRIPTLOG]) the per-tick visibility ray to the civilian is blocked [M drone.rs:17377 `drawn = enabled && visible_to(...)`]. The player's own answer — close the distance: the test now walks Bond toward the civilian (real `step_bond` input) until the ray clears, then runs the unchanged crosshair arms (the weapon-keyed mark [M 0x1a7810:152-183] untouched).
3. **bathhouse_voices_live (2) — GREEN.** Both tests' shouts now roll. Root cause found with a temporary speech-path log (removed before commit): the ch-0x81 trio **woke and hunted but camped in CombatNoSight 0x5c for the whole window — zero AttackTalk sites re-entered** (the sites are the FirstAttack / AimStandFire / CombatNoMove ENTERs [M 0x148010:94; NDrone2_AttackTalk 0x13eac0]), because the tests' teleport put Bond 1.0 BELOW the plate centre — buried below the stair lip, never sighted. The voice RNG coin was never the blocker. Fix: Bond stands ON the plate (the AABB centre, inside [D placement 389's volume y 5.34..6.73]) — the player's own walk-over posture — and the stand windows match the roll timing (the probe's plays landed +552/+786 after the teleport [L, this sandbox]; the old 600/300-tick windows could close before a roll).
4. **bathhouse_shout_mixer_live `the_bunker_trio_shout_reaches_the_mixer_with_the_ambients_live` — GREEN.** Same plate-posture fix; the shout's stream Play reaches the mixer and the four map ambients keep their voices (unchanged pins).

With `on_target` now gating `shoot_point`'s tap and the reload added, every other `shoot_point` caller benefits identically; the four bins this task touched all run green in the foreground:

| bin | result |
|---|---|
| civilian_no_shoot_live | 3/3 |
| bathhouse_voices_live | 3/3 |
| bathhouse_shout_mixer_live | 2/2 |
| walkreal_07000002_live | 1/1 |

No decompile-backed assert was weakened: the only changed pins are harness placements/routes/windows, each cited above.

## Full suite

`tools/test/suite.py` on the final tree (the fixes + both merges), started in the background:

```
<SUITE_TOTAL>
```

<SUITE_NOTES>

## What's left

- The skip-path hand-back pose is still unmeasured on hardware (the -2 report's live check 1 — the port's FFwd lands at the anim end pose; the game's own skip is unverified). The civilian test no longer depends on it (it walks from wherever the hand-back parks him).
- Live check 3 of the -2 report (where the game's Bond stands for the computer shot) is now moot for the test — the corridor-end ray is clear [L bisect] — but a PCSX2 read of the player's floor there (walkway 5.9 vs mezzanine 6.5) would still confirm the game's own approach.

## Live checks for the main agent

1. **07000002, the intro's SKIP-path hand-back** (unchanged from the -2 report): fresh boot, press skip during the intro, read PLAYERS obj+0x30. The port lands at the FFwd end pose (−27.30, 1.23, 28.90).
2. **07000002, the bunker corridor's two floors**: walk the tunnel route to Mayhew's computer and read PLAYERS obj+0x30 (floor 5.9 = walkway or 6.5 = mezzanine?) at the moment the computer is shot — confirms which level the game's own player shoots from; the corridor-end ray is clear from both [L bisect probe: (−55.1, 5.9→7.45, −23.1) and the mezzanine (−54.63, 6.47→8.02, −21.97) both reach the box centre].

## Usage

not visible to me
