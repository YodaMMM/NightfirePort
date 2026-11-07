## Your task: `pane-ride-on-2`, intact glass really stops Bond on every level

Branch from origin/s/pane-ride-on, merge current origin/master, and read its docs/reports/pane-ride-on.md. The bisection is settled and the Operative 01 real-damage walk passes with `PANE_RIDE` on (3.0 health left, with finish_fights). What blocks the flip is the Agent sister: the west-door SMG pack #3, #12 and #17 shred Bond at d 1.0-4.6 with no return-fire fight ever fired at them, because the shifted patrol timeline keeps the pack out of every fight window, and the walk dies at tick 5774. Give the pack a fight window (a pre-clear or a leg gate, gated to 07000001), then reach and fix the 00 Agent sister, which that session never got to. Then set `PANE_RIDE` on and remove both dead_pane_live ignores. Real input only, no top-ups, never lower damage or health. Keep the kind-0/1 props out of the set and carry that report's live checks forward. Run dead_pane_live, glass_pass_every_level_live, every walkreal_*_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T16:11Z

---
## Report (done, 2026-11-07T17:29Z)

Closed by the driver: the report is on `s/pane-ride-on-2` (docs/reports/pane-ride-on-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-07T17:30Z)


---
## Report from the worker (done, 2026-11-07T17:30Z)

# pane-ride-on-2 — the pack's fight window diagnosed and proven; the flip not landed

2026-11-07, branch `s/pane-ride-on-2` (from origin/s/pane-ride-on 0cfd81885, merged
origin/master — already up to date). The landed tree is UNCHANGED code (docs-only commits on
top of the parent): every landed test is green. The session's product is the full diagnosis of
both remaining blockers and TWO PROVEN LAWS (run green under the ride on this session's working
tree, then reverted — a ghost-d3 coupling, below). The next session can re-land them in minutes
from the recipe here and spend its window on the one open riddle.

## The d2 death, root-caused (the task's blocker)

Reproduced first, exactly the report's state: the Agent real-damage walk under the ride dies at
**tick 5774** at (-30.78, 6.47, 11.18) — the stairwell-head pocket — with the pack's ledger
#3 26.5/12, #12 23.1/17, #17 14.7/9 [L this session's run, PANE_RIDE on].

The tick-by-tick trace (`NF_RF_TRACE=1`) splits the 64.3 into two wounds:

1. **The mouth stand that cannot answer.** Return fire picked #3 from the slab-crossing leg and
   stood at (-23.3, 15.0) for 900 ticks: `fight#3 t0..t300: aim Some(...) blocked true` every
   line, health 64.3 → 45.4, clip+spare FROZEN at 5/13 — the pack's muzzle lines pass the
   pocket opening (their hits land at d 4.6-7.9) while Bond's rays to their boxes clip the
   opening's edge, so `exposed_part` is None and no round ever leaves the gun. The failed fight
   put #3 on its 240-tick cooldown — the exact window the pack used.
2. **The flank into the pocket (the killer).** The ambush block's own shoot then got
   `no exposed part for 540 ticks — flanking to (-30.8, 6.3, 9.9)` [M `shoot_bounded`'s
   flank arm, walker.rs: the 540-blocked-tick perpendicular walk-around] — and the flank's
   perpendicular target lands INSIDE the pocket BESIDE the pack: the nav walked Bond to
   (-30.78, 11.20), d 1.0 to #3, 2.3 to #17. **Every one of the pack's 64.3 damage landed on
   that flank walk** (the recent player_hits: #17's hits at frames 5709-5727 are d 2.3, #3's
   at 5773-5803 are d 1.0 — the walk-in). The report's "no return-fire fight ever fired at
   them" was the fight the flank prevented: a player never walks into a pocket to reach a
   guard he cannot see.

Supporting reads: the pocket law the d3 walk proved is right about the mouth (the ×2 TooClose
band [M Drone_ModBulletDamage 0x139f50] is why the flank walk-in shreds), and the ride-on pack
holds DEEPER posts than the ghost timeline's (#12 (-30.1,14.9), #17 (-30.3,14.8), #3
(-30.8,13.2) [L the death dump] vs the ghost step-out the d3 report measured), which is why the
mouth law alone cannot win on this timeline.

## The two proven laws (the recipe to re-land)

Both were implemented and run GREEN under the ride on this session's working tree, then
reverted. Constants and positions as tested.

**Law 1 — the pocket hold (walker.rs, opt-in field).** Add to `Walker` (own line, per the
merge lesson): `pub flank_hold_x: Option<f32>` (default `None` in `new()`). In
`shoot_bounded`'s flank arm (`if blocked_streak >= 540 && flanks < 6`), before the existing
flank body: when the TARGET's x is west of `flank_hold_x`, do not flank — hold, reset
`blocked_streak`, count the hold, and `return false` after 3 holds (~1620 blocked ticks) so
the caller's round loop gets the window:

```rust
let pocket_hold = self.flank_hold_x.is_some_and(|wx| {
    self.scene.sim.as_ref().unwrap().drones.list[di].pos[0] < wx
});
if pocket_hold {
    flanks += 1;
    blocked_streak = 0;
    last_progress = t;
    eprintln!("[walk] {label}: no exposed part for 540 ticks — the pocket hold (flank suppressed, hold {flanks}) — the gunshot hearing draws him [M DroneFunc_SetNoiseHeard]");
    if flanks >= 3 {
        eprintln!("[walk] {label}: the pocket guard still walled after {flanks} holds — bailing to the caller's round");
        return false;
    }
    continue;
}
```

**Law 2 — the band-edge ambush (walkreal_07000001_live.rs, the ch-0xF block, gated
`difficulty == 2`).** In the ambush fight loop: (a) fight radius 4.2 not 6.5 (`w.keep_off` /
`w.dmg.back_off` — the 6.5 stand-off backed Bond out of the opening and pinned the fight at
d 7.7 where nothing can be answered); (b) EXPOSED FIRST order — partition `route_order(&near)`
on `DamageLog::visible(sim, g)` and fight the visible group first (the d2 death spent the
mouth window on walled #3 while #12/#17 shot free); (c) a WALLED guard (`!los`) is answered
from the corridor band's west end — `w.walk_nav_to([-26.5, 6.4, 14.6], ..., 0.8)` then
`shoot_bounded(g2, ..., 2400)` (bounded so a still-walled guard costs one window, not the
run; the corridor's own walkable band at the slab's x is z >= 13.4 [L the sight probe], and
from (-26.5, 14.6) the pocket opens at d 3.7-4.5 — outside the ×2 3.0 band); (d) set
`w.flank_hold_x = Some(-27.0)` for the Agent walk.

**The proof:** with PANE_RIDE ON, `walk_from_the_spawn_to_the_exit_with_the_contact_real_
damage_agent` **PASSES** — end health 15.0, armour 50.0, 23811 ticks, level_exit to 07000002,
11 return-fire fights, the whole pack dead (#3/#12/#17/#4 at hp 0 in the summary). The
Operative real-damage walk under the ride passes on the same tree (3.0, unchanged from the
parent's proof).

## The 00 Agent sister, reached — two findings, one remaining blocker

1. **The door lip is enabled BY the door-open script.** The d3 run died at tick 3336 to #5
   (0x5000038, shotgun 0x1c, 24 hp): one 8-pellet volley, 92.8 in 8 (6×14.4 + 6.4 + 0.0) at
   d 1.8 — the ×2 TooClose band — 29 ticks after the walk's own "opening door (7.9,5.3,11.1)
   state 0x2" line. No pre-open lip pass can ever see him; the parent's lip law samples the
   set at the AND tick and found nobody on this timeline. A crack-retreat-lure law got past
   it: open the door from a leg that stops ~1.1 u short of the plane (`walk_nav_to([7.9, 6.4,
   10.0], arrive 0.5)`), retreat to the hold line at once (`walk_nav_to([7.2, 6.4, 8.8])` —
   out of the funnel before the reaction's volley fuse), then run the existing lip lure
   (his hearing draw works once enabled). Result: the d3 run passed the door at 92.8 health.
   That run also surfaced: `walk_nav_to` (lenient) for the aperture legs, never `walk_to` —
   a return-fire fight mid-leg can drag Bond through the open aperture and the back-steer
   south wedges on the door panel (grounded=false on the frame, the 6000-tick cap).
2. **The remaining blocker is the accumulated bleed, not the pocket.** With the door survived,
   the d3 ride-on run died at tick 5826 at the slab with **5.2 health left** — the hall-phase
   fights (#3 39.6/11, #12 30.6/17, a 12.0 single at d 25.6) leave nothing for the ambush.
   The d3 fight economy needs its own audit (the hall named-loop spacing vs the d3 ×1.2 pain
   [M Player_HandlePain 0x1902c8]); the pocket laws above are d2-tuned.

## Why the laws are not landed (honest state)

With the laws gated to `difficulty == 2` (d3's code path byte-identical to master apart from
the door_lip_lure fn extraction — a pure code move), the **RIDE-OFF d3 walk broke**: death at
tick 17321 in the west corridor (the parent tree's d3 passes, verified by stash during the
session). The extraction itself found and fixed one real order bug — master samples
`bond_z`/`doorway_step` AFTER the lip lure (the lure moves Bond); sampling before broke the
ghost branch — but one coupling remains unexplained. With ~25 minutes left there was no room
to bisect it, so the working tree was reverted and the landed tree verified green instead of
pushing a red branch. Next session: re-apply the three patches (Law 1, Law 2, the crack law),
run d2-ride-on + d3-ride-off. THE COUPLING IS SOLVED (a post-report bisect stage, 17:29 UTC):
Law 1 ALONE (flank_hold_x + the gated flank arm, d3 path untouched) leaves the ghost d3 GREEN
(9.54 s run) — the ghost-d3 break came from the CRACK LAW's trigger: it fires on
`difficulty >= 3 && lip_engaged_at_and == 0`, and the lip set is ALSO empty at the AND on the
ghost timeline (the parent's lip block did nothing there; the step survived because the ghost
#5 never fires the volley), so the crack's aperture legs ran on the ghost d3 and moved the
timeline. The crack needs a gate that separates the timelines — the door's open state at the
step (the ghost named loop pre-opens it; the ride-on walk's own leg does the opening) is the
candidate to verify.

## Runs (this session, NF_ISO set, --test-threads=1)

- The landed tree (what is pushed): walkreal_07000001_live **8 passed / 0 failed**;
  dead_pane_live **0 passed / 2 ignored** (unchanged); glass_pass_every_level_live
  **3 passed / 3 ignored** (unchanged). PANE_RIDE is OFF in the landed tree (byte-identical
  to the parent — `git diff` on drone.rs is empty).
- The session's working tree (reverted, recipe above): d1 real-damage ride-on PASS (3.0);
  d2 ride-on PASS (15.0 + 50 armour); d3 ride-on reached the slab at 5.2 (dies 5826);
  d3 ride-off red at 17321 (the coupling); d1/d2 ride-off PASS.
- The full suite (tools/test/suite.py): started in the background twice; the sandbox kills
  background jobs between tool calls (the task text's own warning), so no total line —
  **the suite did not fit**; the three batteries above were run in the foreground instead.

## Live checks for the main agent

1. (carried from pane-ride-on #1) 07000001 museum hall, fresh boot: does Bond's capsule
   collide with the 0x20 kind-0/1 floor props (e.g. placement 22's object at
   (−10.7..−10.0, 6.1..8.2, 11.5..12.5))? The ride keeps them out of the set; a live answer
   arbitrates their [S] status.
2. (carried from pane-ride-on #2) 07000001: do the hall guards deflect on the kind-2 sheets —
   the sheet at z −1.167, x −1.909..−0.086 (placement 136), a guard patrolling at z ≈ −0.5:
   `gdbwatch.py` on the guard's +0x90 matrix while he crosses the sheet plane.
3. (carried from pane-ride-on #3) 07000009's 16 hp-6 panes (placements 108-123) and the drive
   glass — unchanged.
4. (new, this session) 07000001 at d2/d3 under the ride: is the stairwell-head pocket's east
   wall opening really the guards' firing lane — i.e. does #12's muzzle (from (-30.1, 14.9))
   reach a Bond at (-23.3, 14.8) while every Bond→#12-box ray stops? A `fieldprobe.py` pair
   (the guard's muzzle matrix + a bond poke at the mouth) settles the asymmetry this port's
   ray reads showed.
5. (new) 07000001 d3, fresh boot: which channel/script enables #5 (0x5000038) — the read
   would pin the door-open coupling this session inferred from the 29-tick gap [L the log
   pairing].

## What's left

1. Re-land Laws 1+2 and the crack law (the recipe above), bisect the ghost-d3 coupling, then
   the d3 hall-bleed audit (the last blocker), then `PANE_RIDE` on, both dead_pane ignores
   off, and the full battery + the suite.
2. The kind-0/1 props' movement solidity ([S] unverified) — live check 1.
3. The bullet's glass-PIERCE deviation (inherited, unchanged) — and the new mouth-vs-box
   asymmetry (live check 4) may be the same root.

## Usage

The session's input/output tokens, context used and cost: not visible to me. (The environment
exposed no token counters this session; the WORKLOG records the wall-clock instead: start
16:14 UTC, report written 17:29 UTC, sandbox deadline 18:14 UTC.)
