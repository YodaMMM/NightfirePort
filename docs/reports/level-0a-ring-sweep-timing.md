# level-0a-ring-sweep-timing — report

Branch `s/level-0a-ring-sweep-timing` (cut from origin/s/trigger-plane-records-2 9c5406bc, origin/master dd78d755 merged; final commits 5ad2848a + d2afe75f).

## What was done

**The four red tests are green.** `walkreal_0700000a_live` whole: **13 passed, 0 failed, 2 ignored**
(the four sisters `walk_office_floor_qworm_keyboard_hatch` + the real-damage Operative/Agent/00-Agent
runs among them), plus the required green set held: `low_beam_621_fires_standing` **2/0**,
`fix_truck_live` **2/0**, `interactables_07000011_live` **16/0**.

**The mechanism, pinned.** Beam #614's glist rides key motion: 3 keys z 13.0287 → 10.27843 →
13.0287, one pass 240 ticks [D keys, m_paths; M Trigger_Create 0x1b1720:56-87 + Trigger_Update
0x1b1a78:60-70]. [L probe] the live box parks 12.896..13.173 (t≈224..18) and bottoms 10.146..10.423
(t≈110..148), sliding ≤0.034 u/tick. A standing pass is impossible at ANY phase — the walk's
monotone z always meets the sweep (the probe stamped 0x22 on 16/16 walk phases). The jump is the
dodge: its arc runs 0.1 u/tick with the feet peaking 1.840 [L probe], and the standing toucher's
segment bottom (B = feet−0.485 [M Player_Collision 0x194df0 default arm]; #614's band top 0.5164
[L]) clears the band by more than r 0.55 for arc ticks k 10..37 — ONE jump crosses the whole band,
its low-y ends (k ≤ 9, k ≥ 38, touch radius ≤ 0.54) kept clear of the live box by the takeoff
stands: southbound z ≤ 13.79 behind the arm `box_max ≤ 12.0`, northbound z ≥ 9.52 behind the arm
`box_min ≥ 12.7`.

**The implementation** (`crates/nf-view/tests/walkreal_0700000a_live.rs`): `sweep_jump_beam614`
(the arm watch on the live posed tris — the port re-poses them per tick [M Motion::advance +
step_motion] — the takeoff stand walked on real input, the standing-clearance assert at the
takeoff tick, the one-tick jump, the landing-clearance assert, and the ch 0x22 never-stamped pin)
wired into all three #614 crossings: the keyboard route's southbound, the walk-back nav's final
node 10.6 → 9.0 (the old 10.6 sat inside the bottom park's exclusion) + the northbound, and the
ring's entry. The helper's asserts name the exact pose if any margin regresses.

**The cascade fix.** The #614 re-timing shifts the route's tick phases, and the northbound #621
press 1.0 then fired its takeoff stand INSIDE #621's (x,z) footprint (x −14.8, z −33.4 [L the
621watch round: 0x24 stamped exactly there]). [L] #621 is static (motion false) and its record
planes face the SOUTH side — the southbound crossing from the north stays clean at the same
footprint, so the northbound raw refine reads it. The northbound press widens 1.0 → 2.2: the
takeoff (−15.6, −34.4) is 0.72 3D-clear of the band's west-south corner (plane-side-independent),
and the arc's footprint ticks ride y-safe B ≥ 1.3.

**Merge-collision fixes** (the branch merge's struct fields, mechanical): `walkreal_07000001_live.rs`
`Toucher.seg_only: false` (the 01f plate keeps its master semantics), `exit_aim_hint_live.rs`
`Trigger.planes: Vec::new()`.

## How it was verified

- The compile check of the three touched binaries green (cargo check, 64 s).
- In-sim: the four sisters + the whole 0a binary 13/0/2i, the three named binaries 2/0, 16/0, 2/0,
  all `--test-threads=1` (four parallel level loads OOM the 4 GB sandbox).
- The sweep probes were scratch files (`zz_sweep_probe*.rs`), never committed; their numbers are
  quoted above with the [L] tag.

## What's left

- Every `walkreal_*_live` binary and the trigger/beam/laser family beyond the four named above
  did NOT fit this sandbox (it recycled at ~10:29 UTC with the repo gone; the rebuild + the ISO
 + the fix + the runs consumed the window). My diff touches test files only — no `src/` change —
  so per the merge rules their risk is the branch merge itself; the next session should run the
  walkreal family and the full suite (suite.py) and quote the total line.
- The suite was not run on the final tree (no total line this session).

## Live checks for the main agent

None required — the pin is against the disc key paths + the in-sim live box; no PCSX2 behaviour
is asserted that the port tests don't already cover.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment).
