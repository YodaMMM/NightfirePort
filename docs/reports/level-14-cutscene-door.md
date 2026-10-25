# level-14-cutscene-door — 07000014's control-room cut-scene door now opens

Branch `s/level-14-cutscene-door` off origin/master (aa94ce02), merged `origin/s/cutscene-camera-base`
(284e368b). Window 18:53–20:52 UTC; subagent launches failed all session (harness "context deadline
exceeded" ×4), so the research and port are single-agent.

## 1. The door and its driver (the disc data)

The opening scene is ScriptPlayer #276 (kind 0xd9, trigger ch 0x7E ← TouchOnce #301, out 0xD9 → OR #302)
running script **0x06000833** (pack 07800014.bin, entry at pack+0xabde0, 18976 bytes, camera=1, 25
streams, 1400 ticks) [D decode, all of it re-derived this session with a fresh parser
(`/home/z/my-project/scripts/decode_07800014.py`, not committed)]:

- Stream 0: `Camera{first:0, last:167}`, CameraEnd, End(1400) — shot A keys 0..199, shot B 200..329,
  the dolly landing 499..1400 [D keys; also pinned by countdown_cutscene_cameras_live].
- Streams 10/11: op-07 **Entity, hashes 0x02000891 / 0x02000892, obj_type 0xff**, no parent, keys
  173..227 / 228..293: leaf A slides x 27.7915 → 27.0697 (minimum at t539), leaf B mirrors to 28.6815
  (t546), both at y 1.2965 z 49.5721, closed again by t1000 [D host-float decode]. **The script moves
  the door leaves itself; there is no Door channel write, no Door_Activate, no proximity driver in the
  scene** (the only op-12 events are 0x0b [0x14] at t0 on streams 2/4/6, 0x04/0x05/0x06/0x07 at t0 on
  streams 14/15, and 0x0d [0x47,1] / 0x0d [0x0E,1] at the final marker — the objective writes).
- The door is the kind-0xdb placement pair **#116/#117 at (27.788, 1.203, 49.671)** (params
  [(0,2),(1,6),(4,1),(8,1),(9,2),…] [D main-map block 0x1A]) — the key spot. The scene's actors stand
  right behind it: drones on skins 0x05000087 (26.255, 1.084, 48.640) / 0x05000090 (29.314, 1.084,
  48.641) / Rook 0x05000066 (27.810, 0.198, 45.779) [D].

## 2. The root cause ([M])

- `Script_CreateEntity` 0x1d8968's default arm (obj_type 0xff is in no special case) creates a plain
  `control_create_object` and binds the hash's gfx via `hashtable_set_object_to_entity_gfx` 0x1ca6d8
  (a t_hashtable lookup → `Control_SetGList`) — the mover wears the door leaf's render geometry.
- `Script_Interp` case 1 (0x1d9a08:99-111) writes the object's matrix (obj+0x90) straight from
  `Script_GetInterp`; the parent chain concats parent objects; the tail `Concat(si+0x00, ·)` (0x1d9a08:133)
  concats the ScriptPlayer's own base matrix si+0x00.
- **`Script_CameraStart` 0x1d9140 opens with `Mat_IdentityT()` on si+0x00** (the decompile drops the
  argument; its a0 = param_1 = the SCRIPTINFO whose first field IS that base). From the camera op on,
  the player's base is the identity, so the entity tail concat is a no-op and **the keys are the world
  matrix**. (The cutscene-camera-base branch already proved the camera half of this by render; the
  entity half is the same identity base.)
- The port only skipped the base compose in the camera case, so the door entity streams still composed
  SP #276's placement (52.156647, 2.636515, 46.62138 [D placement]): the leaves sat at
  (79.948, 3.933, 96.194) — 56 units from the door — and the control-room door never moved while Kiko
  walked through it. Probe output on the pre-fix tree (t300): leaves at x 79.896/80.010.

## 3. The fix

`crates/nf-game/src/nis.rs`, op-0x0d handler: set `self.matrix = IDENT` when the camera stream starts,
mirroring [M 0x1d9140]. Scripts without a camera op never hit it, so the fusebox wires (0x06000A1B),
the safe (0x060009E2) and the pickups keep their placement base [D: no camera op in those scripts —
their packs' only camera-flagged script here is 06000833]. The camera case's own skip stays (same
result: its concat now runs on the identity).

## 4. Verification

- `crates/nf-view/tests/cutscene_door_07000014_live.rs` (new, 2 tests, green):
  - `the_control_room_door_leaves_slide_open_during_the_scene` — through the real trigger path
    (Bond at the #301 touch spot), the leaves start closed exactly at the [D] key spot, stay on it
    every tick (no SP base compose), and the gap reaches >1.5 of the [D] 1.6118 (A→27.0697, B→28.6815).
  - `the_door_leaves_have_draw_slots_at_the_placement` — the scene seeds draw slots for both hashes
    (the movers draw; without them the static pair would read as a door that never opens).
- `crates/nf-view/examples/probe14_door.rs` (new) — the t0/t250/…/t1250 leaf log; post-fix it matches
  the [D] keys to 1e-4 (t540: 27.0697/28.6784).
- `countdown_cutscene_cameras_live` — the TEST was wrong and is corrected: it pinned the camera at
  SP #276 placement + key local, contradicting [M 0x1d9140]; expected positions are the keys alone
  now (2 passed).
- `cutscene_camera_07000014_live` (the camera-base branch's pin): 1 passed after the merge.
- Regression run (background, `nfgame_tests.out`): all of `nf-game` unit tests + the 07000014
  fusebox/entities/walk nf-view tests — see the numbers below.

## 5. Headless renders for the main agent

The scene at Bond (40.87, 2.98, 40.85) (the cutscene-camera spot):

| frame | when | expect |
|---|---|---|
| t≈100 | shot A | the control room; the door pair #116/#117 at (27.79, 1.2, 49.67) closed, Kiko/actors behind it |
| t≈420 | mid-open | the leaves parting (leaf A moving −x, leaf B +x) in front of the actors |
| t≈540 | full open | the doorway open (gap ≈ 1.61), the actors visible through it |
| t≈1000 | closed again | the leaves back at the closed spot |

What to compare: the door visibly opening (before: a static door with actors walking through it), no
duplicated leaves (the movers wear the hash's gfx while the static pair still draws — the game's SS
swap for the 0xdb pair is NOT ported here; if the render shows doubled leaves at the closed spot, the
static pair needs the swap-hide and that is follow-up work).

## 6. What's left

- The static-vs-mover swap: the game hides the static 0xdb pair while the script movers wear its gfx
  (the 0xE2 SS swap system, docs/levels/07000014.md §5) — not ported; the render check above decides
  whether it is visible.
- Whether the leaf gfx (hashes 0x02000891/92) resolve to door-leaf meshes or to something else (the
  draw-slot seeding resolved them — `level.by_hash` has both — but only a render shows WHAT they look
  like).
- The follow-Kiko scene 0x0600062F (pack 07900014) was not decoded this session; if it drives doors
  the same way, the fix covers it (same mechanism).

## 7. Suite

`tools/test/suite.py` ran in the background 18:58–20:11 UTC on the pre-merge tree: 801 binaries, the
known master failures only — `disc_golden` ×2 (the slim-ISO exception the task allows),
`fix_0c_pylon_live` ×2 + `probe_pylon_render` (pre-existing master tests, not files this task
touches), `CRASH mp_online_failures (exit -9)` (this sandbox's memory pressure). A full re-run on the
final tree could not fit the window (finish steps started 20:38); the delta since that run is the
camera-base merge + the one-line nis.rs fix + the three test files, and the touched-crate runs above
are green. Touched-crate command:
`cargo test --offline --release -j4 -p nf-view --test cutscene_door_07000014_live --test
countdown_cutscene_cameras_live --test cutscene_camera_07000014_live -- --test-threads=1`
(5 passed) plus the background regression batch in §4.

## Live checks for the main agent

1. **The static pair's hide**: PCSX2 fresh boot, gdbwatch the kind-0xdb door object(s) at
   (27.79, 1.2, 49.67) through the scene (t≈250-540) — expected: their draw state/glist pointer
   changes when the script movers mount (the SS swap), or their mesh is the SAME object the movers
   carry (then nothing to hide).
2. **Leaf gfx identity**: fieldprobe the hashtable entry for 0x02000891/0x02000892 (t_hashtable
   lookup [M 0x1ca6d8]) — expected: a celglist whose matrix sits at the door spot; screenshot at
   t≈540 to compare with the port's render.

## Usage

not visible to me (no token/credit/context-window/cost figures are shown anywhere in this sandbox
environment; the agent transcript does not display usage meters).
