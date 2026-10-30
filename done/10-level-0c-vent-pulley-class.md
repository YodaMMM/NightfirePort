## Your task: `level-0c-vent-pulley-class`, port the 0xE2 vent/pulley and walk the display-eye leg

level-0c-real-crossings-2 (merged) leaves the 0c whole walk green with stand-ins; one is the display eye + the plates reach, carried because the 0xE2 vent/pulley class is not ported (the pad riser is capsule-blocked [D probe]). Port the 0xE2 class from the decompile [M its create/update fns; D the 0c placements], then replace that stand-in in `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with the real-input leg. The walk must stay green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-vent-pulley-class`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T06:10Z

---
## Report (done, 2026-10-30T08:59Z)


---
## Report from the worker (done, 2026-10-30T08:59Z)

# level-0c-vent-pulley-class — report

Task: port the 0xE2 vent/pulley class from the decompile, then replace the display-eye +
plates-reach stand-ins in `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`
with the real-input leg, keeping the walk green.

## What was done

**1. The 0xE2 class did not need porting — it is already in the tree.** [M]
`SS_Create` 0x1a9c58 / `SS_Update` 0x1a9948 / `SS_Operate` 0x1a9df8 / `SS_Activate` 0x1a9ea8
are the existing `crates/nf-game/src/ss.rs` create/tick/activate (the module header cites the
addresses; the grates-02 and fix-0c-hatch tasks landed them). The task's premise ("the class
is not ported") was stale: the ST-1 vent lid #138 "ventmesh-front" and blocker #179 are
already swept by the walk's own laser-watch leg.

**2. The 0c 0xE2/0x2e census [D disc placements, dumped from 0700000c.bin]:**
- `#351 obj_stor1_pully02` (5.92, 14.50, −115.90), `#364 obj_stor3_pully01` (4.73, 11.08,
  −132.80), `#377 obj_stor5_pulley` (3.79, 14.46, −146.26): each carries FOUR keys at the
  SAME position, the only motion a ±0.026-rad wheel sway (quat z), param 2 mask 0x2 (the
  bullet-contact operate class [M 0x1a9df8:35-37]). Dead props — **no travel lift exists on
  the level**; the old "vent/pulley ride" reading of the display-ground access is wrong.
- `#180 "ventmesh"` kind 0xE2 at (53.80, 5.50, −163.59), ch 0x54, 6 keys rate 50 — the lid
  swings up around +X; `#182 "VentLatch"` kind 0x2e at (53.80, 5.90, −165.36); `#184
  "VentmeshBlockingPloy"` keys 2 rate 10 sweeps y 5.43 → 0.00. This is the game's own
  annex vent at the warehouse (the hole down to the 0.90 floor), warehouse-side.

**3. The display eye stand-in is REPLACED by a real-input leg (the task's core).** The
game's own route to the display ground is the staircase, not a ride: the 11.27 hall floor is
missing at x 9..12.4 for z −115.5..−131.5 [D probe rays] and `obj_stor2_staircase01` #359
(kind 0x19 mesh, bounds x 9.97..12.29, y 6.13..12.14, z −127.5..−117.1 [D placement]) spans
the opening — landing 11.12..11.29 at z −127..−128 (level with the hall), ramp down to 6.29
at z −120 [D probe rays]. The walk now: the upper east lane south to the stair head → west
onto the landing → down the ramp (5 real walk_to waypoints) → east onto the south ledge
(6.27) → north to the display pocket → the eye spot (5.4, 7.35, −113.0) → the REAL camera
switch + photograph through `CamSubject_Update` 0x1bc668:14-24 → objective 2 completes
[M 0x1827d8:88-99]. Sister pins added: the shaft has no hall-height floor at (11, −122), the
landing exists at (11.2, −128), the ramp's mid surface at (11.2, −124), and the ledge's west
shaft is still open at (11.8, −115.5) — the route cannot silently turn back into a walkable
floor. No teleport remains anywhere on legs 1–3.

**4. The plates reach keeps the master stand-in, now with closure evidence** (this session's
probes; the stand-in teleport is byte-identical to master's):
- The slab's south face stands at z −155.77..−156.26 from y ~2.5 up across x 2.5..15.25
  [D probe rays], leaving only a 0.23..0.4 slot above the 2.37 deck — no standing or crouched
  capsule passes; the raw south walk stops at the edge z −155.04 standing AND crouched
  [probe runs x 3/x 12].
- The 2.18 deep-floor lane (x 8.5..12.2, the shaft's bottom — reachable: a crouch-walk off
  the display ground's south edge at x 10.5 lands grounded on it at z ~−121) runs south only
  to z ~−150 and steps up onto the same 2.37 pocket, closed south by the same face [D probe
  rays: down z −152..−155.7 floor 2.37, face y 2.6 at z −155.77].
- The east deck pocket (2.37, x 12.5..15.4, z −154..−155.5) is the same pocket, closed south
  [D probe rays]; south of z −156 it has no floor at x ≥ 10.5.
- The annex floor (~5.44..5.99, x 52.5..55.5, z −162..−166) hangs unreachable from the
  display ground too: open air over void at x 15..45 (no floor between y 8 and 0.3), faces at
  x 14.9/31.7/45.2 at y 6.3 [D probe rays]. The vent #180/#182/#184 chain is the
  WAREHOUSE-side connection (drop 5.5 → 0.90 through the hole), i.e. it serves a future
  warehouse-approach leg, not this one.

## Verification

- `walkreal_0700000c_live`: **2 passed, 0 failed** (15.9 s) on the final tree (merged with
  origin/master 535b7d15 = master's batch-1030a).
- Full suite (fast runner, `tools/test/suite.py`, --test-threads=1, NF_ISO=the onfoot ISO)
  on the final merged tree: the run was still executing at this sandbox's deadline — frozen
  at **969 passed, 3 failed, 88 ignored** over 275 test binaries (~08:58 UTC; the per-binary
  sections and the partial sums are in ../suite.log). The 3 failures + 2 crashes:
  - `scratch_l14d_live::l14d_lane_drop` — a FOREIGN untracked leftover of another task's
    session sitting in the working tree (crates/nf-view/tests/scratch_l14d_live.rs); it is
    not on this branch and was moved out after it ran. Not a tree failure.
  - `fix_0c_pylon_live` × 2 — the known environmental pylon fail (level-0c-real-crossings-2's
    report records the same: "the partial suite 446/911 with the environmental pylon fail").
  - `CRASH mp_stress_explosives_live (exit -9)`, `CRASH mp_online_failures (exit -9)` — both
    SIGKILLED by the sandbox (heavy stress binaries; exit -9, no assertion output).
  The touched binary `walkreal_0700000c_live` ran standalone on the final tree:
  **2 passed, 0 failed (15.9 s)**; the suite had not reached that binary when the window
  closed (the runner goes crate by crate; nf-view's walks run late).

## What's left

- The plates-reach real leg: walk the warehouse side (the east-strip yard descent → the
  warehouse floor → up to the annex? or the plates directly) and then the annex vent chain
  (#182 latch → #180 lid sweep → the hole down to 0.90) as the game's display→warehouse
  connection. The 0xE2 machinery for the sweep is ported and proven (#138/#179, the 08 safe
  lid); only the route walking is missing.
- Live checks for the main agent (optional, none blocking):
  1. At the display pocket after the photograph, walk Bond east along the south ledge to
     (13.5, −150) and try to drop off the slab's south edge — expect the game to allow the
     drop to the 2.37 deck (the port's walker refuses; the slot evidence is ray-level, not a
     live measurement).
  2. At the warehouse, E the VentLatch #182 (53.80, 5.90, −165.36) and watch lid #180 —
     expect the swing-up over 50 ticks [M SS_Update rate semantics].

## Usage

not visible to me
