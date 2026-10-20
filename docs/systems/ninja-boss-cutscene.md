# The ninja boss in Double-Cross pt. 4's story cut-scene

Build: ACTION.ELF `e7ffdcb3…`. Tags: [M] decomp / disassembly read here, [D] disc bytes, [L] EE snapshots,
[S] suggested. Port: `crates/nf-game/src/anim.rs` (`script_pose_together`), draw path
`crates/nf-view/src/scene.rs` (`frame_state_for`'s script branch). Symptom (playtest 1017): "ninja boss is
missing from last cutscene of that level" — the final level of the Double-Cross chain, 07000004.

## The cut-scene and its actor list [D]

07000004's main map (map 49; the whole level's 70 placements) carries two ScriptPlayers (kind 0xd9):

| placement | script A | mode (p2) | trigger mask (p5) | channel (p6) | plays |
|---|---|---|---|---|---|
| #3 | 0x06000083 (kind-7, 2272 B) | 1 | 0 | 0 | at once (level start) |
| #2 | 0x06000082 (kind-7, 5440 B) | 0 | 1 (channel) | 56 | when ch 56 ≠ 0 |

The intro 0x06000083 (500 ticks) ends with `Script_EventHandler` event 0xD `[56, 1]` (its stream 6, tick 500)
[D decode] — `switch_channels[56] = 1` [M 0x1d94c0 case 0xD] — which fires #2. #2's state-0 trigger also
gates on Bond alive and the mission running (`glb_blokes+0x894 > 0` — health [M 0x1dbda8:87 +
docs/fields/bldata.md +0x894] — and `Mission_Status() == 1` [M 0x1dbda8:89-92; MissionState 1 = playing,
docs/systems/mission.md]); both hold at level start, so the story scene 0x06000082 (830 ticks) plays right
after the intro. `SP_Hit` 0x1dc470:15-17 sets trigger bit 1 from `switch_channels[p6] != 0` [M]. The port's
sim runs exactly this chain (SP #2 in state 2 by tick ~505; the walkthrough test
`walk_07000004_live.rs` pins the same pair via ch 0x38/0x53, the same numbers in hex).

0x06000082's op-0x0A streams (`Script_AnimStart` 0x1d8f40: u32 anim script, u32 skin, u32 start frame [M;
docs/formats/nis-script.md]) name **three** actors [D decode of the kind-7 record]:

| stream | anim script | skin | who |
|---|---|---|---|
| 1 | 0x06000508 | 0x0500004F | the ally |
| 2 | 0x06000509 | 0x05000009 | Bond (the player skin on 07-1..4 [M 0x1910f8:93-105]) |
| 19 | 0x06000507 | **0x0500001E** | **the ninja** |

0x0500001E is the skin of the level's one placed drone (placement param 0 = 0x0500001E — the assassin the
objective "Defeat the assassin" watches; its death raises ch 0x54 [D placement probe,
docs/levels + walk_07000004_live.rs]). The intro 0x06000083 animates only streams 1/2 (0x0500004F /
0x05000009) [D]. All three skins and both skeletons live in 07000004's own archive [D kind-3/kind-6 scan]:
0x0500001E → skeleton byte 8, one skinned mesh 0x02000292 (resolves in `by_hash`); skeleton 8 is 73 bones
with translation mask 0x1/0/0 — identical counts and mask to skeleton 0 (Bond's).

The ninja's animation: 0x06000507 (kind-5, 831 ticks) carries ONE opcode-0 window → sequence 0x0400047A
[D]; the sequence lives in 07F00004.bin (kind-4, 831 frames, **record skeleton byte 0**, 222 channels =
3·73 + 3·1, flags 0x00) [D]. The cross-rig pairing is shipped content: a skeleton-0 record animates a
skeleton-8 skin.

## Why the game plays it and the port didn't [M]

`AnimObjectNew` 0x1be910 sizes the anim object's bone buffer from the SKIN's skeleton
(`pSkeletons[*(byte *)(skin + 0x30)]` — the runtime +0x30 is the file's skeleton byte, stored by
`AnimProcessSkinData` 0x1bd500:105). `AnimFrameCopy` 0x1c5668 — the per-bone walk — takes its skeleton,
translation mask and stand-in offsets from the **SEQUENCE**'s own record byte
(`iVar6 = pSkeletons[*(byte *)(sAnimSeq + 0xa9)]`, :52; mask test :190; offsets :194-198), and gates each
bone's write on the SKIN's bone-flag bytes (`param_4 + 0x24`, :186-189). Nothing requires the two ids to
be equal: the channel stream is read positionally over whatever skeleton the record names, and the skin
contributes scale, the animated-bone gate and the inverse bind.

The port's cut-scene pose, `script_pose_together`, filtered every opcode-0 window by the strict
`seq.skeleton == skin.skeleton` (nf-game/src/anim.rs, the base picker and the fill loop) — the ninja's
only window (record 0) failed against its skin (8), the base picker found nothing, and the function
returned `None`; `frame_state_for`'s script branch propagates that `None` and the character is not drawn
that frame. The gameplay/drone pose had already learned the lesson — `script_pose`'s channel-count guard
poses the SAME skin (0x0500001e) on skeleton-0 records ("07000004's seated target … the strict filter left
him with no palette and no hit boxes") — the cut-scene path just never got the same arm.

Repro on the pre-fix tree (probe over `nf_view::load` + 700 ticks): chars for streams 1/2 pose (roots
[-1.92, 5.88, 19.78] / [-1.30, 5.89, 20.26] at frame 200), stream 19's `script_pose_together` → `None`.

## The fix

`script_pose_together` accepts a window on the same rule the drone path pins: the record byte names the
skin's own skeleton (the old path: base candidates must `fits` — full walk; fill windows may be narrow,
`bones_covered` leading bones), **or** its channels cover the skin skeleton's full walk
(`Σ 6·has_translation + 3·rest >= channels`) and then it owns all bones. Facial windows (flags 0x8) are
excluded before the check, as before. The pose machinery itself is unchanged: channels are read by the
skin skeleton's mask/offsets (identical here — both skeletons 73 bones, mask 0x1/0/0 — and the shared
human tracks ride the same rig geometry; the seq-byte-drives-the-walk reading is [M 0x1c5668:52], the
port's skin-skeleton walk is the standing [S] convention the drone path already ships).

## Tests

`crates/nf-view/tests/ninja_boss_cutscene_live.rs` (ISO-gated; numbers/ids only):

- `story_cutscene_actor_set_matches_the_disc` [D]: the two SP placements' scripts/modes/mask/channel;
  0x06000082's (stream, script, skin) triples exactly {(1, 0x06000508, 0x0500004F), (2, 0x06000509,
  0x05000009), (19, 0x06000507, 0x0500001E)} and 0x06000083's {(1, 0x06000506, 0x0500004F), (2, 0x06000505,
  0x05000009)} — the sister arm pins the other actors' pairing so a re-pair fails; 0x06000507's single
  window (1, 831, 0x0400047A); the sequence's record byte 0 / 222 channels / flags 0 in 07F00004; the
  skin's skeleton byte 8 and mesh 0x02000292; skeletons 0 and 8 both 73 bones.
- `ninja_poses_in_the_port_cutscene` (the fix, through the real load): after the intro hands over (tick
  ~700), SP #2 is playing, the sim holds `Obj::Anim` src (0x06000082, 19) skin 0x0500001E on anim script
  0x06000507 with an advancing frame, `script_pose_together` returns a finite pose whose root is non-zero
  and moves between two ticks, and the character is not hidden; the sister arm poses streams 1/2's actors
  (unchanged, finite) and would fail if the new arm broke the matching path.

## Live checks for the main agent

- PCSX2, 07000004 fresh boot, the story scene (after the intro): the ninja should stand/walk with Bond and
  the ally during the 830-tick scene (a `--bond` inside the cut-scene camera's view; the scene's camera key
  0 of 0x06000082 looks at the courtyard). Render spot: `--bond <the scene camera's subject>` — the three
  actors together.
- `gdbwatch.py` on the ninja anim object's position `obj+0x30` during the scene: it should advance
  smoothly (the sequence's root motion), not stay at the origin.
