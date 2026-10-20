# Net slot positions — every active slot takes its post-move position

GAPS G-053 item 4. One rule, stated once: **after a net tick's movement, every active
slot's match record takes that slot's body post-move position, written in the same
place for every slot, with no fixed 4.** This doc is the mechanism, the history of the
fixed-4 defect, and the writers' audit (2026-10-20, task mp-net-slots-post-move).

## 1. The state and who reads it

The port's match carries one live record per slot, `MpGame::slots[slot]`
(`crates/nf-game/src/mp/mod.rs:364-400`, `MP_MAX_SLOTS` = 16). Its `pos` is the port's
stand-in for what the game reads live out of each player object: `MP_GetSpawnPoint`
0x185098 walks the slots reading `MPGame + slot·0x30 + 0x1c` (the player obj) and takes
each candidate's live `obj+0x30` position [M 0x185098:123-129 — the decompile's loop
reads the obj pointer per slot, then `Vec_SqDist3D` against `obj+0x30`]; the port keeps
that position in `MpGame::slots[slot].pos` via `MpMatch::set_slot`
(`MPGame + slot·0x30 + 0x1c = obj` is the game's own link [M 0x183a50:127]; the port
stores the position the spawn policy and later phases read — `set_slot`, mp/mod.rs:876-880).

Two folds of `net_state_hash` see the position on every machine
(`crates/nf-view/src/scene.rs:10667-10759`):

- the **bodies** fold — every slot's `bond`/`mp_bonds[slot]` position, yaw, pitch,
  substate, crouch (scene.rs:10686-10700);
- the **match** fold — every `m.game.slots[*].pos`, timers, score, kills, deaths, dead
  (scene.rs:10707-10723).

A slot whose match record does not track its body desyncs the match fold on the
machines that disagree, at the first exchanged tick after it moves.

## 2. The write sites (the audit)

The per-tick post-move write is ONE shared function, `nf_view::write_net_slot_positions`
(lib.rs:498-513): every slot of `0..MAX_AGENTS` ascending from its `mp_bonds` body, the
local slot last from `bond` — the live loop (lib.rs:4735) and `net_tick_comps`
(lib.rs:527) both call it, so the two paths cannot drift apart again. Every other
writer of a slot position, with its bound — all slot-general [M code, the port's own
sources]:

| Site | Bound | Role |
|---|---|---|
| `write_net_slot_positions`, called from lib.rs:527 and lib.rs:4735 | `0..MAX_AGENTS` + the local slot | the post-move write, one place for the tick path and the live loop |
| the MP block's mid-tick ride, scene.rs:2113-2120 | `0..MAX_AGENTS` | pre-move positions so the match rules/spawn policy never see the load-time ones mid-tick |
| the load's spawn walk, lib.rs:1395-1411 | `0..humans` | the match start (`MP_Start` creates every human [M 0x183a50:116-128]; the game's own human cap was 4 [M 0x183a50:138-140], the port extends the walk to the 16-cap settings [S mp-16-slots]) |
| the respawns: scene.rs:9309, 10589-10617; `do_respawn` rules.rs:434-444 | per slot | `MP_ReSpawn` 0x185368 |
| the restart respawn, scene.rs:9400-9410 | `0..human_count` | the round restart's re-picks |
| the late-join spawn, scene.rs:10636-10645 | per slot | `Player_Init`'s port |
| the bots: lib.rs:1425, scene.rs:9569-9673, mp/mod.rs:688 | per bot slot | the bots' positions follow the humans [S mp-16-slots] |
| the replay apply, mp/replay.rs:162-168 | recorded slots | the bug-replay machine |

`set_slot` itself is a plain indexed store (mp/mod.rs:878-880) — no cap inside.

## 3. The defect the GAPS item described, and when it left

- net-match-host (31885a2c, 2026-10-15) introduced the online tick with the live loop
  writing only `for sl in 0..4` [M the commit's lib.rs] — the game's own 4-human era.
- mp-16-slots (e4f7cbf7 and neighbours, 2026-10-18) grew every array to 16 and the load
  walk to every human, but the live loop kept the fixed 4: on the other machines a 5th+
  human's `slots[5..].pos` stayed at its load-time value while its own machine wrote the
  post-move one (`m.set_slot(slot, sim.bond.pos)` runs for the local slot whatever it
  is) — both folds see the disagreement the moment the body moves [M the e4f7cbf7 tree's
  lib.rs:4187].
- 30a3cf69 (2026-10-18, the "desync when I move with bots" fix) widened BOTH sites in
  the same commit — the tick path from `1..MAX_AGENTS`+slot 0 to
  `0..MAX_AGENTS` with the local-slot skip, and the live loop from `0..4` to
  `0..MAX_AGENTS` [M the commit's lib.rs diff]. It reached the MP lineage at the
  876307b5 merge (2026-10-19) and master from there; master e5638828 carries it.

So on current master the code change the GAPS item asks for is present — and this task
narrowed it to one place: the two per-tick write sites (the tick path's all-16 loop and
the live loop's post-30a3cf69 all-16 loop) are now one shared
`write_net_slot_positions` (the drift that produced the fixed 4 cannot reintroduce at
one site while the other stays correct). What the item still lacked was the proof
through the real path — a multi-human loopback session whose machines agree on every
tick while a slot past the first four moves — and a pin that the 4-human shape is
untouched. `mp_stress_explosives_live` (6d89c71c) already ticks 3 machines (locals
0/1/5) through `nf_view::net_tick` with a synthetic shared row; the new
`mp_net_slots_post_move_live` covers the real session path (6 humans over 127.0.0.1),
every tick, with the replays giving every machine's view, plus the sister 4-human
digest pin.

## 4. The tests (mp_net_slots_post_move_live)

- `six_human_loopback_slot5_moves_hashes_agree_every_tick` — 6 real sessions over
  127.0.0.1 (the mp_16_slots_live direct-`Session` shape), 2 simmed machines (the RAM
  bound: one arena load is ~875 MB measured on the 3 GB sandbox; 4 concurrent loads
  OOM — mp_16_slots_live's 2-of-8 precedent), slot 0 and slot 5, both with `net_rng`
  set, 240 ticks through `nf_view::net_tick`. Every tick: the two sims' hashes agree;
  every human slot's match record equals its body's post-move position on BOTH machines;
  slot 5 really travels. The other four sessions feed real inputs and gate every
  `ready_inputs` (the lockstep never runs a tick without all 6). Afterwards each
  remaining local (1..=4) is replayed alone over the recorded rows through the same
  `nf_view::net_tick` — the app's own late-join verification shape
  (`mp_late_join_catchup`) — and must reproduce the recorded hash at every tick: all
  six machines' states and hashes agree, within the sandbox's RAM.
- Sister `four_human_session_digest_is_pinned` — the same harness with 4 humans; the
  final digest is pinned to the value measured on the pre-task tree (master e5638828).
  A digest change means the 4-human path moved; re-pin consciously.

Both can fail, and the shape of the failure is the GAPS item's own: with the shared
writer's loop narrowed back to `0..4` (the experiment, reverted), the 6-human test
fails at tick 0 — slot 5's machine's hash disagrees with the host's (the match fold
sees the frozen record) — while the 4-human sister still passes, because all four of
its slots sit inside the fixed window: exactly the blind spot the pre-16-slot tests
had. The unified writer keeps the pinned 4-human digests bit-identical
(0x5c0bb60a8cdf8938 at tick 0, 0xfb7b58f73e340ec3 at tick 240 — measured on the
task's tree before the refactor and reproduced after it).

NF_ISO-gated (the arena loads from the disc like the other net tests).
