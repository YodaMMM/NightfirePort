# level-14-rook-cutscene — Rook appears in his cut-scene

**Done.** The post-cascade cut-scene at the Delta Sector door on 07000014 is the escape scene
**0x06000848** (SP #733, 950 ticks) and the port now draws Rook in it, at the spot the disc's own
data puts him. One root cause, fixed in `crates/nf-formats/src/anim.rs`, pinned by two tests in
`crates/nf-view/tests/escape_scene_actors_07000014_live.rs`.

## The scene and its actor list [D]

There are exactly three camera scenes on 07000014 — a full enumeration of every kind-7 entry in
`07000014.bin`, `07800014.bin`, `07900014.bin`, `07A00014.bin` (parse failures included) found
cameras only in 0x06000833, 0x0600062F and 0x06000848. The escape scene 0x06000848's camera keys
sit at (27.82, 0.97, −20.38) until t750 then (27.94, 1.64, −21.83) — same x as Rook's post, looking
south (yaw −3.12 rad) at the Delta Sector exit #944 (27.79, 1.28, −25.32 [D placement]). The
driver-quoted camera spot (−34.62, 1.07, 26.19) matches no scene camera key on the disc; it is
Bond's position crossing the escape touch #456, where the walkreal route hands over to the scene
(the nav route target (−33.1, 0.9, 26.8), frame ~7735 of a normal run).

The scene's actor list [D decode, host-float `07A00014.bin!06000848`]:

| stream | op | skin | anim script | windows |
|---|---|---|---|---|
| 1 | ANIM | 0x0500008C (the masked guard) | 0x0600085b | body 0x0400072c [1..951], 222 ch |
| 9 | ANIM | **0x05000066 — Rook's own skin** [D docs/levels/07000014.md] | 0x0600085a | facial 0x04000745 [1..979], 22 ch + body 0x0400072b [1..951], 222 ch |

No ENTITY ops in this scene — no hash can resolve to a wrong object; the actor binding is the
ANIM op's skin.

## What the game binds [M]

A fresh anim object with the skin, not Rook's drone: `Script_AnimStart` 0x1d8f40 reads
(script, skin, start), kills the stream's old object, runs
`control_create_object(0, 0x2d6200, 0x2d6200, 0)` (0x2d6200 = the ORIGIN vector),
`AnimObjectNew(obj, skin)`, `AnimScriptAdd(obj, script)` [M 0x1d8f40:63-74, disasm-checked],
then `AnimObjectUpdate` and `AnimGetFirstTrans` into obj+0x30 [M :93-94]. The placed drone at
(27.81, 0.20, 45.78) (wake ch 0x0D) is not touched by the scene.

## Why the port drew the guard / omitted Rook

The guard's body seq roots at (27.8919, 1.1667, −24.9266) [D 0400072c ch0..2 @1] — in front of the
door; the port drew him exactly there. Rook's body seq 0x040072b roots at
(27.2446, 1.2597, −24.0801) [D ch0..2 @1] — beside the guard, also in front of the door — but the
port posed him at **(29.97, 1.39, −26.49): exactly 1.1× the disc root** (29.9650/27.2409 =
1.3868/1.2607 = −26.4911/−24.0828 = 1.1000). Rook's skin record 0x05000066 carries scale
(1.1, 1.1, 1.1) [D the kind-3 entry +4..+16; the port's `Skin::parse` reads it]. The pre-fix
`local_pose_bones_with` multiplied **every** bone's translation — the root's world position
included — by the skin scale, throwing Rook behind the door plane (z −26.49 vs the door at
z −25.98), out of the shot: "a masked guard in front of the door instead".

The game does not scale the root: `AnimFrameCopy` writes the translation channels scaled
(cases 3/4/5 × Glb_SkinScale/DAT_00319d84/DAT_00319d88 [M 0x1c5668:170-177]) and then **unscales
the buffer's FIRST translation record** with the inverse-scale vector —
`Vec_Mul(param_2, 0x319d90)` [M 0x1c5668:536; the inverses are set per skin in
`AnimObjectUpdate` 0x1befd8:119-124 and reset to 1.0 at :282-287, which also makes
`AnimGetFirstTrans`'s frame-0 seed in `Script_AnimStart` unscaled [M 0x1d8f40:93-94]]. That record
is the root's — the vector `AnimGetFirstTrans` 0x1c7348 copies into obj+0x30 and the record
`AnimFrameResolve` 0x1c35f8:429-432 delta-adds. The root track is a world position (cut-scene
actors) or world-frame motion (the gameplay root track, which the port's `root_translation_at`
already read raw) — never a skeleton-local offset. The tail Vec_Mul exists precisely to keep the
skin scale off it while the other 72 bones scale (Rook is a 1.1-scale model).

## The fix

`crates/nf-formats/src/anim.rs`, `local_pose_bones_with`: the first translation record in the
bone walk reads unscaled (the game's tail Vec_Mul [M 0x1c5668:536]); every later translated bone
and every mask-clear default keeps the skin scale. Consumers verified:

- gameplay drones are unaffected: `Drone::object_pose` zeroes the root translation (the root
  motion integrates into the drone's position via `root_translation_at`, already raw);
- the hip-height read (`local.first().0[1]`) now matches the game's unscaled buffer for
  scale≠1 skins;
- `nf-game --lib` 588 passed, 0 failed, 5 ignored; every cutscene/nis/script binary green (below),
  including the render-verified root pins (cutscene_08_actors, cutscene_models, ninja_boss,
  ride_cutscene_objects with its car-local roots).

## Tests (`escape_scene_actors_07000014_live.rs`, opt-in NF_ISO)

1. `the_escape_scene_actors_live_and_posed_from_the_touch` — the real start path (channel 0x8C
   armed, Bond crosses touch #456 → SEQ_AND #771 → 0x09 → SP #733): both actors pre-built
   (character slots (0x06000848, 1)/(0x06000848, 9) with their own skins), live `Obj::Anim`s bound
   to anims 0x0600085b/0x0600085a, posed finite; the guard's root pinned to the disc's
   (27.8919, 1.1667, −24.9266); **Rook's root pinned to the unscaled disc root
   (27.2446, 1.2597, −24.0801) — the sister assertion fails on the 1.1× root
   (29.97, 1.39, −26.49), naming the address [M 0x1c5668:536]**; stream 9's skin pinned to
   0x05000066 (a guard-skin bind fails it) and the skin scale pinned to 1.1 [D].
2. `the_escape_scene_actors_survive_the_render_path` — with the script camera driving the eye and
   the portal walk feeding `skin_palette`'s room mask (+30/+240/+600 ticks), neither actor's
   palette goes FAR or collapses.

## What I ran (totals)

- `nf-game --lib`: **588 passed, 0 failed, 5 ignored**.
- 41 nf-view binaries — every `*cutscene*`/`*nis*`/`*script*` binary plus the 07000014 set and
  `walkreal_07000014_live`: **100 passed, 0 failed, 4 ignored** (the 4 are pre-existing `#[ignore]`
  probes: probe1b_nis211, probe1b_scripts, script_events_dump ×2). `walkreal_07000014_live`:
  6 passed, 0 failed (its two real-input tests remain `#[ignore]`d — the pre-existing east-duct
  wedge documented in docs/levels/07000014.md, untouched by this change).
- The full four-crate suite could not run as a background fast runner in this sandbox: background
  jobs are killed between tool calls here (three nohup/setsid attempts died with empty logs), so
  the suite driver's per-binary protocol was run per-binary in the foreground for the touched
  area instead (the list above). The fast-runner total line is therefore not available this round.

## Render follow-up for the main agent

- `--bond 27.82,0.97,-20.38,-3.12,0.18` mid-scene (script camera): the masked guard
  (0x0500008C) AND Rook (0x05000066) stand in front of the Delta Sector exit door #944, Rook ~0.7
  left of the guard; pre-fix only the guard was visible.
- Same spot at scene tick ~750: the camera has moved to (27.94, 1.64, −21.83) — both actors still framed.

## Live checks for the main agent

1. PCSX2, fresh boot into 07000014, break at the escape scene (SP #733 running, tick ~30):
   read Rook's scene anim object's obj+0x30 (the object `Script_AnimStart` created for stream 9;
   find it via the SSTREAM's +0x10). Expect ≈ (27.24, 1.26, −24.08) — the UNSCALED body-track
   root — not (29.97, 1.39, −26.49). This confirms [M 0x1c5668:536] end-to-end on hardware.
2. Same break: read the skin 0x05000066 runtime record's +0x10/+0x14/+0x18 (the scale
   Glb_SkinScale/DAT_00319d84/DAT_00319d88 are loaded from). Expect (1.1, 1.1, 1.1) [D record].
3. Door 104 (the exit corridor door at (27.79, 1.20, −25.98)) mid-scene: its open fraction
   (pos01) in the game vs the port — the port sampled 0.500 forty ticks into the scene; if the
   game holds it fully closed (0.0) or fully open (1.0), the port's door timeline needs its own
   pass (it does not affect Rook's visibility in the fixed render — both actors sit north of the
   door plane's midpoint — but pin it while it's cheap).

## What's left

- The two live checks above would upgrade the [M]-chain to a hardware measurement.
- The full-suite fast-runner run on a sandbox that lets background processes live.
- The pre-existing walkreal_07000014 ignored legs (the east-duct wedge) — separate task.

## Usage

not visible to me — this session exposes no token, credit or context-window figures to me
(no input/output token counters, no cost display anywhere in the environment).
