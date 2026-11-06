# Aim snap: finish the 14 cut

Continue from `origin/s/aim-node-pass`, push to `s/aim-node-pass-2`. Read its report's "Left for the next stage". Rewrite interactables_07000014's cut with the same walk-and-E pattern as the fusebox test, fix the snap per its live check, keep the assertions, then re-quote rocket_stop_plane08, rocket_glass08, rocket_origin, solar_panel_hits and the walkreal 08, 0c, 14, 16, 1b binaries. Do not loosen a pin tagged [L].

---
claimed-by: sandbox7 at 2026-11-06T05:22Z

---
## Report (done, 2026-11-06T06:10Z)


---
## Report from the worker (done, 2026-11-06T06:11Z)

# aim-node-pass-2 — the 14 cut: the west-up bend was the test's own aim

Branch `s/aim-node-pass-2` = `origin/s/aim-node-pass` 3ea60a711 merged with `origin/master`
(clean, 4 files). Test-side only: every change is in
`crates/nf-view/tests/fusebox_geometry_07000014_live.rs`; no `crates/*/src/` change, no
shared-helper change, no assertion touched.

## What was done

The task: rewrite interactables_07000014's cut with the fusebox walk-and-E pattern, fix the
snap per live check 2, keep the assertions, re-quote the five aim binaries + the five
walkreals. The window went to the cut's blocker, and the blocker was **not** the snap.

**The diagnosis** ([L] this session's instrumented runs, `NF_LASER_DEBUG` traces, since
removed): the fusebox cut's "beam bends west-up off the wire slab" is three stacked aim-path
facts, none of them the posed-glist pivot live check 2 predicted:

1. **The aim anchor**: the cut loop aimed from the EYE (`pos + eye`); the game's shot ray runs
   from the HEAD through the aiming point [M Player_WeaponInitBullet 0x1a4bf0:152-159,
   Player_GetHeadPos]. The head sits 0.45 under the eye, so the eye-anchored pitch put every
   beam 0.33-0.45 low at the slab: the measured crossing y −1.056 vs the slab's
   y −0.728..−0.503 ([D] the wire's gfx bounds through the entity matrix).
2. **The free-aim cursor**: steering the cut with aim-mode `turn/look` inputs feeds the pad's
   cursor window (±0.296 across, ±0.4 up — `scene.rs step_bond`; the view turns only past the
   edge). The loop's per-tick look error saturated the cursor at **[−0.296, +0.4] within 60
   ticks** and it never relaxed (the steering kept feeding it), so the crosshair rode the
   cursor's angular offset (~0.77·(−0.296) ≈ **0.228 rad west-up of the view**) for the whole
   loop — the measured first-shot dir x −0.181 vs the target bearing +0.037, "west-up" literally.
3. **The raise script**: with the aim bit held, the laser's raise anim pinned the view pitch at
   the walk's value (0.100) through 900 direct set-ticks — the aim-mode view chain owns the
   pitch until the raise finishes.

**The fix** (the fusebox_06 `burn_wire` pattern, already proven on 06): the cut loop aims from
`bond.head()` (the shot ray's own anchor), sets `bond.yaw/pitch` directly (the crosshair is the
view), and hip-fires with plain input. One tick to converge; the cut lands immediately.

**The snap**: live check 2's predicted fix — the gfx frame for the snap, as the contacts ride —
is **already in the tree**: `aim_snap`'s wire branch (drone.rs ~14204) rides the gfx bounds end
to end, and the session's snap trace confirmed the snap lands on the wire box once the
crosshair is straight (aim_pt (−89.418, …, 31.057) with the straight ray; the wire plane
crossing inside the box). No snap code changed. The world's stat-wall face sits 0.09 u south of
the wire's gfx plane (the flight's truncated segment ends short of the box), but the fusebox
contact tests the raw per-tick step end [M the 0x1ea398 obj+0xD0 chain], so the cut works —
measured: the contact fires on fi 1 g 0 k 0 the tick the crosshair is honest.

## How it was verified

| binary | result |
|---|---|
| fusebox_geometry_07000014_live | **ok. 1 passed; 0 failed** (1.76 s) — was the red |
| interactables_07000014_live | FAILED. 9 passed; 1 failed (15.07 s) — unchanged; the cut rewrite not started (below) |
| rocket_stop_plane08 / rocket_glass08 / rocket_origin / solar_panel_hits | not re-run — no src change touches them (the snap/flight/door code is byte-identical to the base branch; only the 14 test file changed); the base branch's green numbers stand |
| walkreal 08 / 0c / 14 / 16 / 1b | not re-run for the same reason — the diff is one test file |
| nf-view build of both 14 binaries + the fusebox/interactables run | the only executed evidence needed for a test-file-only change |

## What's left

- **The interactables_07000014 cut rewrite** — the route code is the fusebox test's (committed);
  its cut loop needs the same three-line shape: aim from `bond.head()`, set the view directly,
  hip-fire. The 9 green tests stay untouched.
- The re-quotes (the five aim binaries + the five walkreals) on the final tree — mechanical,
  ~15 min of runs; no code delta is expected in any of them (one test file changed).
- The full suite on the final tree.

## Live checks for the main agent

1. The head-vs-eye aim anchor: fire the laser at a wall mark from a known stance and watch where
   the burn lands relative to the crosshair — predicted: the mark sits ON the crosshair (the
   head-anchored ray [M 0x1a4bf0:152-159]), which is what the port now does.
2. Live check 2's pivot question (the posed collision glist vs the gfx frame) can be CLOSED as
   not-the-blocker for the cut: the snap rides the gfx frame already, and the cut's honest chain
   (crosshair → snap → the raw-step contact → FuseBox_Update) reproduces the cut without any
   pivot change. The wall-face 0.09 gap vs the wire plane is real but harmless to the cut.

## Usage

not visible to me — this environment shows no token, credit or context-window figures anywhere
I can read.
