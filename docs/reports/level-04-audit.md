# level-04-audit — the 07000004 behaviour audit

Branch `s/level-04-audit` = origin/master (b312d959) + this task. The audit table lives at
`docs/levels/07000004-audit.md` (committed, 44245bf3): every one of the main map's 70 placements
[D] with its creator in `parsemap_create_dynamic_objects` 0x1d04f0 [M], its update/use/damage
handlers, what the port does, and a GAP column; every script archive's streams decoded op by op
(the camera streams, the anims, the events, the sound lines, the hand-backs); the mission record;
the gap list.

## What's done

1. **The audit table** (see the doc for the full 70 rows and the stream decodes). Kinds present:
   Start ×1, the target Drone ×1, ScriptPlayers ×2, Env ×1, Pickups ×4, statics ×40 (the Limo
   parts are 13 hashed cels; the front-door pair is the hashless #28/#29), Trees ×2, Ripples ×4,
   LeafGens ×2, RigidBodies ×2, the 0xEE MultiplexOut ×1, RainBoxes ×3, Sky ×3, Coronas ×2, room
   meta ×2 — zero placements for every other 0x1d04f0 kind [D], matching the census doc's zero
   list. The port covers every kind except the two gaps below; the mission flow (death ch 0x54 →
   the #31 mux → ch 0x64 → `Mission_Update` → `ResetMap_LevelToLoad(0x0730004)`) is already
   pinned end-to-end by `walkreal_07000004_live` (3/3 on this tree).
2. **GAP-1 found and FIXED** (`e4127e84`): a script stream's SECOND sound line never started.
   `Script_SoundStart` 0x1d8820 clears the stream's handle (+0x10 = 0) at its top, before the
   start gate [M 0x1d8820:19]; the port gated on `sound_id.is_none()` without the clear. Any
   stream with two lines lost the second — 07000004's story stream 15 starts 0x5e @298 and 0x134
   @654 [D 0x06000082 decode] and the 0x134 dialogue line was gated off mid-scene.
   - Test (reproduce-then-pin): `crates/nf-view/tests/audit_07000004_live::
     the_intro_and_story_scenes_play_their_scripted_lines` — walks the real path (load → the
     intro SP → the story SP), collects the EE `Cmd::Start`s, and asserts every line: the intro's
     0x50a exactly once at its authored frame 57 [D stream 8]; all eight story ids (0x5d, 0x5e,
     0xa5 ×2, 0xa9, 0xaa, 0x134, 0x255, 0x5c3) [D streams 6/12-18]; the story's first line lands
     ≈500+43 (the intro hands to the story through ch 0x38 [D stream 6]); the NIS music 0x172
     starts and sets `SfxNisMusicTrigger` [M SFXStart 0x115a68]. Red before the fix (0x134
     missing), green after.
3. **The remaining gaps, documented with their [M] chains for own tasks**: GAP-2 the two
   rigid-body lamps (RB_Create 0x1d6bf0 → RB_Init/RB_Update 0x1d6e38 → RB_ApplyImpulses
   0x1d6ff8 → RB_RestingContact 0x1d7168 — a whole physics system; nothing is created or drawn);
   GAP-3 the trees' sway-anim runtime (Env_Tree_Create 0x17c200's AnimObjectNew/AnimScriptAdd;
   TreeFx carries the hashes and the pause arm but the sway doesn't move — draw-only);
   GAP-4 the cut-scene camera hand-back (the verify ledger's Camera_UpdateAll 0x1e2d78 row: no
   camera stack pops on Script_CameraEnd beyond the object delete) — a visual question, render
   spots listed in the audit doc §4 for the main agent.

## How it was verified

- The census: `Level::parse` of the disc archive [D] (70 placements, 0 unresolved, the other 49
  maps empty of placements); the creators read off the 0x1d04f0 dispatch [M] per case; the port
  side grepped per row (triggers.rs case 5 `Trigger_MultiplexOut` 0x1b2ed8, world_fx
  RippleFx/LeafGenFx/TreeFx, env.rs, pickup.rs, sky.rs, corona.rs, drone.rs's param decodes) and
  exercised by the level's own tests.
- The fix: the decompile read first (`001d8820_Script_SoundStart` — the unconditional +0x10
  clear), then the red test through the real input path, then the fix, then the neighborhood:
  cutscene_skip_audio_live 3/3, fix_cutscenes2_live 4/4, movie_audio_layout_live 5/5,
  fix_audio2_live 10/10, walkreal_07000004_live 3/3, interact_07000004_live 2/2,
  walk_07000004_live 3/3 — no regressions from the sound-path change.

## The full suite on the final tree

The sandbox kills background processes (nohup probe failed again), so the fast runner ran as a
resumable clone of `tools/test/suite.py` (same build command and per-binary semantics) in
foreground chunks. Total at report time (23:55 UTC):

```
223/809 test binaries: nf-audio 4/4, nf-formats 26/26, nf-game 89/89, nf-view 105/690
1469 passed, 4 failed, 35 ignored
```

- The 4 failed are the documented slim-ISO disc family (3 `disc_golden` +
  `identity_disc::the_real_disc_passes_the_identity_check`) — allowed by the task rules on this
  image; they fail identically on pristine master.
- **Every binary of nf-game (the crate my fix touches) ran: 89/89, green.** nf-view's remaining
  ~585 binaries are the slow live-test tail (the chunking got ~11/9 min); the touched
  neighbourhood (the sound/cut-scene/04 families) ran green explicitly above.

## What's left

- The nf-view suite tail (~585 binaries) — the tree is final; any later suite run completes it.
- GAP-2/GAP-3 as own tasks; GAP-4's renders + the live checks below.

## Live checks for the main agent

1. **GAP-1's live counterpart**: on a FRESH boot into 07000004, during the story scene (t ≈
   1154) the 0x134 line plays — watch the sound system's stream for id 0x134 (SFXStart
   0x115a68's table entry) and confirm one 3-D voice at the stream-1 anim object's position
   (obj+0xc0 [M Script_SoundStart 0x1d8820:40-44]).
2. **GAP-2's mesh**: at RB_Create (0x1d6bf0) for either lamp, dump the resolved mesh/glist the
   object draws with (the placement carries no hash; RB_Create resolves through the celglist [M])
   — the draw-only stopgap needs that hash.
3. **The renders** in `docs/levels/07000004-audit.md` §4 (four `--bond` spots: the intro's
   opening frame, the assassination framing after the story, the carpark lamps, the pond trees).

## Usage

not visible to me (no token/credit/context-window figures are displayed in this environment)

## ntfy failed

All three fallbacks failed at 23:53-23:54 UTC (the sandbox's network to ntfy.sh was down):
1. `curl -sS -m 20 -H "Title: NightfirePort" -d "..." https://ntfy.sh/nightfire-yoda-8k3x` →
   `curl: (28) Connection timed out after 20001 milliseconds`
2. the python urllib fallback → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
3. `wget -q -O- --post-data "..." https://ntfy.sh/nightfire-yoda-8k3x` → timed out (killed at 40 s)
