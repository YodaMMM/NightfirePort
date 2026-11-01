
## level-11-start-corridor-guard — 2026-11-01

Task: on a fresh 07000011 load with Bond standing at the start, the port has a red-suited guard
walk into the corridor beyond the unlocked door from about tick 200, hit Bond by tick 509 and kill
him by tick 3172. On the PS2 the same corridor stays empty over the same frames [L PCSX2 capture].
Find which drone this is (placement, group, wake channel, route), why it wakes or walks in the port
[M the wake/ENTER gates, Drone_Create 0x136be0's difficulty gate, the patrol/route class], fix the
gap with the address cited, and pin it: an idle Bond at the start is not approached in the first
3200 ticks (sister: the guard IS present in drones.list). Run every *07000011* binary and nf-game;
quote the totals.
