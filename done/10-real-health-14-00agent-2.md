# real-health-14-00agent-2 — the 14 00 Agent walk

Continue origin/s/real-health-14-agent-00agent (merge origin/master first) and read its report's "What's left": re-land the described crossing diffs, then the lid-lip jam (a hop-over in the leg's steer loop) and the legs after it. Real input only; no top-ups; helpers opt-in. Remove the ignore when green; every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox1 at 2026-11-04T13:50Z

---
## Report (done, 2026-11-04T14:41Z)


---
## Report from the worker (done, 2026-11-04T14:41Z)

# real-health-14-00agent-2 — report (2026-11-04)

Branch `s/real-health-14-00agent-2`, cut from `origin/s/real-health-14-agent-00agent` and
merged with `origin/master` (merge commit 5918eb5f, clean). Task: re-land the prior report's
crossing diffs, the lid-lip jam and the legs after it; un-ignore the d3 when green; every
`walkreal_*_live`; the full suite total line.

## What's done

**The re-land is in the branch history and MEASURED leg by leg** (3c011896 = the 971e480b
cc-fix cherry-pick; 71d0651c = the leg diffs):

- **The two SkyBridge crossing releases now drift WEST 15 ticks** (strafe `+1.0` — left of the
  hang's ENE facing [M the move quad `(c·vs, −s·vs)` player.rs:1771-1773], the game's own
  air-control lane). Measured this round: the trench-landing assert PASSES under the STANDING
  capsule — "the crossing landed (45.08, 0.51, 62.49)" — clear of the east walkway's west-edge
  catch (x 44.54, top 2.37 [D probe14_room]) that killed the re-landed walks last session.
- **The four vent-lane windows widened `p[1] < 3.4` → `p[1] < 3.6`** (the standing rest on the
  2.38–2.40 lane = 2.38 + 1.0347 root). Measured: `lane true` at y **3.468** — the crouch-era
  3.4 cap would have failed it.
- **The lid-lip jam: the report's "hop-over in the leg's steer loop" prescription is refuted
  by measurement.** With the standing capsule the leg freezes at (45.13, 3.416, 42.71) with
  vel (0,0,0) for 2500 straight ticks (instrumented run, since removed): the periodic hops DO
  fire but their arcs are squashed at +0.032 by the same contact, and the stand gate's +0.95
  headroom LOS passes UNDER the lintel while crouched (crouch root 2.95 + 0.95 = 3.90 < the
  lintel ~4.4), so a stand press at the mouth stands Bond straight back into the jam. The
  standing capsule cannot hop or stand through; the leg now crosses on **the game's own crouch
  toggle** — real input through `Player_HandleJump` (the crouch press pairs the flag with cc
  [M player.rs:1628-1633]; the stand re-press is the Toggle arm [M]), no pokes. Measured:
  `over true`, Bond reaches (45.13, 3.05, 40.02) crouched.

**The next exposed leg (the connector) is measured and PARKED.** The crawl crosses the lid
but jams at the mezzanine's west lip — STUCK (45.13, 3.05, 39.87), `segment_to_target` hit at
0.124, no progress for the full cap — and NO stand spot exists along the lane (the +0.95
headroom LOS is blocked from z 42.2 down to 40.0: the lane is a low duct). Both jams (the
lintel and the lip) are the **live-push vs dead-tail divergence** the prior report's live-check
3 anticipated [M Intersect_CylGeom 0x109da0:447 — the +0x50 DeltaP lifetime guard kills every
walk contact]. The lib's two experiment switches were tried as the principled fix and BOTH
break earlier legs: `NF_WALK_CLIP=1` panics at the test's line 568 and `NF_WALK_GHOST=1` at
walker_b2.rs:170, each 1.9 s into the walk — hour-scale to re-measure, not this window.

**The tip returns to the wrap-only state (549b5d20).** With the re-land in, the STANDING
capsule re-rolls five master-green walks (measured whole this round: **2 passed / 7 failed /
1 ignored** in 49 s — the_e_use_switch, the_skipped_vent_cutscene, the_vent_drop, both d1
walks, the tripbomb route), and each is an hour-scale re-verify. Exactly like the prior
session's close, the green-preserving tip carries the re-land in history one cherry-pick away
(`git cherry-pick 3c011896 71d0651c`), with today's added measurements in the commit messages.

## How it was verified

Per-binary, release, `--test-threads=1`, real ISO (`NF_ISO`):

- **walkreal_07000014_live (the final tree): 8 passed / 1 failed / 1 ignored** (187 s) —
  master's exact green set. The 1 failed = the d2 agent walk on real damage (the wedge — the
  honest red this task continues from); the 1 ignored = the d3 00agent (un-ignoring was
  gated on green; it is not green — the ignore text stands).
- The intermediate re-land tree (3c011896 + 71d0651c), run whole: 2 passed / 7 failed /
  1 ignored (49 s) — the standing-capsule re-roll documented above.
- The other walkreal levels (the final tree — its ONLY source diff vs the merge 5918eb5f is
  zero, verified `git diff 5918eb5f..HEAD`; every binary is byte-identical to the merged
  master's):
  - walkreal_0700000a_live: **12 passed / 0 failed / 3 ignored** (51 s)
  - walkreal_07000012_live: **3 passed / 0 failed** (24 s)
  - walkreal_0700001b_live: **17 passed / 0 failed / 2 ignored** (123 s)
  - walkreal_07000002_live: **3 passed / 0 failed / 1 ignored** (74 s)
  - walkreal_07000005_live: **14 passed / 0 failed** (76 s)
  - walkreal_0700000d_live: **1 passed / 0 failed / 1 ignored** (18 s)
  - fix_guard_first_shot_live: **1 passed / 0 failed** (1.3 s)
- **The full suite total line did not fit this window** — the time went to the re-land's
  leg-by-leg measurement (each cycle: rebuild ~80 s + run). `npc_doors_live` does not exist
  under that name in the current tree (checked `ls crates/nf-view/tests/ | grep door|npc`;
  the door coverage is door_guards_locked_door_07000014_live, cutscene_door_07000014_live,
  cutscene_door_crossing_07000014_live, 0700000d_gunimp_door_live, door_skip_lifecycle_live —
  not run this window). The walkreal_probe* / walkreal_dump binaries were not run either.
  Nothing under `crates/*/src` or `crates/nf-view/tests/common/` was touched by this branch's
  tip (rule-10 grep for the touched symbols lands inside walkreal_07000014_live alone, which
  was run whole).

## What's left

1. The connector lip + the descent's overhang legs under the STANDING capsule (the report's
   predicted "landing west drop" adaptive loop may already carry some — untested). Each is an
   hour-scale re-tune-and-verify.
2. The principled dead-tail fix: `NF_WALK_CLIP`'s move-clip model is the faithful shape
   [M 0x109da0:447 + M Collide_Intersect 0x1ebaa8:353-373] but breaks earlier legs as-is —
   re-measure it against the full walkreal battery before landing.
3. Then the d3 00agent un-ignore, then the FULL walkreal battery + npc/door tests + the full
   suite total line.

## Live checks for the main agent

1. **07000014, the east vent lane's mouth lintel (z ~42.7)**: with Bond standing on the lane
   at (45.5, 3.4, 42.75), walk south — does the game's Bond pass the lintel (a low ceiling
   over the lane at ~4.4) without a hop, or does the level design force the crouch? If he
   passes standing, the port's live push displacement (vs the game's dead push tail
   [M 0x109da0:447]) is confirmed as the lintel jam's root and the move-clip fix becomes the
   right move.
2. **07000014, the mezzanine's west lip (x ~44.6, z ~39.8)**: walk off the mezzanine toward
   the sloped connector — standing and crouched. The port's crawl jams on the lip (segment
   hit at 0.124 from (45.13, 39.87)); if the game's Bond descends it crouched, the port's
   crawl step-down needs the look; if he crosses standing, the duct's headroom reading is the
   suspect.
3. The prior report's three live checks (the crawl-exit cc pair, the SkyBridge release catch,
   the flush lid lip) all stand — checks 2 and 3 are now sharpened by this round's numbers:
   the release drift lands x 42.9–45.1 and the crossing clears; the lid jams at the LINTER
   (z 42.71), not the lip.

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).
