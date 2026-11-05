
## fx-gas-and-blasts
Started 2026-11-05 02:45 UTC. Branch cut from origin/master 17a784980. Task: port rows 5, 9, 10 of
audit-effect-calls's "Unported calls" table — the gas kinds 9/10 hidden cloud emitters (gas.rs
stub), the Apocalypse flashbang (Apocalypse_Update 0x18d4f0:35, world_fx.rs notes the consumer
unported), and the four MP-only blasts (GT_Create 0x17ffa0:157 chain, GT_Update 0x180498:435
self-destruct, MP_DemolitionProtectionUpdate 0x1870a0:144 script 0x060004f, Drone_Delete
0x136ce8:92 drone-owned-GT). Cite [M] per call, pin through the real path.
