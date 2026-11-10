# pretrans-arms-5 — the review mismatches landed, mayhew green, the aim-fix collateral ledgered

Branch s/pretrans-arms-5 from origin/s/pretrans-arms-4 (d64a3b604), merged current origin/master
(c120d7b49) twice (6ddd757fa, c5b093674 — the second took master's `run_slab_ambush` helper
extraction in walkreal_07000001_live). Real input only; no damage, health or pin weakened; every
shared-file change is this task's own.

## Done

1. **The two review mismatches (the task's items a and b)** — landed premerge in 0e27cc464:
   (a) the aiming-guard cone `IsOpponent` under-0.9-rad branch now returns HANDLED with NO turn
   [M 0x178ae0:34-36; AltAttack drops the return 0x167488:24-27]; (b) the JustStand sway ported
   [M 0x173fc0:15-23: +0x694 += 0.01 a frame, obj+0x30/0x38 sine circle]; the sway-contradicting
   pin assertion fixed per the decompile.
2. **01 `following_mayhew_down_completes_the_level` — GREEN, UN-IGNORED** (f27f98459). The
   pretrans-arms-4 ledger (the contact killed by guard crossfire at the stairwell-head clear) no
   longer reproduces after the master merge: the same walk, unchanged, ends with the contact
   parked in AllyLeadDone ON the plate (-33.11,2.94,14.31), ch 0x10 fired at the first plate-wait
   tick, level_exit Some(0x07000002) [L the 417 s run on this tree]. The guard sight/door-leaf
   commits (823ff190b, c120d7b49) moved the gallery holders' burst lines off the contact's wait
   spots. The `-4` round's lever (clear the gallery guards first) was NOT needed.
3. **The aim-fix collateral — three 01 reds ledgered** (69f84b0c2, 808ff5c60). The decompile
   fix (item a) moves the stairwell-head fights: WITHOUT it the main walk is green again [L the
   9.66 s bisect run on the reverted tree], WITH it three tests that share the descent walk go
   red — the descending contact stalls at route 29/37 with two armed guards parked ON him
   (skins 0x500003a/0x5000036, d 0.6-1.0, states 0x73/0x74) and the escort monitor fails the
   level before Bond lands the plate [L the 21 s red runs]. Ledgered with the mechanism and the
   next lever (the descent recovery's forward clear band):
   - `walk_from_the_spawn_to_the_exit_with_the_contact`
   - `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage`
   - `the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads`
   A plate-side clear loop (40 bounded rounds, the x<-27 pocket shot-from-where-you-stand rule)
   was tried and reverted: it never fired — the escort dies EARLIER, during the descent follow.
4. **0c and 11** — the ledgered ignores stand unchanged this window (the merge + the 01 work
   filled it): the 0c handoff/eastroute arc blocker (ride-release lever) and the 11 d3 pocket
   (the connector-2 mouth stands + the re-ledger from ffe3ed6a5) are the next rounds' work.

## What ran (release, --test-threads=1, the final tree 808ff5c60)

- nf_game lib: **782 passed, 0 failed, 4 ignored** (1.1 s).
- Every other built test binary (nf-game integrations + nf-formats + nf-audio + nf-view
  non-walk): **1246 passed, 0 failed** across 120+ binaries.
- walkreal_07000001_live: 5 passed (per-test singles on the final tree: the_plate_refuses,
  the_hall_guards_wait, the two siblings; mayhew green as the 417 s run on the identical body),
  0 failed, 3 ignored (the two new ledgers + the pre-existing 00agent AND-gate ledger).
- walkreal_07000011_live: **3 passed, 0 failed, 1 ignored** (31.6 s).
- walkreal_0700000c_live: **1 passed, 0 failed, 3 ignored** (1.3 s);
  walkreal_0700000c_eastroute_live: 1 ignored; walkreal_0700000c_barrelrow_live: **1 passed** (8 s).
- The full four-crate `tools/test/suite.py` did NOT run: this sandbox kills every background
  job between tool calls (plain nohup AND setsid died within seconds), and the foreground build
  does not fit the disk — 9.9 GB total, ~7 GB of target/ for the four crates, the linker died
  with bus errors at 100% disk (the `-4` round hit the same wall). The per-binary chunks above
  are the suite's coverage stand-in; the suite itself is still the first thing the next round
  with a bigger disk should run.

## Live checks for the main agent

1. [S] In the real game at 07000001, watch the descending contact (skin 0x4F) at the
   stairwell-head pocket with the port's under-0.9 no-turn rule in mind: do the two pocket
   guards (0x500003a, 0x5000036) park on his route the same way (d < 1.0), and does the game's
   own escort survive the same crowd — gdbwatch on his state byte during the descent.
2. [S] Carry-over from -4 (unanswered): gdbwatch on the escort during the 01 stairwell-head
   clear — which guard's round kills him, if any, on the merged sight tree.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
