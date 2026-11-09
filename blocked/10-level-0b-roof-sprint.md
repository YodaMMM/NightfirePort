# 0700000b: the roof-deck sprint to the parachute

With guards seeing through the roof parapet (masked sight rays, see s/los-mask-join and its report), Bond dies at the parachute on the Agent and 00 Agent walks. Find the player's real way across the roof under the game's rules (cover, timing, which guards to drop first) and make both walks pass on s/los-mask-join's tree; base your branch on origin/s/los-mask-join.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0b-roof-sprint`, report in docs/reports/level-0b-roof-sprint.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T15:14Z

---
## Report (blocked, 2026-11-09T15:58Z)


---
## Report from the worker (blocked, 2026-11-09T15:58Z)

# level-0b-roof-sprint — report

Branch `s/level-0b-roof-sprint` = `origin/s/los-mask-join` (5e36d06ba) + this task's commits.
**Status: NOT finished — both real-damage roof walks are still red.** The session's time limit
(stop 16:58 UTC, sandbox expiry ~17:18) stopped the work mid-investigation; the next window
continues from this branch (see "What's left").

## What's done

1. **Reproduced both reds on the real level through real input** (no teleports): the
   `walkreal_0700000b_live` `..._real_damage_agent` walk dies at tick 15851 (frame 15945) at
   (2.13, 58.11, 40.80) — the parachute take slot — with **21 hits for 100.0**: #4 (skin
   0x5000006, weapon 0x18) 61.2 in 8 hits, "the world" 19.2 in 6 hits, #6 11.2 in 4, #7/#8 2.8
   each. The 00 Agent sister fails the same way.
2. **Read `docs/reports/los-mask-join.md` and its live check**: the deck guards' sight across
   the surface-0x40 parapet band is **verified in the real game** (the main agent's PCSX2 check:
   guard #4 sets los/seen across the band and walks toward Bond). So the port's masked sight is
   right and 0b stays a re-route problem, exactly as that report concluded.
3. **Closed the "which guards to drop first" door with the decompile + the disc**: Bond's only
   conventional weapon on 0b is the Korsakov K5 sleep dart (the 09 kit; the Q-Worm is a gadget)
   [M 0x1a7ef8; L the bullet probe], and the game's dart law drops msg 0x19 for a guard in full
   combat — Attack carries no case 0x19 [M 0x164210; 0x1469b8:315-361]. The deck squad enters
   Attack within 20 frames of spawning (the los-mask-join report's live check), and the squad
   spawns at/after control return (the diag: at frame 15392 no deck guard is present; the two
   leftover exec-floor guards are at y 47.78). No guard can be dropped before or during the
   sprint. Cover/timing is the only lever, as the task's "under the game's rules" implies.
4. **Killed the report's fusebox lead with data**: a diag dump at control return (added to the
   deck leg, real walks only) shows **0b has no fusebox at all** (`sim.drones.fuseboxes` empty)
   and no engaged hurt volume. The "live wire" 19.2 is the **armed GT turret pair #174/#175**
   at (2.26/-4.60, 49.10, −21.60): their hp counters dropped 24/15 shots across the walk and the
   6 × 3.2 world hits land at frames 12761-12804 — the dome-floor lane sprint to the files
   use-plate, under their north-firing cones. `FuseBox_Update` 0x1adfe8's 10-slot law does not
   apply to this level. (The damage log's "live wire / blast / hurt volume" label misleads.)
5. **Located the death's core: the take stand.** The sprint broke at `dxz < 1.0` and the take
   loop then stood ~95 ticks at the slot pushing `forward 0.4` while #4 landed 3 × 11.2 + 5.2
   (38.8) — and ch 0x3F never fired. Changed the deck leg (test-only, `walkreal_0700000b_live.rs`):
   the sprint now runs THROUGH the slot and breaks only on ch 0x3F, and the take is asserted to
   fire on the pass (the d1 walk's own proven pattern — "the take raising ch 0x3F on the pass").
6. **Mapped the take zone with a geometry probe** (`probe_0b_take.rs`, fresh load, no guard
   fire — evidence, not a route stand-in): ch 0x3F fires from essentially the whole band
   z ≥ ~41.0 (x 1.5-3.2), but the **body pushing north from the south is walled at z 40.99**
   at x 2.0-2.6 — the south approach stalls one tick short of the firing band. The d1 walk (same
   code, green) still takes it on the pass, so a crossing x exists; the probe's first push sweep
   used the wrong yaw convention (the walk's yaw is `atan2(dx, dz)`; fixed in the file, not yet
   re-run).
7. **Ruled out the armour detour**: the disc's body armour (kind 3, ch 0x0) at (17.53, 47.68,
   28.46) exists, but the east lane to it is held by leftover guard #1 (state 0x5c at (10.8,
   47.78, 11.9)) plus GT turret #293 at (10.13, 49.06, 8.06); the detour walk died there
   (31 hits for 100.0 at tick 19554). Abandoned (an armour approach would need its own fight
   plan for a lane the game's own route never takes).

## How it was verified

- Every number above is from a run of the real level through the real input path
  (`walkreal_0700000b_live`, `--nocapture`), the sim's own state dumps (the `[0b-diag]` block),
  or the disc/decompile citations inline. The take-zone probe is a fresh-load geometry sweep,
  labelled as such.
- The pass-through sprint change is **work in progress**: the agent walk still dies at the same
  spot (2.26, 58.11, 40.99 — the wall), 21 hits, tick 15851. Nothing got worse; the d1 walk was
  not re-run after the change (time), so its green state is unverified on the final tree.

## What's left (the next window's plan)

1. Re-run the fixed `probe_0b_take` push sweep (yaw 0 = north) to find the x where the body
   crosses the z ≈ 41.0 face line, and check whether the crossing needs run speed (the probe
   pushes from a standstill; the sprint arrives at ~run speed).
2. Route the deck sprint through that crossing x (bend the final leg, e.g. via the west lane
   x ~1.0-1.4), keep the take-on-the-pass assert. Expected margin: the 95-tick stand cost 38.8;
   removing it should leave ~42 health on Agent.
3. Cut the turret pair's 19.2 on the dome-floor transit if cheap (a lane out of #174/#175's
   early cone); not required if 2 leaves enough margin.
4. Run the 00 Agent sister (Bond's pain scale 1.0 there — Plr_DMod_Hard — so the roof cost is
   ~1.25× the Agent run's); re-tune if it needs the turret saving too.
5. Re-run the d1 real-damage walk (the sprint change touches its shared leg) and then the whole
   `walkreal_0700000b_live` binary; the change touches only this test file, so no other level's
   walkreal is affected, but the full suite must run before any merge.
6. No draw changes were made or needed; the parapet sight is game-verified (los-mask-join's
   live check).

## Live checks for the main agent

None this window — the one question the route work turns on (a body crossing of the housing's
south face line z ≈ 41.0) is answerable from the port's own collision data with the fixed probe.
If the fixed sweep finds NO crossing x anywhere, a live check of a player walking into the
housing's open south face on the disc (does the body pass where the port walls at z 40.99?)
would separate a port collision bug from a route question.

## Usage

not visible to me — this session's environment shows no token, credit or context-window figures
(no usage display was available anywhere in the interface I worked from).
