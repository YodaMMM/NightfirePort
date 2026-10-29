## script-channel-events (started 12:15 UTC 2026-10-29)

Task: cut-scene script streams that set switch channels. The port parses Cmd::Event (op 0x12
[M 0x1d8548 stream walk]) but acts only on kind 0xB (weapon); Script_EventHandler 0x1d94c0's
channel-write events — 0xD (set channel, params [ch, value]) and 0x11 (conditional set) — are
not acted on. (1) Census every 0xD/0x11 event in every on-foot level's script streams and
loadable packs [D]: level, script hash, stream, params. (2) Port both events exactly per
0x1d94c0 [M], including skip-path behaviour. (3) Per census row, a test on real input that
plays/skips the cut-scene and asserts the channel written and what it unlocks. Full suite,
quote its total line.

Plan: branch s/script-channel-events pushed from origin/master (1b2309ea). work/ + ISO already
in sandbox from prior session; next: build check, census scan of disc script streams, then
0x1d94c0 decompile reading.

Update (13:45 UTC): census committed (199 rows [D], census_channel_events.rs); 0xD/0x11
exactness pinned per 0x1d94c0 (3 lib tests); the live half — the all-level write audit (every
script-path write is a census row or an SP out mirror), the 07000006 boot NIS lighting the
four searchlights as it plays, and the game's own no-camera skip gate pinned on real input.
Report: docs/reports/script-channel-events.md. Full suite running in the background.
