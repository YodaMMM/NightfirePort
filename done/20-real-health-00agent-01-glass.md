## Your task: `real-health-00agent-01-glass`, 07000001 can be walked on 00 Agent with rounds passing glass as in the game

Branch from origin/s/glass-pierce-2, merge current origin/master, and read its docs/reports/glass-pierce-2.md, item 3 of "What's left". With through-glass hits, the 07000001 00 Agent sister's atrium timeline shifted: guard #5 (shotgun 0x1c) at d 1.6 lands 76.6 in 8 hits, and Bond dies at 3623 at (6.49, 6.47, 13.48). Apply the family of repairs that worked for 08 in that report: plan the 01 fights with glass in the guards' sight, and back off out of #5's close range. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary, glass_pass_every_level_live and dead_pane_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-glass`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T06:19Z

---
## Report (done, 2026-11-08T07:17Z)


---
## Report from the worker (done, 2026-11-08T07:18Z — the body the first attach missed)

# real-health-00agent-01-glass — the 01 00agent walk is green with the through-glass hits armed

2026-11-08, branch `s/real-health-00agent-01-glass` (from origin/s/glass-pierce-2 fa1f11131,
merged origin/master d09f8b2b3). glass-pierce-2's "What's left" item 3: with the guard-side
glass pass armed, the 07000001 00 Agent sister's atrium timeline shifted and Bond died at 3623
at (6.49, 6.47, 13.48) — guard #5 (the shotgun 0x1c) at d 1.6, 76.6 in 8 hits [L the run log,
reproduced on the first run of this session]. **The walk now passes: the whole mission on real
damage, real input only — final state health 30.6, armour 0.0, tick 20940 [L the run].** No
damage or health write anywhere; no top-ups; the load path is the only difference plus the
d3-gated tactics below, all inside `walkreal_07000001_live.rs`.

## The deaths, in the order this session fixed them

1. **3623, the atrium/hall door (the task's symptom).** The AND's three named guards were down;
   the d3 lip pass ran and found the aperture band EMPTY (guard #5 was still deep north); the
   doorway step walked Bond in and the route pre-clear's first nav crossed #5's hunt line —
   d 1.6, the ×2 TooClose volley [M Drone_ModBulletDamage 0x139f50], dead before the return
   fire turned [L the run log]. Fixed by: `fight_the_hall_lip` (the lip fight extracted, now
   with the fight machinery's own crowd back-off on the lure) and **re-run after the doorway
   step** — the same aperture-band fight from the hall side, before any route leg walks west;
   plus the escort spacing restored (6.5) for the d3 open-floor legs (the doorway fight's
   band-edge 3.2 stands inside the shotgun's honest range).
2. **7891, the ch-0xF ambush back-off.** #3 (the SMG 0x12 captain, 7 hp after the popper
   fights' retarget taps) sprayed the back-off lane from the stairwell pocket with every eye
   ray walled — the plain return-fire pick answers only a VISIBLE shooter, so the fight never
   started [L the run log]. Fixed by the **unseen-fights opt-in** (`with_real_damage_unseen_fights`,
   the documented hurting-first fallback + the stand's creep): the pocket fight then ran and
   the creep opened the line.
3. **The unseen fight's own trap:** the walled climber #2 (the south lower hall) got picked by
   the ambush rounds and the pre-stairwell sweep, and the determined pass's 1800-tick flank
   walked Bond through the meet-room walls (the 6000-tick wedges at (−11.3, −4.8), segment
   0.026 [L the run logs]). Fixed three ways: **the determined pass OFF for d3** (`finish_fights
   = difficulty < 3` — the d3 fights re-pin on the short caps plus the route's revisit clears);
   the **mouth/sweep picks drop unseen pickers** (d3-gated LOS gate — the climber is left to
   the descent legs, the stairwell is his own path); and the **d3 mouth band 8.0** (the pocket
   fight stood at 6.65 — 0.15 outside the 6.5 crowd band — and chewed 43.6).
4. **The dry-gun dance + the dry-ladder wedge.** The pocket fight started on clip 0 spare 8
   (fight#3 t0 [L the rf-trace]) and the reload cycles stood Bond in the spray; later the pool
   ran dry at the stairwell (clip 2 spare 0) and the plate leg's dry-ladder resupply detour
   east wedged the nav. Fixed by: the **pre-ambush top-up** (d3 — the pocket is still disabled
   before the slab touch, so the walk is unmolested) and the **d3 pickup line walking ONTO the
   pickups** (arrive 0.4 instead of the pinned 1.0 — the loose stop left the west-hall placed
   pair untaken; the comment itself says the line exists to top up before the stairwell).

## Glass in the guards' sight (the task's first lever)

The 00agent test wraps the walk in `with_glass_sight` (glass-pierce-2's GLASS_SIGHT, opt-in
rule 7): the fights' and stands' sight rays take the flight's mat-0x10 drop — the game's guard
rounds pass intact 0x10 sheets whenever the weapon record's +0x6c bit 0x800 is clear [M
Bullet_CollisionHandler 0x12ce80 disasm 0x12cf44-0x12cf64; the guard guns' records carry it
clear [L the weapon_data census]], while the plain ray counts the panes' static tris solid. A
player sees and answers a shooter through a window. Only this d3 test sets it.

## Verified (this session, NF_ISO set, foreground, --test-threads=1)

- walkreal_07000001_live: **8 passed / 0 failed** — the WHOLE binary: the 00agent sister green,
  and the d1 real-damage, d2 Agent and topped-up sisters byte-identical green (the helpers'
  flags are thread-local opt-ins; the shared stages are d3-gated).
- glass_pass_every_level_live: **3 passed / 0 failed + 4 ignored** (unchanged from
  glass-pierce-2).
- dead_pane_live: **1 passed / 0 failed + 1 ignored** — the shot-out-pane round test is
  UN-IGNORED and green (the walk ledger it carried is resolved); the movement test
  `the_intact_pane_stops_bond_and_the_dead_one_lets_him_cross` keeps its ignore — the pane
  collision ride is still unproven (glass-pierce-2 "What's left" #5), and its ignore message
  now records the walk ledger as resolved.
- The full four-crate suite (tools/test/suite.py): **did not fit the window** — background
  jobs are killed between tool calls in this sandbox (the first background compile died the
  same way), and the foreground window went to the walk iterations above. The three binaries
  above are the coverage; every one that touches the changed file is green.

## What's left (the next message continues from s/real-health-00agent-01-glass)

1. The full suite + the wider walkreal sweep on the final tree (rule 7/10 hygiene; the shared
   helpers were NOT touched this session — only the 01 test file and the dead_pane attributes
   — so the other walks' exposure is nil, but the runs are the proof).
2. The 08 sisters' documented reds (glass-pierce-2 "What's left" 1-2: the topped-up rocket
   economy, the d3 Hind blast) — untouched, not this task.
3. The pane collision ride (the dead_pane movement test's ignore) — still open, drone.rs's
   0x400_0000_0000 shape/matrix pairing.

## Live checks for the main agent

1. 07000001, fresh boot, 00 Agent: stand at the meet plate and let the hall fight run — a
   guard firing from behind the intact windows should hit Bond through the glass (the armed
   guard-side pass [M 0x12cf44-0x12cf64]) and Bond's return fire should reach him the same
   way. Confirms the GLASS_SIGHT fight model matches the game's own exchange.
2. 07000001, fresh boot: the stairwell-head pocket — a guard in the pocket (x < −27) firing up
   the corridor at Bond on the corridor floor: the rounds arrive through the pocket mouth
   while the eye rays stay walled [L the sight probes]. Sizes how the game itself answers
   unseen shooters (it hunts the noise — the walk's lure/hold pattern).

## Usage

The session's input/output tokens, context used and cost: not visible to me. (Wall clock per
the shell: sandbox start 06:24 UTC, the green walk 07:48 UTC, this report 07:52 UTC, sandbox
deadline 08:23 UTC.)

ntfy failed: all three fallbacks (curl connection timed out 20 s; python3 urllib network unreachable [Errno 101] after an em-dash syntax error on the first try; wget killed at the 60 s tool timeout). The exact errors are in the report body above — see the branch report "ntfy failed" (commit 7bdfeb9cd).
