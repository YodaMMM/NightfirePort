# capsule-cull-and-los-masks-6 — the mp digest's mover is NAMED and the pin is RE-PINNED [M]: the game's head-on cull

## What's done

1. **The divergence is named and it is this branch's cull content — proven by tree
   bisect, not memory.** The task's pin (mp_net_slots_post_move_live
   `four_human_session_digest_is_pinned`) reads final 0xbc5e2155a3786e9c on this
   lineage vs the pinned 0xfb7b58f73e340ec3 (tick0 0x5c0bb60a8cdf8938 unchanged on
   every tree). Running the pin binary at each tree:
   - 82362142 (batch-1026d, round 5's last clean master point): GREEN (0xfb7b…).
   - 24af5915 (capsule-1's first commit — the only sim change is
     `LOS_MASKS_ON: false→true` and `HEAD_ON_CULL_ON: false→true`): already reads
     0xbc5e… (RED).
   - 24af5915 with `HEAD_ON_CULL_ON` flipped back to false (scratch, uncommitted):
     GREEN again — **the head-on cull is the sole mover; the LOS masks are
     digest-inert in this session** (nobody sights or shoots in it).
   - 4a155ddf^1 = ca2e7389 (capsule-3 tip, before the 4a155ddf merge): reads 0xbc5e…
     WITHOUT any of the merge's master-side content — **the merge resolution and the
     master side (353b1008's SP-matrix riding, hatch-2's toucher — round 5's surviving
     suspects) are exonerated**; both are master-side and master's own pin is green.
   - First divergence (round 5's measurement, unchanged by all of the above):
     session t142, component **bodies**, slot 3 only — x +0.00122, y −0.00235,
     z/yaw/pitch/substate/crouched identical; the ashdbg dump (this window, at
     ca2e7389) shows slot 3 airborne off the t134 scripted jump and pressed against
     the z −206.70874 wall at t142-146 — exactly the near-flat-face crossing case the
     cull decides.
2. **The new value is the game's rule — verified against the decompile directly, then
   the pin re-pinned with the citations.** The port's inline cull (world.rs, the walk
   contact's `if (nd > 0.999 && f > 0.5) || (nd < -0.999 && f < 0.5) { return false; }`)
   is the decompile verbatim:
   - the skip arms [M Intersect_CylGeom 0x109da0:321-322 — `if (((0.999 < uVar3) &&
     (0.5 < fVar30)) || ((uVar3 < -0.999) && (fVar30 < 0.5))) goto LAB_0010a7b8`];
   - f = the plane-crossing fraction from A, clamped 0..1 [M :310-320];
   - the spine is NORMALIZED before the loop [M the Vec_Normalise 0x1e3000 call at
     0x109f74 — a 0.999 raw-dot threshold is unreachable on the |d| = 0.7597 walk
     spine, so the compare is unit-scale];
   - the walk spine's A = HITTEST+0x20 = BLData+0x760 (pos + up·0.275, the TOP), B =
     HITTEST+0x30 = BLData+0x770 = pos + up·(0.55 − cc) [M Player_Collision 0x194df0 —
     the decompile writes `0.55 − fVar20` into +0x770 at :301/:333 and the top end into
     +0x760] — so with d = B − A pointing down, the arms mean: an UP-facing floor
     crossed in the spine's TOP half passes through (jump-through from below), a
     DOWN-facing face crossed in the BOTTOM half passes, the same faces crossed in the
     other half stay solid.
   The old pin (0xfb7b…) was taken on a tree whose capsule contact LACKED this cull —
   it baked a non-game rule into the expectation, so per the task rule the TEST was
   wrong and is fixed: PINNED_FINAL → 0xbc5e_2155_a378_6e9c with the full evidence
   block in the test. **Both of the binary's tests are green on the final tree.**
3. **The disagreement is recorded, not buried** (lesson 1): the world.rs comment block
   left by level-0a-office-hatch-9 claims the game's arms are the opposite halves
   ("(up-face ∧ f>0.5) ∨ (down-face ∧ f<0.5)", from an [L] fresh-boot corner-walk
   measurement) while the decompile text at :321-322 says these arms. The shipped code
   implements the decompile text verbatim; hatch-9's "corrected arms" were never
   landed (its const is dead). If hatch-9's [L] measurement is right, its A/B reading
   is the thing to re-derive — the decompile line itself is unambiguous.
4. **origin/master moved mid-session** (dcc4a733 → ec917093, batch-1026l: cast-revert,
   display-hatch-4, abseil-hang-pose, glass-01) and is merged clean (9995f255);
   `s/door-state-collision` is NOT in master, so nothing needed dropping. The pin is
   re-verified green on the merged tree.

## How it was verified (what ran, on the final merged tree 9995f255)

- `mp_net_slots_post_move_live` (the re-pinned pin + the 6-human loopback): **2 passed,
  0 failed**. `mp_digest_diag`: 1/0. `gt_los_live` 2/0, `gun_turrets_live` 2/0,
  `interact_07000006_live` 9/0, `interact_07000009_live` 16/0, `level_end_live` 6/0,
  `zoe_gondola_realpath_live` 1/0, `ai_live` 36/1 (see Left), `fix_npc_doors_live`
  1/0, `door_aim_ray_live` 2/0, `door_portal_links_live` 3/0, `door_text_labels_live`
  4/0, `door_hint_boot_count_live` 2/0, `door_glist0_e_dead_live` 2/0,
  `npc_doors_live` 6/0+1ign (the documented escort-reach ignore), and
  `walkreal_07000008_live` **4/0** (the door-state fix's proof, 102 s). All
  `--test-threads=1`, `NF_SILENT=1`, NF_ISO on the slim on-foot image.
- Suite runner status: the background `tools/test/suite.py` dies between tool calls in
  this sandbox (6th sandbox to measure it; this window's run confirmed it again). The
  foreground runner is capped at ~10 min per tool call: its build phase (845 test
  artifacts) exceeded the cap once (no output, killed at 590 s), then completed in
  152 s on the second attempt after a **disk-full incident** (the rootfs hit 100%
  during linking; fixed by deleting 1735 stale pre-session files from
  target/release/deps, 2.5 G freed) — 50 binaries completed clean in that segment
  before the cap killed the run; the families above were then run directly. The full
  845-binary pass still has not happened in this sandbox (3rd window running into
  this); the FAILED/TIMEOUT/CRASH lines across everything that did run this window:
  **exactly one, ai_live::guard_takes_cover_like_the_game, pre-existing (below).**

## What's left

1. **ai_live::guard_takes_cover_like_the_game is RED on a716ffac — the capsule-5 base
   this task started from — so it predates this window and is NOT merge-introduced**
   (verified by running that tree). It was made green in capsule-1 (8b31c613, the
   seed-3 calibration); the failing assert is the state-sequence pin at ai_live.rs:263
   (`[0x58, 0x5a, 0x8c, 0x93, 0x94, 0x95, 0x96]`). The mover therefore sits in
   capsule-2..5: gt.rs's muzzle-offset/track-ray/spread work (7341a8eb, 58d16ebd), the
   drone.rs door-state gate, or the band-settle radial tail. Rounds 4/5 did not run
   ai_live. Next window: bisect ai_live over the capsule-2..5 commits, fix or re-pin
   [M], and only then re-merge master.
2. The full 845-binary suite end to end (needs either a sandbox without the 10-min
   tool cap or a working background runner).
3. Optional sharpening for the cull: a live PCSX2 check would settle the hatch-9
   disagreement (below).

## Live checks for the main agent

1. **The head-on cull's jump-through (settles the hatch-9 arm disagreement).** In
   PCSX2 (fresh boot, god mode), walk Bond off a low ledge/step while airborne
   forward: gdbwatch on the HITTEST spine writes at 0x195444/0x195464
   (BLData+0x760/+0x770) plus a breakpoint inside Intersect_CylGeom at the skip branch
   0x109da0+0x17a-ish (the `goto LAB_0010a7b8` arm at :321-322). Expected if the
   decompile reading is right: while airborne and crossing a near-flat up-facing face
   (|n·d̂| > 0.999, |n·ŷ| > 0.999) in the spine's TOP half (f < 0.5 from the +0x760
   top end), the skip fires (no HITDATA write, no push) and Bond passes through;
   crossing the same face in the BOTTOM half pushes.
2. **The digest's t142 spot (optional).** Ravine MP 4-human scripted session is not
   reproducible in PCSX2; instead: place Bond at (111.74, 41.52, −206.71) yaw −0.066
   falling (the t143 state) and confirm the z −206.70874 wall-press does not push in
   x/y (the pre-cull port pushed Δx +0.00122, Δy −0.00235 there).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures
for this session.
