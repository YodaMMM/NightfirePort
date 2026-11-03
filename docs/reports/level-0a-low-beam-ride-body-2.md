# level-0a-low-beam-ride-body-2 — report

Branch `s/level-0a-low-beam-ride-body-2`, cut from origin/master 6f16146e. The sandbox carried
from the previous session; the pending level-12 finish steps (the queue attach + the ntfy-failed
report section) were closed first.

## What's done

**The law work is complete and cited; the code change is NOT landed** — both prior attempts'
blocker survived the two candidate laws, and the session's net code delta is a diagnostic probe
(`crates/nf-view/tests/probe_sphere12.rs`, `#[ignore]`d) plus this report. Parity is green on
the revert.

1. **The arms table settled [M]** (`00194df0_Player_Collision__FP7obj_tag.c`): the default arm
   (the walk substate AND the vehicle take-control 0x10) is **r 0.55, A = pos+up·0.275,
   B = pos+up·(0.55−cc)** [:315-333 — 0x3f0ccccd = 0.55 the radius, 0x3e8ccccd = 0.275 A's
   offset; the previous session's table was right and the low_beam test doc's "r 0.275" misread
   A's offset for the radius]. **cc = the anim parent object's +0xcc root height** [:86 —
   `fVar20 = *(*(obj+0xdc) + 0xcc)`; the port's ROOT_HEIGHT 1.0347183 [L trace] standing,
   CROUCH_ROOT_HEIGHT 0.55] — so the standing capsule bottoms **at the feet** (pos−1.0347).
   0a's #621 band y 0.13–0.39 [D] is inside a standing body's reach (pos 1.13 → capsule bottom
   0.095): the live firing [L 2026-11-02 fresh boot] needs no sphere — the game's own default
   arm covers the band.
2. **The game's trigger contact gate read [M]** (`001eae80_Collide_PickObj`): by contact type —
   0x200 = the toucher **segment** vs the object's **world sphere** (centre obj+0x80, radius
   obj+0x8c) [:93-107]; 0x800 = the same at r×1.5 **+ the toucher's own radius** [:152-166];
   0x100/0x400/0x1000 = sphere-vs-sphere sums [:226-228]. The world sphere is built by
   `Control_BuildWorldSph` 0x133070: the **glist's LOCAL sphere** (the port's parsed
   `MapObject::centre`/`radius`) through the obj matrix, the radius scaled by |obj+0xe8|
   [:33-49]; a glist-less object keeps the centre at its own translation with the 0.1 default
   radius [:23-30].
3. **Both laws implemented and measured** (on the working tree, then reverted):
   - the `Trigger.sphere` field fed in `build()` from the parsed bounds + a local-bbox-sphere
     stand-in at the script-trigger site (the ch-0x70 park plate is SCRIPT stream 6 [D
     fix-truck], so a placement-only sphere misses it), with touched_by running the
     capsule-vs-sphere gate ahead of the mesh tests;
   - the standing-toucher swap to `player.body` (the game's default arm).
   Results: **the sphere gate alone → fix_truck_live 0/2** (the script triggers' bbox spheres
   fire the standing park plate); **the arm swap + the spheres → fix_truck_live 0/2** (the
   standing plate fires through the feet-level capsule — the plate 0.46 above the bed floor
   [D 37.28 floor vs the 37.7418 plate] is inside [pos−1.0347, pos+0.825] — and the ride's
   chain opener still needs the old box's upper reach at the ch-0x5C gatehouse touch, pos+1.05).
4. **The reconcile gap, now precise**: the game's player-vs-trigger contact must (a) fire #621
   standing — the feet-level capsule does; (b) keep #607 silent crouched — the crouch arm's
   volume on 0a is [0.03, 1.33] (pos 0.68 crouched) and #607's 1.49–1.76 band is ABOVE it, so
   the arms are consistent here; (c) keep the standing ride off the park plate — per the
   fix_truck sister, which is NOT live-verified (the trace's play rode crouched; the decompile
   arms predict the standing ride FIRES it). The conflict is (a)+(c): the same feet-level
   capsule that reaches #621's band also covers the park plate. **The refine step between the
   collide pass's sphere pick and Trigger_Hit's contact list — or the ride's own contact
   window — is the missing law.**
5. **The sphere dump** (the committed probe): 0700000a #621 and #607 both r 2.63; 07000005's
   spawn plate 36 r 1.56; the ch-0x22 gatehouse volume (plc 305) r 11.01 — and the ch-0x5C
   gatehouse touch + the ch-0x70 plate are script-made (they exist only once their script
   entities go live).

## What's left

1. The three live checks below settle the refine law; then the implementation is one pass:
   the refine in touched_by (or the contact-list model), the standing arm swap, the un-ignore,
   and (if live confirms the arms) the fix_truck sister correction with the addresses.
2. The full suite on the final tree.

## Live checks for the main agent

1. **0a, fresh boot, Bond STANDING crossing #621's line** (the live firing): `gdbwatch.py` on
   `Collide_PickObj` 0x1eae80 while the contact list builds — record the HITTEST type word the
   player-vs-trigger pass carries (0x100/0x200/0x800/0x1000) and the toucher shape (the segment
   +8/+0xc, the sphere at +6/+0xc). Expected: whichever arm reaches the band pins the toucher's
   game shape.
2. **0a, the same boot, Bond CROUCHED under #607** (the live silence): the same watch. Expected:
   NO contact despite the world sphere's overlap (#607 r 2.63 swallows the crouched body) — the
   delta between this and check 1 IS the refine law.
3. **07000005, the standing ride** (no crouch after the hand-off): does ch 0x70 fire? The
   decompile arms predict YES (the feet-level capsule covers the plate 0.46 above the bed
   floor); the fix_truck sister asserts NO from the old measured box. Whichever the game does,
   the sister stays or flips with the address.

## What I ran (this branch, release, NF_ISO the slim on-foot image)

- `fix_truck_live`: 2 passed / 0 failed (the parity revert); 0/2 under both candidate laws
  (the evidence above).
- `walkreal_0700000a_live`: 10 passed / 0 failed / 3 ignored (low_beam_621_fires_standing back
  to ignored on the revert).
- The probe binaries (the sphere dump; the spawn-touch comparison: at the 05 spawn only the
  ch-0xBE plunge touches, identically under both touchers — the chain opener is ride-time).
- The full suite: not run — the session went to the law read + the two measured attempts; the
  tree's net change vs master is the probe + the docs (no lib code), so the suite risk is nil,
  but the run is owed on the next pass.

## Usage

not visible to me
