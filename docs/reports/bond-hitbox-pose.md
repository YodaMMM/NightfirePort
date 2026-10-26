# bond-hitbox-pose — report

Task: make Bond's 16 `HumanCollisionBones` hit boxes available on every SP level (the skin
0x05000007 table gap made `bond_boxes` return an empty list — an unhittable Bond), pose them by
Bond's state (crouch vs stand), and get `fix_bond_damage_live` and `anim_events_live` green on the
full suite.

## State found on the branch

`s/bond-hitbox-pose` already carried three commits from the earlier session (branch point
`origin/s/reload-and-bond-hitboxes` 14eef65e, then `origin/master` merged at 051c5240; master drift
0 — the final tree already contains current master):

- 6fbb7f4c — the player's skin on every level [M `Player_Init` 0x1910f8:94-108, the per-level hash
  switch; the loader merges the one missing record, session carriers first], plus the
  `fix_bond_damage_live` staging re-derived onto the torso box centre and the anim-events melee
  flight-tick fix.
- 49abbf38 — `Sim::bond_hit_boxes` + `bond_boxes_every_level_live` (21 story levels assert 16 boxes
  in the disc's `HumanCollisionBones` order [D 0x2c6be8]; 0e/0f/10 have no archive in FILES.BIN —
  documented skips, never a skin miss).
- 5da2b3aa — the crouch pose: `BOND_CROUCH_POSE` [D the SP crouch anim-set descriptor 0x2b91d8 →
  idle script 0x060009ff → seq 0x0400084e frame 1, through the port's own local_pose+world_pose],
  `bond_boxes(..., crouch)` picks it by `Player::crouched`, and the crouch toggle applies the
  settled root drop ROOT_HEIGHT − CROUCH_ROOT_HEIGHT = 0.4847 [L: standing rest 1.086, the truck's
  crouched 0.607].

This session's sandbox had been reset (repo/ISO/target all gone), so the work here was: full setup
again, then **verify everything by running it**, triage every suite failure, and finish.

## Verification (all runs this session, slim on-foot ISO, `--test-threads=1`)

- `fix_bond_damage_live` (nf-view): **7 passed, 0 failed** — including
  `a_round_at_the_standing_head_height_misses_crouched_and_hits_standing` (the task's crouch test)
  and the five formerly-failing damage tests.
- `anim_events_live` (nf-game): **4 passed, 0 failed, 1 ignored** (the ignored one is the
  pre-existing `play_firing_anim…` skip) — `melee_event_creates_the_close_combat_bullet` connects.
- `bond_boxes_every_level_live` (nf-view): **1 passed** — 16 boxes on every story level.
- `mp_perf_16_live` (nf-view): **2 passed** after the digest re-pin (below).

## The one regression this branch introduced, found and fixed

The full-suite chunks flagged `mp_perf_16_live::sixteen_participants_fixed_run_digest_is_pinned`
(digest changed). I bisected with a throwaway worktree over the branch history, running the same
test at each commit with the shared toolchain:

- passes at 051c5240 (merge base), 6fbb7f4c and 49abbf38;
- fails from 5da2b3aa (the crouch pose commit) on.

Cause, not a bug: the run's scripted humans press crouch (`i.crouch = ph % 307 == 0` in the test),
and the crouch toggle now applies the settled root drop (0.4847 [L]) — the port never lowered a
crouched root before, which was the documented deviation. The digest pin has seven prior re-pins
for deliberate sim changes; this is the same situation. Re-pinned to `0xb270_68a8_2582_fec6`
(kills 39 over the 3600 ticks; the two machines' per-tick hashes still agree every tick — that
assert passed in every run), with the reason in the test's re-pin comment. Commit abb44f33.

## Suite failures triaged (evidence per line)

- `fix_0c_pylon_live` (2 tests) and `probe_pylon_render` (1 test): panic
  `adapter: NotFound { … requested NOOP|VULKAN|GL|METAL|DX12… }` — wgpu finds no adapter in this
  GPU-less sandbox. Environment, not code.
- `fix_pickups_trace_live::with_dev_god…` ("drop 7 (weapon 0xe) is taken — the symptom is a drop
  that cannot be picked up") and `walkreal_07000014_live::the_east_duct_route…` (stairwell foot
  y 0.176 vs the −0.91 landing): both **fail identically at the merge base 051c5240** — I built
  the base tree in a throwaway worktree and ran both binaries there (same panics). Pre-existing on
  master/reload-base carryover, not this branch's; left for the tasks that own those pins.
- `mp_perf_16_live` — this branch's, re-pinned (above).

## Suite coverage on the final tree — honest numbers

`suite.py` (post-master-merge tree, 823 test binaries) was run in foreground chunks because this
sandbox kills background processes between tool calls and the tool caps a call at 600 s:
82 + 77 + 28 = **187 binaries ran**, zero failures beyond the five triaged lines above; a fourth
dual-stream round was reaped instantly by the sandbox and produced no output. The remaining ~636
binaries (mostly nf-view's live walk/probe corpus and nf-audio) are **not run this session** — that
is what's left: one uninterrupted `NF_ISO=… python3 tools/test/suite.py` run on this tree (the
branch already contains origin/master, so any checkout can start it directly).

The task's required suites — `fix_bond_damage_live`, `anim_events_live` — are green on the final
tree, run individually as above.

## Live checks for the main agent

- The crouch pose is validated against the idle table's rotations but the game-side crouched
  palette itself has no snapshot in `work/` (no crouched EE snapshot exists). [L] FRESH boot into
  any SP level, crouch, and read the composed head-box origin / `AnimObj+0xCC` (expected
  CROUCH_ROOT_HEIGHT 0.55 semantics — the truck ride measured the settled root 0.607): the port's
  crouched head box lands ~0.5 lower than standing; confirm the game's does too.
- `--bond x,y,z,yaw,pitch` render spots (visual, no GPU here): on 07000005 near the truck
  (crouch-toggle spot), crouched Bond should sit visibly lower (root 0.607 vs 1.086 standing);
  guards firing at crouched Bond at head height should miss.

## What's left

- The ~636 suite binaries listed above.
- The two pre-existing failures (`fix_pickups_trace_live`, `walkreal_07000014_live`) belong to
  their own tasks; baseline evidence in this report.

## Usage

not visible to me — this session's environment shows no token, credit, cost or context-window
figures anywhere I can read.

## ntfy failed

All three fallbacks failed (2026-10-26 ~14:20 UTC):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "bond-hitbox-pose: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "… urlopen(Request('https://ntfy.sh/nightfire-yoda-8k3x', …), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "bond-hitbox-pose: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, still hanging when killed at 60 s (timeout).

The branch, report and finish.sh claim (`OK done 50-bond-hitbox-pose`) were all in place; only the
push notification could not leave the sandbox.
