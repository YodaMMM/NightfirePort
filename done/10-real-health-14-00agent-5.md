# real-health-14-00agent-5 — the 14 00 Agent walk by another route

Fresh-boot [L] on 07000014: Bond placed at (45.6, 3.05, 39.87) facing west stops at x 45.134 and stays there, the same number the port holds. So the mezzanine west lip does not release Bond there and the push-tail rule from s/real-health-14-00agent-4 is not the fix; that branch is dropped, do not reuse it. Merge origin/master first. Find how the 00 Agent walk gets past this spot without the lip (jump, a different leg, or the route the Agent walk uses), then land the d2 with_real_damage wrap and the d3 un-ignore. Run every test that uses changed code.

---
claimed-by: sandbox4 at 2026-11-04T21:18Z

---
## Report (done, 2026-11-04T21:51Z)


---
## Report from the worker (done, 2026-11-04T21:51Z)

# real-health-14-00agent-5 — the 14 00 Agent walk: the lip closed, the real blocker named

Branch `s/real-health-14-00agent-5` = origin/master (690e8fefb) + this window's evidence commits
(e62553cd5, b29ce7581). Status: **the task's lip question is CLOSED with measurements; the d2
`with_real_damage` wrap and the d3 un-ignore are MEASURED and BLOCKED by the same previously
unmeasured wedge — the 1.72-landing hover — whose mechanism is now identified at contact level;
the tree is parked at master's byte state with the whole level-14 walk binary re-run green
(9 passed / 0 failed / 1 ignored, 189.4 s).** The wrap is one line and cannot land while the
walk it wraps wedges; the d3 note in `crates/nf-view/tests/walkreal_07000014_live.rs` carries
the full record for the next window.

## What's done

1. **The fresh-boot hold is the GAME's own behaviour — reproduced on the port, and the -4 rule
   is confirmed dead.** New probe `crates/nf-view/examples/probe14_lip_route.rs`, arm A: Bond
   posed crouched (cc 0.65) at (45.6, 3.05, 39.87) facing west, 600 forward ticks — x pins at
   **45.1339**, the driver's fresh-boot [L] number (45.134). The port holds exactly where the
   game holds; the -4 `honest_ref` "release" was a divergence, and that branch stays dropped.
2. **How the walk gets past the spot without the lip — the ROUTE, measured three ways.**
   - Arm B (the walk's own legs, the walk_to steering law, standing): the lane south leg dips
     west at **z 42.37** — at the flush vent lid, north of the wall — and all three legs reach;
     the crossing ends at (41.898, 2.795, 39.025), on the connector at the #301 band's edge.
     Master's green walks cross here; no leg ever pushes west inside the hold band.
   - Arm C (the hold band's z extent, crouched west crawls): **held at z 39.87; PASSES at every
     z ≤ 39.4** (39.4 / 39.0 / 38.6 / 38.25 all reach x 42.16, the #301 rim). The hold band is
     only **z ≈ 39.9..42.2** — the lane's west wall over the mezzanine-length section, south of
     the lid and north of the connector's south opening.
   - Arm D (the jump candidate): 30 hops at the pin — **held**, x never leaves 45.1339. The hop
     does not clear the lip.
   - Arm E (the stance): a standing west crawl at the same pin — **held identically** (45.1339).
     No stance trick exists; the fresh-boot spot simply sits inside the band.
   So: no collision-rule change is warranted anywhere near the lip — the game holds, the port
   holds, and the walk's route (and any re-land of the -2/-3 crossing diffs) must keep crossing
   at the lid or the south opening, never a straight west push at z 39.9..42.2.
3. **The d2 `with_real_damage` wrap — LANDED IN CODE, MEASURED, and parked red: the wrap is
   blocked by the wedge, not by damage.** With the wrap applied (one line, the d2 test body),
   the d2 run on master's tree reaches tick **25740** and wedges at the "1.72 landing" leg:
   Bond floating at **(33.90, 0.68, 35.38)**, d 0.81 short of the slab-top target, `grounded=
   false`, health **32.5** (36 hits for 67.5, 31 return-fire fights — the -3 report's d2 budget
   number reproduced exactly: the fights' damage SURVIVES). The resupply detour inside the leg
   had already parked him floating at (33.90, 0.68, 31.14). A real-health d2 baseline still does
   not exist, so the wrap was reverted with the tree parked green rather than pushed red.
4. **The wedge mechanism identified at contact level — it is a HOVER DITHER, not a static trap.**
   New probe `crates/nf-view/examples/probe14_landing_float.rs`: a bare pose at both float spots
   grounds in **< 100 ticks** (the fall completes) — so the wedge needs the walk's live state.
   Instrumented run (temporary trace, removed): `grounded` **flickers on single ticks** between
   floats — each ground tick zeroes `vel` [M the grounded tail], the next airborne tick
   re-accumulates ~0.003 of fall, and an up-facing push gives it back — pos never descends,
   `float_ticks` never accumulates, the walker's recovery hop never fires (`grounded_history`
   decays), and the resupply/leg steering makes no progress for the whole 6000-tick cap. This is
   the port's fallback radial push holding the capsule at a fixed offset where the game's
   ref-guarded tail [M 0x109da0:431-447; the -3 report's ref reading: +0x50 boots (0,0,0) [L
   ee_start] and its only writer needs a prior push — structurally dead out here] would give
   none. A tried fix — an opt-in walker `float_settle` (45 dither-tolerant airborne ticks →
   quiet no-input ticks) — was **measured ineffective at the pocket** (60+ quiet ticks, vel 0.0,
   no descent: the hover is not input-sustained) **and harmful at the trench** (it quelled the
   pit-band jump legs' own recovery and broke the trench assert at tick ~1800), and was
   **reverted**; only the measurement survives, in the d3 note.
5. **The d3 un-ignore — blocked by the same wedge**, with the note in the test file rewritten to
   carry: the original measured run, this window's d2-twin measurement, the hover-dither
   mechanism, the closed lip question, and the ordered next levers. The `#[ignore]` stands (the
   walk wedges; un-ignoring would turn the binary red).

## How it was verified

- Every run whole, release, `--test-threads=1`, real ISO (`NF_ISO`), on the exact tree named;
  `git status --short` checked before each commit; only the task's files staged (the two probe
  examples, the d3 note, WORKLOG, this report).
- The parked tree is **master's byte state** for every source file (verified by `git diff` and
  by measurement): `walkreal_07000014_live` whole — **9 passed / 0 failed / 1 ignored** (189.37
  s; the d1 real-damage walk green, the driver's bar). The only code-file change vs master is
  the d3 doc-comment note (comment-only, no behaviour); the probes are `examples/` and are not
  in the test suite. No other crate is touched, so no other binary's behaviour can differ.

## What's left (the next window's levers, in order)

1. **The sim-side hover fix at the pocket** — the principled fix and the one that unblocks both
   deliverables: a SCOPED push rule that lets the held fall complete where the game's push tail
   is structurally dead — scoped to the failed-ref up-facing hold (the -4 mechanism, but NOT the
   dropped branch's global `honest_ref`) or to this pocket's cel (24). Then the d2 wrap and the
   d3 un-ignore are one-liners. The full walkreal battery (rule 7/10) applies to any nf-game
   change.
2. Walk-side alternative: a d2/d3-gated fight-quiet approach to the descent so the fights'
   displacement never reaches the pocket (the displacement, not the collision, is what puts Bond
   in it — d1, with milder fights, lands the slab every run).
3. The stairwell AIM-pair stands (the d3 note's third lever) for the fights' budget headroom.

## Live checks for the main agent

1. **The under-slab pocket (07000014, ~(33.9, 0.7), z 31..35.5)** — with the game's Bond dropped
   into it (a grenade-jump or a fight knockback off the descent's pad): does the REAL game hover
   `grounded=false` over the corridor floor, or land? The port hovers (the fallback push). This
   decides lever 1's shape (a scoped push rule vs a floor-band fix).
2. **HITTEST+0x50 (BLData+0x790) during the pocket hover** (gdbwatch.py, read 4 f32s): if it
   stays ≈(0,0,0) while the hover holds, the game's tail is dead exactly as the -3 report read
   and the scoped rule's guard is `ref ≈ 0` (the failed-ref arm).

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere in this
environment's interfaces for this session.
