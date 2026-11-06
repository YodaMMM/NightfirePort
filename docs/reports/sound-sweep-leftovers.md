# sound-sweep-leftovers

Branch `s/sound-sweep-leftovers` from `origin/master` (c70343b3a). The sound-gates sweep's report
(docs/reports/sound-gates-sweep.md on `s/sound-gates-sweep`) marked three port rows WRONG and left
them unfixed; this branch fixes the two that were still unfixed on master.

## Done

1. **`Sound_StopAllWithId` 0x12/0x3D/0xF0 on the substate change** (0x190630:83-85, disasm
   0x1907b4-0x1907e8; `Player_Init` 0x1910f8:41-43). The game routes every substate change through
   `Player_ChangeSubState`, whose tail stops the loop trio when the NEW substate is in
   [1..3]∪[5..0x10] — substate 0 skips the outer `0 < param_2`, 4-crouch returns at 0x1907b4
   before the calls, > 0x10 jumps past. The port's substate writes were scattered field assigns
   with no stop at all: a live Laser/Stunner beam loop survived a swim/climb/wire entry where the
   game silences it. Fix: `Bond::change_substate` (player.rs) is now the funnel — every substate
   write site (18 in player.rs, the SP grapple, the NIS hand-back, the MP grapple in scene.rs)
   goes through it; the trio queues into the new `substate_stops` outbox and the Sim drains it
   into `drones.audio` with the game's own id-keyed stop, next to the `sfx_out` drain
   (scene.rs:3480; the MP flush at scene.rs:14108). `Player_Init`'s unconditional trio is left as
   the no-op it is: the pool is fresh at the player's creation, nothing can be looping yet.
2. **0x5EF, Oddjob's hat spawn** (`Bullet_init` 0x12c5a0:286-290). The +2 == 0x45 record is the
   hat; its spawn plays the LOOPED 0x5ef and stores the handle at BU+0xF0; `Bullet_Delete`
   0x12e660:13-16 stops it (the hat's post-stick expiry is a silent death but the stop runs). The
   port played 0x5ef nowhere. Fix: the SP throw's `Bullet_init` arm at the scene's Grenade spawn —
   `loop_handle: if base == 0x45 { audio.play3d(0x5ef, origin, None) } else { 0 }` — plus the new
   `Grenade.loop_handle` field and the delete-side stop at the Drones retain (drone.rs, next to
   `grenades.retain`). The MP combat's thrown arm carries `loop_handle: 0` — `MpCombat` has no
   audio sink at all (its stick/bounce sounds are not wired either), recorded as the gap below.
3. **The report's line-201 row** is the "What's left" item 1 continuation — the same
   `Player_ChangeSubState` trio as (1); item 2 of that list, the 0x1DB refused jump
   (0x1916b0:106-112), is already fixed on master (scene.rs:3456/3470: `jump_locked` from
   level 0x07000007 + ch 0x48, the refused press plays the 2-D 0x1DB) — the sweep predated that
   merge. Rows 0x1DB, `Sound_Stop [BL+0x898]` (provably dead code) and the doc-label rows
   (param_7, 0x447, 0x27C) needed no code change.

## Verification

- New binary `sound_sweep_leftovers_live` (NF_ISO, real levels): **2/2**.
  - `the_laser_loop_dies_at_the_swim_entry` — 07000002 from the load, the Laser equipped through
    the real switch input, the beam loop 0x12 live in the pool, then the fix_swim pond walk on
    real input: the entry INTO substate 3 stops the loop. Also pins the helper's branch table
    (4-crouch and 0 stop nothing, a wire entry queues the trio). Red on the old code: the loop
    survived the swim entry (the drain did not exist).
  - `the_hat_spawn_plays_the_0x5ef_loop_and_the_delete_stops_it` — 07000001, the 0x45 record
    thrown through the real state-9 release path (the harness copies the frag record's SP-resident
    fire fields into the 0x45 record; the hat's own +0xb0/+0xb4 hashes are the MP bank's and are
    missing from the SP script table): the spawn plays the looped 0x5ef, the handle is stored, and
    after the stick + the 1500-tick silent life the `Bullet_Delete` stop retires it
    (`finished(handle)`). Red on the old code: no 0x5ef play existed at all.
- `cargo test -p nf-game --lib` (release): **686 passed, 0 failed, 5 ignored** (the 5 ignores
  pre-exist).
- Every binary whose literals or grenade path this change touches:
  `grenade_rattle_live` **4/4**, `blast_smoke_live` **4/4**, `guard_drops_flashbang_live` **6/6**.
- NOT run whole (the window did not fit): the full suite — every `walkreal_*_live`,
  `npc_doors_live`, `fix_guard_misc_live`. The changes are (a) the substate funnel — each site's
  surrounding behaviour is untouched, only the write routes through the helper, and (b) the
  grenade field + the 0x5ef arm (dead for every non-0x45 record; the trio of grenade binaries
  above covers the tick path). The driver should run the full suite before merging, per the usual.

## What's left

1. The MP hat throw stays silent: `MpCombat` has no audio outbox (no stick/bounce/throw sounds
   either). Wiring one is an MP-audio task, not a row fix.
2. The 0x5EF audibility question stands ([L SFXOutputData 0x5ef: looped, weight 0]) — live check
   below.
3. Small sweep gaps (report item 4) untouched: the weapon-change draw site's no-draw-script
   silence; the 0x10E whoosh on the guard path (MP-only); the stick-on-object 0x51C corner; the
   `Drone_ModPlayerHitDamage==0` early-out; the script-0x12 extra (3,1).

## Live checks for the main agent

1. **0x5EF audibility**: in PCSX2 MP, let Oddjob's hat spawn (or spawn any +2 == 0x45 bullet) and
   listen at the spawn: SFXOutputData 0x5ef reads looped with weight 0 [L ee_snapshot] — confirm
   whether the hat's spawn loop is audible in the shipped game before the MP path ever plays it.
2. **Substate trio in the game**: on any level, hold the Laser beam and grab a ladder; gdbwatch
   `Sound_StopAllWithId` 0x1de0c8's `$a0` at the `Player_ChangeSubState` tail (0x1907b4+) —
   expect three calls (0x12, 0x3d, 0xf0) on the climb entry, none on a crouch toggle.

## Usage

Not visible to me: this session runs through an IM gateway and exposes no token, cost or
context-window figures. The only environment signals I can see are the sandbox's own process
accounting (a wall clock of about 1 h 50 m from the first `date -u` at 20:11 UTC), which is not a
token, credit or cost number — so nothing to copy; not estimated.
