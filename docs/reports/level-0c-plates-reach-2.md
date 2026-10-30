# level-0c-plates-reach-2 — report

Task: close the ONE unmapped link from level-0c-plates-reach — the pocket→yard descent — by
working its "What's left" candidates (a) the wider coll-scan west of x −4.5 / south of
z −155.7, (b) the z −158.5 wall's sloping top as a walk, (c) what lies above the yard's
north-west arm; then either rewrite leg 4 of
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` on real
input with no teleport, or pin "no route" with sister asserts. Branch
`s/level-0c-plates-reach-2` = origin/master (a19be699).

## What's done

**All three candidates were worked to a verdict, the descent route is now mapped end to end
with real-input sim measurements, and it is open except ONE bridge hop that the port's sim
blocks.** Everything below is measured in this window's `probe0c_pocket_descent` (committed,
both tests `#[ignore]`d as route records) and `nf-extract coll-scan` dumps over the onfoot ISO
(0700000c.bin) — the scans are [D] (disc collision triangles), the sim walks are the port's
real-input path over that same disc collision.

1. **Candidate (a) — the wider west scans: the prior map was wrong in our favour, but the
   edges are walled.** The 2.37 floor (room_storage01's lower level) runs west to **x −8.25**
   (z −143.11..−155.77 [D scan_swdrop/scan_rampne strips]) — the prior task's "the pocket's
   west edge x −1.8 hangs over void" was a r-2.5 scan-radius artifact. The probe WALKED it:
   the pocket west legs arrive grounded at (−6.07, 3.50, −150.92) [A1/A2]. But every edge of
   the west extension is walled by y 2.37..6.15 faces [D]: x −8.25 (z −143.11..−153.76),
   z −153.76 (x −8.25..−5.57), z −155.77 (x −5.03..15.25) — the sim wedges into all three
   [B0/B1, C1/C2, C4/C5] and the jump-offs land back on the pocket.
2. **Candidate (b) — the "sloping top" is not a walk: it is a vertical face.** The z −158.5
   structure is `room_ex10_5` (map 45, obj 85): its north face z −158.50 spans x −5.65..0.45
   from y −0.01 up to 4.52/4.55 PLUS a vertical sliver 4.52→6.69 (the "slope" is the top EDGE
   rising eastward, all three vertices at z −158.50 [D scan_topbox]). There is no horizontal
   surface along it. The structure's real top is 6.69 strips (x −3.19..−2.40 and −0.34..2.17,
   z −158.5..−159.74) over an OPEN notch shaft (x −2.40..−0.34, floor y −0.01 [D
   scan_pocketvoid]) — and the tower top IS walked: its south edge drops to the y≈0 ground
   with a plain walk-off (H1: lands (0.97, 1.04, −162.86), no fall damage) and a jump
   (E7: (−8.51, 1.09, −173.72), no damage; the E6 west variant takes 14 damage, survivable).
3. **Candidate (c) — above the yard's north-west arm: the room_sk3 ramp, walked.** The y≈0
   floor's north-west arm (x −18.03..−12.77 reaching north to z −156.81 [D scan_arm/
   scan_overarm]) carries `room_sk3`'s ramp decks (3.80..4.83, x −18.03..−9.98,
   z −156.81..−127). The probe walked the ramp's south-west walk-off DOWN to the yard (F2:
   (−15.46, 1.08, −158.28), grounded) and its north deck up to z −127.12 [F3/F4/F5] — the
   yard↔ramp link is real.
4. **The west tower complex is mapped level by level** [D scan_topbox/scan_rampne/
   scan_midgap/scan_westentry]: yard 0 → ramp 3.80..4.83 → the 6.92 walkway (room_sk3,
   x −10.03..−5.65, z −155.02..−110.36) → the 9.24 deck (x −12.07..−7.34, z −162.17..−155.36)
   → the 7.24 platform (room_storage01, x −5.05..−2.04, z −108.67..−116.47) with its south rib
   (x −4.54..−4.37, z −109.80..−116.47). The 6.92 walkway is walked south through its
   cross-wall gap (the wall spans x −8.17..−5.65 [D]; x −10.03..−8.17 open — K1 walked south
   through at x −9) and its west edge drops onto the ramp decks (D4: the jump-off LANDED
   (−17.35, 4.92, −150.95), no damage; K2 mid-drop). The display ground walks west to
   (−0.53, 7.38, −110.35) [the full-route probe's "1 west 1-4"], and the standing jump up onto
   the 7.24 platform's wide part LANDS ((−4.48, 8.36, −109.22) — a 0.97 rise, the E1 class).
5. **The ONE unsolved hop: the 7.24 platform/rib → the 6.92 walkway.** The rib's west side is
   walled by a parapet at x −4.84..−4.79 topping 8.67 (z −112.21..−116.70 [D scan_westentry]);
   every crossing attempt in the sim fails: the west standing jumps from the rib bounce back
   [3b ×2], the rib placements push out east/north [3c ×3], the big west jump from the
   7.1..7.36 surface at x −3.99 bounces at the same west wall [3e], and the running west jump
   off the display floor lands on a ~6.65 shoulder at (−3.82, 7.74, −111.49) without clearing
   [3f]. North of the parapet the room_sk3 7.24 strip stops at z −112.24 and the walkway's
   east portion starts at z −111.86 — no overlap to walk through. The two proven halves do NOT
   join in the port's sim.

**Leg 4 is unchanged** (the master stand-in stays byte-identical; the walk is untouched and
its green status is inherited from the merged tree), because the real-input route is not
complete: the walk must stay green, and a leg with a blocked hop would go red. The "no route"
verdict is ALSO not pin-able: the disc's own collision contains the continuous geometry chain
(display ground → 7.24 platform → rib → … → walkway → ramp → yard), blocked in the sim only at
the parapet hop — so a sister-assert pin of "closed" would assert something the data does not
say. What IS pinned by this window's evidence is in the committed probe (30+ measured
transitions with landings, wedges and fall damage).

## What's left

1. **The parapet hop** (x −4.8, top 8.67, between the 7.24 rib and the walkway/ramp side).
   Next steps in order: (i) scan the parapet's two ends (z −112.21 north / z −116.70 south)
   for a walk-around at 7.24; (ii) probe a crouch+jump or a mantle-shaped input sequence (the
   sim's jump apex is ~1.0, the parapet needs 1.43); (iii) if the sim truly cannot, file the
   live check below — the real game's jump/mantle may clear what the port's cannot, which
   would make this a PORT physics gap, not a route gap.
2. **Then rewrite leg 4** with the proven chain: the display eye → west hops [the full-route
   probe legs 1-2] → … → the walkway south (K1's gap lane x −9) → the west drop (D4) → the
   ramp (F2) → the yard → the old probe0c_plates_route's section-5 yard/bridge/corridor
   waypoints → the pit → the plates' body touch; leg 5 then runs unchanged.
3. **The full suite** on the final tree — see below for this window's run.

## How it was verified

- Every coordinate above is [D] (`nf-extract coll-scan` over the onfoot ISO, 0700000c.bin; the
  scan dumps are named per region in the probe's comments) or a sim measurement from
  `probe0c_pocket_descent` (real `step_bond` input; placements only BETWEEN probe sections,
  never inside a walked leg; each walked leg's landing is logged with fall damage).
- `probe0c_pocket_descent`: both tests `#[ignore]`d (route records, not pins). The touched
  crates: nf-view only, and only this new file + docs — `cargo test -p nf-view --test
  probe0c_pocket_descent` compiles clean (2 ignored, 0 run). `walkreal_0700000c_live` is
  byte-identical to master this window.
- **Suite** (the fast runner, started 14:16 UTC on this tree after the origin/master merge —
  the branch's only code delta vs master is the ignored probe file, so the run is the final
  tree's run): build ok — 936 test binaries in 1192 s; still running at report time (~15:20
  UTC, ~50 min in). So far: FAILED identity_disc + the 3 disc_golden (the slim-ISO expected
  set) and CRASH net_resync_live / mp_shot_effects_live / net_e2e_loopback_live (exit −9 =
  the runner's wall-clock kill of the long network loops), plus FAILED
  fix_0c_pylon_live::the_0c_pylon_fills_the_view_from_the_closeup_spot and
  ::the_0c_pylon_is_on_screen_at_the_user_spot — a render-dependent pair this sandbox cannot
  satisfy (no GPU; the branch changes no render code, so this is pre-existing on the tree and
  needs the main agent's render check, not a port change). The final total line was not yet
  printed when this report was written; the runner's log (../suite.log) carries each binary's
  line as it completes.

## Live checks for the main agent

1. **The parapet hop** [D x −4.84..−4.79, top 8.67, z −112.21..−116.70; the rib top 7.24 at
   x −4.54..−4.37]: in PCSX2 (fresh boot, Bond on the 7.24 platform at ≈ (−4.5, 7.24, −110.5)
   — or walk there west from the display eye), jump west against the parapet and record the
   body's max height and whether it lands west of x −5.06 (the room_sk3 strip) or on the
   walkway (6.92). Expected if the game's jump is taller than the port's: a landing at
   x ≤ −5.06, which opens the whole leg-4 rewrite.
2. **The display drone roof** [D oDrone top 7.36, x −3.74..−2.74, z −109.66..−111.67]: from
   the 7.24 platform's wide part step south onto it and jump west from (−3.2, 7.36, −111.2);
   record the landing (the walkway east portion starts z −111.86 [D]). The port bounces off a
   west wall above spine 8.18 there [3e].

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
