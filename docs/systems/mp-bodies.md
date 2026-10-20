# mp-bodies — drawing the other players in an MP match

Evidence tags: [M] decompile, [D] disc/ELF data, [L] live, [S] port stand-in. Code: `Scene::build`'s MP-body block,
`Sim::mp_body_state`, the MP arm of `frame_state_for`'s `pose_one` (crates/nf-view/src/scene.rs).

## What the game does

`MP_Start` 0x183a50 creates one player object per human slot through `Player_Init` 0x1910f8, whose body skin is
`MP_skins[MPSettings + slot·0x30 + 0x24]` [M 0x1910f8:111]. `MP_skins` (ACTION.ELF 0x26d488) holds 29 records of
16 bytes: skin hashcode, name label, a small count, 0 [D]. Bots get theirs from `BOT_init` 0x1249e8 through the same
table (the record's +0x10 character index). The bodies run the PLAYER animation set (`PlayerAnimSetInitNormal`
0x18fcd8 and its siblings, which branch on the MP flag) [M].

## What the port does (2026-10-18, the user's "they cant see eachother")

- **Build:** `DroneContext::mp_body_skins` carries one skin per slot (`nf_game::mp::slot_skin`: humans
  `0..human_count`, bots `4..4 + bot_count`, else 0). For each, `Scene::build` registers a Character from the skin's
  meshes (main + extra parts as attachments), parked at the origin; `Scene::mp_bodies` maps slot → character. All 29
  skins resolve in 07000023's loaded skins [L probe_mp_skins].
- **Visibility:** `char_hidden` hides a body when `Sim::mp_body_state(slot)` is None. That covers the local slot
  (`Sim::mp_local_slot`, set by `swap_local_slot`), an empty slot, and no match.
- **Pose [S]:** the drone tables' idle / walk / run scripts (`anim_state::STAND_IDLE` / `WALK` / `RUN` →
  `danim_for_state` → `script_for`, the first anim set carrying the danim). The pick uses the slot's measured
  horizontal speed (`Sim::mp_body_speed`, position change per tick; draw-only, never hashed): < 0.5 u/s idle,
  < 4.5 walk, else run. The time is `Script::time_at(frame)`. The matrix is `object_matrix([0, yaw, 0], pos)` with
  the drones' hip height. This is a stand-in until 700v7 ports the player anim set.
- **Not yet:** the held weapon in the body's hands (needs each slot's weapon, 700v6), death / hit reactions.
- **Death / shots (mp-remote-fx):** a dead slot's body plays `DEATH_SCRIPT` (substate 0xd) until the respawn, without
  its gun (`Sim::mp_body_death`; a dead bot's `mp_body_state` is now None). The body's muzzle flash, trails, impacts
  and sounds: docs/systems/mp-remote-fx.md.

## Checks

[L 2026-10-18] two windows on one PC (`--net-host` / `--net-join`, `NF_MP_FACE_OTHER=1`, `NF_NET_SHOT=120`): each
window shows the other player's body standing at its spawn, on the floor (scratchpad mp/b6 captures).
