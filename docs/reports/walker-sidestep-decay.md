# walker-sidestep-decay

Branch `s/walker-sidestep-decay` = origin/master 784ba2d99 + merge origin/s/los-mask-repins-0c-3
(9612e087a) + merge origin/s/los-mask-repins-12-2 (edda1753c; the WORKLOG add/add conflict
resolved keeping both sides' entries). Both reports read before any code moved.

## What's done

**The two level patches are replaced by one walker change** (all in
`crates/nf-view/tests/common/walker_b2.rs` + the route scope in
`crates/nf-view/tests/common/walkreal0c.rs`; walker-only — no sim/nf-game changes anywhere):

1. **The sidestep decays, by default.** The stuck ladder's nudge counter now decays toward 0 on
   BOTH sides every steered tick (`self.sidestep -= self.sidestep.signum()`); the old
   `self.sidestep -= 1` only ended a RIGHT nudge and grew a LEFT one forever (los-mask-repins-12-2
   measured sidestep −753 with Bond strafe-left pinned at the 07000012 door recess [L the
   NF_WALKTRACE run]). The 12-2 opt-in `sidestep_decays: bool` flag and its doc are DELETED; the
   walkreal_07000012 d1 walk's setter is gone (the decay is now what it asked for).
2. **The sidestep resets when a ride or grab scope begins or ends.**
   - Inside `walk_to`: `Act::WireRide` resets at the wire grab (scope begin) and at the drop
     (scope end); `Act::Ladder` resets at the ladder grab (begin) and at the climb's top (end).
   - The direct `step_bond` wire loops that own their own rides (walkreal0c's #284, #282, #283 —
     they never route through `walk_to`) call `reset_sidestep()` at each grab and at each drop —
     the same contract, bound to the scope boundaries themselves.
3. **The 0c-3 patch is deleted**: the post-ride `w.reset_sidestep()` call under the
   `STRIP_RD_POST_RIDE` scope (the patch was "one caller, post-#282") — the reset now fires at
   the ride scope itself, plain and real-damage alike. The scope's mid-ledge STOPS stay (they are
   the 0c-2 ledger's route lever — the thin-floor arrival circles — not the sidestep patch).
   `reset_sidestep()` survives as the walker's scope-boundary primitive with the new contract doc.
4. `walker.rs`'s own `sidestep_decays` flag is untouched (it predates the two patches and belongs
   to the other walker; its users set it explicitly).

## How it was verified

**Red first — both patches proven load-bearing** (each patch neutralized on the merged tree,
before the walker change, real input, whole walk):

- `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage`
  without the 0c-3 reset: **FAILED** — wedged at the 6000-tick cap of "the mid-ledge stop (rd)",
  Bond stuck at **(6.26,1.13,−221.83)** — the 0c-3 report's own measured fall position, reproduced
  exactly [L this session's run].
- `walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative` without the 12-2
  setter: **FAILED** — wedged at the west stand (10.62,1.03,−15.73), the 6000-tick cap [L].

**Green after the walker change, patches gone**: both walks pass whole —
`walkreal_0700000c_live` **4 passed / 0 failed / 1 ignored**,
`walkreal_07000012_live` **4 passed / 0 failed / 0 ignored** [L this session].

**The battery — every walkreal binary that compiles the changed helper (walker_b2) ran whole**,
`--test-threads=1`, release, on the final tree:

| binary | result |
|---|---|
| walkreal_0700000c_live (whole, patches deleted) | 4 / 0 / 1 ignored |
| walkreal_07000012_live (whole, setter deleted) | 4 / 0 / 0 ignored |
| walkreal_0700000c_eastroute_live (walkreal0c → walker_b2) | 1 / 0 |
| walkreal_07000009_live | 9 / 0 |
| walkreal_0700000a_live | 13 / 0 / 2 ignored |
| walkreal_0700000b_live | **5 / 4 failed** — see the repair + the ledger below |
| walkreal_0700000c_barrelrow_live | 1 / 0 |
| walkreal_0700000d_live | 1 / 0 / 2 ignored |
| walkreal_07000011_live | 3 / 0 / 1 ignored |
| walkreal_07000011_route_live | 1 / 0 |
| walkreal_07000013_live | 3 / 0 / 1 ignored |
| walkreal_07000014_live | 7 / 0 / 3 ignored |
| the 32 probe walkreals (probe0c2/3, probe0c_band5/6/8, probe0c_barrelrow, probe0c_blk/2/3, probe0c_bridge, probe0c_catwalk, probe0c_door/2/3/4/5/6, probe0c_eastclimb/3/4/7, probe0c_grapple, probe0c_lip, probe0c, probe0c_northend, probe0c_secretreach, probe0c_stripjump, probe0c_upperlane, probe0c_ventroute, probe0c_voidspan, probe0c_wires, probe11_48) | all ok (1 passed each; probe0c_northend 0/0/1i, probe11_48 2/0) |
| interactables_0700000c_live (walker_b2) | 7 / 0 |
| interactables_07000012_live (tower2 + walker_b2) | 12 / 0 |
| interactables_07000013_live (walker_b2) | 10 / 0 |
| nf-game --lib (whole) | **782 / 0 / 4 ignored** |

**The one shift, and its repair** — `walkreal_0700000b_live`:

- With the decay default, the plain walks wedged at "files n26→n25 (the doorway)": the direct
  line clips the desk row's west corner at (−20.7,48.0,30.6), the stuck ladder's alternating
  nudges now decay instead of growing forever, and the old eternal strafe-left that carried Bond
  around the corner is gone [L the A/B: the pre-change walker_b2 runs 0b 9/0, the decayed walker
  3/6]. **Repair (the level's own file, real input): a west-flank leg** before the doorway —
  the warm walks take `files n26 flank west` (−22.6,47.6,30.2) explicitly. The plain walks are
  green again.
- The four real-damage sisters (real_damage, agent, 00agent ×2) stay **RED, ledgered**: they
  round the desk corner on their own fight timing, but the decay re-times their upstream legs
  (every stuck event resolves differently) and their downstream dome-wire crossing is
  frame-windowed — the run dies at tick 16013 at (−12.55,57.89,12.62) with **39 × −2.40 world
  zaps (93.6 of 100)** — the walk now sits in a LIVE wire window. Gating the flank off the real
  walks did not restore the old timeline (5/4 again) — the shift is the decay's own re-timing,
  upstream of any route lever. **The lever for the next window**: re-time the dome-wire crossing
  against the fusebox windows on the decayed ladder (the 0b-diag block in the file prints the
  world-hit ledger and the fusebox wire states on every real run).

**Not run** (documented): the 36 walkreal binaries that include only `common/walker.rs` —
walker.rs is untouched by this task, those binaries are byte-identical to master's, and the fast
suite covers them next window. **The fast suite did not fit — twice**: the first start was
killed before the red proof (the tree would have changed under its build), and the final-tree
start died in its build phase with the linker's `signal 7 [Bus error]` at **0 bytes free** — the
9.9 GB sandbox cannot hold the full four-crate build (the target dir, the ISO and the
toolchains leave no headroom; the same constraint the los-mask-repins-12, -12-2, -0c-3 and
-08 sessions recorded). The battery above is the substitute: every binary that compiles the
changed helper ran whole, plus the three exposed interactables and nf-game. **The next window
must run the suite on this final tree first** (free `~/.cache/ms-playwright` +
`~/.cache/puppeteer` — 1.2 GB of unrelated browser caches — before starting it).

## What's left

- The 0700000b four real-damage sisters: the dome-wire crossing re-timing (the lever above), then
  un-red. Everything else in the battery is green.
- The fast suite on this final tree (its per-binary pass counts), next window — the disk needs
  the browser-cache cleanup first.
- The pure-walker.rs walkreals (01–08, 15, 16, 1b, 41, 4a, …) ride the same suite run.

## Live checks for the main agent

None new — this task's fix is in the test walker and the test routes; the game-facing facts it
rests on were live-checked in the predecessor rounds (the 543 crossing line, the 0c-2 ride
choreography). If a render pass is cheap: the 0b desk corner at (−20.7,48.0,30.6) facing south
should show the desk row Bond flanks west, and the dome wire Bond dies in at (−12.55,57.89,12.62)
should show the live-wire gap the next window must re-time.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost or
context-window readout to the worker. What was visible: the task message's clock (sandbox expiry
~07:59 UTC), `git` hashes, test outputs, process exit codes. No input/output token counters, no
credit balance, no dollar cost.

## ntfy failed

(filled at finish — see below)
