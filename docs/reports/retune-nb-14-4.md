# retune-nb-14-4 — 14 infiltration to the delta exit, 00 Agent real-damage walk, round 4

Branch `s/retune-nb-14-4`, cut from origin/master 72a68f828. **origin/s/retune-nb-14-3 does not
exist on the remote** (round 3's branch was never pushed; its full report survives in the queue's
`done/50-retune-nb-14-3.md`) — nothing to merge, so the branch is master + this round's test-file
changes only. The round-3 ledger was taken from that report and from the shipped ignore message.

## Status: still red — the ledger and the next levers are pushed into the ignore message

The walk is not green. This round's contribution: the shipped round-3 code no longer even crosses
the junction on today's master (the stream was re-rolled by the master merges), and this round
found and shipped the lever that fixes THAT, measured two of the round-3 levers on the new stream,
and mapped every armour/health pickup near the death chain. All numbers below are [L]: this
sandbox's real-input runs (NF_DMG_TRACE / NF_PICKUP_CENSUS, 2026-11-19).

## What the round measured (each lever its own real run)

1. **The master merges re-rolled the stream — the v15 wedge became the baseline.** The shipped
   round-3 code (the v13 dry-gun scoop + the node-by-node loop with the route_scan stops) on
   today's master WEDGED loop node 5 at the 6000-tick cap: bond (32.47,0.66,24.51) grounded=false
   on the level's hover cells, every attacker ALREADY dead ("drones nearby: none"), 25 hits for
   72.5, 19 return-fire fights. The master commits 22d5836b7 (the Exchange weapons) and
   e4fb1c512 (the weapon-loop substate stop) shifted the junction fights' rolls onto the
   backpedal-into-the-hover-pocket shape the v15 lesson recorded.
2. **v16 STANDS (shipped): the fights' feet tied across the south loop.** `descent_stationary_fights`
   set inside the d3 loop gate and restored before the north leg (the level-13 stretch idiom,
   walkreal_07000013_live's north-west stretch): the fights still answer every shooter with the d3
   pick/aim laws (gun-ray gate, damage-weighted rank, head-hunt) but the stand's feet never move
   Bond — no back_off shove, no creep, no lane-resupply fetches. That is the field doc's own law
   ("the displacement, not the collision, is what puts Bond in the pocket", level-14-landing-hover
   lever 2). MEASURED: the wedge is GONE, the loop CROSSES, and the death moved to the door's west
   face — t25036 at (−60.22,0.73,37.94), health 17.5 there (the loop costs 60.0 → 17.5), 34 hits
   for 100.0, 28 fights: the west-face scan killed #32 (t25029, 17.5 → 15.0) and the door squad
   finished the walk 7 ticks later. The stationary law costs ~−10 health vs the baseline's node-5
   state (a STILL target eats more first rounds under the guards' moving-target accuracy) — traded
   for the entire second half of the walk.
3. **v17 (the round-3 ignore message's lever 2, the hall scoop) measured NO-OP** — kept, it costs
   two read-only comparisons when blocked. The hall's north-west reads ZERO live pickups (the
   clear_hostiles loot passes already took them) and the dry gate never fires there because the
   fusebox cuts leave the Laser 0x4e in hand (clip 499 counts as clip). At the switch band
   switch_to_gun arms the 0x14 at **clip 10 + spare 89** — the door face is reached with AMMO TO
   SPARE. **Ammo is not the binding constraint on this tree; the health pool is.**
4. **v18/v19 (the task's armour/health-pickup scan) measured DEAD — the only armour west of the
   0x85 door is unclosable from the loop's lanes, from BOTH axes.** The NF_PICKUP_CENSUS dumps
   (read-only, env-gated, the runs stay byte-identical) show exactly four kind-3 armours on the
   level's south half: (32.26,0.87,31.14) d 7.2 from the loop mouth (the only one west of the
   door), (42.48,0.09,8.28) 32 u east off route, (−41.29,1.53,36.48) in the Kiko corridor (LIVE at
   the door-face census, d 19.1, ~3.4 u east of the east lane's end (−44.5,34.0)),
   (−27.29,0.87,30.96) in the server room; the display-rack arsenal (nine 0x1a ×20, two 0x15
   ×100, y 1.7-2.34) sits beyond the escape touch. The machine-room armour: the north approach
   (v18, a fixed two-leg rim-line walk under try_walk_to) latched the walker onto the mezzanine
   door (32.0,**2.1**,35.8) — the room's ground entry is the SOUTH door (31.5,1.2,26.9) the route
   crosses — and the junction approach (v19, keyed past that crossing, no peace gate) wedged
   **2.83 short** on the shelf rim (bond (30.72,0.66,28.76) grounded=false — the v11 2.05-short
   shape), the take never fired, and the 6000-tick burn's stream shift died EARLIER (the ascent
   base t21534, 42 fights). Both fetches are REMOVED from the shipped code; the wedge facts stay
   as comments.
5. **The round-3 d1-sister "master regression" is gone on this master.** Round 3 reported
   `walk_the_infiltration_to_the_delta_exit_real_damage_agent` FAILED on their merged tree and
   flagged it for attribution; on today's master it PASSES (the binary run below) — the newer
   master commits fixed it. No d1 fix was attempted or needed here.

## Verification (the required binaries, run whole on the final tree)

- Whole `walkreal_07000014_live`: **8 passed, 0 failed, 2 ignored** (128 s). The ignored two are
  this task's walk (`..._real_damage_00agent`, the round-4 ignore message above) and the d2
  sister `..._real_damage` (round 3's own batch-suite ignore, untouched this round).
- Whole `interactables_07000014_live`: **10 passed, 0 failed** (13 s).
- Diff scope: ONLY `crates/nf-view/tests/walkreal_07000014_live.rs` (+ this report + the WORKLOG
  entry). No `crates/*/src/`, no `crates/nf-view/tests/common/` — every other test binary
  compiles byte-identical to master, so nothing else needed a run for THIS diff (the helper flag
  `descent_stationary_fights` already existed and is set behind the d3+!tops_up gate inside this
  test's body only; the d1/d2 sisters never take the new branches).
- **The fast suite (`tools/test/suite.py`) did not produce a total line for this tree.** It was
  started in the background three times per the driver instruction (`nohup`, then `setsid`) and
  the sandbox killed the job between tool calls every time (process count 0, `suite.out` empty —
  the same experience round 3 recorded); the foreground four-crate `cargo test --no-run` build
  exceeded the remaining sandbox window (the tool call timed out mid-build). The two required
  binaries ran whole in the foreground instead; the crates the diff touches are exactly the two
  binaries above.

## What's left for the next message (the levers, in order, now in the ignore message)

1. **Survive the door face at 17.5 health** — the scans' capped 300-tick stands leave Bond inside
   the squad's closing cone: v18's stream shift (the failed fetch's 12000-tick burn re-rolled the
   door sequence) measured #32 CONVERGED to d 2.6 at hp 20 dealing 10.0/hit — inside the 3.0
   auto-hit band [M DroneWeap_DoBulletAccuracy 0x179740:113-115] — and the second-beat re-park
   read him still alive. A re-park leg that steps BACK west (out of the closing cone) between the
   scan fights, or a longer cap keyed on the door scans only, must land #33/#34 before their
   bursts.
2. **The Kiko-corridor armour fetch** (−41.29,1.53,36.48) keyed after the east-lane leg
   (−44.5,34.0, d 3.4, the same fixed-spot try_walk_to law) — +50 armour before the corridor's
   squad and the Rook's-wake legs.
3. **Then the round-3 ledger's lever 3**: pull the Kiko squad inside the fights' landing band
   (the shipped 300-tick stands and the 90-tick hold fights land nothing at d 9.6-9.9).

## Live checks for the main agent

None required; all measurements are walker-side [L] with the ray facts from this tree's own
candidate dumps. Optional: on PCSX2, walk the real game to the machine-room armour (32.26, 0.87,
31.14) — the port's walker cannot close it from the loop's lanes (2.83 short at the shelf rim
from the junction side, 2.05 short from the north-west, the mezzanine door (32.0, 2.1, 35.8)
latches the north walk). If the REAL game reaches it on its own pathing, the exact input sequence
that does is the missing lever for a +50 armour pool before the junction fights.

## Usage

Not visible to me — this session exposes no token, cost or context-window figures to the worker;
nothing to copy.
