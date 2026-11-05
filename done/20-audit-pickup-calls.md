# audit-pickup-calls — every pickup and inventory effect against the port

The sound, text, effect and HUD sweeps each found gaps tests missed. Sweep Pickup_Handler 0x18e650 and every caller of the inventory and ammo give paths for on-foot play and multiplayer: conditions, amounts, caps, sounds, lines, respawn timing. Fix what is missing or wrong at a clear port site and pin each with a test on real level data; report the rest as a table. Merge origin/master first.

---
claimed-by: sandbox7 at 2026-11-05T02:23Z

---
## Report (done, 2026-11-05T03:07Z)

Four Pickup_Handler 0x18e650 sweep fixes pinned on the real arena 07000023 (head 602bf0061,
report docs/reports/audit-pickup-calls.md): (1) take-message duration 180 SP / 90 MP
[M :324-327]; (2) the MP weapon label from weapon_data+0x3C bare [M :249-256] and the MP ammo
"%d x %s" format [D 0x30cc28]; (3) the take sound for EVERY taker — bot takes keep the
placement's own +0x2A word (the bot arm skips the override block [M :137]), remote humans the
human overrides [M :334-337]; (4) the respawn restore one tick after the 60·respawn·10 limit
[M 0x18e148:183-185] — take_pickup + consume_pickup + the unit test re-pinned. Verified:
nf-formats lib 92/0, nf-game lib 662/0/5i, new mp_pickup_take_live 2/0. KEY FINDING:
Sim::step_mp_pickups has NO caller in the current tree — the MP shared pass (bots/remote
takes) is dormant, MP bots cannot take pickups until it is wired into the MP tick driver.
Also dead: mp::unregister_pickup / reset_pickup_bot_visit_times (game unregisters on delete
[M 0x18e650:341] and expiry [M 0x18e148:257]). The walkreal/live run list did not fit the
window — enumerated in the report. sweep table: 25 rows verified present, no action.
