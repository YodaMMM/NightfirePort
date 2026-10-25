# level-07-guest-faces — 07000007: guests' lips and faces move when they talk to Bond

Branch `s/level-07-guest-faces` (origin/master 7c74d96b + batch-1025i merged). One code change
(`Scene::facial_weights_at`, nf-view) plus the live test and three census probes.

## What was wrong

The guest's voice path was already ported end to end: `NDrone2_BondTalk` 0x1446b0 →
`NDrone2_SpeechSFX` 0x141b00's FACIAL arm (the port's `speech_request`), the `AnimFacialAppend`
0x1c6490 attach stored at DCVars+0x53c (`Drone.facial_talk`/`facial_talk_start`), the script's
op-1 sounds firing per line (`facial_played`). What was missing was the VIEW half — two breaks
in `Scene::facial_weights_at`, the single source both the CPU morph term and
`Renderer::upload_morph_weights` (the GPU half of `SkinIt` 0x10bf30) consume:

1. **The talk slot was hardwired `None`.** The game's weights come only from the three facial
   marker scripts [M `AnimFacialBlend` 0x1be648 builds `AnimListBuild(obj, 0x200, marker)`
   lists; `AnimListBuild` 0x1c0fc0's bit-0x200 predicate is `script+0x80 == marker`;
   `AnimScriptInit` 0x1c1600:58-61 gives every gameplay-started script marker 0, so the body
   anim never contributes in gameplay]. The port blended slot 0 (idle) and slot 1 (blink) —
   both permanently empty (`facial_idle`/`facial_blink` have no gameplay writer) — and passed
   `None` for slot 2 (talk). The attached BondTalk script's weights never reached the face.
2. **A body anim with no facial window returned `None` for the whole array.** The port
   sampled the base from the playing body script's face window and bailed (`?`) when it had
   none. The 07000007 census (probe_guest_face): the guests' idle/walk body anims carry NO
   face window, so a talking guest got "no morphs" at all — the frozen face. In the game the
   base in gameplay is the idle marker script's buffer, and the blend still runs over a zero
   array when nothing else plays; the marker scripts alone drive the face.

## The fix

`Scene::facial_weights_at` (crates/nf-view/src/scene.rs):

- Drone-backed characters with no body face window now blend over a **zero base** instead of
  returning `None`. Script-object (cutscene) characters keep the landed cutscene-faces model
  (`None` — no morph term; the game reads its never-started facial slot's zeroed buffer
  [M `AnimSeqNew` 0x1c7248], visually the same neutral) — pinned by
  `cutscene_faces_live::a_facial_bearing_guest_with_no_face_track_keeps_a_neutral_face`, which
  my first draft had broken; the test stays as it was, the code was fixed to match.
- **Slot 2 (talk) is fed from `Drone.facial_talk`/`facial_talk_start`** (the port's +0x53c
  attach) and passes it to `nf_game::anim::facial_blend` — the |w| > 0.01 override layer,
  exactly `AnimFacialBlend` 0x1be648's lVar6 arm. Slot time runs the talk clock the op-1
  sounds already use (`now_t = frame − start + 1`, 1 on the attach tick) with the window's
  frame = now_t − first + 1; the clock is `Drones.frame` (the value `facial_talk_start` is
  stamped with — `Drone.last_tick` is never stamped anywhere and was always 0, which silently
  disabled every slot on the first draft).
- One decompile note: `AnimFacialAppend` 0x1c6490's C shows a `void` return, but the disasm
  returns $v0 = the `AnimScriptAppend` result (the caller stores it at +0x53c
  [M 0x141b00:18-23]) — the decompile dropped the return, the port already had it right.

No game-side test contradicted the decompile; no test was weakened. The one test my change
broke (`..._keeps_a_neutral_face`) pins the cutscene path my draft had changed — restored.

## How it was verified

- **probe_guest_face** (examples/, the failing-first record): guest d7 (skin 0x05000024,
  voice type 13) takes the FACIAL arm — script 0600037d attached, op-1 sfx 380 sounded —
  while `facial_weights_at` returned `None` pre-fix; post-fix it returns 22 live weights that
  walk the mouth channels during the line (d23 / 05000019 / 06000113 the same).
- **probe_talk_script**: every BondTalk facial script carries exactly one facial sequence
  (flags 0x8, 22 channels), window at frame 1, live weights from frame ~5 — e.g. 0600037d →
  seq 040002ed (64 frames), 06000113 → 04000097 (60 frames).
- **probe_guest_skins**: every party skin is morph-bearing (Skin+0x12 facial = 22, morph
  targets present) — the weights land on real morph targets.
- **`guest_faces_live::a_talking_guests_face_weights_follow_the_line`** (new, opt-in NF_ISO):
  through the real path (load 07000007 → free Bond → approach + aim at the nearest type-13
  guest), the attach happens, the guest's character produces 22 weights that carry live values
  and change across the line (best per-channel range > 0.3 — the mouth flaps), and after the
  script detaches (+0xb5 = 3 stop-at-end) the weights return flat — the mouth stops. Green.
- Regression run (all green, release): facial2_live 2/2, facial3_live 3/3,
  cutscene_faces_live 4/4, facial_gpu_live 3/3, gpu_skin_live 3/3,
  speech_facial_fallback_live 2/2, fix_guest_reactions_live 5/5, guest_faces_live 1/1.

## What is left (the ambient half, not needed for the talking face)

- `AnimFacialUpdate` 0x1be310's idle/blink state machine is ported as a pure function
  (`nf_game::anim::facial_update`, unit-tested) but has **no gameplay caller**: nothing fills
  `facial_idle`/`facial_blink`, so a silent guest has no breathing/blinking face track. Wiring
  it adds the game's real `Rand_Rand` draws (state-0 init, blink cadence, the female roll) to
  the shared stream — gated in the game on the big-skeleton flag + `AnimObjectIsClose(obj, 0xc)`
  (min viewer LOD byte ≤ 12) — so it must be done with the draw order audited against the
  seed-pinned suites. That is the next session's work, not this fix's.
- The eye/lid bytes (animobj+0x97/+0x98 → weight indices 14/13/11/12) and the head-track morph
  bytes (animobj+0x107/+0x108) are not ported; the blend's `eyes` parameter is `[0, 0]`.
- `Drone.last_tick` (drone.rs) is dead — its doc claims it is stamped per drone; it never is.
  Either stamp it or remove it (shared struct, left alone here).

## Live checks for the main agent

1. `gdbwatch` (read) `DCVars+0x53c` of a party guest while he plays a BondTalk line near Bond:
   expect non-zero while the line plays, 0 before and after ([M 0x141b00:18-23 + 0x1c6490's
   stop-at-end]).
2. `gdbwatch` (readwrite) the guest's `animobj+0xb0` array (22 floats) during the same line:
   expect per-tick changes on the mouth channels while the line sounds, all-zero after.
3. `poke` the guest's `animobj+0x109 = 1` mid-line (the speaking byte 2 → 1): the talk script
   keeps playing (the byte is the ambient state machine's, not the talk gate's) — confirms the
   +0x109/0x99 split the port models.

## Headless renders for the merge (named spots, 07000007)

- `--bond 43.5,44.0,-76.7,-0.9,0.15` — Bond ~2.2 in front of guest d7 (skin 0x05000024,
  pos [43.41, 44.00, −78.92], type 13) aimed at his head. Expected once he talks: his mouth
  opens/closes across the line; pre-fix it was a stone face.
- `--bond 44.9,44.0,-60.3,-2.25,0.15` — guest d0 (skin 0x05000020, pos [44.32, 44.00, −62.13])
  face-on. Expected: the same flap while his line plays; neutral closed face between lines.
- `--bond 122.8,43.75,-85.2,-1.4,0.15` — the second crowd (d6 [122.78, 43.75, −83.19] /
  d11 [122.94, 43.75, −87.40]). Expected: either guest flaps when the crosshair sits on him
  and the line sounds.

(The render needs the sim ticking with Bond aimed at the guest so the BondTalk cadence fires;
a single frozen frame between lines shows the closed neutral face, not a bug.)

## Suite

The fast runner (`tools/test/suite.py`) was started at 16:38 UTC, killed by a shell exit
(empty suite.out — background jobs die between tool calls here), and restarted detached
(setsid) on the final tree (origin/master batch-1025i merged) at 17:15 UTC; its final line is
quoted in the WORKLOG append and was checked for FAILED/TIMEOUT before the finish. The eight
nf-view test files touched by this change were also run directly (listed above, all green).

## Usage

not visible to me (this session shows no token, context-window or cost figures anywhere).
