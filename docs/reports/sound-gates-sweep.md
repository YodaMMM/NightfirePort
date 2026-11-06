# sound-gates-sweep

Sweep of every PORTED row of `docs/systems/sound-coverage.md` (48 rows + the summary rows): for
each, the branch conditions around the call in the decompile were re-read (with disassembly
wherever the C looked incomplete — ids re-taken from `$a0` at the jal), the flag/state named, and
the port's gating compared branch-by-branch. Flags were traced to their writers rather than
trusting the row's label. Six parallel audits (player, movement, bullets, world ×2,
drones/music/script); every claimed WRONG re-verified in the decompile/disasm before any change.

## Result per row

MATCH rows cite the decomp condition and the port's current site (many old port cites had
drifted; the table in `sound-coverage.md` is refreshed). WRONG means: the port fires on a
different event than the game, or the row's label was wrong and the port followed it.

### Player on foot

- 0x1DB refused jump (0x1916b0:106-112): **WRONG (doc)** — the row said PORTED `scene.rs:2294`;
  no 0x1DB play exists anywhere in the port (git history: never), and the port launches the jump
  where the game refuses. Condition pinned [M dis 0x191770-0x1917c4]: `GameState==0x0700007 &&
  switch_channels[0x48]==0` + action-7 edge + grounded-history ≠ 0 + head-hit flag clear →
  `Sound_Play(0x1DB)` + return without launching. Relabelled MISSING (doc).
- 0xA3 jump launch (0x1916b0:184): MATCH — `player.rs:1673-1694` (`sfx_out` → `scene.rs:3461`),
  branch-by-branch (action-7 edge, BL+0x940 grounded history, flag 0x20 clear, headroom LOS).
- 0x5FB/0xA3 landing (0x1916b0:195-198): MATCH — the head-bump-surface-sound corrected reading is
  faithfully ported (`player.rs:2045-2062`, measured [L] in-place deviation documented there).
- 0x88 pain (0x1902c8:113): MATCH — 1-in-4 roll after the pain gate; port `bullet.rs:356` →
  `drone.rs:21104` (+ drown/hurt-vol/fall paths). Nit: the game skips the roll on type-7 pain
  with `CheatInfo != 0`; the port has no cheat model.
- 0x600/0x412/0x171 obj hit (0x194458:231-255): MATCH — port `drone.rs:21108-21123`; 0x600 is
  the MP head-zone arm, correctly dropped in SP; 0x171 gated `record+0x10 & 1`. Nits: the game
  returns early (no sounds) when `Drone_ModPlayerHitDamage` returns 0; the port plays on a
  0-damage hit.
- 6/0x89 death (0x190810:41,88): MATCH — `death.rs:223-241` (one-shot, mp gate, unconditional
  0x89) + `music.rs` event 6.
- 0x60B/0x60C/0x27 air (0x191a70:106-141): MATCH — `scene.rs:3565-3580` + `3525-3529`, swim
  latch `sound.rs:55-58`; 0x60B + type-7 pain every 30 ticks at air < 1.0. Deltas (not
  verdict-changing): the game's substate 8/9 (zero-g) forces the surfaced arm — unreachable
  (no zero-g level has water cels); the game's 30-tick cadence is the global frame counter, the
  port's a local counter.
- Sound_StopAllWithId 0x12/0x3D/0xF0 (0x190630:83-85 + 0x1910f8:41-43): **WRONG (port), not
  fixed** — the game stops the three loop ids when the NEW substate ∈ [1..3]∪[5..0x10] (disasm
  0x1907b4-0x1907e8: substate 4-crouch returns early with NO stops; 0 skipped) and
  unconditionally in `Player_Init`; the port has no substate-change stop — its only id-keyed
  stops fire on weapon switch (`scene.rs:4288/4303`), swim (`scene.rs:3576`), script op-4.3
  (`scene.rs:4947`), and drone anims (`drone.rs:20686`), so a live gadget loop (0x12/0x3D
  laser/stunner, 0xF0) survives a substate change where the game silences it. The exact
  condition is in the doc row.
- 0x283/0x280/0x281/0x282 weapon change (0x1a2e38:227-295): MATCH — gadget id [0x4A,0x5E] →
  0x280/0x281 else 0x283/0x282, keyed old (holster)/new (draw), `scene.rs:4297/4328/4374/4441`.
  Gap: the port's 4374 site also plays when the new record has no draw script (game silent).
- 0xEC micro-camera shutter (0x1a4bf0:262): MATCH — `drone.rs:14102-14109` (+2 u16 == 0x54, per
  bullet) + test.
- Sound_Stop [BL+0x898] (0x1a1a70/0x1a24c8): **WRONG (label; port = documented deviation)** —
  [BL+0x898] is a single u32 id and an ELF-wide immediate scan (ACTION.ELF, SLUS_205.79,
  DRIVING.ELF) finds only zeroing writes: the game's two stop sites are provably dead code. The
  port's stop at the weapon switch (`scene.rs:4288/4303`, "the weapon-switch path itself
  performs NO stop [in the game]") is the stand-in. Relabelled in the doc; matching the dead
  code literally would kill loop sounds.
- Music_Event 0xD creep/wire (0x1969d0:85 / 0x197658:107): MATCH — `scene.rs:3497-3506`, per
  tick, board latches tolerate it. Deviation: the port's `on_wire` includes SS_ZIPLINE (game
  substate 0xf silent); dead in shipped data (both live wires type 0).
- 0x5FC/0x1D4/0x1D5/1 objective banner (0x19c310:148-235): MATCH — `scene.rs:6416-6434`, same
  partition and windows (live-pinned by `objective_banner_timing_live.rs`).

### Weapons and bullets

- Bullet_init:273 `param_7` (0x12c5a0:273): **WRONG (doc label)** — the guards pass **0xffff**
  (disasm 0x17940c) and the player's WeaponInitBullet passes 0xffff too: both are silent here;
  their real fire ids ride FireWeapon's own `Sound_Play3D(+0x60)` (0x179490, ported
  `drone.rs:11283-11287`) and the fire script's op-1 (0x1c1b38:320, ported with the class
  remaps). Only the anim-script fire op feeds a real id (nothing in ported SP paths). Doc fixed.
- 0x10E whoosh (0x12c5a0:277-282): MATCH — `drone.rs:14098` (+0x90 == 0x0E), handle kept and
  stopped on delete; test. Gap: the guard `fire_weapon` path never plays it (no SP 0x0e
  loadouts; MP bot loadouts carry them).
- 0x5EF (0x12c5a0:286-290): **WRONG (port + doc)** — the +2 == 0x45 record is **Oddjob's hat**
  (base 0x45, +0x60 = 0x5EF, looped; [L] sweep), not a "grenade-bounce kin". The port plays
  0x5EF nowhere (the grenade-throw play was removed correctly; the hat spawn is silent).
  Relabelled MISSING with the audibility open question ([L SFXOutputData 0x5ef: looped,
  weight 0]).
- CollisionHandler impacts (0x12ce80:316-432): MATCH — 0x4A7 the 0x100-family stick clang, 0x446
  the 0x80000 stick, 0x51C the even-count bounce block, 0x15/0x13 the 0x2000/0x20000 first
  bounces; the surface-id leg is callee `Effect_Bullet` 0x1ce8a8:86-90 (flags 0x61 ← weapon
  +0x70 & 2). Port `drone.rs:21312/21369-21372/21008-21010` one-for-one. Unpinned corner: a
  0x100-family stick hitting a class-2/3/0x15 object routes through the bounce block (0x51C at
  count 0) — the port plays the clang only.
- 0x1DC fly-by (0x12d890:250): MATCH — weapon +0x6c & 0x40000, not the player's own shot,
  closest-point SqDist < 9, 1-in-5 roll; `bullet.rs:264-268` → `drone.rs:21128`.
- 0x16/0x14 (0x12d890:292,411): MATCH (label corrected) — 0x16 the stun-grenade flash pop, 0x14
  the smoke canister pop (first expiry, re-arms); neither is the frag blast. `drone.rs:21429/21461`.
- 0x447 (0x12d890:442): **WRONG (doc label)** — it is the satchel-charge countdown beep
  (quarter-second digit change, (qs&3)/sec<10/sec<5 cadence); there is no smoke-trail sound in
  DoTrails. The port already fires on the exact game event (`drone.rs:11657-11661` →
  `21324-21327`). Doc fixed.
- 0x5DD (0x12d890:473): MATCH (label direction corrected) — the charge's burn counter ends under
  the beam (not the beam's life); `drone.rs:15464-15495` + tests.
- Bullet_update 0x27C (0x12e7b0:104,290,327): **WRONG (doc label)** — 0x27C is the tripbomb's
  laser-arm sound (the 0x80000 arm-clock expiry), not a bounce; :104 the loop follow, :290 the
  state-3 stop. Port faithful (`drone.rs:14809-14812/15969/21318-21319`). Doc fixed.
- Bullet_Delete 0x28D (0x12e660:14,53): MATCH — SetGrapplePoint's only caller is Bullet_Delete,
  so latch ⇔ sound; port `scene.rs:3193-3205` (at the fire-press latch — no bolt flight).
- Casings (0x12faa0:97): MATCH — `casing.rs:276-282/320-331`; the id range is 0x1FD-0x204 (the
  default-mesh 0x1FD was missing from the doc row).

### World objects

- Door +0x60/+0x62/+0x64 (0x135618 + 0x135be0): **WRONG (port, one-way arm) — FIXED this
  sweep**: Door_Activate's approach gate computes dot = **(door − activator)·facing**
  (`_vsub(door+0xc0, act+0xc0)`, disasm-verified; `Mat_GetDir` of door+0x90), mode 1 refuses
  dot < 0, mode 2 dot > 0 (one-shot, a pass clears +0x76). The port computed (activator −
  door)·facing with the same tests — both refusals mirrored (door.md carried the same flipped
  reading). Fixed in `door.rs:703-708` + test `one_way_door_refuses_from_the_game_side`
  (mode-1 refuse + clunk from the facing side, open from the back; mode-2 one-shot + clear).
  The rest of the row MATCHed (state-8/0x10 edges, lock channel, flags reject).
- Lock 0x249 (0x1ae3d8:92): MATCH — `lock.rs:172-181` → `drone.rs:15423-15434` on `+0x105 < 4`.
  Also: the doc's 0x5DB/0x5DA row still said MISSING, but the picklock beeps are implemented
  (`lock.rs pending_sounds` + `lock_sounds_live.rs`) — status fixed to PORTED.
- Searchlight 0xE5/0xE6 (0x1cb900:401,455): MATCH (labels corrected) — 0xE5 the state-3 off→on
  relay clunk, 0xE6 the >0-damage bullet glass break; `searchlight.rs:377-385/605-623`.
- Sensor 0x4AA/0x4A9/0x4DD (0x1af618): MATCH — identical gates incl. the 0.5/0.25/0.125 cadence
  and the sweep midpoint+wrap servo; `sensor.rs:293-298/312-334/344-351` + `sensor_live`.
- Pickup +0x2A (0x18e650:336): MATCH — `drone.rs:17286-17294` (same refusals fall through, same
  overrides 0xF5/0x141/0x14C, 0xffff sentinel).
- Hint (0x1b3380:26): MATCH — `scene.rs:3383-3396`; nuances: the game plays 2-D `Sound_Play`
  (port 3-D at Bond's head) and deletes the marker when the OFF channel rises (port skips).
- SoundTrigger (0x1b2790): MATCH — `sound.rs:160-187` branch-for-branch incl. the create
  mapping; sole edge: the port tests `channels.on(off)` unguarded where the game guards
  off != 0 (matters only if p4==0 and channel 0 is ever set).
- MusicTrigger (0x1b31a0:24): MATCH (label + location corrected) — a channel-gated 30-tick
  repeater posting `Music_Event(id, 2)`, in `worldobj.rs:514-529/566-574`; the port never
  deletes on the p1 channel (all surveyed placements carry p1=0 [D]).
- 0x286 anim object (0x1befd8:205): MATCH — the ladder rung step (Bond, stride bit, substate 1
  climb); `scene.rs:3598-3630`; the game's `ScriptCam==0` gate is the port's NIS-disabled
  stand-in.
- AnimProcessScriptCmds (0x1c1b38:320,388): MATCH — same (from,to] window, same six held-weapon
  id remaps (`scene.rs:42-52`), op-4.3 stop; the port omits the random pan and plays view-script
  sounds at the camera.
- RB_RestingContact (0x1d7168:111/0x1d73a0:121): MATCH — class-5 gate, id rb+0x158 with the
  0→0xFFFF fallback (`rigid_body.rs:722-729/874-899/1203`); footnote: the blast path's second
  collection (`drone.rs:14466`) drops its sounds (only the control-walk drain plays).
- View_DrawSky 0x5D/0x5E (0x1e5ad0:96-143): MATCH — countdown leaving 1, LightningType written
  by the same loop's flash-start roll; `sky.rs:139-182` + `drone.rs` sky_frame.
- HandleMapSoundAllocation (0x1dd118:66-79): MATCH — inside-R start / 1.2·R² stop, same
  strictness, `sound.rs:78-89`.

### Drones, guards, music, script

- NDrone2_PlaySFX (0x141848:121): MATCH — `foley_entry`'s 30-frame post-load gate + repeat gate
  + the `foley_reqs` drain. The SeenSurrenderedDrone 1-in-6 `PlaySFX(0x23)` site is dead data
  (SfxInfo 0x23 sfx id 0 [L ee_snapshot]).
- NDrone2_DoHitEffects (0x1458e0:108-111): MATCH — `drone.rs:15571-15576` (damage > 0.5, the
  firing weapon's +0x70 & 0x180 clear, part==5 → 0x601 else 0x413).
- SpecialDeath 0x148 (0x163c68 case 5): **WRONG — FIXED this sweep**: case 5 is the ANIM_DONE
  message (the death script's completion; disasm 0x163f38-0x163f80: `$a0=0x148`,
  `$a1=glb_players+0x30`), and the cry plays at the PLAYER's position. The port queued it on
  state ENTER at the drone (right id, wrong event and position; its own test baked that in).
  Moved to the ANIM_DONE arm (`drone.rs:4745-4756`); test rewritten to drive the completion and
  pin the player position (`special_death_0x0600021f_plays_0x148`).
- DroneWeap_FireWeapon (0x178f68:277): MATCH — `drone.rs:11283-11287` (weapon_data +0x60, skip
  0x5EF; the pitch draw deliberately not consumed — fix-guard-weapons3).
- BOT pain (0x125ed8/0x126618): MATCH — same pools; the `Sound_Playing_Ref` suppress gate
  deliberately unmodeled (MP determinism).
- Music events 0xF/0x2/0xE/0x10: MATCH except **0x10 was WRONG — FIXED this sweep**: the
  HostageSaved ENTER queued `music_reqs.push(0x10)` but **nothing ever drained that outbox**
  (decl + 2 inits + the push; no consumer anywhere) — the hostage-saved cue never reached the
  music board. Drained in `Drones::tick` (`drone.rs:20585-20592`), next to the sfx_reqs drain;
  test `hostage_saved_posts_the_0x10_music_event` (fail-first: the unfixed outbox keeps the cue
  forever). Gaps: `Drone_EnableAll`'s 0xF rides the 0x600001b script (not live on ported
  levels); InitComms' +0x100-aware arm unmodeled [S].
- Mission 8/7/6: MATCH — `drone.rs:17955-17956` + `death.rs:232-234`.
- Trigger_Activate (0x1b2318:109): MATCH — `drone.rs:19585-19592`; deviation: the port also
  fires (3,1) on the script-0x12 LoadLevel path (`drone.rs:17049`) where the game posts nothing.
- Script sound streams (0x1d8820/0x1d9860:83/0x1d9a08): MATCH — `nis.rs:639-654` →
  `drone.rs:16932-16951/19343`; the SetVolume fade unmodeled (the game's volume never reaches
  the driver — sound.rs documents the forced 100).
- NIS music markers (0x1d9140:48/0x1dac78:55/0x1d9860:74/0x1db588:40): MATCH —
  `drone.rs:16863-16872/17176-17177`, same 0x40-flag gate and three-hash set.
- Sound_Ready (0x1dceb8:108-109): MATCH — `nf-view/lib.rs:510-512` at load.
- Sound_UpdateSounds (0x1dda70): MATCH — `sound.rs:273-309` (the 1.1× outer cull, the 5-tick
  START_GRACE documented) + the subtitle hook (more faithful than the doc's "presentation").
- UpdateMusicalEvents (0x114e70): MATCH — `music.rs:200-266` → `nf-audio` (start/jump/fade).

## Verification

- `cargo test --offline -p nf-game --lib`: **689 passed, 3 failed, 5 ignored** — the 2
  `mp::bots::fell_out` failures pre-exist this branch (verified on a clean stash of the same
  tree); `one_way_door_refuses_from_the_game_side` failed only before its own assert was
  corrected to expect the open sound (285) alongside no clunk.
- Changed-code binaries (NF_ISO, release): `sound_coverage_live` **5/5** (incl. the rewritten
  0x148 completion test); `ai_missing_states_live` **10/10** (incl. the new
  `hostage_saved_posts_the_0x10_music_event`).
- **The full suite did not fit in the window** (deadline ~14:54 UTC). NOT yet run whole: every
  `walkreal_*_live`, `npc_doors_live`, `fix_guard_misc_live`, and the remaining binaries. The
  door change (`activate` one-way arm) and the drone changes (0x148 arm, music_reqs drain) are
  covered by the lib tests + the two binaries above, but the driver should run the full suite
  before merging.

## What's left

1. `Player_ChangeSubState` loop stops (0x12/0x3D/0xF0 on substate change into [1..3]∪[5..0x10],
   NOT 0 or 4-crouch) — WRONG, not fixed; condition pinned in the doc row and above.
2. 0x1DB refused jump (07000007, ch 0x48 off) — MISSING; the port launches where the game
   refuses; condition fully pinned.
3. 0x5EF hat-spawn arm — MISSING (MP record; settle the [L] weight-0 audibility question first).
4. Small gaps worth rows/tests: the weapon-change draw site's no-draw-script silence; the 0x10E
   whoosh on the guard path (MP-only); the stick-on-object 0x51C corner; the
   `Drone_ModPlayerHitDamage==0` early-out; the script-0x12 extra (3,1).
5. Run the full suite (esp. every `walkreal_*_live`, `npc_doors_live`, `fix_guard_misc_live`).

## Live checks for the main agent

1. **One-way door side**: on a level with a p10 != 0 door (approach mode 1 or 2 — census needed;
   `door_research_07000001` prints the column), approach from both sides and gdbwatch
   `Door_Activate` 0x135be0's clunk site 0x135cf8: expect the +0x64 play from the decompile's
   side (mode 1: activator on the facing side; mode 2: the opposite). Settles the `_vsub`
   reading beyond the disasm-verified argument order.
2. **0x148 cry**: on a level with the 0x0600021F special death, let the scripted death play out
   and gdbwatch `Sound_Play3D` 0x1dd880's `$a1`: expect the PLAYER's position (glb_players+0x30)
   at the script's END, not the dying drone's.
3. **0x5EF audibility**: throw Oddjob's hat in MP (or spawn any 0x45 bullet) and listen:
   SFXOutputData 0x5ef reads looped with weight 0 [L] — confirm audibility before porting the
   hat-spawn arm.

## Usage

Not visible to me: this session runs through an IM gateway and exposes no token, cost or
context-window figures. (The subagent harness reported per-call token totals — the six audit
calls ranged ~122k-224k total tokens each — but those are the agents', not mine.)
