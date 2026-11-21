# retune-nb-14-14 — the 14 infiltration to the delta exit 00 Agent walk: GREEN

Branch `s/retune-nb-14-14`, cut from origin/master b69bacbc9 with `origin/s/retune-nb-14-13`
merged in first (it was not on master; the merge was clean — the round-4..13 reports, the
ledgers and the WORKLOG entries carried over; WORKLOG appended only).

The sandbox was fully reset this round (no repo, no work/, no ISO, no rust): the clone,
work-slim.7z + the ISO archive (verified at exactly 369823203 bytes before extraction, py7zr
for both), rustup stable 1.100, `cargo fetch` and the ALSA user prefix were redone (the
session's leftover user prefix was intact and the `libasound.so` symlink resolved).

## What's done

**THE WALK IS GREEN. The `#[ignore]` on `walk_the_infiltration_to_the_delta_exit_real_damage_00agent`
is REMOVED. The round-14 lever 1(a) — the far-guard #58 pre-emption — was executed as the
POCKET AMBUSH and walked the whole route: Rook killed on the walked gun path, the room
cleared, the delta exit taken, 07000015 loaded warm.**

- **The v0 baseline re-read from the round-13 logs (the untracked scratch logs were still on
  disk; the round-13 v0 single-run certification carries as the shipped-stream proof)**: the
  death t38906 = Rook's first 0x6e volley −10.70 at d 9.8 into the 10.7-hp pool; #58's burst
  −7.0 (t38808/38812, skin 0x0500008c, hp 10, (29.1,1.0,7.0)) is the WHOLE deficit. Two
  revisions to the round-14 ledger's premises came out of this round's runs [L 2026-11-21]:
  (1) the v0 trace's fight counter (48) exceeds its printed fights (47) by exactly the fight
  that opened ON #58 AT his own burst — Bond died INSIDE it, before the fight's end-of-fight
  print; the far guard was answered, just too late. (2) #58 STARTS the chase at
  (28.47,1.0,43.15) — the far guards MIGRATE with the chase (the round-14 "pre-empt him at a
  fixed post" framing was wrong; the drone index is the identity).
- **Lever 1(a), four forms, each on its own real run (v1-v4), then the shipped v5**:
  - **v1** (the sight-open break + the shipped fight): the pre-empt sight (walker_b2's new
    opt-in `preempt_on` — a read-only `exposed_part` ray test at the %10 walk_to gate) WORKED
    and broke the chase leg at nav 8 +31 ((27.25,−10.13)) — but the round-9 went-silent
    pass-break (walker_b2.rs:742, opt-in on `switch_on_first_blood && finish_fights`) broke
    EVERY pre-empt fight at +0: a pre-empt's target is bloodless BY DESIGN, so the 240-frame
    window always holds another shooter's blood. The 3-attempt budget burned on instant
    breaks; "Rook must die" panicked with the walk ALIVE at 17.7 — and the stall itself
    measured the break spot BLIND (the pool held 17.7 for 210+ ticks, nobody shot — the
    round-13 v5 pocket law, now on the live stream).
  - **v2** (the pass-break scoped OFF, the plain fight): the fight ran (d 19.5, GL clip 5
    spare 79) but the GL landed ZERO on #58 in ~280 ticks at that range, and the plain
    `shoot_bounded`'s own slide law dragged Bond 4.4 u east off the pocket into #53's
    point-blank line (d 1.8, −7.0) — death t38939.
  - **v3** (the slide-off pocket hold, `shoot_bounded_fast`): Bond HELD the park, but #53 —
    alive because the break skipped his nav-9 free-kill fight — advanced from d 9.8 to 6.3
    and burst −21.2 in 6 hits: death t38781, WORSE than v0.
  - **v4** (the nearest-exposed ambush at ≤ 12 u): fighting WHOEVER exposes nearest from the
    quiet pocket (the GL provably kills hp-10 guards at ≤ 10 u — the chase's own kills
    #50/#46/#47/#53 all fell to GL fights at that range) kept the v0 nav-9 free kill AND
    cleared forward: #53 killed t38854 (d 4.0) and **ROOK HIMSELF killed t38973 (d 9.9, the
    walked gun path — the kill rounds 4-13 could never take)**. The gap: #57 (hp 2, the
    Rook-fight GL splash) sat at d 14-18 BEYOND the band, and his two retreat volleys
    (−3.50, −3.70 at d 18.3) ate the 7.2 pool on the delta legs — death t39216.
  - **v5 = SHIPPED GREEN**: the ambush band widened to 20 u, and when nothing exposes but a
    hostile still lives within 20 u the loop steps ONE nav leg toward the nearest (the leg's
    own fights — the unseen fallback's creep — open the line; lever 1(c)'s own "step one more
    nav leg and re-read" design, applied to the room-clear). Measured chain [L the v5 +
    final-cert runs]: the break at nav 8 +31 ((27.25,−10.13)), #53 killed t38854 (d 4.0),
    Rook killed t38973 (d 9.9), #57 killed t39013 (d 14.2, ZERO pool cost), #58 killed
    t39201 (d 17.2, ZERO pool cost — his −7.0 burst never fired), pool 7.2 held through the
    delta legs, **walked out in 40649 ticks, stuck 0, falls 0, the exit hold drained at
    (26.997,1.133,−24.847) and 07000015 loaded warm, arrival (94.50,41.37,0.0019) on the
    silo gantry**.
- The levers the ledger carried beyond 1(a) are MOOT for this round: the ambush cleared #57
  (1(b)'s target) and #58 with ZERO pool cost; the blind-pocket bombardment completion
  (1(c)) is superseded by the same ambush loop (the step-leg arm is its design).

## How it was verified

- **walkreal_07000014_live (whole binary, FINAL tree, `--test-threads=1`)** [L 2026-11-21]:
  **10 passed / 0 failed / 0 ignored, 210.46 s** — round 13's 9 passed / 1 ignored became
  10/0/0: the sisters and the other walks stay green (byte-identical — the helper change is
  opt-in, `preempt_on: None` default, the hook dead while None), and the 00agent walk now
  RUNS and passes.
- **interactables_07000014_live (whole binary, FINAL tree)** [L 2026-11-21]: **10 passed /
  0 failed, 12.57 s** — matches rounds 5-13's counts.
- The un-ignored 00agent single run was re-certified on the final tree (42.74 s, green, the
  same 40649-tick chain).
- Every lever form was judged on its own real run (v1-v5 plus the final cert, six runs
  total), all with NF_DMG_TRACE ledgers captured (lever_a_v1.log … lever_a_v5.log,
  final_cert14.log, whole_binary14.log, interact14.log — untracked scratch outside the repo).
- Changed files: `crates/nf-view/tests/common/walker_b2.rs` (the opt-in `preempt_on`/`preempt_hit`
  fields + the dead-while-None hook at walk_to's %10 gate + walk_nav_to's break-consume),
  `crates/nf-view/tests/walkreal_07000014_live.rs` (the un-ignore, the round-14 ledger
  comment, the d3-gated g58 resolve + ambush loop), `docs/WORKLOG.md` (appended), and this
  report. **No file under `crates/*/src/` changed** — the sisters' byte-identity is by
  construction (the new fields default None and only the d3 00agent chase sets them).
- **The shared-helper sweep (lesson 7)**: the level-walk binaries are SLOW in this sandbox
  (each needs its own release build; walkreal_07000001_live alone exceeded a 280 s window
  mid-test). The fast suite (`tools/test/suite.py`) was started in the background on the
  final tree per the task order — see below for what it produced. The two required binaries'
  whole runs above are the primary evidence.
- **The fast suite**: started in the background (nohup) per the task order on the final tree
  (the tree is merged with origin/master — the branch carries the merge commit 087f9cb06 of
  master b69bacbc9 + s/retune-nb-14-13). The sandbox killed it between tool calls with an
  empty `suite14.out` — twice (the round-13 lesson stands: the runner writes its per-binary
  lines only after the four-crate `--no-run` build, and a background process does not
  survive the tool-call boundary here). **No total line exists this round; the two required
  binaries' whole runs above are the evidence, as in rounds 4-13.**
- **The shared-helper sweep (lesson 7), attempted**: the six level-walk binaries
  (walkreal_07000003/04/06/07/09/0a_live) were batch-built successfully in one cargo pass
  (16.57 s — the dependency graph was already cached from the level-14 runs), but each
  binary's full walk exceeds the per-binary windows this window could give (100-280 s:
  07000003 was still mid-route at 200 s, 07000001 exceeded 280 s) — the sandbox's 2-hour
  budget went to the five lever runs that GREENED the walk. **No cross-level verdict was
  obtained; the byte-identity argument stands on the change's construction**: the two new
  Walker fields default to `None`, the %10-gate hook short-circuits on the None, the
  `preempt_hit` flag is reset and consumed only inside `walk_nav_to`, and only the d3 00
  Agent chase ever sets the flag — the other walks' instruction streams are unchanged.
  A next window should run the six built binaries whole (they sit in target/release) as the
  belt-and-braces check.
- Facts tagged [L] come from this window's runs on the real ISO with the NF_DMG_TRACE
  ledger; code citations are [M] with the addresses in the test comments; no test was
  weakened (the round-13 00agent ignore message's ledger moved into the test's doc comment
  verbatim-plus-addendum, and the ignore attribute itself is GONE because the walk passes);
  the game bytes, the token and the URLs stayed out of the commits.

## Still left

Nothing for the walk itself. The carried live checks for the main agent stand below (the
second kind-27 trap's trigger; Rook's 0x6e cadence). The round-9 levers 2/3 die with the
walk's green.

## Live checks for the main agent

1. **The second kind-27 trap's trigger** [D/script] (carried from rounds 12-13): at
   (−4.4,1.7,−25.3) r 4.0, what arms/fires the blast — answer ONE-SHOT vs CYCLICAL. The
   green walk never triggered it (the v5 chain crossed without a linger).
2. **Rook's 0x6e volley cadence when chased** [D for the answer] (carried): the v4/v5 runs
   add one datum — his volley fired −2.83 (t38703, d 9.8→10.2) on the ambush stream, i.e.
   the per-round damage varies with the accuracy law, not a fixed −10.70.
3. **The pocket's sight geometry** [D/script] (carried from round 13): the v1/v4/v5 runs
   answer half of it — the sight to #58 flips OPEN at the nav-8 pocket ((27.25,−10.13))
   while the v5 round-13 census read every guard eyefalse there: the guards MIGRATE with the
   chase, so the pocket's blindness is time-dependent, not fixed. What remains: which
   script/geometry owns the room-mouth sight line.

## Usage

not visible to me

## ntfy

The three fallback attempts run after this report's push (the task's step 3); their exact
outcomes follow in the queue transcript. [FILL_NTFY_FINAL]
