# level-4a-every-objective-4 — the scoped drone bullet flight with the 08 fusebox walk green

Branch `s/level-4a-every-objective-4` = origin/master (814c8976) + the -2 scoped bullet-flight
hunk (cherry-picked as the single bullet.rs hunk from `origin/s/level-4a-every-objective-3` —
merging -3 wholesale would have dragged its older-master test tree over master's newer walks)
+ the 08 walk hardening. `walkreal_07000008_live::walk_the_whole_mission_on_real_input` now
walks the WHOLE mission on real input with the game's cel-scoped drone bullet flight:
**3 passed; 0 failed**, the mission walked in 31855 ticks, falls [].

## What's done

1. **The scoped flight is on the branch and stays.** Master's `Bullet::update` flew every round
   (player and drone — "Drone-fired rounds run the same Bullet_update") through the FLAT
   `segment_bullet`; the -2 change queries
   `segment_bullet_scoped(world.find_cel(self.pos), self.pos, end)` — the game's cel-scoped pick
   [M Bullet_update 0x12e7b0:343 seeds `Collide_RayIntersect` with obj+0x20, re-fed from the live
   position each tick (disasm 0x12ef04 `lw $a2,0x20($s5)` + 0x12f0cc-0x12f0d8); Collide_Pick
   0x1eb320:125 → Collide_StraddleCels 0x1eac98]. Carried onto master verbatim (one hunk,
   `crates/nf-game/src/bullet.rs:210`).
2. **The repro, on the merged tree**: `walk_the_whole_mission_on_real_input` fails at "shooting
   the fusebox wires raises ch 137" (2 passed / 1 failed) — the -3 result, reproduced.
3. **The mechanism, measured this session** (temporary instrumentation, removed from the tree;
   the flat/scoped A/B is two runs of the same test with only the bullet.rs hunk toggled):
   - The wedges the -3 report blamed are NOT the delta: the flat (green) run wedges the SAME
     legs ("the station doors nav 10" and "the lower landing" give up at the 3000-tick cap on
     BOTH trees) and the walk tolerates them by design. The one divergence is the NEXT leg,
     "by the fusebox nav 1" into the stair down to the walkway: flat arrives
     (146.15, 43.26, −119.85) at +85 ticks; scoped drops Bond into the station's pit — measured
     y 44.85 → 37.13 at (148.18, −119.16), floor 36.0 — and the leg then fires the laser from
     the pit: muzzle y 38.26, the 4.0-range segment [L weapon_data +0x4E·0x10C +0x1C] topping at
     y 42.0, 1.4 below the wire box (154.35, 43.41, −119.67 [D]) — ch 137 never rises.
   - **What changes for the walk**: under the scoped flight the station guards' rounds REACH
     Bond on the approach — the instrumented run counted 228 `PlayerImpact` hits on Bond by the
     lower-landing wedge (the -3 probe's 838 drone-round events, confirmed at the impact
     level). Every hit runs `Player_HandlePain`'s 1-in-4 pain-grunt roll [M 0x1902c8:111 — the
     port's `PlayerHealth::apply`], a draw from the shared bit-exact RNG the guards' AI also
     draws; under the flat flight those same rounds die at walls (no hit, no draw). The AI's
     random decisions (paths, positions, door approach timings) therefore diverge, the walk's
     stuck-mitigation sidesteps land on different spots, and the lower-landing wedge spot —
     IDENTICAL starts, (146.35, 44.85, −115.82) scoped vs (146.34, 44.84, −116.41) flat —
     changes the bearing into the stair. A mitigation sidestep at the stair's edge drops Bond
     into the pit. Which guard fired the rounds: the station's placed wing (sw_ch 42 wakes on
     the ch-0x2a plate [D]); the trace shows guard #8 standing at (146.4, 44.2, −117.2) beside
     the stair through the wedge — the rounds come from the woken station wing, not one
     sniper. What the rounds hit: Bond (228 impacts, damage 2.0 each, suppressed by the walk's
     top-up) and the station walls — **no breakable dies** (the dead-breakable dump at the
     fusebox moment is EMPTY in the scoped run — the walkway floor is world geometry, not
     glass), no door state flips (the stuck dumps' doors-nearby lists are empty at every wedge),
     and no explosion is involved (the port's `spawn_blast` applies pain only — no velocity
     kick [M the port's drone.rs:13215 body; the game's blast stagger is a drone-side anim]).
   - **What the game does there**: the flight rule is the game's [M 0x12e7b0:343 + 0x1eb320:125]
     and the -3 report's live checks stand (the real console's station is hot; the walk's
     pit-fall is the WALK's fragility, not a port AI bug — no port AI side was found wrong:
     no knock, no false pain stall — the player's movement code never reads pain — and no
     guard geometry blocks the capsule, which the port never registers into the collision
     world, same as before this task).
4. **The fix — the walk's approach hardened for the hotter station** (test-side only,
   `crates/nf-view/tests/walkreal_07000008_live.rs`; the flight, the AI and the shared walker
   are untouched):
   - The last two nav legs into the fusebox are replaced by the measured green line's
     square-up waypoints: `the stair head` (146.6, 44.4, −117.6, r 2.2) → `the walkway west`
     (146.3, 43.3, −119.8, r 2.0) → `the walkway line` (149.5, 43.2, −120.3, r 2.2) → `by the
     fusebox` (153.0, 43.4, −118.5, r 1.5) — each an explicit `walk_to` whose arrive radius the
     AI wobble cannot outrun: a leg that arrives within its first stuck sample never enters the
     mitigation ladder, so no sidestep can fire at the stair edge. The descent line is the
     green run's own crossing (its nav-1 arrival (146.15, 43.26, −119.85) sits on the walkway's
     west end after a straight south descent — measured, quoted above).
   - A retry version was tried first (re-walk the legs on a wrong stance, walk back to the
     lower landing between attempts) and REJECTED on evidence: each attempt re-randomises the
     same wobble (attempts 2/3 ended at y 46.8 / 46.4 with dxz 15.9 / 12.8 — worse), burning
     ~50 s per roll. The square-up kills the fragility instead of gambling on it.
   - The shot's stance is pinned by the run itself: the fusebox fire now happens from
     (152.39, 43.10, −119.78) — the walkway level, beside the box — and ch 137 rises.
5. **The rest of the mission stays green on the scoped flight**: the safe bolts ("bolt 0",
   "bolt 1"), the briefcase stretch (26446 ticks), the ride ("bond [111.0, 13.5, −179.7]"),
   the Hind shoot-down on real aim (23 trigger pulls, 360.5 damage vs 299 hp), falls [].

## How it was verified

- `cargo test --offline --release -p nf-view --test walkreal_07000008_live -- --test-threads=1`
  on the final tree: **3 passed; 0 failed** (91.58 s) — the whole-mission walk green WITH the
  scoped flight; the other two tests in the file (the ally pins) green.
- The A/B: the same binary with only the bullet.rs hunk reverted → the flat tree stays green
  (the filtered single test 1 passed; the full file 3 passed pre-change) — the hunk is the
  delta, the fix does not depend on the flat flight.
- The full four-crate suite (the fast runner): see the Suite section.

## What's left

- Nothing in the task's scope. The -3 report's live checks (the real console's station heat,
  the real fusebox stance) remain the main agent's items — this session's measurements sharpen
  them: the game's player crosses the same stair under live guard fire, and the wire cut must
  happen from the walkway level (a pit shot cannot reach [L]).

## Live checks for the main agent

1. 07000008 fresh boot: stand at the lower-landing wedge spot (≈146.3, 44.8, −116) under live
   guard fire — do the station guards' rounds hit the player on the stair (the port's scoped
   flight says yes: 228 impacts by the wedge)? And does the real game's pain flinch the player's
   walk (the port's pain is health+flash only — no movement stall [M the port's apply()]?
2. 07000008 fresh boot: the fusebox cut — confirm the player cuts the wires standing on the
   walkway beside the box (y ≈ 43.1–43.4), not from the pit below.

## Suite

The fast runner cannot run in this sandbox (background processes die between tool calls — the
nohup'd suite died twice with an empty suite.out), so the sanctioned chunked variant
`tools/test/chunk_suite.py` ran (same build, same per-binary limits, state across calls;
`--budget 540` to fill each call). The build: **915 test binaries in 544 s** after the
bullet.rs change rippled through nf-game. The run's totals as of the stop line are quoted at
the tail below. The touched binary `walkreal_07000008_live` is green in its own run (above).

## Usage

not visible to me
