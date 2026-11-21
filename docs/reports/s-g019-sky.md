# s/g019-sky: sky behaviours against View_DrawSky (G-019)

**First: one walk test goes red. Keep this on the branch.** `walkreal_0700000b_live
walk_fresh_spawn_to_the_mission_end_real_damage_agent` fails on the branch: Bond dies at tick 17389
at (-5.07, 58.07, 19.37). It passes on master. Two runs isolate the cause. With the gate forced
open the test passes, and with the gate on but the thunder sound muted it also passes. So the
thunder plays at different moments (it now pauses indoors), and that changes the fight outcome
through the sound voice pool (`sound::Sounds::alloc`, the 256-node pool the walk's headless sound
frees by voice length). The game's `Sound_Play` takes a voice too. Which sim read couples a voice
slot to the fight was NOT traced. The other seven 0b walks, all 13 runnable 0a walks and all 19
1b walks pass.

RNG: the shared random stream is unchanged. The sky timers draw from the port's own `sky_rng`,
and the new gate only changes when that private stream is read.

## What a player would notice
- 0700000a / 0700000b (storm): thunder and the Thunderhead flash timers now pause while Bond is
  somewhere the sky is not seen from (every room the view reaches is a no-sky room), as the game
  does. Before, thunder kept rolling deep indoors.
- 0700001B (space): the sun position the lens flare and the sees-the-sun ambient read now turns
  with the rotating sky every frame, instead of staying where the level load put it. This changes
  the flare and the view-model brightness on 1b: it needs a render look before merge.

## Found and ported (logic and data)
1. Viewer sky gate [M]. `parseentity_fixup_entity` 0x1d00c0 writes 0x1804c023 / 0x1804c047 for a
   room whose parameter 3 is non-zero. `Vision_Portal_Recurse` 0x1e86e8:33-41 sets `viewer+0x20`
   when any VisCelList cel lacks bit 0x10000000. `View_DrawSky` 0x1e5ad0:91 runs nothing without it:
   no timers, no thunder, no `SuperHappyLightningFlag` clear, no 1b count, no slot draws.
   Port: `celgraph::CEL_NO_SKY` (set in `scene::parse_cel_graph`), `celgraph::chain_sees_sky`,
   `Sky::frame(level, paused, sees_sky, rng)`, `Sky::no_sky_view`. `Drones::sky_frame` reads the
   gate from the same VisCelList walk the drawn gate uses (sim eye/aim/linked cel), only on
   levels with sky state (flash slots or 1b). Disc counts [D]: 0a 23 of 30 rooms flagged,
   0b 15 of 23, 1b 0.
2. Sky slot table [M]. `Cloud_Create` 0x18d620 also calls `View_AddSkyObj`: slot = param 4, the
   cloud object in the +0x18 handle, group 1 (a stack argument the decompile drops; the
   disassembly at 0x18d724 stores 1). A normal call writes both normal handles, so a later
   placement on a shared slot replaces the earlier one. On the disc [D]: 07000003 slot 0
   (moon_old, then SkyBox1-terra), 07000004 slot 1 (mountains group 1, then moon group 0),
   07000006 slot 3 (the cloud, then Sky_Tower12) and slot 0 (moon, then skybox), 07000014 slot 0
   (two Light_rays). Slots now also carry the placement translation and rotation.
   `Sky::from_placements`, `Sky::placement_of`.
3. 1b sun [M]. 0x2d6230, which the sky block copies as its base matrix, is `CONSTANT_MAT_IDENT`
   (symtab), not a camera matrix. So `SunVector` = RotMatrix([count/1440,0,0]) applied to
   spherical(1000, 0x409f2c8e, 0xbf29c920), rewritten on every slot pass (paused frames too, the
   count held). `sky::rotating_sky_sun`; `Drones::sky_frame` writes `sun_vector`. The decompile's
   only other SunVector users are Env_Create, Env_Reset and `HUD_UpdateLensFlarePane` 0x19d770. So
   the open "sun-vector effect on lighting" question has this answer: it reaches lighting only
   through the flare pane's sees-the-sun byte (BLData+0x962).
4. Checked, no change needed: the thunder sound ids 0x5d / 0x5e at volume 100, the flash/flicker
   rolls, the interval scaling, the cloud drift (`cloud_update` 0x18d768; both disc clouds drift
   positively, so fmodf and the port's rem_euclid agree). No script event writes sky state: the
   only writers of GfxList's sky slots are `View_AddSkyObj` and `Env_Reset`'s memset [M grep].

## Draw-side differences for the renderer (not changed here)
- Shared slots: `Level::sky_slot_instances` / `sky_instances` still return every sky placement,
  so 03, 04, 06 and 14 draw objects the game never draws (04's mountains, 06's moon and cloud
  layer, 03's moon_old, the first 14 light-ray mesh). The slot table in `drones.sky` is the
  game's: draw only the handles left in each slot.
- Group for shared slots: on 04 slot 1 ends in group 0 (drawn after the world), not group 1.
- Clouds: the game draws the cloud through the object path at (player x + phase x,
  param 0 + viewer y, player z + phase z) with obj+0xe8 scale [M 0x1e5ad0:163-175, 0x18d768].
  The port draws the static placement. `CloudFx::phase` already holds the drift.
- Gate: with `drones.sky.no_sky_view` set, the game draws no sky slot at all.
  `HUD_UpdateLensFlarePane` 0x19d770:68 also skips the sun line-of-sight test (flare off,
  BLData+0x962 = 0) while the gate is shut. The port's flare and `ramp_char_ambient` ignore it.
- The sky pass state `View_DrawSky` sets: tweak ARGB ff, colour blend 2, lights off, fog
  (0, 10.0, group, 0xff, 0, 0) before the loop and (0, 20.0, 1, 100, 100, 0x96) after it.
  Not compared against the renderer here.
- `Game_Draw` 0x1e6f50 runs the sky pair once per active viewer. A second viewer (split screen,
  or the extra viewer list at 0x2d88c0) would tick the timers again. The port ticks once a frame.
  Only matters in multi-viewer play. Not ported.

## Tests
- nf-game: sky_view_gate (4, pure), sky_slots_live (3), sky_lightning (4), sky_rotation (3),
  sky_flash_live (4), sensor_live (3).
- nf-view: sky_view_gate_live (3), sky_spin_1b_live (2), light_sun_live (2),
  walkreal_0700000a_live (13 ok, 2 ignored as before), walkreal_0700001b_live (19 ok),
  civilian_slide_0a_live (ignored as before), walkreal_0700000b_live 7 ok / 1 FAILED (see the top).
- Fail-first, run on the old sources: sky_slots_live clouds + translation fail (2), the
  shared-slot test passes on old code too. The old 0x2A path already overwrote the glist, and the
  06 cloud was missing from it. That test is a pin, not a fail-first test.
  sky_view_gate_live: all 3 fail on old code (no flagged cels, timers running indoors, 1b sun
  static). The pure sky_view_gate tests use the new API, so they cannot compile on the old code.

## Unproven
- No live capture of the gate. The claim that thunder stops indoors on 0a/0b is [M] plus [D]
  only. The sim's VisCelList walk stands in for the render camera's (sim eye, aim, the +200
  back plane), so the exact tick the gate flips can differ from the game's.
- The one-frame order between the 1b sun rewrite (the draw) and the flare read (the update) is
  not pinned.
- The exact ZMSK during the sky draws stays a live-check item.
