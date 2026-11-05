# local-match-bot-pickups — report (2026-11-05)

**DONE (the wiring) + the pins green; two long online tests could not run on this sandbox.**
Branch `s/local-match-bot-pickups` (cut from origin/master 17a784980, `origin/s/audit-pickup-calls`
602bf0061 merged first — it was NOT on master and it edits `step_mp_pickups`).

## The symptom, reproduced first

A local match (real MP settings, no net session) via `nf_view::load_mp_carry` on the arena
07000023: a bot placed on a weapon pickup through its own out-of-bounds recovery
(`BotBrain::fell_out_map`), the sim ticked normally — the pickup stayed live. The shared take
pass `Sim::step_mp_pickups` (scene.rs, the port of `Pickup_Update` 0x18e148's humans-then-bots
taker loop) had NO caller: `update_pickups` handles only the local player's touch
(`mp_shared_pickups` defaults false [M drone.rs:13034]) and `update_pickups_net` runs only on a
net session's `Drones::net_bodies` row (scene.rs:3641) — the audit-pickup-calls finding #1,
confirmed by the pre-fix test run (2 of the 3 pins failed exactly on the take and the mark).

## The fix

1. **The pass is called from the per-tick MP driver** (inside `Sim::step_bond`, scene.rs): right
   after `step_mp_bots` and before any body moves — the point the pass's own doc names (every
   taker's position is last tick's on every machine). Gated to LOCAL matches only
   (`mp.is_some() && net_rng.is_none() && net_rng_shared.is_none()`): the online path keeps
   `update_pickups_net` (the session's `net_bodies` row) untouched, verified by the online
   suites (below).
2. **`mp::unregister_pickup` wired** through a new `Sim::reconcile_mp_pickup_rows`, run at the
   driver head BEFORE the bots' availability refresh (the refresh would mark the row first and
   the helper's row-registered gate would never fire): a gone pickup's row is zeroed and leaves
   `MpSettings::registered_pickups` [M Pickup_Handler 0x18e650:338-344 — the `+0x2c == 0` delete
   arm calls MP_UnregisterPickup under the MP gate; MP_UnregisterPickup 0x18a058:21-31; the
   lifetime expiry 0x18e148:254-258]. A hidden (state 2, respawning) pickup's row stays — only
   the delete arm unregisters [M 0x18e650:348].
3. **The bots' pickup rows read the REGISTERED law** — `pickGoal` skips only a row whose
   registered object is GONE [M 0x126b88:386-390, read first-hand this session: the skip gate is
   the row's null obj pointer / null emitter], so the row builder's `available` is `!p.gone` (it
   was `!gone && state != 2`, which also skipped hidden rows and kept the bots off them). The
   game's touch walk has no hidden gate either (the audit's own table: "Hidden (respawning)
   pickups stay takeable"), so a bot parked on a respawning pickup takes it on restore — the
   port now matches both.
4. **`reset_pickup_bot_visit_times` wired at BOTH BotDead ports** — the slot-death walk and the
   combat kill in `step_mp_bots`/the MP damage site, wherever `respawn_at = frame + 180` is
   stamped [M NDrone2_DSTATE_BotDead 0x171518:17 calls MP_resetPickupBotVisitTimes on the death
   msg, before the 180-tick stamp; the helper is 0x18be50] — so a respawned bot re-competes for
   pickups.
5. **The Auto Aim detector now sees local matches**: the pass sets `mp_shared_pickups = true`
   every tick, which is the mark lib.rs's detector reads (`net.is_some() ||
   mp_shared_pickups`) — the game picks the row on MPSettings+0x180 [M Player_AutoAim
   0x1a7c98:39-45]; the detector's mark was [S], the game's flag is the MPSettings word.

## The pins (`crates/nf-view/tests/local_match_bot_pickups_live.rs`, 3 passed / 0 failed)

Through the real per-tick driver (no direct pass call in the take path):
- `bot_takes_a_weapon_in_a_local_match_through_the_tick_driver` — the bot reaches a weapon
  (its own `fell_out_map`), the driver's pass takes it (state 2), the placement's own 0xF5
  plays AT the pickup [M 0x18e650:137 → :334-337; read off a second take driven straight into
  the pass, because the driver's own sound lands at the tick head and the same tick's AI
  hearing pass consumes the hearing-list copy], the respawn timer is the game's
  `60·respawn·10 (+1)` [M 0x18e650:348 + 0x18e148:183-185], the bot's visit stamp lands in its
  row [M 0x18e650:126-127], and the take reaches the bot's arsenal.
- `a_local_match_marks_itself_for_the_mp_auto_aim_row` — `mp_shared_pickups` true in play (the
  MP row's detector input).
- `a_deleted_pickup_unregisters_its_bot_row_and_a_hidden_one_keeps_it` — two real driven takes,
  then A into the delete arm's own state (gone): A's row unregisters while B's (state 2) stays
  registered. (The memset + count are the helper's own unit pin in bots.rs.)

## Existing tests re-pinned (each with the reason)

- `mp_bots_live::bots_roam_and_hunt` — the firing window 3600 → 7200 ticks: the bots now SEEK
  the registered pickup rows (the game's law above), so the first minute tours pickup spots.
  Green 3/0.
- `mp_perf_16_live` — `PINNED_DIGEST` re-pinned to 0x6b6f_2ae6_bf54_2614: the two game-faithful
  changes land inside this ONLINE run, both simulated identically on both machines (ha == hb
  every tick — the lockstep assert holds). Green 2/0.
- `pickups_g029_live` — the respawn pin 1200 → 1201: the audit's fix-4 law (the restore lands
  on `elapsed == limit+1` [M 0x18e148:183-185]); the old pin held the one-tick-early value the
  audit's code no longer writes — the audit session never ran this binary (its remaining list
  omits it). Green 4/0.
- `mp_pickup_take_live::mp_take_message...` — the sound read moved off the old order's accident
  (the take ran in the drones phase pre-wiring; now at the tick head, so the hearing-list copy
  is consumed same-tick): the law is read off a direct second take, and the message assert reads
  the driven take's own `taken` entry. Green 2/0.

## What was run (direct binary runs, --test-threads=1, slim ISO)

- Every `mp_*` + pickup binary in crates/nf-view/tests (45 files): **41 green** —
  fix_pickups_ammo 4/0, fix_pickups_remainder 1/0, fix_pickups_trace 2/0,
  local_match_bot_pickups 3/0, mp_16_slots 3/0, mp_bodies 3/0, mp_body_anim 2/0,
  mp_bot_armour_online 1/0, mp_bot_objectives 10/0, mp_bot_weapons 2/0, mp_bots 3/0 (after the
  re-pin), mp_bots_moving 3/0, mp_clock_gate 1/0, mp_combat_weapons 11/0, mp_core 5/0,
  mp_death_effects 3/0, mp_death_hands_online 2/0, mp_debriefing_flow 6/0, mp_defaults_setup
  12/0, mp_host_match 2/0, mp_hud_16 2/0, mp_hud_real 4/0, mp_lobby_one_screen 7/0,
  mp_match_end 2/0, mp_menu_flow 11/0, mp_menu_lists 5/0, mp_net_lobby 11/0,
  mp_net_slots_post_move 2/0, mp_perf_16 2/0 (after the re-pin), mp_pickup_take 2/0 (after the
  re-pin), mp_pickups_visible 2/0, mp_remote_fx 2/0, mp_respawn_ammo 3/0, mp_scenarios 19/0,
  mp_screens 5/0, mp_shot_effects 2/0, mp_shot_gate 5/0, mp_site_damage_offline 4/0,
  mp_status_strings 1/0, mp_world_sync 8/0, pickups08 5/0, pickups_g029 4/0 (after the re-pin),
  probe_tripbomb_pickups 1/0.
- `pickup_live` **1/0**; `nf-game --lib` **662 passed / 0 failed / 5 ignored**.
- **mp_online_failures and mp_stress_explosives_live did not run to completion on this
  sandbox** — both are killed (SIGKILL) past the 5-10-minute mark; verified pre-existing by
  re-running mp_stress_explosives on the BASE tree (my changes stashed): killed the same way.
  The sandbox cannot host these two runs this session.
- The full suite (`tools/test/suite.py --limit 900`) started in the background on the final
  tree; its progress/total is reported below if the window allowed.

## What's left

- The two killed binaries above need a box that can host them (they are pre-existing heavy
  online flows, untouched by this task's gating — the local-match gate excludes them by
  construction, and the digest pin proves the online lockstep holds).
- `MpSettings::registered_pickups` is 0 after an arena load (the register side never bumps it —
  the audit's finding list does not name it; the reconcile's decrement is saturating, so the
  count stays 0). Wiring the REGISTER side (Pickup_Create's MP arm) is a natural follow-up if
  the count's hash role ever matters.

## Live checks for the main agent

1. A local match (Quick Start with a bot), stand near a weapon: when the bot grabs it, the take
   sound (the placement's +0x2A word) should play FROM the pickup, and the weapon should appear
   in the bot's hands.
2. With Auto Aim Off on the SP row and On on the MP row (the options' multiplayer page), a local
   match should aim-assist per the MP row (the port reads the MP row via the mark; the game's
   own pick is MPSettings+0x180 [M 0x1a7c98:39-45]).
3. A bot that dies should re-compete for pickups after respawning (the visit rows zeroed on
   death [M 0x171518:17]) — before this task a respawned bot starved behind its old stamps.

## Usage

not visible to me (no token/credit/context-window figures are exposed to this session; nothing
in the sandbox reports them).
