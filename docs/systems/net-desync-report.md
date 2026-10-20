# Net desync report (net-desync-detail, 2026-10-19)

When two machines' state hashes disagree we only learn the tick. This system makes every
desync diagnosable: the machines name the first diverging component and slot, and each writes
a report zip that replays.

Everything here is the port's own online infrastructure [S port protocol] — the PS2 game has no
network mode; the determinism it diagnoses is the shared bit-exact RNG + the fixed 60 Hz tick.

## The named component hashes

`Sim::net_state_hash` (crates/nf-view/src/scene.rs) is now the fixed-order fold of seven named
component hashes (`Sim::net_component_hashes`, `crate::scene::fold_component_hashes`):

| # | name | contents (each hashed exactly once) |
|---|--------|---------------------------------------------------------------|
| 0 | `frame_rng` | the tick counter + the shared RNG stream (x, y, draws) |
| 1 | `bodies` | per-slot body poses (pos, yaw, pitch, substate, crouched) — the local slot reads `bond`, the others `mp_bonds`; an empty slot folds the `0xffff_ffff` sentinel |
| 2 | `vitals` | per-slot health, dead-ticks, armour |
| 3 | `combat` | `MpCombat::hash_into` — the shared rounds, thrown grenades, rides |
| 4 | `match` | the match/game state (state, clock, limit, the slots, the settings) + `nf_game::mp::hash::state_hash` (the whole scenario state) |
| 5 | `bots` | per bot: `BotBrain::state_hash` + health + dead |
| 6 | `world` | `net_world_fold` — pickups, doors, triggers, world channels, destroyables, breakables, SS objects, hurt volumes |

The order is part of the wire contract (`PT_HASHCOMPS` carries the components in this order;
`NET_HASH_COMPONENT_NAMES` is the name table). The combined hash value changed with the
regrouping (the same fields, folded per component and then folded again) — re-pinned on
purpose; no test pinned a literal. Hashes from a protocol-7 build never match an 8 build's;
the handshake refuses that mix.

## The per-item breakdown

`Sim::net_component_items` returns each component's items — one item hash per hashed element,
in the fold's order: bodies/vitals one per agent slot; combat the rounds count, one per round,
the thrown count, one per thrown, one per ride slot; match/frame_rng a single item (the
component hash); bots one per bot; world one per pickup, then per door, then per trigger, then
per world-channel entry, then per destroyable, per breakable, per SS object, per hurt volume.
The report maps a world item index to a name from the counts it prints (`world items:` line in
`report.txt`).

## The history and the exchange

Each machine keeps the last 600 ticks of component hashes (`desync::HashHistory`, pushed every
executed tick beside `submit_hash`) plus, at the exchange cadence (`Session::hash_every`,
default 15 — a desync is only ever detected at an exchanged tick), the per-item breakdown.

On a desync at tick T (both machines see `Event::Desync` at the same tick — the compare loop is
symmetric), the machines exchange:

1. `PT_HASHCOMPS` (type 19): tick u32 + 7×u64 components.
2. `PT_HASHDETAIL` (type 20): tick u32, component u8, count u16, count×u64 — the first
   diverging component's items (at most `HASH_DETAIL_CAP` = 160 items ride one packet; a bigger
   world truncates the tail).

`desync::Watcher` is the per-machine state machine: it sends the local components on the
desync, names the first component whose hashes disagree once the peer's arrive, sends that
component's items, and completes a `Diagnosis { tick, component, item, local/peer comps, local/
peer items }` when the peer's items land. The packets cross (each side may see the peer's
components before its own desync event), so every handler pumps the machine to quiescence —
at most one `PT_HASHCOMPS` + one `PT_HASHDETAIL` per desync. Protocol version 7 → 8.

## The report zip

`desync::desync_report_files` builds the bug-report format (`zip_store` entries,
`report.txt` `key: value` lines):

| file | what |
|---|---|
| `report.txt` | `kind: desync`, report_id, version, os, level, seed, desync_tick, first_component, first_item, the combined hashes, the replay line, the world-item ranges |
| `components.txt` | the 7 components' local vs peer hashes, the first diverging one marked |
| `hash-history.txt` | `tick c0 … c6` per kept tick (the reporter's view) |
| `match-settings.bin` | the tag-1 match-start payload (the wire MatchSettings + the seed) |
| `match-log.bin` | first_tick u32, tick_count u32, then per tick the 16 wire inputs — the full input log since the match start |

The app (`App::finish_desync_report`) saves it through `bugreport::save` into the reports
folder and offers the same "send report" path as F10: the GitHub issue form opens pre-filled
(`[Desync] <level>: <diagnosis>`) and the file manager shows the zip. The fail box carries the
diagnosis line (`menu::FrontEnd::desync_report` / `Diagnosis::ui_line` — the draw code renders
it). While the exchange runs (≤3 s) the session stays up; on a timeout the fail box says the
exchange did not complete.

## The replay

`nf-view --bug-replay <report.zip>` recognises `kind: desync`: it rebuilds the match from
`match-settings.bin`, replays `match-log.bin` headless (the lockstep transcript is every
machine's view — `net_slot0_setup` + `net_tick_comps`, the live loop's tick shape) and names
the first tick/component where the replay's component hashes differ from the reporter's
recorded `hash-history.txt`. The replay reproduces the honest state, so that first difference
is the symptom the reporter's machine carried; the peer's own report (the same exchange ran
there) gives the other view.

## Tests

`crates/nf-view/tests/net_desync_report_live.rs` (NF_ISO-gated): a 2-machine loopback match
through the real session layer; a test-hook perturbation of (a) one rocket's position, (b) a
bot's goal, (c) a door (Sub Pen 07000025 — 07000023 carries no doors) on one machine at tick
30. Each names the right component and slot on both machines (combat/item 1 — the count item
then one item per round; bots/bot 0; world/pickups+0 = door 0), the report zip holds the input
log + settings + seed + history, and `--bug-replay` of the door report names `FIRST
DIFFERENCE: tick 30 component 6 (world)`. Sister: the unperturbed match never desyncs and
writes no report. Unit tests in `desync.rs` pin the history cap, the first-diff order, the
exchange state machine (including the crossed-packet order), the match-log round trip and the
report contents.
