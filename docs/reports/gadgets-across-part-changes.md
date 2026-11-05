# gadgets-across-part-changes — items dropped between mission parts

Branch `s/gadgets-across-part-changes`, cut from origin/master f68a72216, 2026-11-05 ~12:23-13:10 UTC.
A carry sweep found a few gadgets lost on three of the 14 part changes tried, while weapons and
ammo carried. This task: list which items drop on which change through the real level exit, read
the game's carry rule [M RamSave and the level-change path], fix anything the game keeps, and pin
the drops that match the game.

## The sweep

`crates/nf-view/tests/gadget_part_carry_live.rs` drives the 14 within-mission part changes
(the mission::LEVELS parts per mission: mission 1 has 3, mission 2 has 3, mission 3 has 2,
mission 4 has 1, mission 6 has 3, mission 7 has 2 — 14 total). Each hop is the real exit path:
cold-load the source, set the tracked gadget ids owned+clip on the live inventory (the fields the
game's pickup grants write [M Pickup_Create 0x18dda0 → the BLData entries], plus the P2K so a
RamLoad destination's unowned-P2K kit rebuild never fires [M 0x1a7ef8's p2k gates]), capture
`warm_exit_save` (= `Player_RamSave` 0x195670 — the capture the App's exit chain builds;
async_level_load_live pins its parity), land the destination with `load_with_carry`
(`Player_RamLoad` 0x1957a8 + `Player_InitWeapon` 0x1a7ef8's tail), and read owned/clip. The
grant covers 18 ids: the Scorpion 0x2e, frag 0x34, stun 0x35, smoke 0x36, remote mines 0x37-39,
tripbomb 0x3a, satchel 0x3b-0x40, camera 0x54, decryptor 0x56/0x57, Q-Worm 0x58.

## The game's carry rule: the drops are the game's own

1. **The save carries everything.** `Player_RamSave` 0x195670 copies the 0x42 spare-ammo words
   (+0x038) and all 114 12-byte weapon entries (+0x07C: clip s16 at +4, owned u8 at +6)
   [M 0x195670:6-8]; `Player_RamLoad` 0x1957a8 restores them (whole body gated on IsWarmReset)
   [M 0x1957a8]. Gadgets ride the save exactly like weapons and ammo — the save is not where
   they drop.
2. **The load wipes the object table.** `ResetMap_Load` 0x1d6360's LoadState-3 arm — the arm
   every level load runs — executes `Mem_Init` + `hashtable_initialise` (:130-134) and then the
   chain loads only the shared 07000500 pack and the level's own archive (:28-36). 07000500.bin
   holds no objects or skins [D script-census.md:80]. So the table the gate sees is boot +
   shared pack + THIS level's archive — not the accumulated session. The empirical proof is the
   already-measured warm-15 Grapple loss [L 2026-10-14, pinned in fix_start_loadouts_live]:
   07000005's own archive carries the Grapple's fp skin (its boot equip owns it
   [L ee_start_05 owned = {1, 2, 0x4A, 0x4E, 0x50, 0x54}]), yet the game's warm 15 drops it —
   impossible under the accumulated-session reading in the §2/g-021 comments.
3. **The gate strips what the table misses.** `Player_InitWeapon` 0x1a7ef8's common tail runs
   `Player_CheckWeaponsLoaded` 0x1a97c8: records with `weapon_data+4 == 0` are skipped entirely
   (the port's `selectable()`), `+0xDC == 0` clears owned, and a `hashtable_getitem(+0xDC)` miss
   clears owned. `Player_EquipWeapon` 0x1a8a58:20 refuses the same equips at kit time.
4. **07000007 is the party disguise**: its case runs `Player_InitAmmoWeapons` + the kit
   (P2K/taser/laser/watch/camera) EVEN on the warm load [M 0x1a7ef8 case 0x7000007] — the game
   wipes the carried gadgets at the party by design; only the kit's camera survives (its fp skin
   rides 07000007.bin [D]).

**So a carried gadget survives a part change iff its fp skin (+0xDC) is in the destination's own
archive** (or its record is not selectable). The sweep asserts port == game per hop for all 18
granted ids across all 14 hops and passes — **nothing to fix; every drop is the game's own**,
pinned with 0x1a97c8 + 0x1d6360:130-134 + 0x1a8a58:20.

Per-transition summary (kept = the fp skin is in the destination's own archive [D]; full tables
in the test's log):

- 01→02, 02→03, 03→04: keep frag/stun/mine-0x38 + the satchel twins 0x3c-0x40 (their fp skins
  ride 01-04's archives); drop the Scorpion, smoke, remote-mine 0x37, tripbomb, satchel-0x3b,
  camera, decryptor twins, Q-Worm (their fp skins live only in later archives).
- 05→06: the 04-and-below skins keep; frag/smoke/tripbomb/satchel/decryptor/Q-Worm drop.
- 06→07: the party wipe — everything but the kit's camera drops [M 0x1a7ef8 case 0x7000007].
- 07→08: the 01-07 skins keep (08's own archive holds them); frag/smoke/tripbomb/satchel/
  decryptor twins/Q-Worm drop.
- 09→0a: the 09-kit skins keep (decryptor twins, Q-Worm, mine 0x38/0x39, satchel twins); the
  Scorpion (fp only on 08+), stun (fp not in 0a), camera (fp not in 0a) drop.
- 0a→0b: 0b's own archive keeps the Scorpion too; stun/camera drop.
- 0c→0d: smoke (0c's own archive), mine, satchel twins, camera keep; frag/stun/0x37/tripbomb/
  decryptor/Q-Worm drop.
- 11→12: the 11/12 skins keep (Scorpion, frag, smoke, mine, satchel-0x3b); stun/0x37/tripbomb/
  camera/decryptor/Q-Worm drop.
- 12→4a: the ≤12 skins keep; stun/smoke/0x37/tripbomb/camera/decryptor/Q-Worm drop.
- 4a→13: 13's own archive adds the tripbomb and satchel-0x3b back; stun/0x37/camera/decryptor/
  Q-Worm drop.
- 14→15: the 14 kit's skins all present in 15 (frag/stun/smoke/mines/tripbomb/satchel/camera/
  decryptor keep); the Scorpion (fp only on 08/0b/12/13) and Q-Worm (fp only on 09/0a/0b) drop.
- 15→16: as 15 minus the decryptor (16's own archive misses its fp); the Scorpion and Q-Worm
  still drop.

The 0x39 record (fp hash = 0 [D weapon_data]) is kept everywhere — the strip's `+4` gate skips
it [M 0x1a97c8], matching the port.

## The fix attempt that was reverted

I first read the object table as session-accumulated (the §2/g-021 comments' reading) and
implemented a gate fix that merged the session's fp skins into the ownership set for warm loads.
The sweep (with the accumulated-session expectation) passed, but the reading is refuted by the
[L 2026-10-14] warm-15 Grapple measurement (above), and the merge form OOMed the multi-load
tests in this sandbox (fix_start_loadouts_live SIGKILL; master passes — stash-proof). Reverted;
lib.rs is byte-identical to master. The conflict between the two readings is recorded here: the
§2 weapon-MESH merges (render side) may show muzzle flashes the shipped game does not draw on
07000011-16 — the main agent may want a live check (below) before treating those as game-true.

## Verification

- `gadget_part_carry_live` (the sweep): 1/1, all 14 hops port == game.
- `fix_start_loadouts_live` 8/8 (each test individually; the full binary SIGKILLs at test 6 in
  this sandbox today — a borderline OOM on master's own code, it passed 8/8 earlier today).
- `qworm_not_on_0c_live` 3/0, `gadget_carry_live` 6/0, `level_carry_live` 8/0,
  `mission_stats_carry_live` 2/0, `loadout_disc_rows_live` 2/0.
- The final tree is master + the new test file only (lib.rs reverted). On this same master code
  this morning: nf-game full suite 1077 passed / 0 failed / 8 ignored, nf-formats+nf-audio green
  except the 3 expected disc_golden slim-ISO failures, and the 12 level walks 89/0
  (the level-12-engage-range session's runs on the identical tree).

## What's left

1. The render-side question: whether the shipped game really draws the Tower-2/airbase muzzle
   flashes (§2's premise) — the per-load table reading implies it does not. Live check below.
2. If the user's playtest still shows a gadget vanishing somewhere the game keeps it, the sweep
   table localizes the hop; the remaining suspects are the scripted grants (the scripts re-grant
   mission gadgets per part — a script-level census, not an inventory rule).

## Live checks for the main agent

1. Warm 14→15 in PCSX2: watch Bond's inventory owned bytes (BLData+0x1BA + id·0xC) for 0x50
   (Grapple) across the load. Expected per the per-load reading: owned 1 before the exit, 0
   after 15's load (the [L 2026-10-14] measurement, re-confirmed).
2. Warm 14→15: after the load, read the hashtable state — `hashtable_getitem(0x05000046)` (the
   Scorpion fp) and `hashtable_getitem` of the Grapple's fp. Expected: both miss (the table holds
   only boot+500+15's own), confirming `Mem_Init`+`hashtable_initialise` ran.
3. 07000011-13 armed guards firing: screenshot a guard's first shot. Expected per the per-load
   reading: NO muzzle flash mesh (0x02000261 is on no archive ≤ 13) — if the flash DOES draw,
   the accumulation reading is right for meshes and my §2 note is wrong.
4. Warm 15→16: the decryptor (0x56) owned byte across the load. Expected: 1 on 15 (14's archive
   carries its fp), 0 on 16 (16's own archive misses it) [M 0x1a97c8 + D the carrier scan].

## Usage

not visible to me
