# port-map-bullets — the bullets-family function map

Branch `s/port-map-bullets` from `origin/master` (22d5836b7). The table is
`docs/port-map/bullets.md`, in the drones-map format (one line per function: ported with
the port location, or not reached with the proof).

## What's done

1. **The family enumerated**: every `Bullet_*`, `Gun*`, `Weapon*`, `Explode_*`, `Effect_*`
   function in ACTION.ELF — **32 in total** (13 `Bullet_*`, 5 `GunImp_*`, 5 `Explode_*`,
   9 `Effect_*`; no `Weapon*`/bare `Gun*` names exist in this build — `Player_Weapon*` is
   weapon-switching's map, `DroneWeap_*` the drones map's). DRIVING.ELF and the SLUS ELF
   carry none of the family.
2. **Every address swept in `crates/*/src`** and each citation resolved to its
   implementing code by reading it (comment mentions alone not accepted). Result:
   **31 fully ported, 1 ported-with-a-gap-slice, 0 not reached** — all 32 are dispatched
   on real levels (bullets fly, casings eject, blasts fire, impacts draw, 0700000d's
   mount reaches the GunImp set).
3. **An independent audit subagent** re-read the 8 thinnest rows against their decompiles:
   7 stood (two with folds worth naming: the spawn-time propagate for
   `Explode_CollisionHandler`'s once-latch, the gate-release restore for
   `GunImp_Deactivate`); **2 defects found and fixed**:
   - `Bullet_trail_effect_smoke` cited `init_gt_smoke` — the real fn is
     `init_trail_smoke` (gas.rs:243, the row now says so).
   - `Bullet_heatseekable` overclaimed ("plus copter bodies and Bond"): the decompile has
     FOUR arms; the port had only the class-2 drone arm — the rest was comment-only.
4. **The gap proved, then fixed (own commit).** Two new opt-in probes:
   `homing_record_probe.rs` — the 0x20000 homing carriers in weapon_data are
   **0x2e, 0x30, 0x45, 0x6d** [L ee_start_05.bin @0x2bf150, 115 × 0x10C]; and
   `copter_homing_probe.rs` — the copter-vs-pickup sweep: **0700000b copter #148 + the
   0x2e pickup #15d; 07000012 copter #707 + pickups #2aa/#2f0** [D]. So Bond-owned homing
   rounds fly where a copter lives: `Bullet_heatseekable`'s class-0 arm (the CopterList
   walk — a copter BODY is seekable while drawn, NO health read [M 0x12f248]) is REACHED,
   and the port had no copter arm. Fixed in the `PlayerBullet` homing block
   (drone.rs: `target_copter` selects the index space, the copter bodies scan after the
   drones with the same 1000-unit/0.66-dot/LOS gates, the steer reads the body's live
   pose); `nf-game --lib` 686 passed 0 failed with the fix in.
5. **The live red/green harness is on the branch, `#[ignore]`d with its blockers written
   down** (`homing_copter_07000012_live.rs`): it loads 07000012, equips the Scorpion
   through the panel-give arm (`equip_weapon(..., true)` — SOLVED this round; the loadout
   case never carries the 0x2e), holds fire off-axis inside the 0.66 cone, and asserts the
   round turns onto and locks the copter body. The OPEN blocker: the port's `Copter.matrix`
   reads (0,0,0) at load on both census levels — the copter waits on its gate channel
   (07000012 gate 0x25, 0700000b gate 0x2a) and 600 ticks of a raised gate never posed the
   body (the flight script's own chain must run; the dressing body rides script entities
   per `Copter_Register` 0x17e530). The next round drives the activation chain (or reads
   the body part entity's pose) and runs it red-then-green.

## What's left

- The class-3 arm of `Bullet_heatseekable` (Bond, `BLData+0x894 > 0`): comment-only in SP,
  present only in the MP fold (`mp/combat.rs:462-489`). REACHED by the copter's own 0x6d
  rounds at Bond on non-07000008 levels [M copter.rs:147 — the copter fires 0x62 on
  07000008, 0x6d elsewhere; 0x6d carries the homing bit [L]]. This is the family's one
  missing slice — next round.
- The live red/green run (blocker (2) above).
- The full suite did not run in the window (this sandbox kills background jobs between
  tool calls; the foreground chunks did not fit). What ran: `cargo test -p nf-game --lib`
  (686 passed, 0 failed), the compile of every touched test binary, and the earlier
  task-shared binaries were not touched this session (no shared helpers changed).

## Live checks for the main agent

1. On 07000012 (or 0700000b), give Bond the AT-600 Scorpion (0x2e), stand ~140 units from
   the parked/hovering helicopter, fire past it inside ~45°: the round should visibly
   curve INTO the airframe and detonate on it. A screenshot before/after the turn pins the
   render too.
2. With the debugger: while a homing round flies, watch `BU+0x40` (the bullet's target
   pointer) — expect the copter body object's address once the class-0 arm selects.
3. The copter's own 0x6d missiles on a non-07000008 level: their `BU+0x40` should name
   Bond's object (the class-3 arm) — the port currently cannot (the missing slice).

## Usage

not visible to me (no token, context-window or cost figures are exposed to this session;
the driver's message carries no numbers for this worker).
