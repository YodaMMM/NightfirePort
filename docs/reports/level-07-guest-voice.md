# level-07-guest-voice — the guests speak their lines aloud and turn their heads to Bond

Branch `s/level-07-guest-voice` (from origin/master 11e19082, then merged `origin/s/level-07-guest-talk`
2ae22f68 — the task's starting state — and the moved origin/master c64b2b80). Changed:
`crates/nf-game/src/drone.rs` (the SpeechSFX facial arm + the head turn), `crates/nf-view/tests/speech_facial_fallback_live.rs`
(the two pins re-derived onto advance-and-play), `crates/nf-view/tests/fix_guest_reactions_live.rs` (the walk-up test extended),
`crates/nf-view/examples/probe_facial_ops.rs` (the one-shot op-1 census), `docs/WORKLOG.md`.

## What the game does, and what the port now does [M]

- **The facial arm of `NDrone2_SpeechSFX` 0x141b00** — ported into `Drone::speech_request` +
  a per-tick ControlSTANDARD pass in `Drones::tick_inner`. The game's shape [M 0x141b00:15-35]:
  while no facial line is playing or pending (bytes +0x36/+0x37 both clear — the u64 window at
  +0x30 [M :16]), `AnimScriptPeekSfx` 0x1c6ca8 peeks the facial script's own op-1 sound command;
  found → the script id queues at +0x4c with +0x37 = 1 and +0x48 = now + 0x14 and
  `SFXPreCache(sfx, 0x14)` [M :24-28]; not found → `AnimFacialAppend` 0x1c6490 appends the script
  at once (+0x53c = the handle, +0x35 = 0, +0x36 = 1 [M :18-23]). Either way the arm returns 0 —
  a facial line never returns or stores a handle (+0x4d0 untouched) [M :35] — and stamps the
  `Drone_SfxInfo` entry's `lastFrame = now` only once +0x53c stands [M :32-33], which arms the
  entry's repeat gate [M :9-12]. ControlSTANDARD appends the pending script at T+21 [M
  `NDrone2_ControlSTANDARD` 0x1490f0, speech.md §1.4] and the attached script's own op-1 command
  then plays the line — `Sound_Play3D(sfx, pos, vol 100, outer 100, inner 0, pitch Rand(500)−250)`
  with one shared-RNG draw per fired op-1 [M `AnimProcessScriptCmds` 0x1c1b38:319-322], the handle
  discarded; the stop-at-end script (+0xb5 = 3 [M 0x1c6490]) clears +0x53c at its end [M §1.4].
  Port folds, each marked in code: the SfxInfo stamp is per drone (the game's table is global —
  the 07 BondTalk cooldowns 480/600 exceed every shared entry's repeat except 0x8e/0x95/0x96's
  2112 [D the SfxInfo dump, scripts/dump_sfxinfo.py], which no test drives cross-drone) [S];
  `SFXPreCache` rides the play (the start-latency fold of speech.md §4.5, headless-optional) [S];
  `AnimScriptAppend`'s slot refusal is not modelled [S].
- **The head turn** — `NDrone2_HeadTrackObj(0, players, 200, dcv)` 0x14ada0, called from every
  BondTalk arm that speaks (cases 9/0xd/0x10/0x11 [M 0x1446b0:67/125/174/211], case 0xe's
  `DroneFunc_DoBondTalk` [M 0x144488:63], the 0x12 jump-table steps 0/1/3/5 at 200 and step 6 at
  600 [M 0x144d94-0x144db4]) [M]. Ported as `Drone::head_track_obj`: the prop-0x4c gate
  (`behaviour_util_getProperty(0x4c, +0x4d8)`, 0 → no-op [M 0x14ada0:13-16]), then +0x2c = 1 and
  +0x358 = now + time [M :25-29]. `DoHeadTrackingBehaviour` defers to the window (`now <= +0x358`
  → return [M 0x147578:37-39]) — the port's head-ease block runs the talk branch first (the want
  angles toward Bond's eye within SetHeadTracking's 3.0 range [M 0x14ada0:18], the ease speed
  model+0xe0 = 0.07 [M :19-21]), and the sight path resumes on expiry.
- **BondTalk's advance** — unchanged code, now live: `h == 0 && !facial_attached()` returns
  without advancing; once +0x53c stands the cooldown (600 for set 0xd [M 0x1446b0 case 0xd]) and
  the step advance [M 0x144488:50-63]. `facial_attached()` reads the real +0x53c stand-in now.

## The [D] census that pins the sound (crates/nf-view/examples/probe_facial_ops.rs)

On a live 07000007 load: every present BondTalk facial script's FIRST op-1 command's sfx id
equals the entry's sfxId (0x58..0x83 present → 463, 374..396, 398; 0x84..0x8c → 418..426; 0x8e →
427; 0x95/0x96 → 450/453; 0x97..0x9a → 454..457 — all `eq true`), and the absent ones are exactly
0x70/0x72/0x73 (0x06000977/978/979, matching the 2026-10-22 census). The queue arm therefore
fires for every guest line, and the played id is pinnable to the exact entry sound.

## Tests (each pin cites its evidence; the real input path only)

- **`speech_facial_fallback_live` re-derived** onto the game's advance-and-play behaviour:
  `bond_talk_plays_the_absent_script_lines_direct_and_the_present_ones_through_the_facial` — the
  absent-script draws (397/399/400) still play DIRECT with a handle [M 0x141b00:18-28] and the
  present-script draws now sound through the facial op-1 (ids ∈ the present census set, never an
  absent id) and advance the bookkeeping [M the case 0xd join];
  `a_present_script_talker_plays_through_the_facial_and_advances` — the vt-14 talker's line
  sounds (the census asserts each set-0xe script's op-1 == sfxId [D]), `+0x4d0` stays clear [M
  0x141b00:35] and the bookkeeping advances once +0x53c stands [M 0x144488:50-63].
- **`fix_guest_reactions_live::walking_up_to_a_guest_acks_the_talk_gates_and_does_not_scare`
  extended**: the voice-type-13 pin (the port's `voice_type(skin)` == +0xd8), the line-id pin from
  the guest's dtype table (`0x58 + rand(11)*4 + variant`, variant by the skin byte [M 0x1446b0
  case 0xd:109-127]), the sound-emitted pin (the requested line's own sfx among the played [D
  census]), the head-track pin (the HeadTrackObj window stood [M 0x14ada0:25-29] AND the guest's
  head yaw turned — the hold runs at a 0.7-rad offset off the front line so the want angle is
  geometrically real, still inside the ±π/2 facing cone [M 0x1446b0:14]) — and the unchanged
  no-scare pins. 5/5 green, no ignores.
- The old "present scripts stay silent / never advance" assertions are GONE — they contradicted
  the decompile once the facial arm landed (the 0x843-era port model), replaced by the
  advance-and-play pins above, per the task's re-derivation instruction.

## How verified (the fast runner cannot run here — background jobs die between tool calls,
verified again this session with the suite runner and the ISO fetch; foreground slices ran instead)

- nf-game: **887 passed / 0 failed** (547 lib + the live bins) on the pre-merge tree, and the lib
  547 re-run green on the final tree.
- nf-formats + nf-audio: **133 passed / 3 failed** — the 3 failures are exactly the allowed
  `disc_golden` tests on the slim ISO (bigf_header_fields_behave_as_documented,
  bigf_tables_match_the_golden_manifest, directory_tree_matches_the_golden_manifest).
- nf-view slices (99 passed / 0 failed): fix_guest_reactions 5, speech_facial_fallback 2,
  talk_barks 2, fix_party_mission 9 (6 ignored), party_start_rules 6, walk_07000007 2,
  walkreal_07000007 2, bathhouse_voices 3, bathhouse_armed_girl 2, bathhouse_map_ambients 2,
  bathhouse_shout_mixer 2, civilian_first_sighting_alert 4, civilian_mission_wait 3,
  civilian_no_shoot 3, escort_follow_07000001 1, fix_mission 4, facial2 2, facial3 3,
  facial_gpu 3, fix_heads07 2, head_look 2, alert_status 1, consider_alerted 4,
  cross_process_determinism 3, ai_traces 4, walkreal_07000005 8, walkreal_07000001..04/06/08
  3+1+4+3+0(2 ign)+4. The final tree re-ran the guest/fallback/barks (9/0) and party (17/0)
  slices green after the origin/master merge.

## What's left

- **Lip-flap visuals** (the task's optional item): the port models the facial state and the
  sound, not the mouth-flap morph playback — the scene's facial-weight path (fix-facial3's
  idle/blink slots) does not yet sample the talk script's face window.
- The per-drone `SfxInfo` stamp fold [S] (above) — a live check can pin the game's global-table
  behaviour for two same-set guests inside one repeat window.
- The body anim op-1 route (`Drones::tick`'s `anim_sounds` play) does NOT draw the game's
  `Rand(500)` pitch while the NIS `Request::AnimSound` route and my facial ticker do [M
  0x1c1b38:319-322 says every fired op-1 draws] — a pre-existing divergence this task did not
  touch; worth an audit pass (it can shift the shared RNG on levels whose body scripts fire op-1s).
- The `SFXPreCache` start-latency fold [S] (speech.md §4.5 marks it optional headless).

## Live checks for the main agent

1. Fresh boot of 07000007, walk unarmed up to a standing guest from its front, stand ~2 units:
   the guest should turn its head to Bond (HeadTrack 200) and a short line should play ~22 +
   (opFrame − 1) ticks after the gates pass [M §5], rotating through its dtype table with a
   600-tick cooldown (set 0xd). A gdbwatch watch on the drone's +0x4c (the queued script id) and
   +0x53c would show the T+21 append; a trace of `Sound_Play3D`'s pitch argument pins the
   `Rand(500)` draw per line.
2. Two guests of the same skin talking within one entry's repeat window (the set-9 lines 0x95/
   0x96 carry repeat 2112 [D SfxInfo]): the game's shared `Drone_SfxInfo.lastFrame` should block
   the SECOND guest's SpeechSFX until 2112 ticks pass — the port's per-drone fold allows it [S].
   A gdbwatch on `Drone_SfxInfo+entry*32+8` (0x2812d0-based) across both talks pins the fold.
3. Port-side renders (`--bond`): a spot 2.0 units in front of a standing guest, unarmed — the
   guest's head toward Bond while the window stands; after drawing the P2K, the guest cowers
   (no head track). The talk line itself is audio-only (no new art involved).

## Usage

Task-system token/context figures are not visible to me this session (no token, cost or
context-window readouts are exposed in this environment). Session wall-clock: branch pushed
02:20 UTC, report written 03:40 UTC (2026-10-25).
