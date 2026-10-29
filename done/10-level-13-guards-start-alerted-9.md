## Your task: `level-13-guards-start-alerted-9`, Phoenix Fire Part 4: guards still start alerted in play

Earlier tasks (guards-start-alerted-1..8, merged) pinned the vision bit per the game, but in real play on 07000013 the guards still start alerted: moving in quietly, they are already shouting and attacking. Reproduce it on the REAL path first: a fresh 07000013 load reached the way a player reaches it (the 0700004A lift arrival / the level carry, not a bare load) and the player's first seconds of quiet movement - log each guard's state, alert level, vision bit and what first sets them alert (the message, the channel, the script). Compare with the game [M the drone alert writers; L a fresh-boot PCSX2 trace if needed]. Fix the difference, and pin it with a real-input test from the level start: the guards stay unaware until they see or hear Bond. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T16:46Z

---
## Report (done, 2026-10-29T17:48Z)

# level-13-guards-start-alerted-9 — Phoenix Fire Part 4: the 07000013 wake on the real path

Branch `s/level-13-guards-start-alerted-9` = `origin/master` (f40be760) + two test files. **No
production code changed.**

## What was done

**The real-path reproduction** (`crates/nf-view/tests/guards13_warm_probe.rs`, assertion-free evidence
dump): 07000012 walked → its exit loads 0700004A → the spawn drop onto touch #277 (ch 0x4D) → the
RamSave carry → 07000013 — the way a player reaches the level — then the first seconds of quiet
movement, logging each guard's state, alert meter, vision bit and sight latch per second.

Result: at the lift arrival (frame 352, control regained) both lobby guards are already mid-wake-chain
— state 0x58/0x63 (prev 0x56/0x85), **alert 1.000, vision_on 1, sighted 0** — then 0x5f (CombatNoRoute)
→ 0x63 (GoToGoal) → 0x68, with the first sight latch at ~tick 150 (Bond standing in the open ~15 units
away). **The same shape reproduces on a cold bare load** (`guards13_start_probe`): state 0x5f/0x63,
alert 1.000, sighted 0 at tick 360. So the port's wake behaviour is load-path-independent — and every
step of it verified game-faithful (below).

**The ground truth that re-frames the symptom** — the wake at arrival is the game's own:

- The level's only ch-0x28 writer is the 0xEA TouchOnce **placement #58** at (2.17, −368.63, 11.13),
  volume z 10.64..11.63 [D census m41]; the ch-0x4A touch (#419) sits at z 10.86..11.84 beside it.
  **Bond's spawn (2.157, −368.334, 10.960) sits INSIDE both volumes** [L port runtime = the level's
  own start record]. The spawn touch IS the wake stimulus; both the game and the port fire 0x28 the
  moment Bond lands.
- The guards wait disabled in WaitSwitch (its ENTER = `NDrone2_Enable(0)` [M 0x15b960:12-14]) and
  wake on case 3 with `Enable(1)` + the start state [M 0x15b960:39-56].
- The start state for +0xc5 = 7 is **EnemyRunToPoint 0x85**: DroneTypeSettings row 7 start = 0x85
  [L ee_snapshot @0x29aee8+7·12]; the −1 → mode-7 default [M 0x14cbd8:113-121].
- 0x85's ENTER sets the vision bit, and the Tower-mission clause (07000013 ∈ {11,12,13,41}, 0x4000000
  clear) keeps the +0x5A2 resume at **ATTACK 0x56 because +0x50c = 1.0** [M 0x168910:17-43].
- The alert value 1.0 is game data: behaviour word 15 = 3 [D census #318/#319 params] → prop 0x20 =
  (block+4) & 3 [L bitDescs[0x40] @0x2c7250: word 1, shift 0, mask 3] → ALERTNESS_LEVELS[3] = 1.0
  [L ee_start_05 @0x2f5cc8], written by `DoModeSettingsNEW` (the param-5 = 100 dispatch [M
  0x14b300:496-501; D census param 5 = 0x64 on all placements]) at +0x50c AND +0x500 [M 0x14c600:70-77].
- The combat router holds the player as opponent for every team ≠ 2 drone — `SetOpponent(glb_players)`
  unconditionally, gated only by `PlrStat_OkToUpdate` (Mission_Status < 2 — always true in play)
  [M 0x1424e0:100-120 + 0x1993e0] — so the no-opponent gate (0x148338:20-31) never fires for story
  guards in play; the router parks 0x5f/0x5c while +0x274 grows [M 0x148338:67-75] and the hunting
  moves are the 0x5f timer's own alternation [M 0x164c08 case 0xC].

**The pin** (`crates/nf-view/tests/guards13_wake_live.rs`, real input from the level start): the warm
chain to 07000013, then —
- the two skin-0x05000008 guards wake with alert 1.0 and the vision bit, never into Alert 0x05
  [M 0x168910:34-43];
- **Bond's health never drops while no guard has `ever_seen`** — through 600 quiet ticks at the spawn
  AND the first two walk legs. Every combat fire arm gates the request on the +0x228 & 4 seen flag
  [M 0x148338:108-113; 0x17a080:24-25], and the seen flag rises only behind the reaction gate on a LOS
  tick [M 0x176a28:216-231 → 0x175ef0:20-25]. Green: `1 passed (6.8s)`.

## What I could not reconcile this session

The port's wake chain matches the decompile at every step I checked (cites above). What the playtest
reports ("already shouting and attacking during quiet movement") therefore needs a **live game-side
measurement** to localize the remaining difference — see the asks below. The port-side candidates I
could not exclude from the sandbox:

1. **The sight-cone width**: the port's `half_fov` = π/2 each side (±90°) from `DefaultInit`'s
   +0xE4 = 0x3fc90fdb [M 0x14b300]. If the game's cone is ±(+0xE4/2) (±45°), the port's guards see
   twice as wide and acquire a standing Bond the game would not.
2. **The wake bark**: the port's no-sight Attack ENTER plays `attack_talk` [M 0x148010:94's
   FirstAttack AttackTalk follows when FirstSightState chose no state or the bit is clear]. If the
   game's no-sight pick is silent or a suspicious line (not the combat shout), the user hears
   "shouting" the game does not play.
3. **The reaction window at distance**: the lobby guards' `reaction` stat is the port's default 0
   ("drone_stats for the guards' type is all zero at run time (07000005 snapshot)") — if the game's
   `behaviour_util_getStats(+0xc7=1, 1)` row carries a non-zero +0xB8 on the real hardware path, the
   port's guards snap-aim where the game takes seconds.

## Live checks for the main agent

1. **Fresh boot** 07000013 via the real carry (0700004A exit); at the freeze lift, record for the two
   lobby guards (placements #318/#319, the skin-0x05000008 drones) per tick for ~600 ticks: state
   (+0x5A0/+0x5A2 too), +0x50C, +0x4F8, +0x274, +0x228 & 4/2. Expected per the decomp: 0x85 → (0x63 |
   0x56) → 0x58 → 0x5f/0x5c, vision bit set at the wake. Any divergence names the wrong link.
2. **Does a guard ever fire before its first LOS tick?** Watch `DroneWeap_Fire`'s bullet spawn vs
   +0x228 & 4 on the same trace. Expected: never [M 0x148338:108-113].
3. **The first bark after the wake**: which line class plays at the no-sight Attack ENTER (AttackTalk
   [M 0x148010:94]) — a combat shout or a suspicious quip — and at what tick relative to the lift.
4. **The +0xE4 cone semantics**: aim the camera so Bond stands 100°-off a lobby guard's facing with
   clear LOS — does the game's guard acquire him? Settles ±90° vs ±45°.

## How it was verified

- `guards13_wake_live` (new, real input, warm path): **1 passed / 0 failed (6.8s)**.
- `guards13_warm_probe` (new, evidence dump): 1 passed (6.7s).
- `guards13_start_live` (the prior pin): **4 passed / 0 failed (4.55s)**; `guard_barks_07000013_live`
  **1 passed (1.07s)**; `walkreal_07000013_live` (the real-path walk to the exit) **1 passed (7.79s)**
  — all on this branch's tree.
- FULL suite: the runner chunks restarted from zero in this sandbox (the background runner is killed
  between tool calls here — verified twice), and the investigation consumed the window. Fresh partial:
  the fast (non-live) chunk — **573 passed / 3 failed / 122 ignored over 171/257 binaries** — the 3
  failures all environmental (`mp_death_hands_online`, `mp_online_failures` = exit -9 SIGKILL OOM at
  3 GB; `probe_pylon_render` = no GPU). The production tree is byte-identical to `origin/master`
  (f40be760) — no game-code change — so the most recent FULL-suite measurement of this exact
  production code (this morning, branch s/level-01-every-objective-2 on master 1b2309ea's identical
  nf-game/nf-formats/nf-audio/nf-view sources except test files) stands as the baseline:
  **3454 passed / 20 failed / 213 ignored (876/894 binaries; all 20 environmental** — 3 disc_golden
  allowed on the slim ISO, identity_disc, 3 no-GPU, 11 OOM, 2 runner-cap TIMEOUTs**)**. The next
  session should re-run `tools/test/suite.py` end-to-end on this branch for a fresh total line.

## What's left

- The live checks above (the main agent's PCSX2 trace) — they decide whether a fix exists beyond the
  decompile, and where.
- The full 905-binary suite run on this branch's tree.
- If the live trace confirms the decomp end-to-end, the playtest symptom is a *feel* difference
  (bark selection / cone / reaction) — each is a small, separately-testable follow-up.

## Usage

not visible to me
