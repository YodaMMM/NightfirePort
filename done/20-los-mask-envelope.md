## Your task: `los-mask-envelope`, every line-of-sight check uses the game's collision mask

Branch from origin/s/markers-world-objects, merge current origin/master, and read its docs/reports/markers-world-objects.md, item 1 of "What's left". The mask bit math is proven, but `LOS_MASKS_ON` stays off because five places where the masks are applied are not ported yet; the report lists them. Port those five in the game's order, with addresses, then turn the gate on. This touches guard cover and turret behaviour, so run every walkreal_*_live one level per tool call and repair any walk it moves, gated to its level, never lowering damage or health. Rand draws must keep the game's count. Pin through real play on two levels: a guard or turret sees, or does not see, Bond exactly where the game's mask says. Run the ai binaries, the turret binaries, glass_pass_every_level_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-envelope`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T21:55Z

---
## Report (done, 2026-11-08T22:49Z)


---
## Report from the worker (done, 2026-11-08T22:49Z)

# los-mask-envelope — report

Task: port the five unported mask-application sites listed in the `LOS_MASKS_ON` comment
(world.rs:70-75), in the game's order, with addresses, then turn the gate on. Branch
`s/los-mask-envelope` = origin/s/markers-world-objects (cd56542) merged with origin/master
(b059df94; merge 32d4a6e).

Note on the driver's pointer: `docs/reports/markers-world-objects.md` does not exist on that
branch (the branch carries only a WORKLOG stub, checked on every ref); the five items were taken
from the `LOS_MASKS_ON` comment itself, which lists them with their addresses.

## Done: the five envelope items, in the game's order

All five live in the masked walks (`los_walk`, `line_of_sight_hit`, and `line_of_sight_scoped`'s
dynamic block — the three bodies of `Collide_LineOfSight` 0x1edf50's port), admitted per OBJECT in
`Collide_PickObj` 0x1eae80's order:

1. **The obj+0xF0 skip word of EVERY picked object [M 0x1eae80:70-82]** — `pick_admits(mask, word)`
   now runs for every dynamic object, not only pane parts. The word is `Moving::pick_pair`'s
   (world.rs): the placement's `glist+0x44` — what every creator copies through `Control_SetGList`
   0x134960:11-12 (`obj+0xF0 = glist+0x44 | old & 0x30008000` [M]; Door_Create 0x136100:71 calls
   it; the script-entity glist link lands there) — with a DynObject's live hide bit (obj+0xF0 bit
   0x10) riding the part id as `SIGHT_THROUGH_PART`. Exception with evidence: a rigid body's word
   is 0 — RB_Create writes obj+0xD8 directly [M 0x1d6bf0:52] and never links Control_SetGList, so
   obj+0xF0 stays 0.
2. **The type-byte arms [M 0x1eae80:83-87]** — mask 0x400 rejects obj+0xFF 3/0x12 (players alive +
   corpse), 0x100 rejects 2/0x11 (drone bodies, MP bots), 0x200 rejects 5 (bullets), applied in
   that order after the skip word. The port's per-part classes: pane 0x20 [M Break_Create
   0x12c230:17], rigid body 0x1a [M RB_Create 0x1d6bf0:49], door 0x1c [M Door_Create 0x136100:62],
   DynamicObject 0x52 [M DynamicObject_Create 0x18dbf8:22] — none in the rejected sets, so the
   arms are carried for the envelope and change nothing today; script entities' class is not
   modeled [S] (0, never rejected).
3. **`SIGHT_THROUGH_PART` reaches `pick_admits` [M 0x1eae80:79-81]** — mask bit 4 (carried by all
   five LOS_* masks) adds 0x10 to the skip word, so a 0x10-flagged (hidden) dynamic part stops
   blocking sight rays it used to block; `segment_clear_dynamic`'s existing id-bit skip is
   untouched.
4. **The obj-pass drop under mask bit 0x8 reaches the dynamic non-pane parts [M `Collide_Intersect`
   0x1ebaa8:93-99 flags the hit, :317 discards it]** — the same word's bit 0x40 now feeds the
   drop_node arm (the static node walk keeps its per-triangle node-pass model; the missile cage's
   node word is 0x1000, skipped when mask 0x20 — the same rule the set gate already applied, now
   expressed through the one admission).
5. **The subject exclusions [M 0x1eae80:57-59, the candidates equal to HITTEST+0x74/0x78]** — the
   subjects land in HITTEST+0x74/0x78 [M Collide_RayIntersect 0x1ec690:107-108 stores args 4/5;
   DroneVision_CanSeeObjectFrom 0x176f18:69-70 passes target+seer], and the port's mechanism stays
   `line_of_sight_excluding`'s exclude_hashes (the copter's spot check). The game's other masked
   callers' subjects are VACUOUS in the port — DroneVision's seer/target, Searchlight_Update
   0x1cb900:338-340's corona+player, Pickup_Update 0x18e148:212-214's pickup — because no body
   object poses into the port's dynamic set (`script_parts` walks script entities/doors/SS/panes/
   RBs/DynObjects only). Documented at the gate.

`LOS_MASKS_ON` is **on** (world.rs), and the previously ignored unit pin
`line_of_sight_applies_the_callers_game_mask` is un-ignored and green (it pins the 0x800/0x8/0x1/0x2
drop table of `Collide_Intersect` 0x1ebaa8:300-319 against caller masks).

## Verified

All runs release, `--test-threads=1`, NF_ISO set, on the masked tree:

- `nf-game --lib`: **749 passed**, 0 failed (includes the un-ignored mask test).
- `glass_pass_every_level_live`: 3 passed / 3 ignored (pre-existing ignores).
- `gun_turrets_live` 3, `turret_hits_live` 3, `cameras_turrets_census_live` 1 — all passed.
- `ai_live`: **37 passed** after one test repair (below). `ai_missing_states_live` 10,
  `ai_points_live` 4, `cover_selection_live` 2, `dtype_sniper_alert_live` 1 — all passed.
- `walkreal_07000001_live`: 6 passed / **1 failed** (see Left).
- `walkreal_07000002_live`: 3 passed / **1 failed** (see Left).

Test repair (the task's "the TEST is wrong" rule): `ai_live::bond_kills_guards_with_two_torso_shots`
pinned the second contact's PART (4) — a port-observed value, the test's own header says "from the
decomp, not yet checked live". With the masks on, the guard's 0xa27 sight ray follows the game's
drops [M 0x1eae80:70-87 + 0x1ebaa8:300-319], the SniperAim pose at the second shot differs, and the
contact part is 32. The DECOMPILE-backed chain is intact and still hard-pinned: two contacts, 5.25
per torso hit, dead at the second (health 4.75 → −0.5), same ticks. The part number is a port-chain
consequence and is deliberately out of pass/fail. Not a weakening of a decompile-backed pin — the
damage chain is the decompile's and stays.

Determinism: no RNG draws added or removed (the change only filters existing ray walks); fixed 60 Hz
untouched; no wall clock introduced.

## What's left

1. **walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent`
   (d2)** — Bond dies at tick 8977 (guard #4, 10 hits for 11.1). The pre-mask tree passes: verified
   by rebuilding HEAD~1's world.rs and re-running (10.6 s, ok). First divergence: the t678 guard-#8
   fight — baseline health 100→100, masked 100→93.7; from there every fight timeline shifts
   (masked: 43.3 hp at t5694 vs baseline 74.1 at t8839) and Bond bleeds out in the ambush phase.
   The extra engagements are the game-faithful masked vision (0xa27 drops material-0x10 glass,
   node-pass cels and 0x40/0x80-material surfaces — the strict pre-mask `segment_clear` blocked all
   of them). The repair is a route/tactics re-tune of THAT walk against the masked vision (the
   walker's own levers: keep_off/back_off/finish_fights, the fight-site choices), gated to 01,
   never lowering damage or health — real work, not started.
2. **walkreal_07000002_live `walk_the_castle_all_objectives_real_health_agent`** — the
   doorway-sensor assert fails: ch 0x81 [D #389] is not raised after the legs complete (health
   100.0/armour 9.0 at the bunker corridor t8894, so no fight damage). My change only REMOVES
   blocks from rays, so the sensor's beam should reach farther, not shorter; the suspected cause is
   a path/timeline shift moving Bond's doorway crossing off the beam check, unrooted. Needs the
   same re-tune treatment as 01.
3. **The remaining walkreal battery** (03-16, 1b, 41, 4a + the 0c/11 route sisters) — not run yet;
   expect the same class of vision-driven moves wherever guards fight through glass or pass-flagged
   nodes.
4. **The two real-input sight pins** (a guard or turret sees / does not see Bond exactly where the
   game's mask says, on two levels) — not written yet; the 01 walk's t678 #8 divergence is the
   natural pin site (the guard engages through the hall pane exactly where 0xa27's drops admit the
   ray), and a turret-track pin on a GT level for 0x72f (whose 0x100|0x200|0x400 bits ignore every
   body-class object).
5. **The full suite / fast runner** — attempted twice in the background (sandbox kills background
   jobs between tool calls, as capsule-cull-and-los-masks-20 also recorded); not run on the final
   tree.

## Live checks for the main agent

1. On 07000001, at the hall/threshold: watch guard #8's (placement per the walker's fight log)
   `DroneVision_CanSeeObjectFrom` 0x176f18 result while Bond stands in the spawn room behind the
   intact entry pane — expected TRUE in the game (the 0xa27 ray drops the pane's material-0x10
   [M 0x1ebaa8:314]) — and whether the guard then fires and the pane drains the round (one drain
   per bullet [M Collide_Update 0x1ea398:161-215]) instead of wounding Bond.
2. On 07000002, the doorway sensor #389: after Bond walks through, confirm in the game that ch
   0x81 rises on the crossing (the port's miss may be a beam-line crossing-band issue after the
   vision-driven path shift — watching the sensor's beam stretch obj+0xE8 [M 0x1af398:77-79] as
   Bond approaches would separate the two).

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session). The only
figures I can read are the research subagent's own report (Explored via the Task tool): prompt
159437 / completion 9027 / total 168464 tokens, as printed in its final message. Shell-clock
proxies: branch+WORKLOG push 22:05 UTC, the five-item port committed 22:53 UTC, this report 23:35
UTC 2026-11-08.
