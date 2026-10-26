# level-0a-audit — report

Branch `s/level-0a-audit` (from origin/master 30598272). Deliverable: `docs/levels/0700000a-audit.md`
— the per-placement behaviour audit of 0700000a's main map (800 placements [D]), every script
stream (13 kind-7 entries) and every mission record, one row each, game behaviour [M/D] vs the
port's, GAP column included.

## What's done

- **The game side, per kind**: the `parsemap_create_dynamic_objects` 0x1d04f0 dispatch read from the
  decompile for all 36 kinds the level carries; each creator's class byte and per-tick update
  traced through `control_funcs` @ 0x26f890 (every row verified from the raw ELF bytes — the table
  is pure .data the decompiler only shows as indirect calls); use/damage handlers named
  (Trigger_Activate/Hit, Door_Activate/ReactToHit, Switch_Activate, Lock_Activate/Unlock/ReLock,
  SS_Activate, PCQWorm_Activate, Monitor_Activate, GT_Activate/ApplyDamage, Pickup_Handler,
  Sensor_ApplyDamage, Break_ApplyDamage, Hurt_GetDamage/Type). The static kinds (0x19 ×474,
  0x47 ×30) never reach the dispatcher (0x1d1b68:93's `(flags & 0xa000) == 0` gate) — world cels
  from `parsemap_block_map_data_static` 0x1d1998.
- **The disc side, per placement**: `nf-extract placements-all / triggers / nis / script-info /
  level-summary` over `0700000a.bin` — every dynamic placement enumerated (index, name, position,
  params), the 62-record trigger dump digested, all 13 script streams decoded with their key ops
  (no camera ops, no player disable/enable ops anywhere; every stream ends with the hand-back op),
  the mission records with their channel writers cross-checked.
- **The port side, per kind**: created/updated/drawn/usable/reacts/sounds with file:line (§3 of the
  audit doc), the per-tick order read off `Drones::tick_inner` (drone.rs:15168).
- **The calltree sweep** (`tools/calltree/calltree.py`, the 08 audit's method) over the 20
  update/control entries of the level's classes: **TOTAL distinct never-mentioned reachable: 0** —
  the strongest single result: no reachable game code behind this level's classes is unmentioned
  in the port.
- **Corrections written back** to `docs/levels/0700000a.md` with addresses (the audit's §2): the GT
  row was stale (gt.rs exists — created drone.rs:12578-12636, updated 15273-15392, census pinned);
  "no fuseboxes (0x31) / grapple points (0x3A)" was wrong (13× 0x31 — `FuseBox_Create` 0x1b0448
  shells whose SP hash 0 becomes the dummy 0x2d6200 that never loads [M 0x1dc7e0:69-71], inert
  either way; 1× 0x3A ported); the MusicTrigger row was stale (created + ticked into the music
  event board, worldobj.rs:414-420/466-474).
- **Verify**: no code change was needed — every behavioural row matches on the port side. The
  three gaps below are each bigger than a direct fix, per the task's own rule.

## The gaps (each with its section in the audit doc, §5)

1. **GAP-1 (large, code): the keypad E-path.** `Lock_Activate` 0x1aed00 → state 3 (Lock_Update
   0x1ae3d8:236-258: KeyCodes copy, MenuManager_Create action 0x40000021, the 0x2000014 keycode
   hint) → state 4 (:261-296: digit match → `Lock_Unlock` 0x1b0650 → the unlock channel) → doors
   #724 (ch 48) / #725 (ch 49) [D — the 0x2e locks' p5 pair]. The port has the lock object, burn,
   re-lock and decryptor scan but no E activate and no keypad menu: on a real pass the two keypad
   doors are decryptor-only. Needs its own task (a menu UI + the state-3/4 machine).
2. **GAP-2 (medium, code): the gold-medal kind-6 create gate.** Pickup_Create 0x18dda0:92-96
   refuses kind-6 without `PlrStats_HasGoldMedal` 0x199a40 (= Menu_HasMedal(BaseMapHCode, 3, 0)):
   0a's three "Pickup 007" (#726-728) would not exist on a fresh boot. The port creates them
   (pickup.rs:166) behind the documented D-004 take-path skip. Not fixed here: three live tests
   assert BOND_BONUS exists/takes (interactables_07000011:283, fix_progress:88,
   mp_pickups_visible:102) and an MP arena's kind-6 would read a mission medal MP does not define
   — the medal store + its MP rule are their own task.
3. **GAP-3 (draw-only): the hologram rotor's spin is not posed onto the drawn mesh** (#93
   `office_N_hologram`, object idx 35, by-index hash ffffffff [D]; sim angle ticks at
   worldobj.rs:452, the drawn matrix stays the static placement matrix, scene.rs MISC pass
   8043/8122). Plus the five tracked draw-census rows (placement-draw-census.md:119-125) named in
   the audit's §5.

## What's left / open

- GAP-1/2/3 as follow-up tasks (the [M] chains are written down).
- The GT scripts 0x06000071/72 are not in the archive [D — likely cached-model refs, open].
- The six not-placement-referenced ambient script chains (0x06000027/41/42/4F/52/7C4 — the atrium
  rain-leak/drip show among them): who starts them on a real boot (§6 live check 4).
- ch 0x37's writer on 0a (carried from 07000009 [S], unchanged); ch 0x3c confirmed writerless
  (already answered in the level doc).

## Live checks for the main agent

1. Keypad flow (GAP-1): fresh-boot 0700000a, E at `Door_Guardroom`'s keypad (6.8, 1.3, 0.3):
   `Lock_Activate` 0x1aed00 read-hit; the lock's +0xF4 → 3 → 4; MenuManager_Create 0x40000021;
   `switch_channels[49]` (0x26fc90 + 49) rises after the code; #725 opens.
2. Medal gate (GAP-2): fresh boot (no medals): the three "Pickup 007" must NOT exist at
   (9.4, 0.3, 32.0)/(34.8, 0.3, 34.4)/(−38.8, 0.3, −3.3) — probe around Pickup_Create 0x18dda0:92-96
   (the PlrStats_HasGoldMedal 0x199a40 call) and the pickup list count.
3. Rotor (GAP-3): render `--bond` at the security room looking at office_N_hologram (−0.3, 1.8,
   2.9) at t = 0 and t ≈ 5 s — the hologram yaw should differ (worldobj.rs's angle).
4. Ambient chains: NF_SCRIPTLOG on a real walk — do 0x06000052 (the atrium drip show) and its
   companions play on boot without any placement referencing them?

## What I ran

- The fast runner (`tools/test/suite.py`) was started detached twice; the first (03:18) died
  silently between tool calls (suite.out 0 bytes, process gone — the sandbox reaps background
  jobs), the restart (03:58, setsid) was still running at report time; its total line is appended
  below when it finished.
- No code changed on this branch (docs only), so the touched-crates rule has no test surface; the
  sweep + suite cover the tree. `nf-extract` was rebuilt (release) for the disc dumps.

## Usage

not visible to me
