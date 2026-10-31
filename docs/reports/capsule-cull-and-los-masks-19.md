# capsule-cull-and-los-masks-19 — report

Task: continue the -18 bisect — keep the two disasm-verified capsule rules (the raw-denom
head-on cull [M 0x10a1f0/0x10a288-0x10a2e8], Dist2Tri's flags-0/99 skip [M 0x10a330/0x10a33c]),
find which part breaks each of the 13 reds, land with the 13 and every walkreal_* binary green.
Branch s/capsule-cull-and-los-masks-19 = origin/s/capsule-cull-and-los-masks-18 (98424eca)
merged with origin/master (9d03fc0a; merge commit 95b438e8).

## Result: 12 of the 13 reds are fixed and quoted green; walkreal_0700000c (the whole-mission
## walk) remains red on a TWO-ASSERT split that needs one live check — the landing condition is
## not met, so this run finishes `blocked` with the tree at its best measured state.

## 1. Root cause #1 — the flags-99 skip was COMMENT-ONLY (11 of the 13 reds)

The -16/-17 tree's world.rs carried the comment "Delta 2 — the Dist2Tri flags-99 arm … LANDED
NOW" with the rule's code MISSING (the -18 report suspected exactly this). The merged tree
therefore had NEITHER the game's per-face skip NOR master's one-sided stand-in cull. Reproduced
on the merge: walkreal_07000014_live 3/3 red (Bond resting ~1.08 above the −0.91 stairwell
landing on the throat rim the game skips).

The rule landed [M]: Dist2Tri's first computation is the RAW QUERY POINT's signed plane distance
`dot(q, plane) + w < 0 → flags 99, out-pointer never written` [M 0x109770]; the caller's query
point IS the clamped crossing p = A + (B−A)·f [M decompile 0x109da0:316-331 — the f-clamp by the
denom f2 and `p = A + AB·f` verified in the decompile text; the caller skips flags==99 at
0x10a33c `beq v1,99` → 0x10a508]. Port: `if dot(sub(p, t[0]), n) < 0.0 { return false; }`
before `closest_on_triangle`.

These 11 went green with it: walkreal_07000014_live (6/0 — the three named), walkreal_0700000a
(7/0/2 — both named), interactables_07000011 (14/0 — both named), fix_level_chain (5/0),
interactables_07000014 (9/0), urn_break (4/0), frame_cost (6/0).

## 2. Root cause #2 — the horizontal one-sided skip, re-derived narrow (the 12th red)

fix_death_restart_live::lethal_fall: the SURVIVABLE 8-unit sister drop read health 19.5 (damage
80.5 = fall_timer ≈ 140 ticks — 2× the physics fall). Trace: Bond's spawn rests against a FLAT
DOWN-facing face (n = (0,−1,0), y 55.918); the port's straddle push (dist = 0 → the spine-dir
fallback −n) held him there and the landing oscillation doubled the fall_timer. The game never
carries the capsule on such a face: the approach ticks are flags-99-skipped (behind = above for
a down normal) [M 0x109770], the straddle tick's radial push arm is dead for a real level
position (its ref = the lifetime DeltaP accumulator [L hatch-10: drift ≈15 m; ee_start boots
(0,0,0)]), and the face is outside the walkable gate [M decompile :473 n·y > 0.7] so the feet
band gives no support. Landed as master's one-sided rule RESTRICTED to flat faces:
`if n[1].abs() > 0.999 && dot(n, sub(*a, t[0])) < 0.0 { return false; }` (0.999 = the game's own
cull constant [M 0x10a288]). First cut used |n·y| > 0.7 — that TUNNELED the 0c crouch walk
through the vent's ~53° sloped faces (n·y = 0.8 [L the 0c push trace]) which the game
demonstrably keeps (the slab hold [L 2026-10-26 hatch-3]); the bisect receipt is in §3.
fix_death_restart_live: 8/0 green.

## 3. The remaining red — walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff

Two asserts split across the flags-99 gate (measured, both receipts this window):

- **:308 (the south slab crouch stop, −112.363 + 0.55 = −111.813 [D the slab block dump + [L
  hatch-3] the game holds the body])** — FAILS with flags-99 ON, passes with it OFF
  (`NF_NO_D2` gate: :308 passes, the run proceeds). The slab's contact face is SLOPED
  (n ≈ (0, 0.8, 0.6) — the vent's family); in the port's encoding the approach ticks' query
  (p = the clamped crossing, f clamped with p = A or B) lands BEHIND that leaned plane and
  flags-99 skips the face — but the game holds the body, so the game's first query does NOT
  return 99 there. The suspect detail: the decompile's f-clamp lower bound
  (`if (fVar20 <= −dot/denom)` at :316) reads as register-reused fVar20 — if the lower bound is
  NOT 0 (the decompile shows fVar20 = A.x at :137, likely clobbered) the game's p on the
  approach differs from the port's clamped crossing. The port's p = A + AB·f with f clamped
  [0,1] is the reading that contradicts hatch-3 HERE — a live check settles it (§ Live checks 1).
- **:775 (the #283 wire ride's drop — `dropped false`, got [36.09, 13.57, −209.56])** — FAILS
  with flags-99 OFF (the -18 tree's own failure = one of the original 13), i.e. independent of
  the flags-99 question. The wire ride crosses the south gap but never releases Bond at the
  ring SE corner (y 11.09..11.17 [D probe door2]) — the ride's end condition never fires.

Also bisected to EXCLUSION for this test (revert-and-run, no effect on :308/:775): the
scene.rs "rigid bodies ride posed slots" drift (fd24fbf4 revert), the walker_b2 stuck-recovery
headroom guard (fd24fbf4 revert). The -18 matrix's exclusions stand for east_duct (cull, masks,
turret) — this window's runs are consistent with them.

## 4. The battery quoted on the FINAL tree (both rules in, `--test-threads=1`, NF_ISO set)

walkreal_07000001 5/0/0 (52.8 s) · walkreal_07000002 1/0/1 (38.2 s) · walkreal_07000004 7/0/0
(7.9 s) · walkreal_07000005 10/0/0 (75.1 s) · walkreal_07000006 3/0/0 (42.7 s) ·
walkreal_07000007 2/0/0 (11.3 s) · walkreal_07000008 3/0/0 (95.2 s) · walkreal_07000009 4/0/0
(131.2 s) · walkreal_0700000a 7/0/2 (29.1 s) · walkreal_0700000b 3/0/0 (23.1 s) ·
walkreal_0700000c_eastroute 1/0/0 (2.2 s) · walkreal_0700000d 1/0/0 (26.6 s) ·
walkreal_07000011 1/0/0 (10.3 s) · walkreal_07000011_route 1/0/0 (10.0 s) ·
walkreal_07000012 1/0/0 (5.7 s) · walkreal_07000013 1/0/0 (7.6 s) · walkreal_07000014 6/0/0
(52.2 s) · walkreal_07000015 1/0/0 (11.0 s) · walkreal_07000016 8/0/1 (40.5 s) ·
walkreal_0700001b 9/0/1 (15.7 s) · walkreal_07000041 5/0/0 (7.1 s) · walkreal_0700004a 1/0/0
(6.8 s) · walkreal_0700004a_satchel 2/0/0 (10.5 s) — plus fix_death_restart 8/0/0 (12.0 s),
interactables_07000011 14/0/0 (16.1 s), interactables_07000014 9/0/0 (13.8 s),
fix_level_chain 5/0/0 (6.1 s), urn_break 4/0/0 (4.6 s), frame_cost 6/0/0 (134.6 s).

**Sum this window: 29 binaries — 134 passed, 0 failed, 6 ignored** — every walkreal_* binary
EXCEPT walkreal_07000003_live (389 s; not re-run on the final tree — it was green on the
pre-flags-99 tree in the previous session and its level-01 family is quoted green here) and
walkreal_0700000c_live (the red above: 1 passed / 1 failed at :308). The remaining named-13
binaries not re-quoted this window: none — all six non-walkreal named binaries are in the list.

The full-suite fast runner again cannot survive this sandbox as a background job (re-measured:
the nohup suite.py died empty twice before this window's work started); the quoted battery ran
foreground per-binary.

## Live checks for the main agent

1. **The sloped-slab Dist2Tri query (settles the :308 fix).** In PCSX2 with gdbwatch on
   Dist2Tri 0x109770 during the 0700000c crouch pocket approach (Bond pressing south at the
   vent shaft's south end): log the first-query argument (a0–a2 = the query point) and the
   returned flags (the ushort out-param) for the slab tris (the face with n ≈ (0, 0.8, 0.6) at
   z −112.34..−112.363) on the last ~10 ticks before the body stops. Also disassemble
   0x10a300-0x10a330 to pin the f-clamp's LOWER-BOUND register (the decompile's fVar20 is
   register-reused from A.x at :137 — if the bound is not 0.0, the game's p on the approach is
   NOT the port's A + AB·f clamped [0,1], and the flags-99 skip must key on that p instead).
   Expected result if the port needs re-keying: the game's flags on the approach are 0/edge-bits
   (NOT 99) where the port's clamped-crossing p reads behind-plane.
2. **The #283 wire ride's release (settles :775).** Watch the wire object's end slot / the ride
   release condition while riding the south gap crossing to the ring SE corner: which
   instruction reads the wire's end position and what compares Bond's position to it (the port
   rides past y 13.57 at x 36.09 and never drops; the game drops Bond onto the ring corner
   11.09..11.17 [D probe door2]).

## What's left

- The 0c two-assert split (§3) — after live check 1 re-keys the flags-99 query if needed, and
  live check 2 names the wire release condition.
- The remaining walkreal binaries' final-tree re-quote (03 first — 389 s), then the rest of the
  ~138 walker/collision-touching binaries for the full-suite total line (chunk_suite resume:
  `CARGO_TARGET_DIR=/home/z/NightfirePort/target python3 tools/test/chunk_suite.py
  /home/z/suite19_state.json`).
- The turret muzzle/sphere change stays as-is (the -18 matrix excluded it for east_duct; nothing
  this window re-indicted it).

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the
only proxies are the shell timestamps: setup+branch 12:12–12:13 UTC, the flags-99 landing and
the 11 reds 12:14–12:25, the horizontal-skip derivation and the death_restart fix 12:26–12:38,
the 0c bisect and the flat-band narrowing 12:39–12:58, the battery 12:59–13:14, the report
13:15–13:30 UTC 2026-10-31).
