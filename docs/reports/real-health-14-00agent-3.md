# real-health-14-00agent-3 — report (2026-11-04)

Branch `s/real-health-14-00agent-3`, cut from `origin/s/real-health-14-00agent-2` and merged
with `origin/master` (fast-forward — the -2 tip is already in master's batch-1104j merge
b57ad91a8). Task: settle the live-push vs dead-tail divergence [M Intersect_CylGeom
0x109da0:447], then the connector/descent legs, the d2 `with_real_damage` wrap, the d3
un-ignore.

## What's done

**1. The guard question is SETTLED (the task's first ask).** A subagent traced the whole
chain in the decompile; I verified the key files. The verdict:

- The push gate is **NOT** the +0x51 flag gate for the player walk. `Collide_Update`
  0x1ea398:94 passes the SELF HITTEST's `+0x9a` as `Collide_Intersect`'s third argument
  [M 0x1ebaa8:178 ORs it into the candidate's +0x51]; the player's HITTEST lives at
  BLData+0x740 [M Player_Init 0x1910f8:40], so +0x9a = BLData+0x7da — **no writer exists**
  (grep over all 3039 decompile files: zero hits; Player_Collision 0x194df0 writes
  +0x7d4/+0x7d8, never +0x7da; the obj+BLData block is memset-zeroed at creation [M
  control_create_object 0x1344e0:17]) — the player's gate value is 0x0000 forever.
- The pick's candidates arrive with +0x51 clean [M Collide_Pick 0x1eb320:199/303/348
  memset(0) and no +0x51 writes]. So `Intersect_CylGeom`'s push arm runs structurally for
  every walked contact.
- The REAL gate is the **ref-guard** inside that arm: the push fires only when
  `|q − HITTEST+0x50| < radius`, direction `(q−ref)/|q−ref|`, magnitude `radius − |q−ref|`
  [M 0x109da0:431-447], and a triangle that fails it is skipped entirely (no push, no
  best-hit record — the record tail is reached only through the push's `goto`). `HITTEST+0x50`
  boots (0,0,0) [L ee_start_0700000a 0x1b78ee0 = 00×16] and its ONLY writer is the DeltaP
  add-back [M Collide_Intersect 0x1ebaa8:353-373] which requires a prior push — a
  chicken-and-egg that never fires for walks far from the world origin (the port's own [L]
  evidence: 0a's walked contacts sat |closest| ≈ 10-25 u ≫ r 0.55). **The game's player-walk
  push tail is structurally dead on walked lines away from the origin; the live snapshot's
  ~15 m +0x50 drift [L ee_snapshot] says it can lock near the origin — where and when is a
  live check, listed below.**
- The position move is the CALLER's, not the sweep's: `Intersect_CylGeom` pushes only its
  stack-local copy [M Collide_Intersect 0x1ebaa8:157 memcpy]; the per-candidate world DeltaP
  goes into the self HITTEST's +0x10/+0x20/+0x30/+0x50/+0x60 [M :353-373], +0x10 is zeroed
  every frame [M Collide_Update 0x1ea398:50], and `Player_CollisionHandler` 0x1949a8:131-140
  adds BLData+0x750 to obj+0x30 once per frame when the pick list is non-empty
  (control_funcs[3] = Player_Update/Player_CollisionHandler [D action-symtab 0x26f890]).

**2. The port's divergence is now pinned to one arm.** `capsule_tri_contact`'s push decision
(crates/nf-game/src/world.rs:798-834) implements the ref-guarded radial arm faithfully but
adds a **spine through-push fallback** (`(spine_dir, r − dist)` at :804, marked [S]) that the
game's decompile does not have: where the ref-guard fails the game pushes NOTHING (and
records nothing), the port pushes. Measured consequence on 07000014's mezzanine west lip:
with the re-land applied, ALL SIX non-agent walks jam at the same spot — STUCK
(45.13, 3.05, 39.87), `segment_to_target Some(0.124)`, crouched, grounded — and a raw
600-tick forward-input loop (real input, the game's own move quad) holds x 45.1338 the whole
time: the port's push cancels the westward move where the game's ref-guarded arm would not
fire. That is the live-push vs dead-tail divergence, settled and measured.

**3. The coupled re-land chain, measured state by state** (all runs whole, release,
--test-threads=1, real ISO):

- **State A — the wrap-only tip (master's tree): 9 passed / 0 failed / 1 ignored** (194 s).
  Includes the d2 agent walk green UNWRAPPED (it top-ups — the merge owner's flag stands).
- **State B — + the cc fix (bd007c18f): 1 / 8 / 1.** Every test with the SkyBridge release
  fails at "the release over the open gap lands Bond on the trench floor, got
  (43.97, 3.17, 63.42) grounded" — the honest standing capsule catches the east walkway's
  west-edge corner. The strafe drift (74f660327) exists to fix exactly this: the two are a
  coupled pair.
- **State D — cc fix + strafe drift, windows back at 3.4, master's lid line: 2 / 7 / 1.**
  The release lands STANDING at root 3.468 — over the reverted 3.4 lane window ("lane false",
  panic at :764). The window widening is required by the cc-paired standing release.
- **State C — the full re-land (cc + drift + 3.6 windows + the lid crouch toggle): 2 / 7 / 1**
  (48 s, twice this session and once in the -2 report). The lid crosses (`over true`) but the
  stand never takes (`stood false` — the +0.95 headroom gate is blocked from z 42.2 down to
  40.0, re-measured), Bond crosses the connector CROUCHED and jams at the lip (above).
- **The d2 wrap measured**: `with_real_damage` + the d2 route in state A survives the DAMAGE —
  32.5 health left at tick 25740 (36 hits for 67.5, 31 return-fire fights) — and WEDGES at
  walker_b2.rs:641, the 1.72-landing float (Bond hung 0.41 over the floor by the slab skirt's
  ny-0.42 push — the exact wedge the cc fix was written to repair, and the cc fix re-rolls the
  world per state B). `back_off 6.0` (hold more distance under fire) measured WORSE: died at
  tick 5594 with 45 hits for 100.0 — the lane fights' strict gun-ray aim loses the corner LOS
  at 6.0 u; 4.0 stands.

**The tip therefore returns to the wrap-only state** (the -2 pattern): the tree is
byte-identical to a3b83fcae (verified `git show a3b83fcae:<file> | diff`), the green set is
master's 9/0/1 measured this session on this exact tree, and the re-land + the cc fix + the
d2 wrap ride the pushed history (`git cherry-pick bd007c18f 74f660327`, plus the wrap commit
55f0b4055) one step away for the next window.

## What's left

1. **The lib fix the task pointed at**: delete the spine fallback at world.rs:804-805 so a
   failed ref-guard means no push and no contact (the game's rule [M 0x109da0:431-447]), and
   re-verify against the FULL walkreal battery (rule 10 — every walkreal_*_live plus the
   door/guard tests). Both jams (the lintel under the standing capsule and the lip under the
   crawl) should dissolve with it; NF_WALK_CLIP/NF_WALK_GHOST stay off (both measured breaking
   earlier legs — the -2 report's numbers stand).
2. Then the re-land chain lands piece by piece (cc fix + drift + windows + lid crouch), the
   d2 wrap goes in, and the 1.72-landing wedge is re-measured under the honest cc — the
   damage budget already survives (32.5 left), so a green d2 is plausible the moment the
   wedge clears.
3. The d3 un-ignore after the d2 (its wedge is the same leg; the fights' 50.0-by-tick-26340
   health line needs the smoother early game or the AIM-pair stands).

## Live checks for the main agent

1. **HITTEST+0x50 (BLData+0x790) during a fresh-boot 07000001/07000014 play** (gdbwatch.py,
   read 4 f32s): does it stay (0,0,0) through walked corridors, or does it move on contacts?
   If Bond's route passes within ~0.55 u of the world origin once, the tail should lock there
   and start tracking — that would reconcile the [L] boot zero with the snapshot's 15 m drift.
2. **07000014's mezzanine west lip (x ~45.1, z ~39.9), crouched walk west**: does the game's
   Bond walk off the lip and drop the ~0.25 onto the connector without stopping? (The port's
   push holds x 45.1338 against 600 ticks of forward input.) A pass confirms the dead-tail
   reading at this spot and clears the lib fix for landing.
3. **The same lip STANDING** (a player walking off normally): confirm the stand gate is
   blocked only on the lane (z 42.2-40.0) and the connector itself takes a stand — if yes, a
   test-side stand at the lip is an alternative to the lib fix.

## How it was verified

- Per-binary, release, `--test-threads=1`, real ISO (`NF_ISO`): walkreal_07000014_live
  **9 passed / 0 failed / 1 ignored** (194 s) on the tip tree (byte-identical to a3b83fcae —
  `git show a3b83fcae:crates/nf-view/tests/walkreal_07000014_live.rs | diff` clean); the
  state B/C/D runs quoted above from the intermediate trees (16-48 s each).
- **The full suite total line did not fit this window**: the time went to the guard trace
  (the subagent's decompile sweep), the four state measurements, and the d2/back_off
  iterations. No `crates/*/src` or `crates/nf-view/tests/common/` file was modified by the
  tip (the tree is byte-identical to a3b83fcae, which is master's), so the other levels'
  binaries are exactly master's — the suite run on the final merged tree is the next
  window's first background job, before the world.rs edit.
- Nothing was committed but the test file's history-exploring commits, the WORKLOG and this
  report; `git status --short` clean before each.

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).

## ntfy failed

All three fallbacks failed at ~19:12 UTC 2026-11-04:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "real-health-14-00agent-3: …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'…', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "…" https://ntfy.sh/nightfire-yoda-8k3x`
   → timed out after 60 s and was killed (no output).

The report and the finish.sh step (`OK done 10-real-health-14-00agent-3`, queue commit
15d9cbaf5) completed before the notification attempts; only the ntfy ping is outstanding.

