## Your task: `retune-nb-01`, 01 spawn-to-exit-with-the-contact real-damage walks green again (Operative, Agent, 00 Agent)

Branch from current origin/master. Commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt attack from the fire states; commit 6fe4339a1 re-routed the escort through the hall's north opening, and the three walks walk_from_the_spawn_to_the_exit_with_the_contact_real_damage, _agent and _00agent are still ignored, each with its measured death in the ignore message. Re-tune them for the new fights, Operative first: read each ignore, find where the new close-range attacks or bunching cost the health, and change the route, stance or fight order there. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-01`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-21T07:27Z

---
## Report (done, 2026-11-21T08:53Z)


---
## Report from the worker (done, 2026-11-21T08:56Z)

# retune-nb-01 — 01 spawn-to-exit-with-the-contact real-damage walks (Operative, Agent, 00 Agent)

Branch `s/retune-nb-01` (from origin/master e0adca26e; origin/master 8f2ff8608 merged into the
final tree). The three walks are **still red but much deeper**: each ignore message carries its
refreshed 2026-11-21 ledger and the named next lever, per the task's fallback. Everything below
is judged on its own real run through the real input path (`walkreal_07000001_live`), no top-ups,
no guard-code changes, no damage/health changes.

## What was measured first (the baselines, [L] runs on this tree)

- **d1 (Operative)** died t28813 at (-29.86,6.47,13.64): #17 (skin 0x5000039, SMG, 12 hp) dealt
  50.8 in 48 hits at d 1.9 and the 0x5000036 melee guard 36.8 in 65 punches at d 0.7 — the
  plate nav's return-fire fight vs #17 lost its exposure at keep_off 6.5, then flanked INTO the
  pair standing on the plate approach lane, with the contact parked between them.
- **d2 (Agent)** died t14006 at (-22.47,6.47,13.64): the ambush retry for the walled 24-hp SMG
  captain #3 approached into the north lane at clip 1 spare 9 (the "successful" resupply fed a
  wheel pool, not the held gun) and #3/#7 shot Bond down; cost buckets: #7's hall volley 33.6,
  #12's popped-ambush fight 29.4, #3's retry 24.4.
- **d3 (00 Agent)** died t2324 at (7.04,6.19,9.67) IN the door lane: the lane fight tapped #7's
  torso at 0.9-1.2/hit (the SMG aim ladder rode dy 0.05 = the feet) while #7's pellets landed
  4.1/hit — a 5:1 race loss; 15 body taps left #7 at 9 hp.

## Levers landed (all level-01 test code; helpers untouched)

1. **d3 SMG head-box aim** (`walkreal_07000001_live.rs`, the lane-stand fight): the SMG ladder
   0.05-first → 0.75-first. The landings ride the aim minus 0.0-0.2 on this stream; the part-5
   head box sits at dy ~0.55-0.65 (measured on the run: part 5 at +0.57 = 12.0 dmg, part 4 at
   +0.39..+0.50 = 1.2). **Result: the door phase is WON** — #6 (12 hp) dies in 1 head tap,
   #7 (24 hp) dies to head taps with ZERO volleys taken (the volley race won at t2375 [L]).
2. **d3 nearer-shotgunner-first** (the volley-race sort's tie-break flipped): each dead 0x1c
   guard halves the volley clock; d1/d2 keep agent_order (the sort is d3-gated).
3. **d3 volley-break windows** (the hall-4 peek-dance law, d3-gated): hp0 taken before the stand
   walk, one wound ends the peek, walk home to the deep line quiet, wait 75, re-peek, 4 windows;
   the walk-ins run with return fire off (the v3 run's return-fire trade paid 57.6 for the kill).
   d1/d2 do one pass (the break is d3-gated) — byte-identical streams.
4. **d2 dry-retry gate** (the ambush loop, d2-only): the walled-guard retry only runs when the
   HELD gun holds ≥12 (the resupply can feed a wheel pool while the held gun stays dry). The
   eight dry pushes skipped on the run; the t14006 death is gone (now t28819, the plate lane).
5. **d1/d2 pre-plate lane clear** (in `walk_to_the_exit_at`, real-damage-only via `!w.tops_up()`):
   round 0 fights the VISIBLE guards from the east lane stand (-24.9,14.5); rounds 1-2 step to
   the pocket edge (-30.0,10.2) — the shoot flank's own proven exposure spot. The stand walks run
   quiet. Walled guards never take a flank from a stand (the flank passes #17's face at 1.1-1.3 u
   inside the 3.0 forced band [M 0x179740:100-110] — the v5/v8 deaths).
6. **Tried and reverted**: opening the d3 east-armour fetch to d1 measured a LOSS — the side door
   (7.9,5.4,2.9) wedged the direct out-leg (d 4.21, 3000 ticks, armour untaken) and the lost
   ticks left the escort unattended during the west-end pre-clear's guard-2 grind (10800 ticks);
   the contact died at (-6.1,13.6) t56723 with Bond at 86.0. The gate is back at d3; the fact is
   recorded in the code comment.

## What is still red (the ledgers live in the ignore messages)

- **d1**: Bond reaches the plate lane at 50.8; #17 parks ON the stairwell-head lane
  ((-30.1..-31.1, 13.1-15.0)) and every approach passes inside the 3.0 forced band. Next lever:
  a d3-style converged tap loop from the pocket edge (the machinery that won #7's race) — the
  walker's shoot machinery flanks instead of standing.
- **d2**: t28819 at (-25.73,13.63) — #7's hall volley 33.6 (the d2 lure/hold still stand; the
  d3 crouch law is d3-gated), #12's fight 29.4, the same plate-lane pocket pair (#17, 24.4
  sustained). Next levers: the d2 lure/hold crouch, then the pocket tap loop.
- **d3**: t75294 at (-25.98,14.46) — the ENDGAME: the pool ran dry (clip 1 spare 7; the corridor
  boxes taken, the dead-guard drops no_fetch-banned) and the stairwell-head trade vs #3 (24-hp
  SMG captain, exposed at 5.0) took 17.6. Next levers: allow the WEST-corridor dead-guard drops
  (the ban is hall-interior-scoped), the #11 window cap (the pool thief), the pocket tap loop.

## What was run

- `walkreal_07000001_live` whole minus the three ignored walks on the final merged tree:
  **5 passed / 0 failed** (483.6 s) — `walk_from_the_spawn_to_the_exit_with_the_contact`,
  `following_mayhew_down_completes_the_level`, `the_plate_refuses_bond_alone`,
  `the_hall_guards_wait_for_the_meet_scenes_channel`, `the_exit_fade_starts_...` — the green
  walks stay green (the levers are real-damage/difficulty-gated; the shared helpers are
  untouched).
- `interactables_07000001_live` whole: **10 passed / 0 failed** (26.0 s).
- The three real-damage tests individually per lever iteration (the baselines and the v2-v9
  runs cited above; each run 12-33 s).
- The fast suite (`tools/test/suite.py`) on the final merged tree: **no total line** — the
  foreground run was killed at 560 s while `cargo test --no-run` was still building the merged
  crates (the sandbox kills background jobs between tool calls, so the runner cannot outlive a
  call here). The two binaries above were run whole by hand on the same tree instead.

## What is left

The three walks stay ignored with their refreshed ledgers and named next levers. The next
session should: (1) port the d3 converged tap loop to the plate-lane pocket (d1/d2), (2) open
the west-corridor dead-guard drops for d3's endgame ammo and cap the #11 windows, (3) the d2
lure/hold crouch, then re-judge each walk on its own real run.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no cost or usage numbers were displayed anywhere in the tooling).
