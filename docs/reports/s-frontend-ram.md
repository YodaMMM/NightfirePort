# s/frontend-ram: the front-end map stops building weapon meshes and combat pools

## What was wrong
Every load of the front-end map 07000048 (the boot load and every return to the menus) ran the
session weapon-mesh merge. The merge takes every story archive with a smaller level code, and
0x07000048 sorts after all of them, so it cloned 44 maps from 8 archives (1b, 16, 13, 0d, 0b, 08,
06, 04). `Scene::build` then built the combat/effect draw pools from them (tracers, beams, glows,
projectiles, swooshes, impacts, occluders, scan brackets, rain, leaves, satchel digits).

07000048 places no world instance and no character, Bond owns no weapon there, and the sim does not
tick while the front end is up (the tick loop runs only with `fe.is_none()`). The menu code never
takes the `Scene`; it draws from the HUD tables (font, strings, textures). The Weapons, Gadgets,
mission and character wheels show textures (0x03 hashes), not meshes.

## Measured (headless, `tests/front_end_ram_live.rs`, one load per process)
| path | before | after |
|---|---|---|
| boot load (`nf_view::load`) | 1.69 s, peak private 1365 MB, working set 736 MB | 0.10 s, 133 MB, 138 MB |
| return load (`load_front_end_gc`) | 1.91 s, peak private 1367 MB, working set 736 MB | 0.10 s, 133 MB, 138 MB |
| scene | 6353686 vertices, 13246116 indices, 103 materials, 12632 attachments | 0 / 0 / 0 / 0 |

These are CPU-side numbers. The window also uploads the vertex and index buffers to the GPU, so
GPU memory drops by the same buffers; that part was not measured.

## Change
- `load_impl_gc`: skip `merge_weapon_models` when the archive is the front end (`is_front_end_archive`).
- `Scene::build`: the pool blocks read `pools` (the drone context, except on the front end) instead of `drones`.
Every other level builds exactly as before (load_scene_equality unchanged).

## Pages checked
Headless `--page` shots, old binary vs new, byte-identical PNGs on all 15: start, main, codename,
difficulty, missions, dossier, records, rewards, gadgets, weapons, cnselect, nfbonus, credits,
wingame, screenadjust. The MP lobby pages have no headless shot; they are covered by the MP menu and
lobby tests below, and the menu code takes no `Scene` input.

## Tests
- `front_end_ram_live::front_end_builds_no_combat_geometry` (new): fails on the old tree (6353686
  vertices, 12632 attachments), passes now; also checks 07000001 still builds its pools.
- load_scene_equality, fe_frame_buttons, fe_pages_draw2, menu_audio_flows, menu_music_flow,
  campaign_flow_live, pause_results_flows_live, mp_lobby_one_screen, mp_net_lobby, mp_menu_flow,
  mp_debriefing_flow, movie_over_scene_live, lib flow_audit.
