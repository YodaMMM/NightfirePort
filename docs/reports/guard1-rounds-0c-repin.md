# guard1-rounds-0c-repin — report

Branch `s/guard1-rounds-0c-repin` = `origin/s/pretrans-arms-0c-11` 7a89b686b (origin/master
784ba2d99 was already an ancestor at branch time; no newer master commits existed all session).
Task: the 0700000c first guard's rounds test green with the game's guard sight. All changes are
in `crates/nf-view/tests/guard1_rounds_0700000c_live.rs` and docs — **no `crates/*/src/` code
changed**, no shared-helper (`common/`) change (the diff on the sim sources and on
`crates/nf-view/tests/common/` is empty), so the every-walkreal rule is not triggered; the
required binaries ran whole anyway.

## What moved — found by running the test on master before and after the sight merge, as asked

`guard1_rounds_at_the_ring_real_health_fight` (the real-health pin), run single, `--nocapture`,
deterministic (bit-exact sim, each run repeatable):

- **823ff190b^ (before the sight merge): GREEN** — Bond ends at 11.2 hp, 37 hits for 88.8
  (the far dealers #20 32.0 @61.5 u, #19 16.0 @72.2 u, #15 16.0, plus the small chips) [L the
  run].
- **823ff190b (the sight merge itself): GREEN** [L the run].
- **origin/master tip 784ba2d99: GREEN** — Bond ends at 75.0 hp, 11 hits for 25.0 (#1's single
  16.0 at d 3.6 the largest) [L the run].
- **This branch: RED** — Bond dies at tick 9708, 14 hits for 100.0: #19 48.0 (3 × 16.0 at
  d 9.6), #15 16.0 (@31.8 u), #1 27.6 (1 hit at d 2.4 — inside the 3.0-u band where every
  round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115] and TooClose doubles the damage
  [M Drone_ModBulletDamage 0x139f50]) [L the run].

**The task's premise — a casualty of the guard sight landing — is disproven**: master is green
before AND after 823ff190b. The actual mover is this branch's own sim delta vs master: the
[M]-backed root-motion-zero landing (`Drone+0x38`, commits 3c6a8e503 + 0e27cc464 — drone.rs
+121/−26, the only sim diff; the test file and the shared helpers it uses are byte-identical
master↔branch, verified by diff). Fight-level proof of the same attribution: master tip's
deck-south stand kills deck sniper #19 (`killed true` at 5332); the identical fight code on
this branch fails (`killed false` at 5426) and #19 lands three 16.0s [L the NF_RF_TRACE runs
both sides]. Per the task's decision rule this is a **walk timing** → re-pin gated to
0700000c. (The test file is level-local, so the re-pin is inherently 0700000c-gated.)

## The re-pin — the ring corner, with the measured numbers

The red run's failure chain [L the NF_RF_TRACE run]: the aimed stands DO land (mid-crossing
killed #18; ladder-top killed #15 — 2 ticks after #15's own beat fired, so his 16.0 stands;
the at-the-door shot at #19 missed mid-swing and his 48.0 stands). The killer is guard #1:
**the ring choreography walked the north-west-corner leg right past him** — #1 patrols at
(10.85, 12.09, −204.21) when the diagonal-2 leg ends (9.3 u, gun rays BLOCKED [L the probe]),
the rays open only at the corner (bond (12.25,−201.90), #1 (11.58,−203.36) — 1.6 u [L the
probe]) — and the ring-corner stand's aim-raise was still starting when #1's beat fired
(frame 9710, the 27.6 doubled-band kill; the stand's t0 print is the trace's last line).

The fix (test-file-local): **split the corner leg at z −200** — `the ring corner approach`
(Wp (12.0, 11.1, −200.0) r 1.0) — where the probe measured the rays OPEN (bond
(12.75,12.14,−199.40), #1 (11.69,12.09,−202.94), d 3.70, exposed true [L]) — and run the
`ring-corner` sweep there. The stand's round kills #1 at d 3.7 (`impacts DRONE #1 part 4` [L
the green run]) — outside the doubled band, ~2 ticks ahead of the beat that used to land at
9710, and #1 dies without firing. The late `ring-corner` call stays for late acquirers (it is
a no-op on this tree — #1 is already dead).

**Result [L the runs on this tree]**: `guard1_rounds_at_the_ring_real_health_fight` GREEN —
Bond ends at **27.6 hp** (13 hits for 72.4: the #19 48.0 and #15 16.0 stand as the measured
cost of the root-zero-era fights; no doubled-band hit), the walk completes the hold window,
and the reload-rhythm asserts pass (guard #1's clip bookkeeping: he dies before his first
reload — the top-up sister still pins the 5-round clip law on the same tree). Both tests of
`guard1_rounds_0700000c_live` green.

## What ran (foreground chunks — the fast runner cannot live in this sandbox)

- `guard1_rounds_0700000c_live` whole: **2 passed, 0 failed** (21.9 s).
- `walkreal_0700000c_live` whole: **2 passed, 0 failed, 2 ignored** (the pre-existing
  real-damage/Agent sister ledgers, untouched) — 20.6 s.
- `nf-game` whole (all targets): **0 failed** — the lib 782 passed, 4 ignored; every
  integration binary ok (118 test-result lines, all ok).
- Suite chunks: nf-formats + nf-audio **142 passed, 3 failed — exactly the 3 `disc_golden`
  tests the slim ISO allows**; nf-view --lib **138 passed, 0 failed**.
- The fast runner (`tools/test/suite.py`) was started in the background per the task and died
  within seconds (nohup, 0 survivors — the same wall the -0c-11/-5/12-2 reports recorded; this
  attempt also shows it needs `cargo` on PATH, which the nohup environment did not carry). The
  full four-crate `--test-threads=1` total line therefore **did not fit**; the chunks above are
  the same binaries on the same tree.

## What's left

- Nothing on this task's own bar: the pin is green with the sight-era fight laws, and the
  before/after-sight-merge experiment the task asked for is recorded above.
- Carry-over ledgers stand untouched: the two ignored `walkreal_0700000c_live` sisters (the
  real-damage/Agent ledgers from their own tasks) and the full-suite-on-a-background-capable-
  sandbox wish.
- A [S] observation for a future round: on master tip #1 stops at d 3.6 at the corner; on this
  tree he closed to 2.4 (the root-zero-era close-out). Whether the GAME's own #1 stops at 3.6
  or 2.4 there is unverified — a live check below settles it; the re-pin does not depend on it
  (the walk kills him at 3.7 either way).

## Live checks for the main agent

1. At Operative, walk the patrol ring to the north-west corner ((12.3, 12.1, −201.9)) WITHOUT
   shooting and watch guard #1 (placement #3, skin 0x5000093): what distance does he hold when
   he opens fire (the port's two trees say 3.6 and 2.4)? gdbwatch on his +0x38 root-zero writes
   during the close (expected per [M 0x166328:118-122]: the zero sets while the aiming class
   plays — whether the game's own close-out stops at 3+ u or walks into the 3.0 band decides
   whether the 27.6 kill shot is game-true or a port artefact).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy

(pending — filled below after the finish steps run)
