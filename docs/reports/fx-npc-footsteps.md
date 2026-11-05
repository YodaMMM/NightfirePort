# fx-npc-footsteps — the NPC footstep Effect_Create (audit row 2)

Branch `s/fx-npc-footsteps` (cut from origin/master 17a784980, which already carries the
audit-effect-calls merge). Shell casings out of scope, untouched.

## What's done

Row 2 of audit-effect-calls's "Unported calls" table listed two call sites; both are ported.

1. **The per-frame NPC footstep (`AnimObjectUpdate` 0x1befd8)** — commits c974610a6 + e336b3d96.
   The game's raise is per-frame and per-object: `ScriptFootstepType` (0x30ce10) is cleared at the
   TOP of every `AnimObjectUpdate` call (disasm 0x1bf0c4 `sb zero, -0x31f0($at)` — the decompiler
   DROPPED this store, the C looks unconditional-set), set by the anim script's op-4 footstep
   markers during that object's own script tick (`AnimProcessScriptCmds` 0x1c1b38:372-401: op 0
   toggles `ScriptFootstepFoot` 0x30cde0, op 4 → false, op 5 → true, then `ScriptFootstepType` =
   1 for scripts 0x6000099/0x60000ae else 2), and consumed once right after [M 0x1befd8:191-208]:
   `Effect_Create(1.0, obj+0xdc hitdata, obj, ScriptFootstepFoot ? 8 : 4 | 0x42)` — suppressed
   ENTIRELY while `ScriptCam != 0` [M 0x1bf25c-0x1bf260]. The distance accumulator (stride 1.65,
   0.4125 when running) is the OTHER raise path and belongs to obj kind 3 = the PLAYER only
   [M 0x1befd8:159-167, `Player_Init` 0x1910f8:76 sets obj+0xff = 3]; its run branch
   (`Sound_Play3D(0x286)` + `Sound_SetAlertness`) never fires for NPCs. The port consumes the
   markers at the tick's drain (drone.rs `Drones::tick`): the surface's footstep sound
   (`Effect_Player` 0x1cec48:59-70) AND the footprint decal (:84-136, gfx = `EffectInfo[mat]+8`
   foot 4 / `+0xc` foot 8, x/z the foot-bone world matrices (palette slots 55/51), y = ground
   + 0.05 behind the |foot_y − hit_y| ≤ 0.2 gate, aligned up, cel-hosted), behind
   `Effect_Create`'s material early-out [M 0x1ce688:59], coalesced to one consumption per drone
   per frame (the game's +0x60&4 is a flag, one `Effect_Create` per object per frame
   [M 0x1befd8:157-208]).

2. **The Script_Interp case-2 footstep (the NIS anim streams) — this session.** A stream-driven
   anim object ticks `AnimObjectUpdate` SYNCHRONOUSLY inside `Script_Interp` case 2
   [M 0x1d9a08 case 2, disasm 0x1d9f2c `jal 0x1befd8`], and its op-4 markers raise the same flag;
   the INNER consumption stays suppressed because a script camera is running, so the stream's own
   trailing block consumes instead [M 0x1d9f64-0x1da020]: a fresh 4-unit downward ray from
   obj+0xc0 (`Vec_Copy2(obj+0xc0, A, B); B.y -= 4.0`, `build_FindCel(A)`,
   `Collide_RayIntersect(A, B, cel, obj, 0, &hits, 0, 0xf0d)` [M 0x1d9f7c-0x1d9fd0]), the gate
   `rayhit.y − hitdata+0xcc ≤ 0.2` [M 0x1d9fdc-0x1d9ffc, one-sided — rejects props ABOVE the
   stored standing ground], then `Effect_Create(rayhit, obj, ScriptFootstepFoot ? 8 : 4 | 0x42,
   1.0)` [M 0x1da01c]. **No ScriptCam gate** — the trailing block reads only `ScriptFootstepFoot`
   and `glb_world` [M disasm 0x1d9f30-0x1da028]; the cut-scene's own script camera is WHY the
   inner consumption is suppressed, not a gate on this one. Port: `Request::NisFootstep { at,
   ops }` (nis.rs) — case 2 collects the stream script's op-4 footstep markers in the (from, to]
   window (`World::anim_ops4_footsteps`) and the request carries them in command order;
   `Drones::drain_script_requests` applies them to the ONE global alternator
   (`Drones::script_footstep_foot`, the port's `ScriptFootstepFoot` 0x30cde0) then consumes: the
   same full Effect_Player arm as the drone path, ray hit 4 units under the stream object
   (`segment_surface_host`), sound at the ray hit, decal x/z from the posed foot bone
   (`World::anim_foot_matrix`, palette slot 55 foot 4 / 51 foot 8, stride 0x40 [M 0x1cec48:89-110:
   55·0x40 = 0xdc0, 51·0x40 = 0xcc0]), y = hit + 0.05, the ≤0.2 gate, no camera gate.

## How it was verified

All on the real ISO through the real runtime, `--test-threads=1`, release:

| Binary | Result |
|---|---|
| `nf-game npc_footsteps_live` | 2 passed — the per-frame path: one footstep start per marker, ev 4 → foot 4 (mesh `EffectInfo[0x4]+8` = 0x02000567 [L parsed EffectInfo]), ev 5 → foot 8 (+0xc = 0x02000568), y = ground + 0.05, cel host, and `ScriptCam != 0` suppresses the step entirely |
| `nf-game nis_footsteps_live` (NEW) | 1 passed — the case-2 path: 07000005's ledge cut-scene (SP 298, NIS 0x06000994 [D ledge_live's decode]), started through its real plate chain (lock 0xb3 → plate out 0x43), Bond's stream 14 (`Anim { script 0x06000995, skin 0x05000007 }`) leaves THREE prints in the first 90 ticks from the script's real markers ev 4 @8 / ev 5 @56 / ev 4 @90 [D the parsed script] with meshes foot 4 / foot 8 / foot 4, each y = ray hit + 0.05, and the surface's footstep sound starts — WHILE the cut-scene's script camera runs (no ScriptCam gate) |
| `nf-view footprint_decals_live` | 1 passed — Bond's own walked segment (the audit's fix 1, regression-watched) |
| `nf-game nis_handback_census` | 1 passed, 1 ignored |
| `nf-game nis_skip_handback` | 2 passed |
| `nf-game ledge_live` | 2 passed |
| `nf-game anim_events_live` | 5 passed |
| `nf-game mp_map_objects_live` | 4 passed |
| `nf-view npc_doors_live` | 6 passed, 1 ignored |
| `nf-view fix_guard_misc_live` | 6 passed |
| every `walkreal_*_live` (107 binaries, nf-view) | see the run list below (drone.rs touched → all run) |

Full-suite note: `cargo test -p nf-game -p nf-formats -p nf-audio -p nf-view` did not fit the
session window (the nf-view test-binary compile alone exceeds it); the binaries above are every
one that reaches the changed code (drone.rs drain + `World` impl, nis.rs case 2 + `Request`).
The three `disc_golden` tests were not run this session.

## Values used (all cited)

- Raise/consume per frame [M 0x1befd8:157-208; the `ScriptFootstepType` clear the decompiler
  drops: disasm 0x1bf0c4], markers [M 0x1c1b38:372-401], `ScriptCam` suppression [M 0x1bf25c],
  accumulator = player kind 3 [M 0x1befd8:159-167 + 0x1910f8:76], run branch `Sound_Play3D(0x286)`
  player-only [M 0x1befd8:200-207], `Effect_Create` material early-out [M 0x1ce688:59],
  Effect_Player decal arm [M 0x1cec48:59-136], foot palette slots 55/51 [M 0x1cec48:89-110,
  stride 0x40 by 0xdc0/0xcc0], case-2 trailing block [M 0x1d9a08 disasm 0x1d9f30-0x1da028:
  ray mask 0xf0d, B.y −= 4.0, gate `hit.y − hitdata+0xcc ≤ 0.2` one-sided, foot 4/8, `| 0x42`,
  scale 1.0].
- Disc values pinned by the tests: 07000001 walk script 0x060000ca markers (ev 4 @30, ev 5 @64),
  EffectInfo[0x4] foot meshes 0x02000567/0x02000568 [L the parsed tables]; 07000005 NIS
  0x06000994 stream 14 anim script 0x06000995 (ev 4 @8, ev 5 @56, ev 4 @90) [D the parsed
  script], its scene's material row read live under the object.

## What's left

- Nothing known for row 2. The two [S] stand-ins stand (documented at both consumption sites):
  the port has no per-NPC/per-stream stored hitdata, so (a) the drone path's ground is this
  tick's feet ray and the NIS path's second gate `rayhit.y − hitdata+0xcc ≤ 0.2` is subsumed by
  the short ray + the material gate (a prop hit slightly above the standing ground would decal
  where the game would not — NIS casts stand on open floors, low risk), and (b) the case-2 sound
  plays at the ray hit (the game plays at the ray hitdata+0x40 — the same point).
- The game's ScriptFootstepType=1-vs-2 distinction (scripts 0x6000099/0x60000ae) changes nothing
  at the consumption (only `!= 0` is read [M 0x1bf22c-0x1bf238]) and is not ported.

## Live checks for the main agent

1. During 07000005's ledge cut-scene (step on the plate left of the castle gate), with a
   breakpoint on `Effect_Create` (0x1ce688): three `Effect_Create(1.0, hit, bond_obj, 0x46)`
   then `0x4a` then `0x46` calls in the first ~90 frames from the markers ev4@8/ev5@56/ev4@90 —
   flags 0x40|0x2|foot, foot 4 = 0x46, foot 8 = 0x4a — and the hitdata+0x50 material = the
   courtyard's. Access type: breakpoint reads of a2 (flags) and a0 (hitdata).
2. Mid-cut-scene, read the stream object's `obj+0xc0` and `hitdata+0xcc` (obj+0xdc): the ray in
   case 2 must start at obj+0xc0 and `hitdata+0xcc` should hold the standing ground y (confirms
   the [S] subsumption of the second gate — write access type: who stores +0xcc, a watchpoint).

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session).
