# level-14-claw-route-retune

## Done

1. **The claw question answered from the decompile** (the driver's first ask): the game's claw
   does NOT reach the SkyBridge grapple point #658 (45.2888, 5.0961, 62.9362) from the duct
   stand — the duct wall really is in the way. The mechanics, with addresses:
   - The bolt dies on its first solid contact: `Bullet_CollisionHandler` 0x12ce80 sets the
     bullet's state +0xf4 to 2 on a solid hit, and a state-2 bullet gets +0xf0 |= 0x10 at the
     handler's tail (the delete flag) [M 0x12ce80].
   - The latch: `Bullet_Delete` 0x12e660 only calls `Player_SetGrapplePoint` when the hit
     object's kind byte +0xff is 0x51 ('Q'), passing the bolt's +0xc0 position and the object
     [M 0x12e660:50-55]; `Player_SetGrapplePoint` 0x195da0 stores the point at BLData+0x140,
     the object at +0x154, and ChangeSubState(2) [M 0x195da0].
   - The reel: `Player_SSGrapple` 0x192860 pulls 0.05 → cap 1.5/tick (SP, MPSettings==0)
     toward the point and hangs at dist < 2.0; the release is the phase-2 action-9 branch
     [M 0x192860:56-66, :79-121].
   - The geometry: from the duct stand (40.8, 2.40, 65.5) the ray to #658 crosses the wall
     plane x 41.53 at (41.53, 2.84, 65.08) — inside the duct east wall's band (mat 5, floor to
     roof) — so every bolt dies on the wall before reaching the point. The port's
     `interactables_07000014_live::the_qclaw_aimed_through_the_duct_wall_does_not_latch` pins
     the same result and passes (10/0). **The crossing stand-in stays.**
2. **The `_agent` sister is un-ignored and green**:
   `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_agent` walks
   out in 22633 ticks ("walked out ... 07000015 loaded warm"), 0 stuck, 0 falls. The pit
   band's east jump the ignore note named is no longer failing on this master tree — no route
   change was needed for it. The two new real-input laws below run behind it and did not move
   its deterministic path (same 22633-tick total as before them).
3. **Two real-input laws added for the real-damage descent** (both `if !w.tops_up()`-gated so
   the master/top-up walk keeps its timing exactly):
   - The room guard #284 (drone 5) is killed from the machine-room lane start (34.01, 0.65,
     33.21) — the master-proven kill spot — after `switch_to_armed`. Measured basis: on this
     tree the guard wakes under the game's sight while the drop loop perches on the landing's
     west edge and deals the full 100.0 in 50 hits by tick 4104 (Bond still at y 2.35 on the
     edge); the landing perch shot cannot land (the landing slab's west face, top 2.37, eats
     the floor-level ray — the converged-aim no-damage loop in the trace); `clear_hostiles`
     from the landing is wrong here (its loot pass walked Bond into the south gap's ambush —
     death at tick 5926, 0.3 from #10's stand (33.92, 30.85)).
   - The hall guards are cleared before the box A fusebox cut (two `clear_hostiles(24, 900)`
     passes at the hall west end), because with the enemies alive the cut window is not quiet
     (see What's left).
4. **The plain real-damage walk's state is documented in place**: the OLD ignore blockers are
   gone on this tree (the descent lands (29.51, 0.65, 37.77), inside the pin box; the leg-2
   start and the Kiko band are reached; the walk gets to the box A cut at tick ~33300). One
   real-damage blocker remains (below), so the test stays `#[ignore]` with the measured state
   and next levers in its note.

## How it was verified

- `walkreal_07000014_live` whole binary: **8 passed, 0 failed, 2 ignored** (the plain
  real-damage walk and the 00 Agent walk — both with measured notes), 130 s, `--test-threads=1`.
- `interactables_07000014_live` whole binary: **10 passed, 0 failed**, 11.9 s.
- The master top-up walk `walk_the_infiltration_to_the_delta_exit` re-run green after the
  changes (the gates keep its timing; the descent pin and both fusebox cuts pass as before).
- Crates touched: only `crates/nf-view` (one test file). Rule-5 command run for nf-view test
  builds (`cargo test --offline --release -j2 -p nf-view --test ... --no-run`).

## What's left

- **The plain real-damage walk dies in the box A cut window** to the GENERATOR BLAST: a world
  explosion at (-88.7, 0.9, 32.6), radius 10.0, 500 base damage, d 2.4 from the stance
  (-88.38, -1.07, 31.34) [L NF_DMG_TRACE] — death at ticks 33324/33336/33348 across three
  runs, identical 40 hits for 100.0 and 13 return-fire fights. Master never sees it (its
  enemies are disabled; its cut is 44 quiet ticks; its h_mid asserts pass). The pre-cut clears
  find nothing within 24 — the igniting round is not a cleared guard's. Levers: identify the
  igniting round (NF_DMG_TRACE shows the blast, not the bullet — candidates: the Laser spray
  blocked by a fight corpse at the stance, or a beyond-24 sniper's over-flight), a corpse-free
  stance candidate in `cut_fusebox`, or a wider pre-cut clear. The blast is the game's own
  object destruction — never lower the damage.
- The `_00agent` sister keeps its own ignore (the hover-dither wedge — out of this task's
  scope, unchanged).
- **The fast runner did not survive this sandbox**: `suite.py` was started in the background
  in the first minutes (15:52 UTC) but was killed between tool calls (`suite.out` empty, no
  process left — this sandbox kills background jobs between tool calls; same as the previous
  session). In its place the two touched binaries above were run whole, plus the rule-5 build
  command for nf-view. The final-tree full suite re-run is the next session's first step.
- Live checks for the main agent: none required — no visual work; the blast source question
  would benefit from `tools/trace/gdbwatch.py` on the generator object near (-88.7, 0.9, 32.6)
  (which instruction lights it) if the next session wants the answer the fast way.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for
the session.

## ntfy failed

All three notification paths failed from this sandbox (ntfy.sh is unreachable from here; the
same as the previous sessions):

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-14-claw-route-retune: ..." https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`, exit code 28.
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'level-14-claw-route-retune: done', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`.
3. `wget -q -O- --post-data "level-14-claw-route-retune: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output, killed by a 25 s timeout cap (exit 124; the connect hangs past the driver's
   own 20 s window).
