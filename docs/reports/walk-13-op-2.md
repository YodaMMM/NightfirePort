# s/walk-13-op-2: the 13 warm-chain real-damage walk — the ledger's lever taken, the block split at its measured spots, the frontier at the door drain (t6232), still red

Branch from origin/master 7562f1e70; origin/s/walk-13-op (round 1) merged in first (README +
the gadget census from master's newer side; the walkreal_07000013 ignore message keeps round 1's
ledger). Task: read the round-1 ledger, skip the measured-dead levers, take the next untried lever
("run the west-mid ANSWER BLOCK at the sweep's kill spot — the re-anchor end, the pre-emptive's own
pick law … one leg EARLIER than the west-mid end; the floor answers (shoot_loose, the charger law)
follow at the same spot"), judge each lever on its own real run.

## What was done

**The ledger's lever taken, then refined three times — each state judged on its own real run**
(NF_DMG_TRACE / NF_RF_TRACE captures, /tmp, not committed):

1. **The block moved to the re-anchor end AS LEDGERED** — death **t4200** (worse than round 1's
   t4337), INSIDE the block's mezzanine re-stand: the pick re-took the pre-emptive's own FAILED #5
   (nearest exposed dy>2; the balcony #4's box never opens from the re-anchor end) and re-stood
   into his fire, −38.5 across the two stands [L the lever-1 run]. The block's floor-loop skip-set
   law (lever-4) never reached its own mezzanine loop — the move exposed the defect.
2. **TAKEN — the same-end skip-set** (`failed_at_end`, the floor loop's own lever-4 law at END
   scope, shared by the pre-emptive + the block's loops): #5 never re-stood; the floor answer
   KILLED the re-fed #10 at the re-anchor end for the first time (the stand cost −21.0, all of it
   #5's covering fire — #10 dealt 0), but death **t4839** INSIDE the second floor stand: #6 (the
   re-fed wave rifle, hp 24, at the spawn-pocket sliver (4.9,30.2) d 6.3-7.3) took 8 pistol rounds
   with ZERO lands (the body-point aim's rounds stop on the pocket edge) and his first −7.0
   volley killed the 7.0-hp walk [L the lever-2 run].
3. **TAKEN — the FACE LAW + THE SPLIT.** The face law in every mezzanine pick (the pre-emptive's,
   the answer blocks': dy>2 targets with x < −16.29 are unstandable from the lobby floor — the
   ledger's own measures: every 0x1c pellet AND 0x19 rod stops on the face, hp 12 intact all run
   [L the MEZZ_DRY_END ledger + the walk-13-op runs]; round-1 lever-1's pre-emptive stand on #5
   cost −9.0 for nothing). The split: the mezzanine answer back at the WEST-MID END — the only
   spot #4's box opens (the 0.02/0.028 band converged on him at 21.4 u killed-true [L the nb-13-2
   ledger]) — and the re-anchor floor answers honour the charger law's own d ≤ 12 range (the
   lever-3 run's third stand took #9 at d 17.2 OUTSIDE that envelope, hp 24, clip 2 spare 6, and
   the statue cost the leg its window).

**The final run's shape** [L the lever-4 run]: 47.5 intact into the west-mid leg (the face law
took NO pre-emptive stand), the west-mid leg crossed, **#4 killed true at the west-mid end**
(health 3.5 after — the leg + his bursts + the in-flight volleys cost −28.0 in 8), the north-west
watch **killed #9**, the walk reached the security-room door drain and died **t6232** at
(−24.27,−368.18,38.17): the pool ran DRY (the #11 stands at clip 0 spare 0, "out of ammo at +61"
twice [L the same run]) and the re-fed #6 (hp 24, d 28 BEHIND Bond at (3.4,42.1)) fired the −3.5
kill during the drain's failed-stand + quiet waits. That is 1895 ticks past round 1's frontier.

**Measured, not repeated:** the re-anchor floor answers that killed #10 and #6 in the lever-3 run
(−16.0 / 0.0) found NOTHING in the lever-4 state — the pick's own debug dump (temporary, removed)
read #10 at dxz 5.3 and #6 at 8.2 with exposure **false** in the split tree's state, while the
pre-split lever-3 state had both exposed and both killed [L both runs]. The mezzanine loop's
absence from the re-anchor block is the only code delta; the exposure at the pick is
state-sensitive in a way this session could not pin down (the traces are byte-identical through
the sweep's kill print in both runs).

## How it was verified

- The ignored d3 walk alone, four runs (the ledger's lever + three refinements), each judged on
  its own run (the death tick + the full damage/return-fire breakdown above).
- The WHOLE walkreal_07000013_live binary on the final tree: **4 passed, 0 failed, 1 ignored**
  (the plain walk, the Agent sister, the Operative sister and the route-flag thread test — the
  sisters stay byte-identical: every lever lives inside `real_damage && difficulty == 3` gates,
  the d2/d1 paths are untouched).
- interactables_07000013_live (final tree): **10 passed, 0 failed**.
- `git diff origin/master --stat`: only crates/nf-view/tests/walkreal_07000013_live.rs (+ docs),
  so no other binary's behaviour can differ from master; no sim, draw, RNG, damage, health or
  guard-code change; no top-ups; real input only.

## Suite (the final tree, the resumable chunk runner)

The background suite dies between tool calls in this sandbox, so tools/test/suite.py ran through
scripts/suite_chunked.py (the runner's own per-binary logic, one JSONL state file) on the final
tree (f4b1334aa, the refreshed ledger included). **The full sweep: 414 test binaries, 1759
passed, 7 failed, 72 ignored.** The 7 failures are the branch-point allowed set exactly:
fix_0c_pylon_live (2) + probe_pylon_render (1) — the wgpu tests need a GPU this sandbox lacks,
driving_ai_follow (1), and disc_golden (3) on the slim ISO. No FAILED/TIMEOUT line belongs to
this task's change; walkreal_07000013_live ran inside the sweep green (4 passed, 1 ignored).

## What's left

- The walk is still red (the ledger above; the refreshed ledger + the next lever are in the test's
  ignore message): the door phase needs a POOL (the north-west fetch after the watch drain is newly
  available — #4 dies at the west-mid end now, so the margin-6 measured-dead blocker is gone), and
  the re-anchor floor answers' exposure sensitivity wants the live check below.

## Live checks for the main agent

1. (carried from round 1) The mezzanine balustrade face x −16.29: from the re-anchor spot
   (3.29,−368.39,31.77), does a FRESH-BOOT player round at the mezzanine pistol guard #5
   (−17.9,−362.4,45.4) land on his box, or stop on the face? Expected per the port: stopped.
   If a real player CAN kill him from the lobby floor, the port's bullet trace at that face is
   wrong — that would be the root fix, not a route lever.
2. NEW: the re-fed wave guards' exposure at the re-anchor end is state-sensitive: in the
   pre-split state #10 (dxz 5.3) and #6 (8.2) read exposed from the re-anchor spot and both die
   to the loose stands for −16.0/0.0; in the split state (the only delta: the mezzanine loop
   moved past the leg) the SAME two drones read exposure false at the same spot [L both runs].
   A fresh-boot probe (can a player hit a lobby guard standing d ~5-8 east of the re-anchor
   spot?) would say which reading is the port's truth.

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed in this
environment; nothing to copy).

## ntfy failed

All three fallbacks failed. Exact outputs:

- curl: `curl: (28) Connection timed out after 20002 milliseconds` (exit 28).
- python urllib: the `urlopen(..., timeout=20)` call hung; the wrapper `timeout 25` killed it
  (exit 124), no response printed.
- wget: `wget -q -O- --post-data ... https://ntfy.sh/nightfire-yoda-8k3x` hung; the wrapper
  `timeout 25` killed it (exit 124), no output.
