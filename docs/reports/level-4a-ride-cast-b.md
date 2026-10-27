# level-4a-ride-cast-b — the anim-object placement decoded to the metal; the compose site is provably NOT in the script path and the deciding live check is specified

## What was asked

Find the mechanism that places the 0700004a ride landing cut-scene's cast (pack 0x060006cf, SP #289
base (0, -53.53, -3.38)) in the car despite the car-local sequence roots (0x040005fd root
(2.35, 0.85, -0.64) [D]), port exactly that, and keep every other cut-scene's cast where it is.

## The decode [M — the decompile + mipsdis over work/elf/ACTION.ELF]

1. **The object's position IS the sequence's root track.** `Script_AnimStart` 0x1d8f40 creates the
   object at the origin (`control_create_object(0, 0x2d6200, 0x2d6200, 0)` [M 0x1d8f40:69;
   0x2d6200 = the origin vector [L ACTION.ELF]]), then seeds obj+0x30 with `AnimGetFirstTrans`
   — the root translation at the start frame, gated on the anim script's 0x800000 flag [M
   0x1d8f40:94; 0x1c7348], which `AnimScriptTick` 0x1c2630:43 sets on EVERY ticked script — and
   `AnimFrameResolve` 0x1c35f8 adds the root track's per-frame delta to obj+0x30 afterwards.
   The renderer's bone palette is built under the object's OWN matrix obj+0x90..0xcc [M
   psiBuildMatrixPalette 0x111498:134-160] — nothing in the script path rewrites obj+0x90 for
   script anims (AnimObjectUpdate only rebuilds it on the 0x20000-aim arm / the +0xff==3
   footstep arm, neither of which script anims hit).
2. **The per-tick anim stream composes no base** [M Script_Interp 0x1d9a08 case 2:140-199 — it
   advances the frame, calls AnimObjectUpdate, flags 0x20, footsteps; no GetInterp, no matrix].
   (The ENTITY streams' case 1 DOES compose the parent chain + the SCRIPTINFO base [M case
   1:91-138] — the port already models that.)
3. **The anim-start GetInterp call is a dead compose.** The disasm shows `Script_AnimStart`
   passing GetInterp's base-concat flag SET (the o32-stack arg `sw a2,0(sp)` with a2=1 at
   0x1d90b4/0x1d90d8 — the callee's prologue maps it via `lb v0,208(sp)` [M 0x1da158:8], and its
   tail is `if (base) Concat(SCRIPTINFO, out)` [M 0x1da158:195-197]) — but the output matrix
   (sp+16) is never read by this caller [M disasm 0x1d90f8-0x1d9118: after the jal only obj+0x20
   is tested for the room link]. The SP base composes onto the interpolated matrix at anim-start
   and the result is dropped.
4. **The parent hierarchy exists but script anims have no parent.** obj+0x1c = the parent obj_tag*
   [M Control_CreateObjEx 0x1343c8:26 writes it; `AnimProcessScriptCmds` 0x1c1b38:282-301 composes
   a command placement against the parent's matrix rows +0x90/0xa0/0xb0 and position +0x30] — but
   `control_create_object` (the Script_AnimStart path) never sets it, AnimObjectNew memsets the
   sAnimObject (the PreTrans callback at +0x20 stays zero), and no other writer of obj+0x1c exists
   in the ACTION.ELF decompile set. `Script_Update` 0x1da998 is the clock driver only;
   `Script_Run` 0x1d8548 is a pure opcode interpreter; the renderer (`View_DrawObjList` 0x1e69a0 →
   `AnimObjectDraw` 0x1c4490) passes the object through unchanged.
5. **The pack's data [D]** (pinned in ride_cutscene_objects_live now): the guard's body sequence
   0x040005fd carries root ch0/1/2 = (2.35, 0.85, -0.64) with header flags 0x0 (222 channels);
   the facial 0x040006e6 has flags 0x8 (22 channels, zero root). The port draws the guard's root
   at y 0.56; the game shows him in the car's hatch (the car parked at y -54.2 [L ride frame
   700]; SP∘root ≈ (2.35, -52.68, -4.02) — the deck).

**Conclusion:** the compose that puts this pack's cast in the car is NOT in the script/stream
path and NOT in the object's placement fields — it must happen in a render-side or object-level
link this decompile set does not expose, or the live object's fields differ from every static
path above. The deciding measurement is the live check below; landing a guess (the SP-base
compose on every cast) broke 07000014 once already (d6af2214's revert), so this round lands the
decode + the pins instead of a wrong mechanism.

## What landed

- `crates/nf-game/src/nis.rs` — the full placement decode documented on `Script::anim_start`
  (this section, [M]-tagged).
- `crates/nf-view/tests/ride_cutscene_objects_live.rs` — the car-local roots PINNED [D]: the
  body sequence 0x040005fd's root channels (2.35, 0.85, -0.64), flags 0x0, the facial 0x040006e6
  flags 0x8; the drawn-y diagnostic keeps the open mechanism printed (not asserted) with the
  game's expected deck height.
- No behavior change: every other cut-scene's cast draws exactly as before (cutscene_models_live
  and the 07000014 scene stay green — see the suite record).

## What I ran

- `cargo test --offline --release -j2 -p nf-game --lib -- --test-threads=1` — **578 passed / 0
  failed / 5 ignored**.
- `ride_cutscene_objects_live` — **1 passed / 0 failed** (the new [D] pins included).
- The fast runner (tools/test/suite.py) on the final tree, targeted set:
  **cutscene_models_live, ride_cutscene_objects_live, cutscene_08_actors_live,
  walkreal_07000001_live, walkreal_0700000a_live, walkreal_07000005_live,
  walkreal_0700001b_live — 31 passed / 0 failed / 4 ignored (2 min)**, no FAILED/TIMEOUT/CRASH.
- The fast-runner fast-unit chunk (226 non-live binaries): it outran the tool-call window; the
  partial record shows the environment-shaped failures only — `probe_pylon_render` (the no-GPU
  sandbox, the fix_0c_pylon family), `identity_disc` (the slim image's disc check) and
  `mp_online_failures` (exit -9, the sandbox's kill). The full 845-binary suite does not fit the
  2-hour window in this sandbox (~13.6 s per ISO-live binary measured); the touched surface is
  the targeted set above.

## Live checks for the main agent

Fresh boot of 0700004a, the lift ride, during the landing cut-scene (pack 0x060006cf playing,
SP #289):

1. **The guard's anim object fields** (skin 0x0500004, stream s3 — find the obj via the skin's
   hashtable): read obj+0x30 (the position), obj+0x90..0xcc (the draw matrix), obj+0x1c (the
   parent pointer) per frame around ride frame 700. Expected: if obj+0x30 reads ≈ (2.35, 0.85,
   -0.64) (car-local) while obj+0x90's translation reads ≈ (2.35, -52.68, -4.02) (in-car), the
   compose happens between them — hook the writer of obj+0x90 (a write-watchpoint) and name the
   function; if obj+0x1c reads a non-zero obj pointer, the parent compose [M
   AnimProcessScriptCmds 0x1c1b38:282-301] is the mechanism and the port parents the cast to
   that object.
2. **The 07000014 control-room scene's cast object** (the same reads, SP base (52.16, 2.64,
   46.62), roots near (26..34, 1.2, 48)): its obj+0x90 should read the world roots directly —
   the contrast between the two scenes' obj+0x90 values IS the discriminator the port needs.

## What's left

- The compose-site identification (the live check above) and the port of exactly that mechanism,
  gated so 07000014/world-root scenes stay untouched.
- The three walkreal_0700000a trio un-ignores and the full-suite total line — carried from the
  level-0a band-settle line (the branch's sibling report covers that task's open items).

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
