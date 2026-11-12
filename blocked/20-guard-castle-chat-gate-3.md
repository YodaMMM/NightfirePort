## Your task: `guard-castle-chat-gate-3`, the castle chat pair without touching other guards

Start from origin/s/guard-castle-chat-gate-2 merged with origin/master. The chat pair itself checks out against the game [L PCSX2 fresh 07000006: both pair drones in CastleChatGuard1 0x36 from frame 0, unchanged for 2400 frames with Bond idle at the start, ch 0x11 0; the port on this branch matches]. But the full suite on master + this branch has 4 reds that pass on master: guard_parity_live::a_tower_guard_outside_his_civilian_states_can_attack_on_sight and guard_ai_other4_live::investigate_stands_down_to_idle_when_nothing_is_found (generic guard AI - the change reaches guards outside the chat pair: find which shared arm, e.g. look_for_opponent's sight-bit gate or the enemy_alerts route, and keep it to what the decompile says for those states), rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway, and walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage. Each red gets a cause with the address; a test that was wrong against the decompile gets corrected with the address; nothing is weakened. Then the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-castle-chat-gate-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-12T05:51Z

---
## Report (blocked, 2026-11-12T07:33Z)


---
## Report from the worker (blocked, 2026-11-12T07:34Z)

# guard-castle-chat-gate-3 — the four suite reds on master + the chat branch

Task: the chat pair itself checks out against the game [L PCSX2 fresh 07000006]. This task took
the 4 suite reds that pass on master and fail on master + guard-castle-chat-gate-2, gave each a
cause with its address, and corrected what the decompile proves wrong. Status: **blocked** — the
two synthetic reds are corrected (re-verification pending, see below); the two real-walk reds
have their causes pinned but are open, with the evidence the main agent needs.

## The shared change under all four reds

gate-2 changed `Drone::look_for_opponent` (nf-game drone.rs, both arms) from the sticky
`self.seen` latch to the per-tick form `los && seen_ticks >= reaction_time(level)`. That is the
game: `DroneVision_EnemyLookForOpponent` 0x177ed0 refuses unless the sight bit `+0x228 & 4` is
set [M 0x177ed0:26-28]; HaveOpponentSight sets bit 4 only while the accumulated sight counter
(+0x270) holds past `DroneFunc_ReactionTime` [M 0x175ef0:41-44] on a tick whose LOS ray (bit 2)
held, and ProcessDroneSight clears bit 4 and zeroes +0x270 on the first pass without held sight
[M 0x176a28:200-202]. Bit 2 is cleared and re-taken inside each probe [M 0x175cb8:22-26,104] and
persists on ticks the round robin does not probe [M 0x176a28:153-179]. The old sticky latch
attacked from memory of a past sighting; the game does not.

## Red 1 — guard_parity_live::a_tower_guard_outside_his_civilian_states_can_attack_on_sight

Cause: the test modelled "opponent in sight" with `g.seen = true` (the ever-sighted latch).
The decompile gate is bit 4 [M 0x177ed0:26-28], so a sticky-seen guard without held sight never
lands the sighting (0x177ed0:76-80 is reached only past the bit-4 gate). **The test was wrong
against the decompile; corrected** (guard_parity_live.rs): the setup now sets the port's model of
bit 4 (`los = true`, `seen_ticks = 600` — past every reaction time) and keeps the assertion
(the sighting lands, [M 0x177ed0:76-80]) unchanged. The send() path runs no vision pass, so the
setup reaches look_for_opponent intact. **Verified green on the branch tree**: guard_parity_live
whole 20/0.

## Red 2 — guard_ai_other4_live::investigate_stands_down_to_idle_when_nothing_is_found

Cause: same sticky-latch modelling in the sister arm (Investigate's UPDATE calls
DroneVision_EnemyLookForOpponent [M 0x168070 case 3 → 0x177ed0]). **The test was wrong against
the decompile; corrected** (guard_ai_other4_live.rs): `los = true` + `seen_ticks = 600` instead
of `seen = true`, plus `opp_bond = false` so the sim tick's vision pass skips the probe (the
round robin probes opponent-pointer drones; a skipped tick keeps bit 2 in the game [M
0x175cb8]) — without it the pass overwrote the setup. The stand-down arm and both assertions are
unchanged. **Verified green on the branch tree** (after a forced lib rebuild — the first
re-run had linked master's nf-game out of the shared target dir; the ailog format proved it):
guard_ai_other4_live whole 4/0.

## Red 3 — rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway

Cause (pinned, not fixed): on this branch the switch-16 guard d#3 (skin 0x0500_002e, ch 16,
placement mode 0 [D census (30.3, 41.1, −16.8)]) sees Bond during "the guardhouse court" leg
(dist 13.5, reaction 10, state Idle 0x4 — NF_AILOG), enters Attack 0x56, and his FirstSightState
pick comes up 0, so FirstAttack's alternate-state tail runs ChangeToAttackMode — mode 2 → 0x19
(25), stationary [M 0x148010:100-131 + 0x14a648:7-36] — and he fights on as a stationary sniper
(SNIPER_AIM 0x2a / SNIPER_FIRE 0x2b). On master the pick was a combat move (STEP_AIM_LEFT 0x73),
which returns from FirstAttack before the alt tail [M 0x148010:41-45], so the mode stays 0. The
pick is the prop-0x1b coin (`rng.rand(2)`) [M 0x147c48 / 0x148010] — a shared-RNG draw whose
stream shifted because the chat pair's break-off timing changed: on master the pair left
PlayScript on the sticky latch (early), on the branch they hold 0x36 until the held sight
[the game's rule]. The whole guard population's state cycles differ between the two logs
(/tmp/rook_master.log vs /tmp/rook_branch.log: guards 12/13/14 cycle 0x68→0x5c→0x5f ×140 on
master, entirely different paths on the branch). The test's `d3.mode == 0` pin at
"before door #47" is therefore an old-model outcome, not a disc fact — but the game-accuracy of
the NEW outcome must be shown on the real game before the pin is touched. Not weakened; left red.

## Red 4 — walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage

Cause (pinned, not fixed): Bond dies at the door band exit — end health 0.0, 60 hits for 100.0;
drones #12/#13 (skin 0x500002f, weapon 0x16) each land 25 hits at d 5.0-7.4, where master's
fights were 4/2 hits at d ~10. 07000005 has no chat pair [D census — only 07000006 carries the
0x06000124/26 specials], so the only reach is the look_for_opponent gate itself: guards now
break off on held sight only, approach while unseen/unheld, and engage at close range. Walker
panic: crates/nf-view/tests/common/walker.rs:379 (the real-damage survival assert). The branch's
gate is the game's [M 0x177ed0:26-28]; the walk's survival pin was taken under the old model.
Open: either the walk leg needs re-deriving for the game's engagement timing (a test-route
change, not a pin weakening), or a PCSX2 measurement shows the game's guards there engage
earlier than the port's model — the live checks below decide which.

## Live checks for the main agent

1. rook06 (07000006 fresh boot, debugger on drone #3 = the switch-16 guard at (30.3, 41.1,
   −16.8), +0xC5 = mode): walk the test's route (truck → dock west end → west lane → well
   courtyard west → hedge gap → guardhouse court, before door #47). Expected under the branch:
   when Bond first rounds the hedge end and stands in the open court, d#3's +0x228 bit 4 set +
   +0x270 accumulating past DroneFunc_ReactionTime (read +0x270, +0x228 each frame); then
   either FirstSightState's prop-0x1b coin → a combat move state (mode stays 0) or the
   ChangeToAttackMode tail → +0xC5 = 0x19 (25) + +0x28 stationary. Record which the real game
   takes. If the game takes 0x19 like the branch, the rook06 pin gets corrected to the game
   with that evidence; if the game holds mode 0, the port's draw stream diverges and the chat
   pair's 0x36 path needs the draw census re-checked.
2. 07000005 fresh boot at the door band exit: watch drones #12/#13 (skin 0x500002f, weapon 0x16)
   — do they engage Bond at d ~5-7 with held sight, or from ~10 on first reaction-sight? Record
   +0x228 bits 2/4 and +0x270 for both while Bond walks the door band route. If the game
   engages close (bit 4 per-tick), the walk leg gets re-derived on the branch; if the game
   engages early, the port's los/seen_ticks model has a divergence to find.
3. guard_parity/guard_ai_other4 synthetic corrections: no live check needed — they cite
   [M 0x177ed0] directly.

## What ran

- guard_parity_live (branch binary, pre-correction): 19 passed, 1 failed (the red above).
- guard_ai_other4_live (branch binary, pre-correction): 3 passed, 1 failed.
- walkreal_07000005_live (branch binary): 16 passed, 1 failed (the red above).
- rook06_realpath_live (branch binary): 0 passed, 1 failed; master binary: 1 passed.
- guard_ai_other4_live (branch tree, corrected): 4 passed, 0 failed.
- guard_parity_live (branch tree, corrected): 20 passed, 0 failed.
- The full suite did NOT run: this sandbox has 2 cores and a 9.9 GB disk; the 4-crate test
  build alone (672 binaries) filled the disk to 100% twice (ld bus errors) and background jobs
  do not survive between tool calls here, so the fast-runner flow was not available. The
  corrected binaries' rebuild was forced but not re-run before the deadline (shared target dir
  had linked master's nf-game; branch sources are committed).

## Usage

not visible to me

---
## ntfy failed (2026-11-12T07:37Z)

All three fallbacks timed out from this sandbox: curl(28) Connection timed out after 20001 ms; python urllib hung past 90 s (first attempt SyntaxError on a non-ASCII dash); wget hung, no output. Full errors in the repo report.
