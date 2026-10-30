# drone-bump-collision-12 — the 08-family red mapped to the every-stance push; the port work stays out

## What was done

Branch `s/drone-bump-collision-12` = `origin/s/drone-bump-collision-11` (a3a2b7f3) merged with
`origin/master` (21fbeed3, clean). The task: split the branch's changes (the kind-word arming +
every-stance Jointy push, the straddled-cel pick filter, the door-zone opponent gate, the
boundary-clamp removal), find which one moves each of the 14 named reds, fix the port where it
is wrong, and land only what keeps the full suite free of new reds.

**The landing: the four port-code areas and the branch-calibrated test files are restored to
master's bytes (commit b35ccece).** Nothing of the -1..-11 port behaviour ships. Kept: the
docs trail and two assertion-free `#[ignore]` probes. The reasons, per group:

### The 08 family (7 of the 14 reds) moves with the every-stance Jointy push — established

Baseline on the merged tree (all four groups active): `fix_party_mission_live`
`the_07000008_safe_chain_works_in_play` fails at the watch burn of safe bolt out 135; bond's
own print shows him FALLING at the bolt-135 stand: y 42.84 → 38.28 → 26.92 → 8.75 over 150
ticks [L the run log]. `fix_safe08_live` + all four `pickups08_live` tests fail at the same
step ("the watch burns bolt out 135").

**Experiment 1** — restore exactly one line, the -11 stance gate
(`if !ctx.bot_capsules.is_empty() && !self.grounded`, player.rs :1903): `fix_party_mission`
9/0, `fix_safe08` 5/0, `pickups08` 5/0 — all green, arming/filters/unclamp still active. So
the stance-gate removal is the mover for 6 of the 7; nothing else in the branch is needed to
explain them.

**The probe (`probe_guest_routes08.rs`, kept on the branch, run with `--ignored`)** — two
facts against the "game-true shove" reading:
1. With bond idle, NO bot ever approaches the bolt stands in 600 ticks (the guests — dtype 1,
   team 1 — sit static at (151.8, 43.0, −120.7) and around, none with a route). The shover in
   the failing tests is not a wandering guest following its disc route.
2. With bond teleported to the bolt-135 stand and every-stance ON, bond is ejected on the
   first tick (+1.10 x, +0.64 y, +0.83 z) and falls to y 7.45 — with NO bot within 5.0 m at
   any sample. The ejector is not a bot capsule either. What the game does at this stand
   (does its Bond get shoved / can it burn the bolts with company around) is exactly the -11
   live check, still open — see "Live checks".

With the game-truth unresolved and the tests un-derivable from disc evidence in-window, the
every-stance push stays OUT per the landing rule; master's airborne-only gate (its own comment
marks it `[S] unverified — the PORT CHOICE gate`) stays.

### spots_07000008_safe_bolts_in_reach — a different mover, isolated only this far

Still red under Experiment 1. The failing assert is an eye ray to bolt #207; on the branch the
blocker is STATIC world geometry — `segment_hit_static` blocks at the identical t=0.418
(normal [0,0,1] at ≈ (162.29, 44.03, −115.76)), no drones and no door objects stand in the
corridor, and the block is insensitive to ±0.10 eye shifts in y and z (a wall face, not a
knife edge) [L probe_guest_routes08's ray section]. A static ray cannot differ between trees
with identical world bytes — so a reverted port delta changed the STATE the ray sees (the
settle eye after `place_and_ground`, or an SS-door/intro divergence during `free()`). On the
final tree (master port code) the test passes: `playtest_spots_live` 18/0. The exact
intermediate is not isolated; the probe prints to reproduce it are one `--ignored` run away.

### The remaining reds — green on the final tree, per-group attribution left with exact commands

Direct runs on the final tree (`--test-threads=1`, NF_ISO set): `escort_follow_07000001_live`
1/0 (25.5 s), `fix_npc_doors_live` 1/0 (159.7 s), `speech_facial_fallback_live` 2/0 (4.0 s),
`walkreal_07000011_live` 1/0 (5.8 s), plus the 08 family above and `playtest_spots_live`
18/0 — every one of the driver's named reds that these binaries carry is green.
`walkreal_07000001_live`, `walkreal_07000014_live`, `walkreal_07000002_live` were not re-run
in-window; their test files and the port code are byte-identical to master on the final tree,
so they carry master's verdict.

## What ran

- Baseline (merged tree): `fix_party_mission_live` 8/1, `fix_safe08_live` 4/1,
  `pickups08_live` 1/4, `playtest_spots_live` 17/1 — reproducing the driver's list.
- Experiment 1 (stance gate restored): `fix_party_mission` 9/0, `fix_safe08` 5/0,
  `pickups08` 5/0; `playtest_spots` 17/1 (unmoved — its own section above).
- Final tree: the eight binaries above, all green (the numbers are in the section before).
- The full suite: the final tree differs from master only by ignored probes + docs, but the
  runner could not produce a total line in this window — the sandbox kills background jobs
  between tool calls (verified twice today: `suite.py` launched with nohup was dead at the
  next tool call, `suite12base.out` empty), and a foreground 892-binary build+run does not
  fit the remaining minutes. See "The full-suite total" below for what was run instead.

## What's left

- The per-group attribution for `fix_npc_doors`, `escort_follow_07000001`,
  `speech_facial_fallback`, `walkreal_07000011` and the walkreal trio on the BRANCH tree.
  The next window should re-land ONE group at a time from the -11 tip and run exactly:
  `cargo test --offline --release -j4 -p nf-view --no-fail-fast --test fix_npc_doors_live
  --test escort_follow_07000001_live --test speech_facial_fallback_live
  --test walkreal_07000011_live -- --test-threads=1` (~200 s), then
  `walkreal_07000001_live` (215 s) and `walkreal_07000014_live`; drop any group that reddens.
  The Experiment-1 state (arming + filters + unclamp + master's airborne gate) is the
  recommended first candidate — it is already proven green on the whole 08 family.
- The spots-ray intermediate (settle eye vs SS-door state) — the probe prints identify it.
- The full-suite total line on a tree that re-lands the surviving groups.

## Live checks for the main agent

1. 07000008 fresh boot: after the safe chain, stand bond at the bolt-135 spot
   (162.24, 43.3, −115.6) with party guests in the room — does the game's bond get shoved
   while burning the bolts, and can the watch burn bolt #208 (out 135) at all with company
   present? (Settles whether the grounded Jointy shove at the stands is game-true — the -11
   check, still open; the port's fall at the stand reproduced with no bot within 5 m, so the
   ejector itself is also worth the watch.)
2. 07000008 fresh boot: from the stand (162.24, 43.3, −115.6) facing the bolts, is the
   doorway's upper area (≈ (162.29, 44.03, −115.76)) clear at head height? The branch tree
   blocked a static face there; master's port code clears it. Which matches the disc?

## Usage

not visible to me
