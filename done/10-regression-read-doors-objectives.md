# regression-read-doors-objectives — Doors, switches and objectives: read the recent commits for player-visible breaks

Read every commit since 2026-10-20 touching: crates/nf-game/src/triggers.rs crates/nf-game/src/mission*.rs crates/nf-game/src/nis.rs. For each, read the diff and ask what a player would see change. Check the claim against the game [M], then prove any suspect by running it through the real App path (HeadlessApp keys, at 1/240 as well as 1/60), not by reading alone. Name the commit that broke each thing and its first tag. Fix what you prove, with a test that fails first. Not casings, driving or gamepad. Report every commit range read and what was clean.

---
claimed-by: sandbox1 at 2026-11-05T15:07Z

---
## Report (done, 2026-11-05T16:06Z)


---
## Report from the worker (done, 2026-11-05T16:07Z)

# regression-read-doors-objectives — report

Task: read every commit since 2026-10-20 touching `crates/nf-game/src/triggers.rs`,
`crates/nf-game/src/mission*.rs` and `crates/nf-game/src/nis.rs`; for each diff ask what a player
would see change; check the claim against the game [M]; prove any suspect through the real App path
(HeadlessApp keys, at 1/60 as well as 1/240); name the commit that broke each thing and its first
tag; fix what is proven, with a test that fails first. Not casings, driving or gamepad.

Branch `s/regression-read-doors-objectives`, cut from origin/master 82405afdb.

**Outcome: no player-visible break found in the window; nothing owed a fix.** Two suspects were
chased to ground — one resolved CLEAN against [M] plus a disc probe (the use pick's front-only
window), one confirmed as a real-but-honest [S] scope divergence whose player-visible consequence
could not be proven and whose stale comments are now corrected (comment-only commit). Every commit
in the three ranges got a diff read and a verdict.

## Ranges read

- `triggers.rs`: 36 non-merge commits, fafd1771c (10-20 21:04) → 38fea222a (11-04 15:02).
- `nis.rs`: 48 non-merge commits, a8ba7f355 (10-20 21:16) → 0712c4308 (11-05 03:44).
- `mission.rs`: 3 non-merge commits (da8e52a0e 10-21, 272cea833 10-22, 2b0602cf4 10-31). The
  `mission/` per-level tables: 0 commits since the cutoff (last touch 8d8ee59ef, out of window).
- Merges: all 42 merges since the cutoff touching the three paths diffed against their first
  parents. 37 automatic. 5 with conflict resolutions: 4eb7dc280 (union, clean), f810967da (union,
  clean), 0df471687 (union, clean), ca1fa461a (documented deliberate drop of `root`/`anim_root`
  for the general `matrix` — verified no stale refs), 3c6a96ba2 (the keys_alone rule swap, below).
- Reverts/reapplies verified byte-exact: 04e642347 vs e521dea42 = 0 lines; 400eb288c's triggers.rs
  blob identical to pre-e7f80cd6b; 8cecb1b7d's reapply matches c691219e0 (the original merge, not
  819741a9b) except one dropped doc line, and the wrapper_end_timer removal it pulled in was
  re-landed the same morning by d60efab50 — no partial state.

Method: four parallel readers (one per range plus the merge check), each reading the restricted
per-commit diffs; every suspect re-verified by hand against `work/decomp/ACTION.ELF/` and probed.

## Suspect 1 — the use pick's plane window: RESOLVED CLEAN, no break

5706105a8 (2026-11-01, use-pick-two-stage-2, first tag **v0.1.29**) replaced aac593c72's
(v0.1.29) two-sided window (`x < -r || x > r` refuses) with the front-only
`dp > 0 && dp <= r` in `CollisionWorld::sphere_meets_tris`. The two commits cite the same
decompile with opposite readings — aac593c72 pinned "fVar25 = −radius" off a
[D walk_07000015_live] observation (the use point up to 0.45 behind the exit planes when E
loads), 5706105a8 pinned "fVar25 = 0" off HITTEST+0x9a. The [M] settles it:

- `Intersect_PointGeom` 0x1e92d8: `fVar25 = 0.0` when `HITTEST+0x9a & 8 == 0`, else `−radius`
  (decompile :110-116).
- The use pick's flags word is 0: `Player_Activate` 0x1a5170 calls
  `Collide_SphereIntersect(1.0, head+dir, cel, obj, BL+0x7e8, hits, 0, 0, 0)`;
  `Collide_SphereIntersect` 0x1ec940 stores `uStack_1d8 = param_9` into the HITTEST it hands
  `Collide_Pick` → `Collide_Intersect` (the mask-0x101 arm memcpys the caller's HITTEST, 0xa0
  bytes) → `Intersect_PointGeom`.
- The window's signed distance X is the glist **plane record's** `n·p + d` (0x1e92d8: the record
  table at glist[3] + idx·0x40, normal 3×i16·2⁻¹⁴ read via the tri record's u16 at byte +6), not
  the winding cross — so the remaining divergence candidate was the port evaluating the front-only
  window against the **winding** normal.

The probe (`crates/nf-view/tests/probe_use_planes_15.rs`, print-only, NF_ISO) closes it on
07000015: exit #445 and the gated #343 have **zero** record-vs-winding sign flips (X identical to
3 decimals on every tri); the standing-still use point is behind all planes under BOTH laws (min X
−0.551 — the game refuses it too), and the walk-in window (+0.299) is admitted by both. The walk
test's "press while walking in" is the real behavior, not a cover. aac593c72's behind-the-plane E
reading was the misreading of its own walk data — its two-sided window made the port ADMIT a use
point the game refuses, and 5706105a8 fixed that. Also verified: no decompile site writes
HITTEST+0x9a (read-only everywhere), so the flags word cannot inherit bit 8 from another caller.

No fix needed. Clean.

## Suspect 2 — the standing segment's raw-law scope: DIVERGENCE CONFIRMED, no break proven

c56bbe871 (2026-11-04, first tag **v0.1.31**) scoped the standing segment's raw one-sided law to
glists whose flags are EXACTLY 0x645 (`Trigger.seg_beam`), while its own citation says the game
routes EVERY bit-0x40 glist through the raw refine ([M `Collide_Intersect` 0x1ebaa8:93-99 sets
HITDATA+0x51 |= 2 from obj+0xf0 & 0x40; D 0700000a #607/#614/#621 = 0x645 but 07000011 #235 =
0x50]).

The census (`crates/nf-view/tests/census_trigger_glist_flags.rs`, all 36 level bins, print-only):
bit-0x40 non-0x645 trigger glists are the NORM — 0x50 dominates every level (7 volumes on
07000001, 28 on 07000002, 22 on 07000003, 20+ on 07000005, …). The port's scope is far narrower
than the game's rule.

Why no fix today: the scope is an honestly-labelled [S] and was forced by measurement — at
af7e4164d the raw law ran for the standing segment on every record-bearing trigger and "on floor
plates the segment fired ahead of the box and moved scripted sequences across levels (17 tests)"
(c56bbe871's own message). The port's `seg_touches_tri_raw` does not yet match the game's cylinder
refine (`Intersect_CylGeom` 0x109da0's plane-crossing/centroid anchor + `Dist2Tri` 0x1dfca8's
one-sided entry) on those 0x50 floor plates; widening the scope before the model matches would
re-break the walks. The HEAD delta vs master is ADDITIVE (0x645 glists gain the segment law;
everything else rides the measured box, exactly as master).

The stale comments this left are corrected (comment-only commit): drone.rs claimed the standing
segment "keeps 011's Bond-Moment plate #235 firing" — false at HEAD (#235 is 0x50 → `seg_beam`
false → the segment returns false; the measured box fires it); triggers.rs's "the strict law runs
for the standing segment" claimed a wider arm than the gate implements. Both now state the [S]
scope and what closing it needs.

## The other findings

- **912b1f112** (v0.1.22): created kind INVERTER from flags 0xF9 citing "[M
  InverterTrigger_Update 0x1b30a8]" — the decompile contradicts it: parsemap case 0xF9 →
  `SoundTrigger_Create` (001d04f0:504-510); the inverter is case 0x41. While live (one day) it
  armed 18 phantom inverters on 07000014. Fully reverted by 88067cc2a (verified: no INVERTER at
  HEAD; worldobj.rs owns 0x41). Historical.
- **39ede9453** (v0.1.18) — the skip catch-up subtitle "last wins" (the message reads as comment
  dispositioning but ships behavior): checked against [M] — the game flushes subtitles once at the
  skip (0x1dac78:25-29) and `Text_Update` 0x1b1510:8-32 re-points the single SubTitleMsg sprite,
  so the LAST add of the fast-forward is what survives; the port's last-wins retain matches the
  visible content. Residual nuance: the port starts the surviving line's duration when the view
  drains (end of skip) where the game started it at the add's catch-up tick — timing-only,
  render-side. No fix.
- **Transient same-day states**, each superseded within hours and verified gone at HEAD:
  dee509f7b/d99aa5875 (`Toucher::player` r 0 → 0.55, reverted by d24dff3ec to the master box);
  e7f80cd6b/8c0cc7136/9c5406bca/af7e4164d (the r>0 → seg_only → all-records raw-law churn, settled
  by c56bbe871); 33f8797af/2bb24981c (the scoped keys_alone rule lost at merge 3c6a96ba2,
  re-derived by 9842b2605's hash-0 anchor carve-out).
- **3c6a96ba2** (v0.1.23): the merge resolution deleted the scoped keys_alone rule INCLUDING the
  07000008 gondola carve-out; the taken side (9842b2605) re-derives the gondola via the hash-0
  carve-out and 071bc5b61 re-gained the 062f arm — HEAD's rule (nis.rs:1275-1284) matches the last
  stated form (`keys_alone = camera_started && parent < 0`, hash-0 kind-0x19 never composes, hash-0
  anchors compose unless 0x0600_062F). No orphan found; pinned by zoe_gondola_realpath_live.
- **Message-vs-diff disagreements recorded**: 400eb288c ("seg_touches_tri_raw stays" — the diff
  deletes it); 9c5406bca (the box/crouch/drones claim — the `r > 0` gate still put them on the raw
  law until af7e4164d); a877342a8 (ffwd's doc header still says the skip fade is "not modelled"
  while the same commit models it); 0f437196b ("four producers" — three live in drone.rs; its
  natural-start `Text_FlushAllSubtitles` deferral is unverified); 0712c4308 (the +0x60 |= 4 raise
  attributed to 0x1c1b38 is actually in `AnimObjectUpdate` 0x1befd8:186).
- **Residual risks noted, no break proven**: 992b17bb2's (v0.1.20) hash-0 kind-0x19 carve-out is
  [L]-pinned on 01/4a only (census gap for other levels; the mechanism is openly unresolved in the
  decompile); 0712c4308's (v0.1.32) footstep sound skips the game's stored-ground gate (the decal
  keeps it) — a stream anim over a ledge could play one footstep the PS2 would not; a877342a8's
  single view_fade slot vs the game's per-viewer fades and the per-script `script_cam` vs the
  global ScriptCam (census: 18 camera SPs, all non-looping — the overlap case cannot arise on
  disc).

## mission.rs

2b0602cf4 (v0.1.28, seed earlier parts' objective records): the seed and the carry-restore are
mutually exclusive on the same `Option::take` (drone.rs:17833) and the seed only writes rows cold
init skips — it cannot clobber a carried load. Verified against `Mission_Init` 0x182270 (the
prior-part arms only read the record count) and `Mission_MonitorObjectives` 0x1827d8 case 0. Open
item for the disc: the driving return is a fresh ACTION boot per the repo's own docs, where the
disc would RE-ANNOUNCE the prior parts (pristine .data) — both port paths suppress that. Live
check below. 272cea833/da8e52a0e: comment-only. Clean.

## Tests run (pass counts)

- `nf-game` lib: 678 passed / 0 failed / 5 ignored.
- `interactables_07000011_live`: 16 / 0 (whole binary).
- `walkreal_0700000a_live`: 13 / 0, 2 ignored (their ignore messages carry the documented live
  checks) (whole binary).
- `low_beam_621_fires_standing`: ok (also inside the whole run above).
- `walk_07000015_live`: 2 / 0 (whole binary) — the use-exit E-press path green through the real App keys at HEAD (suspect 1's App-path pin).
- The two new census/probe tools are print-only and `#[ignore]`d; both ran with `--ignored`.
- No production behavior changed (two comment blocks + two new ignored test files), so per lesson
  10 the touched-code binaries above were run whole.

## Not run

The full workspace suite (the other levels' walkreal_*_live, disc_golden, …) — the diffs are
comment-only plus new ignored print-only tests, and the 16:53 UTC stop-work line; the touched-code
binaries were run whole instead.

## Live checks for the main agent

1. 07000015 exit E standing still: Bond wedged at the #445 doorway stop (head z ≈ −24.15), press
   E — the [M] window ((0, 1] on the record plane) says the disc refuses it (X = −0.55). Confirm
   the disc refuses (no level load); closes suspect 1 with disc behavior instead of the [M] chain.
2. `Intersect_CylGeom` 0x109da0's raw refine on a 0x50 FLOOR plate: gdbwatch the refine branch
   across 07000002's #341 plate (flags 0x50) — does the disc's cylinder take the one-sided
   record-plane entry there? The port's raw model over-fired those plates (17 walk tests), which
   is why the scope is [S]; this decides the anchor fix the scope widening waits on.
3. Driving return (07000500 → a prior part): does the disc re-announce the earlier parts'
   objectives on the resume level (pristine .data per `Boot_LoadPTPData` 0x1c8560), or does
   something restore them? The port suppresses the re-announce on both its paths.
4. Skip-subtitle timing: on a skipped dialogue scene (07000004 story stream), is the surviving
   subtitle's remaining duration the game's own (started at the add's catch-up tick)? Render check.

## What's left

Nothing broken was found in the window, so nothing is owed a fix. The one real divergence carried
forward is suspect 2's [S] scope (documented in-code now); closing it needs the cylinder-refine
model work plus live check 2 above.

## Usage

Figures this environment showed me (copied exactly; source in parentheses):

- Reader subagent A (triggers.rs range, Explore agent-462d7257-2e46-4cd4-bdc4-4371fcb1ede2,
  footer of its final report): `Tokens -> prompt:155229 completion:9865 total:165094`.
- Reader subagent B (nis.rs 10-20..10-27, Explore agent-8f306822-6c86-4355-a297-910d454ec196,
  footer): `Tokens -> prompt:100953 completion:4625 total:105578`.
- Reader subagent C (nis.rs 10-27..11-05, Explore agent-311068cd-a22f-4506-a9ca-4f73f7c2291d,
  footer): `Tokens -> prompt:127639 completion:8687 total:136326`.
- Reader subagent D (mission.rs + merges, Explore agent-fa02771a-5957-4c70-95ae-400577069dc8,
  footer): `Tokens -> prompt:128029 completion:4507 total:132536`.

My own session's input/output tokens, context-window fill and cost: not visible to me.
