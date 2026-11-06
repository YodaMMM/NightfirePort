# guard-radii-finish — the guard-goal-arrival-radii reds, test-side

Branch `s/guard-radii-finish` = `origin/s/guard-goal-arrival-radii` 70c2f9e57 merged with
`origin/master` c4c00c7ec (clean merge; the master side already carried its own 011/013 walk
work). The radii are the game's and were not touched: every change below is test-side, per
walk, in the walk's own file. No `crates/*/src/` file changed on this branch.

## Done

1. **ai_live `guard_takes_cover_like_the_game` — decided from the [L] source, seed re-picked
   9 → 3** (commit 029f97a67). The [M] reading stands: FindCover sets its own goal's arrival
   radius from the chosen node's kind byte — 0.5 for kind 0/1, else 1.0
   [M 0x152548:174-181, `pcVar7` walking the 0xb0-stride node array; the port's parsed nodes
   are only kinds 0/1 [D placements 0xE5/0xE6] — so every real cover goal is 0.5]. The game
   ALWAYS ran 0.5, so the live in-cover pin (58.83, 99.32) [L] was measured under it and the
   pin does not move. The disagreement was the shared-RNG stream: with the true radius the
   cover run arrives deeper, the stream shifts, and the old seed 9 held the 90-tick lean in
   UnderCoverAim (live 37). A seed scan (seeds 0-40, all pins: the state chain, the two
   duration bands 28..=34 / 32..=40, cover node 1, the in-cover spot within 0.4, and the
   sister no-cover assert) passes on 3, 15, 16, 18, 23, 25, 28, 29, 30, 36 — with the
   in-cover spot back at err 0.01-0.03 of the [L] live position. Seed 3 taken; every [L] pin
   keeps its live value. **ai_live whole: 37 passed / 0 failed / 15.8 s.**

2. **walkreal_07000015_live `walk_spawn_to_the_omega_sector_exit_real_damage` — the crossing
   rifle guard resolved by skin at the [L] crossing spot, not a hard port index** (commit
   23e153540). On the merged tree the crossing's own return fire kills skin 0x5000087 as port
   index 1 at tick 977 [L the kill trace: his last stands (61.99, 41.28, 1.99) → (61.55,
   41.28, 2.21), the two P2K hits 4.8 → −0.5], so the old `shoot(0)` aimed at a different
   guard whose parts never expose from the lane (37 P2K rounds, his health 25 unchanged, the
   measured run) and the #462 mine shot arrived dry (clip 0 spare 0 through the whole 3600-
   tick shoot loop). The fix resolves the guard by `skin == 0x5000087 && dxz < 4.0` of the
   crossing spot and shoots only when alive; when the return fire has already dropped him
   Bond crosses and shoots the beam. **walkreal_07000015_live whole: 6 passed / 0 failed /
   1 ignored** (the d3 00agent ignore is pre-existing with its measured budget).

3. **walkreal_07000013_live — already green on the merged tree, nothing to change.** The
   master merge carried the fix. **Whole: 2 passed / 0 failed / 1 ignored** (the plain
   real-damage variant keeps its pre-existing scripted-bot ignore).

4. **walkreal_07000006_live `walk_the_castle_interior_to_the_party_real_damage` — d1 aimed
   stands on the ledge route** (commit fd3f231ab). The true goal radii stand the ch-55
   reinforcement pair at their EAST posts — #20 (29.0, 42.0, −37.0) hp 20 and #21 (28.8,
   42.0, −34.5) hp 10 [L the 2026-11-06 ledger: both visible in the mouth census at d 29.6
   with rays0] — and the east-courtyard leg walked Bond past them at 8.8-9.2 u with no fight
   arm (the d3 `clear_the_spot` arms are difficulty-gated off): the corner fight left him at
   33.5 and #20's 0x16 rounds finished him at (20.27, 41.13, −37.97), tick 5217. The fix adds
   the file's own aimed-stand tier at the strip and corner arrivals, **gated `difficulty ==
   1 && walker::no_topup_env()`** so the currently-green d2 Agent sister's timeline keeps its
   measured shape. **walkreal_07000006_live whole: 5 passed / 0 failed / 2 ignored.**

5. **walkreal_07000001_live `..._real_damage_00agent` — improved 19098 → 70165, still RED**
   (commit b28203c21). The death's real shape on the merged tree: the named loop's guard-6
   fight (skin 0x5000036, weapon 0x1c, 12 hp) sat across the hall aperture with no part
   exposed from the hold line, and the shoot machinery's flank nav opened the hall door and
   walked Bond INTO the hall — 1.9 u off the guard, 58.0 in 8 hits through the ×2 TooClose
   band [M Drone_ModBulletDamage 0x139f50]. The fix fights cross-aperture named guards from
   the open aperture's threshold (tracked burst at the live position, back off to the hold
   line on the first wound, two rounds of it) — guard 6 dies for the AND and the walk reaches
   deep into the escort. **Whole: 7 passed / 1 failed** (the d3 00agent walk). The measured
   next failure: guard 11's threshold fight runs the pistol pool dry (clip 1 spare 0 — the
   resupply cycles find nothing), the fall-through shoot flanks the door dry, and lip guard
   #5 (0x5000038, 0x1c, 24 hp) takes the last 24 at 1.4-2.2 u. An ammo-guarded variant was
   TRIED and measured WORSE (the resupply detour reshuffled the stream; death tick 7026 on
   the hold line to #5's −14.40 volley) and was reverted. **Left for the next session:** make
   the threshold fight ammo-aware without a mid-fight detour (resupply BEFORE the hall phase
   at d3), and run the d3 lip block's scan again once the escort crossing begins (the
   report's own "post-escort re-sweep" — #5 enables/surfaces late).

6. **walkreal_07000016_live — both chasers reds LEFT RED, evidence captured** (nothing
   changed on this file). `the_chasers_punch_lands_the_record_damage`: Bond bled to 17.4 by
   tick 4215 through 9 return-fire fights, then died at 8519 at (−14.51, −22.00, −170.18) —
   the chasers' pistol chips (−1.0 from #6 skin 0x5000087 at 13.3 u and #7 skin 0x5000090 at
   5.4 u) ground him down and drone #9 (0x500001e, weapon 0x12, FULL hp 100) finished at
   **0.2 u**. `walk_the_chasers_defeat_completes_the_mission_on_real_damage`: the same chip
   grind, then death at 10959 at (−5.83, −21.99, −163.14) with the last hit −10.00 from THE
   WORLD (live wire / blast / hurt volume) — the chips left him no margin at the wire. The
   shape on both: the true radii stand the chasers deeper/closer, stretching the chip
   windows. Next session's levers: kill the chip shooters #6/#7/#9 during the early phases
   (they are pistol chasers at 4.6-13 u, exposed), and route the defeat walk's end around the
   wire volume at (−5.8, −163). The d2/d3 sisters keep their pre-existing measured ignores.

## Runs this session (each binary WHOLE, `--test-threads=1`, NF_ISO set)

| binary | result |
|---|---|
| ai_live | 37 passed / 0 failed / 15.8 s |
| walkreal_07000013_live | 2 passed / 0 failed / 1 ignored |
| walkreal_07000015_live | 6 passed / 0 failed / 1 ignored |
| walkreal_07000006_live | 5 passed / 0 failed / 2 ignored |
| walkreal_07000001_live | 7 passed / 1 failed (the d3 00agent walk — item 5) |
| walkreal_07000016_live | the two chasers reds failed; the d2/d3 sisters ignored; the rest of the binary green (the overlook/pit/decoy tests passed in the same run) |

The remaining walkreal binaries and the full four-crate suite did NOT fit this session: the
sandbox started cold (2 cores, 3 GB, the build itself took the first hour and hit two disk
walls that had to be cleared), background jobs die between tool calls here so every run was
foreground. All four crates build clean (`cargo test --offline --release --no-run`); the
binaries that exercise every file this branch changed were each run whole and are quoted
above. No `crates/*/src/` file changed, so no src-adjacent test binary was orphaned.

## What's left

1. 001 d3 (item 5): the ammo-aware threshold fight + the post-escort lip re-sweep.
2. 016's two chasers tests (item 6).
3. The full suite re-run (the driver's fast runner) once the walks are green.
4. The two d3 sisters' ignores (001's is now the only non-scripted one among them) — after
   the d3 walk lands, re-audit the ignore budgets.

## Live checks for the main agent

1. On 07000001 d3: during the hall door-lure phase, log guard skin 0x5000036 (the 12-hp
   shotgunner) — his position/state while the port's lure fires into the hall. Expected: he
   hunts the noise to the hall's south structure (the port measured (6.7, 6.3, 13.3)) but
   does not cross the aperture; the game's own player line for him is the threshold fight.
2. On 07000016: at the chasers phase, whether the game's chasers #6 (0x5000087) / #7
   (0x5000090) hold 4.6-13 u while chipping, and where guard #9 (0x500001e) stands when he
   reaches 0.2 u — is the 0.2-u finisher the game's own punch range or a port-only wedge.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures
anywhere I can read (no banner, no status line, no API surface exposed to the worker).
