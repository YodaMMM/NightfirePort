# 0700000a: a guard has no animations

The guard seen through the window from (4.59, 0.65, 2.43), about frame 7485, slides with no animation. Find which guard and state, and why no animation is set. If animation state is right and only the draw is wrong, say so.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-0a-guard-no-anim`, report in docs/reports/level-0a-guard-no-anim.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-07T23:48Z

---
## Report (done, 2026-11-08T00:47Z)


---
## Report from the worker (done, 2026-11-08T00:48Z)

# level-0a-guard-no-anim — the sliding window guard (0700000a)

Task: the guard seen through the window from (4.59, 0.65, 2.43), about frame 7485, slides with no
animation. Find which guard and state, and why no animation is set. If animation state is right and
only the draw is wrong, say so.

## Verdict

**The animation state is right; the draw side is wrong.** The sliding "guards" are the office
CIVILIANS (all 20 of 0a's drones are team-3 civilians — skins 0x5000029/0x500002a, weapon 0x1 = the
fists [L sim roster]). The one at the tester's vantage is the atrium-lane patroller **placement
#695** (sim index g7), in **AI state 0x14 CivilianPatrol playing the walk danim 392 (anim state
0x37)** — the walk the game itself plays. No animation "fails to be set": the anim machinery is
correct. What freezes him is the port's `drawn` flag (the stand-in for the game's `obj+0xFC`), which
reads false whenever the sightline carries a thin occluder — and the security centre's west sightline
carries two.

Per the task's "no draw changes": this report names the draw-side divergence and does not fix it.

## The mechanism, with addresses

1. **The game**: `obj+0xFC` is the *draw pass's* output — `View_AddObjects` 0x1e62d8:71 writes 2 into
   it when the object's world sphere passes `Vision_InView` over the **portal-visible cels** [M; the
   same citation the port's own `visible_to` doc carries]. There is **no occlusion raycast** in the
   game's drawn decision: any guard whose cel the portal walk reached is "drawn" and animates, even
   when a thin post or a plant stands in the exact sightline.
2. **The port's stand-in**: `Drones`' control pass assigns `dr.drawn = dr.enabled &&
   visible_to(player, d.world, dr.pos, dr.head())` (crates/nf-game/src/drone.rs:20177). `visible_to`
   (drone.rs:11993-12002) = the camera frustum test (`in_view`, slack 1.0) **AND** at least one of
   five `segment_bullet_clear` rays (head, pos, ±0.5 side, pos−0.8 down) from the eye.
3. **The slide**: `Drone::update_anim`'s not-drawn arm (drone.rs:3001-3011) ports
   `NDrone2_DoAnimation` 0x149f88 [M]: a drone with `obj+0xFC == 0` playing a danim whose
   `Drone_AnimInfo` flag 4 is set and with no blend pending **does not animate — it advances along
   its yaw by the danim's speed** (0.02/tick for walk 392). That is exactly "slides with no
   animation".
4. **Why he is on screen at all**: the port's renderer draws every present+enabled guard —
   `char_hidden` (crates/nf-view/src/scene.rs:12004) checks `present && enabled` (plus the
   script/MP cases) and never reads `drawn`. So the civilian renders behind the thin occluder —
   visible, in a frozen walk pose, gliding. The game would have shown the same picture *animated*.

## The tester's instance

The readout (4.59, 0.65, 2.43) is inside **cel 29** (x −7.61..7.02, y −0.13..4.65, z −4.72..9.16),
the Office_N-security room — the security centre's floor is y 0 [D coll-scan; the sim's
`floor_below` reads 0.00017]. Looking west from there, the sightline to #695's patrol lane
(x ≈ −11, z ≈ 2.5) crosses, in order [D coll-scan + L probe rays]:

- **Blockinpoly_Holoprojector** (map-37 object 349: x −1.08..0.51, y 0.84..4.02, z 2.38..3.63,
  glist 0x1010, mat 0x40) — the translucent hologram panel: almost certainly the "window" of the
  report. Its material 0x40 carries the port's bullet-pass bit, so the drawn gate's rays pass it
  (`bullet_pass` = node pass || mat & 0x40, world.rs:370) — it is NOT the blocker.
- **LaserTrackVert** (object 562: the laser track's post, x −0.07..0.07, z 0.01..2.78,
  y 0.03..1.77, mat 0x06, glist 0x0) — 0.14 thick. All five gate rays to the patroller die on it
  (probe: head/pos/±side/down blocked at ≈ (0.5, 1.7, 2.5) [L run]).
- The one ray that misses the post dies further west on **office-N-ob2bigplant** foliage (object
  274, at ≈ (−7.2, 0.7..1.2, 1.7..2.0)) [L probe].

Five-of-five rays blocked → `drawn == false` every tick the patroller is behind them → the not-drawn
slide arm advances him 0.02/tick with the anim frozen [M 0x149f88]. In the game the post and the
plant are mere visual occluders; the patroller's cel is portal-visible from the security centre, so
`obj+0xFC` reads 2 and he walks. (Live checks below pin that game-side reading.)

The same divergence slides the other patrollers wherever their sightlines carry thin occluders. On
the plain real-input route to the security keyboard (no teleports) a temporary env-gated trace
(added to the shared walker for the run, reverted before commit) logged **2374 slide-ticks** across
g0/pl54 (932), g10/pl700 (918) and g7/pl695 (524) — every one in the view frustum on danim 392 with
`drawn` false [L run].

## Reproduce (committed)

`crates/nf-view/tests/window_guard_0700000a_live.rs` ::
`window_guard_slides_drawn_false_while_the_walk_danim_is_right` — **1 passed**. Real input only from
the level load (arrival plate → out of the cab → lobby → the crouched west-hall crossing under beam
#607), then a slow aim sweep across the security centre's west sightline while the civilians patrol.
It pins, at the first occurrence of the tester's symptom:

- a team-3 civilian **in the view frustum** on a **movement danim** with **`drawn == false`** (the
  slide condition `update_anim`'s not-drawn arm answers [M 0x149f88]);
- the animation state is the RIGHT one: AI state `0x14` CivilianPatrol, danim `392`, anim state
  `0x37` — the assertions that fail if anyone later "fixes" this on the anim side;
- all five gate rays blocked, each hit west of the eye or inside the LaserTrackVert post's column
  (the occluder family above).

This test is the red marker for the eventual draw-side fix: when `drawn` stops reading false for
thin-occluded sightlines, its `drawn == false` expectation goes red and should be flipped to assert
the fixed gate. No anim-side change can turn it green — which is the point.

Run: `NF_ISO=... cargo test --offline --release -p nf-view --test window_guard_0700000a_live --
--test-threads=1` → 1 passed (0.9 s). The route binary that shares the harness,
`walkreal_0700000a_live::walk_office_floor_to_the_security_keyboard`, re-run green after the
temporary walker trace was reverted (1 passed, 4.8 s).

## What is wrong on the draw side (named, not changed)

`visible_to` (drone.rs:11993) requires a physically clear bullet ray to one of five body points.
The game's drawn bit is a **per-cel** decision (`View_AddObjects` 0x1e62d8:71 over the portal-visible
cel set — no rays). The two disagree exactly when a *thin or translucent* occluder (the laser posts,
plant foliage, the hologram panels) sits in the sightline while the guard's cel is visible: the game
draws and animates him; the port marks him undrawn and the not-drawn arm slides him **on screen**.
The port already knows this shape of bug — the guards-0c fix moved the gate's rays onto the bullet
segment so pass-flagged glass/foliage drop out (the `visible_to` doc cites the 0c museum slide) —
but mat-0x06 thin posts and non-pass foliage still block. The faithful shape is the game's: the
drone's cel is portal-visible from the viewer's cel (the port's sim has no cel graph today; the
view side does — `CelGraph` + `portal_visible_cels_*` in nf-view), which is a draw-side change and
out of this task's scope by instruction.

## What's left

- The draw-side fix itself (per the task: not mine to make) — either a cel-visibility-based drawn
  gate or a thin-occluder-aware ray set.
- The live checks below to pin the game-side `obj+0xFC` reading at the tester's vantage.

## Live checks for the main agent

1. **The game draws the patroller through the panel.** PCSX2, level 0700000a, after the keyboard,
   stand Bond at ≈ (4.59, feet, 2.43) facing west. With `tools/trace/fieldprobe.py`, read
   **placement #695's obj+0xFC** while he walks the x ≈ −11 lane: expected **2** on every frame he
   is in the visible cel set (i.e. on screen through the holoprojector panel), never an extended 0
   while visible.
2. **The write source.** `tools/trace/gdbwatch.py` on that obj+0xFC word for ~10 s of the same
   stance: expected repeated writes of 2 from the draw pass (0x1e62d8:71's store), and no write of
   0 while he is on screen — the port's `drawn=false` interval is the divergence.
3. **The render comparison.** A screenshot from the same stance in the port: expected the civilian
   visible through the translucent panel, frozen walk pose, gliding westward at ≈ 0.02 u/tick; in
   the game the same stance shows him walking his gait.
4. **If (1) reads 0 in the game too** while he is on screen: then the game itself slides him, the
   port is faithful, and the tester's report describes game-faithful behaviour — re-check the
   tester's vantage before any port change.

## Usage

Token, context-window and cost figures for this session: not visible to me (no token/cost display
is shown in this environment; the IM gateway metadata carries no usage fields). Nothing estimated.
