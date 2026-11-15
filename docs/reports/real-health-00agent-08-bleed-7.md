# real-health-00agent-08-bleed-7 — 07000008's 00 Agent walk: the tail sprint-dodge lever taken and measured; the homing missiles stand

Task: take the ledger's next lever toward surviving the Hind fight's tail (~2250 more ticks /
~20-30 more health needed after the measured five-rocket roll: the Hind 489.6 of 598, Bond
dies t33393 with 8.95 left, the next good launch after rocket 5 at fight t8391), judge it on
real runs, keep every block gated to difficulty 3, and run the whole
`walkreal_07000008_live` binary and `interactables_07000008_live`. Branch
`s/real-health-00agent-08-bleed-7`, base origin/master 854334126 (the 07000008 ledger commit
c65f36857 on top), with origin/s/real-health-00agent-08-bleed-6 merged for its report (its
walk file was superseded by master's newer iterations; the conflict resolved to master's).

## What's done

The lever is TAKEN and measured on eight real runs (each the full mission walk on real input
from the load, no top-ups, no health or damage writes), and the walk is still red — the
refreshed ledger and the next levers are in the ignore message on
`walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent`
(update 8). All changes are in `crates/nf-view/tests/walkreal_07000008_live.rs`, all d3-gated;
the d1/d2 sisters and the ally tests stayed byte-exact through every iteration (whole binary
5 passed 1 ignored on every shipped tree).

Shipped d3-gated blocks (all inside the Hind fight loop):

1. **The sprint law, tail-gated** — the dodge episode stands Bond up (the crouch press edge,
   one tick [M Player_SSCrouch 0x1924c0:84-105, the Toggle default Input_Init 0x17edd8]),
   strafes standing (≈0.077 u/tick vs the crouched+aiming 0.011 [L run-bleed6-1: 0.31 u in
   28 ticks]), and re-crouches the tick the episode ends, skipped while a launch tick holds
   the gate. The stand-up is gated to `rockets >= 5`: every sprint episode before rocket 5
   perturbed the deterministic chain and re-rolled the overhead pass's frame-graze lottery
   (below), so the measured five-rocket roll is kept BYTE-EXACT — runs bleed7-5/6/7/8 all
   reproduce the same launch ticks t674/946/1142/5223/5558, the same 489.6 damage, and the
   same chip frames 30591-31103.
2. **The launch holds while the re-crouch is owed** — `ready` requires `!sprint_standing` on
   d3: a launch on the episode-end tick spawns the round from the standing eye (measured:
   the run-bleed7-1 variant without it died at t28214 to a −52.00 detonation ~1 u from the
   round's spawn).
3. **The dodge side is AWAY from the threatening round's flight line, tail-gated** — the
   perpendicular that grows the miss distance (the anchor-ward pick stays for the
   pre-rocket-5 episodes, byte-exact).

## How it was verified (the eight runs)

| run | tree | death t | rockets | notes |
| --- | --- | --- | --- | --- |
| baseline | master 854334126 | 33393 | 5 (489.6) | the ledger's own numbers reproduced exactly on this sandbox (34 hits for 100.0); the killing −8.95 at frame 33394 after the 2291-frame lull |
| bleed7-1 | full-fight sprint law | 28214 | 1 (101.0) | rocket 2's overhead pass re-rolled: the release bearing grazed the shot-out east pane top frame 1.05 u out for −52.00 (the baseline's pull cleared to the 4.66 u ceiling, −25.05 — the accepted bill) |
| bleed7-3 | + the launch holds while the re-crouch is owed | 28214 | 1 | same death — the launch was crouched; the graze is the release-frame bearing, not the stance |
| bleed7-4 | + the l > 20 stand-up gate | 28214 | 1 | the pre-rocket-2 episodes ran at l > 20 anyway; the perturbation starts at the fight's first dodge |
| bleed7-5 | rockets ≥ 5 stand-up + the effective-aim slit guard | 33147 | 4 (388.6) | the guard BLOCKED rocket 5's own window — taken off again |
| bleed7-6 | rockets ≥ 5 stand-up, anchor-ward side | 33393 | 5 (489.6) | the five rockets byte-exact; the killing −8.95 landed at the SAME frame 33394 |
| bleed7-7 | + the away-from-the-line side pick | 33393 | 5 (489.6) | the same −8.95 at the same frame from a spot 0.9 u away — the rounds HOME |
| bleed7-8 | + the 40 u threat window | 33393 | 5 (489.6) | the same death — the displacement class is exhausted, not the window |

The conclusion: bleed-6's binding constraint (2) is confirmed with the sustained standing
sprint — the machine's tail missiles home on Bond (the chase's per-tick steer
[M Bullet_update 0x12e7b0:177-244]) and both dodge sides, 30 u and 40 u threat windows, land
the killing −8.95 at the same frame. The machine's volley cadence is on-rails (the chip and
blast frames identical across all tail variants), so dodge displacement cannot move the
incoming schedule either.

Binary results on the final tree (commit before the report):
- `walkreal_07000008_live` whole: **5 passed, 0 failed, 1 ignored** (the ignored one is the red
  00agent walk carrying the ledger) — 115.6 s.
- `interactables_07000008_live`: **15 passed, 0 failed** — 16.2 s.

## What's left

The five-rocket roll is preserved byte-exactly and the fight still loses by one missile blast.
The tail needs ~20-30 more health that the displacement class cannot supply. The next levers
(in the ignore message, update 8) come from OUTSIDE the fight's dodge class:
1. **The east deck head fight's 40.0** (t684-1114, four 0xa guards at 4-10 u) — the pre-fight
   bill's biggest item; a faster kill there re-rolls the ride and the fight, judge on full runs.
2. **The gondola ride's 12.3 standing armour** — the ledger's crouched-ride note measured the
   fight re-rolled worse, but that was on the pre-update-7 tree; re-measure on the current
   five-rocket tree before retiring it.
Live checks for the main agent: pin in PCSX2 (fresh boot) whether the Hind's tail missiles'
homing steer rate at 10-30 u really outruns a sustained 0.077 u/tick perpendicular sprint
(the steer clamp constant in Bullet_update 0x12e7b0:177-244's chase), and the cannon rounds'
impact band inside the cabin at the anchor (the x rows the −2.0 chips land in, x 111-113.5).

## Suite note

The fast suite (`tools/test/suite.py`) could not run in this sandbox: the disk is 9.9 GB and
nf-view alone compiles 1113 test binaries (~16 MB each) — the target dir filled the disk
(9.3 GB used, the linker died with `ld terminated with signal 7 [Bus error]`) at ~340
binaries; detached processes are also killed between tool calls (three starts died, as in
bleed-6). The complete affected-test evidence is above: the diff touches ONLY
`walkreal_07000008_live.rs` (no shared helpers under `tests/common/`, no game code, no other
crate), so the tests that can be affected by it are exactly the two binaries run green on the
final tree. The next session should run `tools/test/suite.py` once on the merged tree as
usual.

Scratch logs (run-bleed7-*.log as /tmp/*.log, whole8/interact8, suite.out) are untracked,
never committed.

## Usage

Not visible to me: this session exposes no token, credit or context-window figures to the
worker (no usage metadata is available in the sandbox).
