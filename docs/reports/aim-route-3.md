# aim-route-3 — the hold-counting recipe built and measured; the 48-burst gate survives both challengers; the chain arithmetic pinned; 08/0c/16 untouched

Branch `s/aim-route-3` (from `origin/s/aim-route-finish` 85ce433f4, merged `origin/master`
a6665d115, clean). One file changed: `crates/nf-view/tests/walkreal_0700001b_live.rs`
(test-side only). The [L] 08 pins and the 0x30000000 fix are untouched.

## 1. The recipe (aim-route-finish §2) — built

In `shoot_consoles`:

- **The answered-hold counter.** `answered[pi]` counts the script player's `SP_UnPause`
  activations — the damage tail's one-tick flag [M 0x1dc2f8-0x1dc318, sampled right after each
  tick]. One activation = one 20-damage block consumed = one of the console UI script's three
  holds answered [D probe1b_scripts: event 3 on streams 1/2/3]. Measured: the count reads
  exactly 3 per console before every latch (two extra activations land 2-4 ticks AFTER each
  latch — the ended script's own out-channel self-unpause arm [M nis.rs state-2 arm] drains
  while the channel is on; harmless, they never touch the pre-fly).
- **The pre-fly.** When the current console's count reaches 3 (its script is playing to its own
  t682 channel write — nothing left to shoot), the no-hold float target switches to the NEXT
  un-latched console's station, both two-phase clamps applied.
- **The deadline pin.** Every latch log carries
  `deadline = missile_start[k−1] + P0_LEN[k−1] − 1`, where `missile_start` is the measured
  frame `MissileDeploy` goes 0→1 [M 0x1aa218] and P0_LEN is the disc decode [D space_missile.rs].
  **S is measured, not estimated: ch19 (the launch order) rises at frame 2747 and all eight
  missiles leave state 0 the same tick** — the shipped fast state-0 countdown (512.0/tick)
  expires every seed [D 105/40/20/0/60/105/105/60] in one gated tick. The walk now prints the
  margin per console.

## 2. The measurement — three whole runs of `walk_arrival_to_drakes_defeat`

Deadlines with S = 2747: k3/k4/k5 → 4546, k6/k8 → 5146, k7/k1/k2 → 6046.

| run | levers | latches (console@frame, margin) | MissileDeploy |
| --- | --- | --- | --- |
| 1 | recipe, the 48-burst kept | 3@4017 (−529), 4@4425 (−121), 5@4629 (+83), 6@5043 (−103), 8@5505 (+359), 7@5961 (−85), 1@6388 (+342), 2@6531 (+485) | [3,3,2,2,3,2,2,3] |
| 2 | recipe + the steady cadence (clip ≥ 16 replaces the 48-burst) + the interleaved fire pick | 3@4017 (−529), 4@4299 (−247), 5@4737 (+191), 6@5169 (+23), 8@5919 (+773), 7@6081 (+35), 2@6377 (+331), 1@6586 (+540) | [3,3,2,2,3,3,3,3] |
| 3 | recipe + the steady cadence only (the pick reverted) | 3@4017, 4@4299, 5@4737, 6@5169, 8@5919, 7@6081, 1@6544, 2@6676 | [3,3,2,2,3,3,3,3] |

- **The pre-fly fires on all eight consoles and gains ~6 ticks total** (run 1 vs the
  predecessor's best run: 8: 5511→5505, 1: 6370→6388, 2: 6513→6531). The report's hypothesis
  was wrong in one detail: the flights were ALREADY overlapping the end-plays — the old
  anticipation arm's failure was not the missing departure trigger; the departure trigger only
  moves the start by the latch-to-3rd-answer gap, which measures **2 ticks**.
- **The steady cadence is a measured regression** (runs 2/3): k4 improves −126, but k5 +108,
  k6 +126, k8 +414, and **k7 arms too**. Mechanism: the 48-burst threshold RATIONS the clip —
  the early consoles (3/4, with 500+ ticks of slack) burn only their 3-tap bursts and the
  threshold keeps the recharge running toward the late chains; the ≥16 cadence lets the early
  chains fire at every 12-tick alignment, the phase's tap supply (24 taps × 12 rounds = 288
  rounds vs the +1-per-4-frames recharge [M 0x193eb8-0x19405c] = one tap per 48 ticks
  sustained) drains at the slack end and the dry stretches land on the deadline-critical
  chains. REVERTED — the 48-burst gate survives with both runs' numbers in the comment.
- **The interleaved fire pick measured DEAD, not harmful**: run 2 is bit-identical to run 3
  through k7. The eight UI scripts DO hold in parallel from ~S+1009, but the level look pins a
  tap to the eye's own height band (a level shot cannot reach a sphere more than ~r above or
  below the eye), so the only opportunistically-shootable targets are the height twins
  (3/4, 5/6, 7/8, 1/2) — and those are exactly the pairs the serial flow already answers in
  ~36 ticks. The pick's one visible effect was the late-phase order (2 before 1 in run 2).

## 3. What else the chain needs (run 1's decomposition)

| cost | ticks | where |
| --- | --- | --- |
| same-height twin taps | ~36 | k3→k4, k8→k7 (tap spacing + re-aim) |
| height re-floats between twins | ~78 | k4→k5 (the eye −3.5 → −5.8) |
| clip dry gaps | 289 + 360 | k5→k6, k7 h1→h2 (the 48-burst's re-arm) |
| the real flights | 336 + 353 | 6→8 (cross_high), 7→1 (climb-out) |
| each chain's own plays | ~160 | hold₁→hold₂→hold₃→t682, measured 59-96 per hold gap |

- The measured span (first hold ~3757 → k2 6531) is 2774 ticks vs a ~1500-1800 floor
  (the tap supply 1152 + the flights ~700, the chains riding under both). The dry gaps are the
  slack — but both direct attacks on them moved the lateness INTO the late consoles.
- **The next levers, in data order**: (a) the fight-exit clip top-up — the walk enters the
  phase with ~64 rounds and idles ~1000 ticks before the first hold anyway (the UI scripts
  only hold from ~S+1009); topping to 100 in that idle window is free and adds exactly the
  buffer the dry gaps ask for; (b) a deadline-rationed burst gate — keep 48 while the primary's
  margin is positive, relax toward 16-24 only when it goes negative; (c) the mid-bank stance —
  8's screen cannot be shot from the 6→8 crossing path (the bank blocks the line,
  `sphere_shot_ok` fails through the crossing); a stance that sees BOTH 6's and 8's screens
  would let 8's chain start inside 6's end-play, which is the only remaining ~300-tick block.
- The report's S+2550 estimate for 8 is confirmed shape-wise: 8's latch = 6's 3rd answer +
  2 + the flight (336) + the chain (~160) ≈ the measured 5505; the deadline 5146 needs ~360
  back, and only (a)/(c) above can fund it — the departure side is already optimal.

## 4. Not done this window: 08 x1, 0c x1, 16 x1

The 1b item consumed the window (three ~300 s runs + two challenger builds). No code changed
for 08/0c/16; the aim-route-finish §3 classification stands: **08** the station legs' hurt
volume (death at d 84.8-92.3 chip exposure + a 78.0 world hit); **0c** the far-sniper drain
(the finishers at d 61-90 left at full hp) — with this task's hint recorded for the next
window: check what Bond HOLDS before any stand, because pistol-class weapons carry range 50
and rifles 150+ [weapon_data +0x1c, the driver's decode — verify the offset against the decomp
before building on it]; a pistol cannot answer the d 61-90 finishers, so the stand is the
wrong lever if the held weapon is the pistol; **16** the point-blank melee swarm (three
swimmers in the 0.8-1.3 always-hit band).

## Verification

- `walkreal_0700001b_live` `walk_arrival_to_drakes_defeat` (+ `_on_real_health` shares it):
  three whole measured runs, table above; both still red at the MissileDeploy gate
  (0 passed / 2 failed / 2 ignored per run — the gate assert at the disarm block).
- The shipped state = HEAD (the recipe + the instrumentation, the 48-burst untouched);
  builds clean. The challengers' code is NOT in the tree — their numbers live in the branch
  history (runs 2/3) and in this report.
- The full suite did NOT run this window (time): the branch changes one test file, no
  `crates/*/src/`, nothing under `crates/nf-view/tests/common/`; `shoot_consoles` is called
  only by the two drakes walks, which ran 3× whole. The 1b binary's other 15 tests do not
  touch the edited fn.
- The aim binaries (interactables/fusebox/rocket/solar) and walkreal 08 / 0c / 14 / 16 did not
  run: nothing they read moved on this branch.

## What's left (next window)

1. 08 x1 / 0c x1 / 16 x1 per §4, 0c's weapon check first.
2. 1b: the fight-exit clip top-up and/or the deadline-rationed burst gate; then the mid-bank
   dual stance if the budget still shorts 8; then the real-health drakes walks green.
3. The sp-panes re-pacing note: the measured S = 2747 holds ONLY for the shipped fast
   state-0 countdown; the disc-accurate 1/60 step (REC_FRAME_RATE) moves every deadline by
   seed·60 — re-pin the margins when that lands.

## Live checks for the main agent

1. (carried from aim-route-finish) The 14 kind-2 door at (32.0, 2.1, 35.8): stand Bond west of
   the jamb (30.7, 0.55, 35.2) — does the GAME's door cycle open/closed the way the port's
   does ([M 0x1363c8:52-68, the moved-sphere proximity scan])?
2. The missile p0 start frame S on the REAL game: watch ch 19's rise and each missile's
   state-1 entry with the disc-accurate countdown — the port's 2747 is the fast-countdown
   value; the real S per missile = ch19 + seed·60 [D seeds 105/40/20/0/60/105/105/60], which
   re-spaces the eight deadlines and may loosen the 1b gate on its own.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
