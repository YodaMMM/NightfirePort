# trigger-hittest-segment — the report

Task: port the game's player-vs-trigger contact so 0a's low laser beam #621 trips on a STANDING
walk-through, with the full chain (the glist narrow phase + the ref-gate/extent, the ride's
class-5 truck contact, the 0a office-ring route). Branch `s/trigger-hittest-segment`, cut from
origin/master a71f9233.

## What was pinned this session — beyond the prior task's chain

1. **The trigger cels' glist flags decide the refine branch.** The trigger placements' target
   celglists carry flag bit 0x40 — [D] measured with the new `crates/nf-formats/examples/
   dump_trigger_glist_flags.rs`: 0700000a #607/#614/#621 (the laserbeams) = 0x645, 07000011 #235
   ("Trigger Bond Moment") = 0x50. `Collide_Intersect` 0x1ebaa8:93-99 turns `obj+0xf0 & 0x40`
   into `HITDATA+0x51 |= 2`, and the refine's asm (0x10a530-0x10a538) routes a set bit-1 PAST the
   ref-point law to the RAW per-triangle distance. The "ref-gate/extent" the prior report chased
   is this branch select — and the ref point itself (HITTEST+0x50) has NO writer anywhere in the
   player's path (BLData+0x790, whole-decomp-tree grep; the collide pass only pushes it,
   0x1ebaa8:353-373), so it only ever governed non-0x40 glists.
2. **The raw law, from the disassembly** (work/elf/ACTION.ELF, capstone; trigger-hittest-segment
   probes): per triangle — the plane-crossing parameter `t = clamp(−(A·N+d)/(N·(B−A)), 0, 1)`,
   anchor P = A + t·(B−A) [M asm 0x10a200-0x10a2f0]; a segment parallel to the plane
   (|N·(B−A)| ≤ 0x3951B717 ≈ 1.78e-4) anchors P at the projection of the triangle's CENTROID
   onto the segment [M asm 0x10a3f0-0x10a4fc] — this is what fires 011's wall-mounted #235;
   the directional skip `(dot > 0.99987 && t > 0.5) || (dot ≥ −0.99987 && t < 0.5)` [M asm
   0x10a278-0x10a2e8 — the decompile's second arm is a branch-likely mangle; unreachable for the
   player's 0.76 segment]; `Dist2Tri` 0x1dfca8's ONE-SIDED entry (the signed plane distance of P
   < 0 = the code-99 miss) [M 0x1dfca8:58-68]; the strict `dist < radius` refine [M asm
   0x10a5c0-0x10a5d0]. The port's existing AABB prune (grow r+0.1) is the game's own box grow
   [M 0x109da0 head 0x109e48-0x109e6c].
3. **The windings/geometry checks that fell out**: #235 is mask 0 → the create's default 2
   (class-5 bullets), a vertical wall quad the standing segment plane-crosses degenerately; #614
   is the ROTATED laserbeam (the line crosses the corridor at z ≈ 13.03, x −24.71..−19.27, band
   z 12.89..13.17); the 0c bay-volume rest miss under the raw law comes from the one-sided entry.

## What was wired and measured — then reverted

`crates/nf-game/src/triggers.rs` gained `seg_touches_tri_raw` (the law above, the asm cited at
the site) and it was wired into `touched_by` for r > 0, with the standing toucher switched to
`player.body` (the game HITTEST segment). Measured on the branch (commit e7f80cd6 in the history):

- **walkreal_0700000a_live: 9 passed / 4 failed / 2 ignored — `low_beam_621_fires_standing`
  UN-IGNORED and GREEN** (0.81 s run), with the crouch-under (#607), the jump-clear (#614) and
  the crouched-#621 sisters all still passing. The 4 failures = the office-ring walk + its 3
  real-damage sisters: the walked route tripped #614's line — the walk's own jump arc crossed the
  line at +0.40 and the raw refine's side-face endpoint distance stayed under 0.55 (the game's
  [L] arc clears at +0.43; the press-vs-arc-height relation is set by the TICKS since the press,
  so the retune is press ≈ 1.8 — the route-side legs are in e7f80cd6).
- **fix_truck_live: 0/2** — the 05 spawn-reveal plate ch 0x45 went silent (the [L] says the game
  fires it 0.557 below the hip on the boot tick), so the truck chain never started.
- ROOT CAUSE of both breaks: **the one-sided entry's sign**. The game's Dist2Tri reads the
  glist's OWN plane records (each tri's 4th u16 indexes a 4-i16 `(n×1e-4, d)` quad — asm
  0x10a283-0x10a1e8) — the AUTHORITATIVE side; the port's parse carries only the tri winding and
  computes N from it, and the signs disagree per cel (0a's beams agree; 05's spawn plate and the
  ring's #614 case disagree).

The wiring is REVERTED to the box + the inclusive capsule — master behavior byte-exact — and
`seg_touches_tri_raw` stays in the tree with the plane-record lever documented at the site.

## State

- `low_beam_621_fires_standing` is `#[ignore]` with the UPDATED message (the chain is pinned and
  implemented; the landing needs the glist plane records parsed).
- The truck ride's class-5 contact (trigger-port-check #11) stayed untouched (master behavior).
- Next session: parse the glist plane records in the level parse (`collision`'s 4th-u16 plane
  table → per-tri (n, d)), feed `seg_touches_tri_raw` the record's plane, re-wire `touched_by`
  (r > 0) + the standing `player.body` (the diff is e7f80cd6), retune the ring's #614 press to
  ~1.8, then re-run the battery below.

## Binaries run (each whole, --test-threads=1, release, NF_ISO set)

On the WIRED tree (e7f80cd6): `walkreal_0700000a_live` 9 passed / 4 failed / 2 ignored (the
target GREEN); `fix_truck_live` 0/2.
On the REVERTED tree (the pushed HEAD): `walkreal_0700000a_live` **12 passed / 0 failed / 3
ignored**; `fix_truck_live` **2 passed / 0 failed**. (Master's own HEAD carries the ignored 03/11
walks — unchanged here.)
The full suite did not fit the window (the disassembly + the two wiring/verify cycles consumed
it); the two binaries above are the ones the touched code can affect, and the pushed tree is
byte-exact with master's behavior in them.

## Live checks for the main agent

1. **The 05 spawn plate's glist plane record** (ch 0x45, the boot-tick fire): read the tri's 4th
   u16 plane at obj+0xD8 and the normal's y sign vs the tri winding — confirms the plane-record
   lever before the parse work.
2. **The standing truck ride** (unchanged from the prior report): does ch 0x70 ever pulse
   standing? It decides whether the ride pin moves to the class-5 truck contact path.
3. **The 0a office ring**: Bond's footpath xz vs the port's walker waypoints at the #614 line —
   the game's own arc clears at +0.43 with the press 0.9-1.3 [L]; the port's full-speed arc
   crosses at +0.40 — a PCSX2 press-distance sweep would pin the game's arc shape.

## Usage

not visible to me (no token/credit/context-window figures are exposed in this environment; the
session's boundary was the ~01:55 UTC sandbox expiry — the wrap-up ran to ~02:10 UTC).
