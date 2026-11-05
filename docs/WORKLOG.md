
## fx-gas-and-blasts
Started 2026-11-05 02:45 UTC. Branch cut from origin/master 17a784980. Task: port rows 5, 9, 10 of
audit-effect-calls's "Unported calls" table — the gas kinds 9/10 hidden cloud emitters (gas.rs
stub), the Apocalypse flashbang (Apocalypse_Update 0x18d4f0:35, world_fx.rs notes the consumer
unported), and the four MP-only blasts (GT_Create 0x17ffa0:157 chain, GT_Update 0x180498:435
self-destruct, MP_DemolitionProtectionUpdate 0x1870a0:144 script 0x060004f, Drone_Delete
0x136ce8:92 drone-owned-GT). Cite [M] per call, pin through the real path.
Progress 2026-11-05 04:03 UTC: rows 5/9/10 ported. Row 5: Gas kinds 9/10 — the hidden bit
(+0xf0 |= 0x10 [M Gas_Init 0x1aacc8 LAB_001ab1e4]), kind 10's life halved at creation [M
pfVar1[0xe] *= 0.5], the child-spawn record (Gas_Init's own args stored at the data table [M
:46-52]) as Gas::child, and gases_smoke's cases 9/10 [M 0x1ab2d0:146-186] — the 20-tick creation
warmup then kind-8 children on GameState&3==3 (vel (0,REC,0) [M 0x1aacc8's kind-8 arm]), and
kind-7 children on GameState&7==7 with the given (0,2·REC,0) through the generic ·REC store;
GasPool::update takes the GameState frame now; the nf-view gas draw skips hidden puffs. No game
caller in the port's scope creates kind-9/10 parents (the audit's census) — the mechanics land
for the callers that do. Row 9: Apocalypse_Update's flashbang arm [M 0x18d4f0:33-40] —
Player_SetFlashBang(remaining, glb_blokes, min(remaining·382/total, 255)) EVERY armed tick
[0x196040: BL+0x963 = the byte, BL+0x918 = BL+0x91C = the duration] — WorldFx::apocalypse_flash
(remaining, alpha) applied to health.flashbang/flash_total/flash_alpha in Drones::tick right
after fx.tick (the stun-grenade path is the other writer of the same state). The whiteout arm
(viewer+0x200 = remaining·5/total) was already carried as ApocalypseFx.fade; the camera-shake
consumption stays the main agent's render work (audit row 1). Row 10 (the chain + the
self-destruct per the task text): GT_Create's same-owner chain [M 0x17ffa0:150-166] —
GTurret::create_chain at both creation sites (the placement walk + GT_DeployMiniGun's deploy),
each blown sibling = delete_flag + the (5.0, 50.0, 0x6000052) blast; the MP wreck self-destruct's
Exploded event now lands its Explode_Create [M 0x180498:435-441] in pending_blasts (the emit side
was already gt.rs's DESTROYED arm). The demolition site [M 0x1870a0:144 — 5/5/500, script
0x600004f, at obj+0x80, with the MPGame round-end flow] and the Drone_Delete owned-GT site [M
0x136ce8:92] are reported, not ported — the port has no MP demolition mode flow and the task text
scopes rows 10 to the chain + self-destruct. Pins: gas.rs kind9/kind10 emitter tests, world_fx.rs
apocalypse_flashbang_carries_the_per_tick_write, gt.rs create_chain_blows_the_same_owner_turrets.
