# guard-castle-chat-gate-2 — the castle chat pair work rebased and finished

Task: start from origin/s/guard-castle-chat-gate, rebase onto origin/master keeping master's
versions of the three tests that branch had defined twice, then finish what that branch's report
left open (the chat pair still breaks off early; the ch 0x11 raiser census in the NIS event
layer). Required runs: nf-game vision_bit_census, nf-view fix_npcs_live, walkreal_07000006_live,
the full suite.

## What was done

1. **The rebase resolved.** The gate branch's base already equals master 08a276ce0, so the
   "merged onto master it does not compile" came from INSIDE that branch's file: its session had
   appended a second copy of `walk_the_castle_interior_to_the_party_real_damage_agent`,
   `..._real_damage_00agent` and `the_wellhead_seals_the_shaft_on_real_input` after the
   originals. The appended copies are removed; master's versions stand. The branch's own ignored
   chat test was replaced by the castle-chat-real-path green version (the 15 u watch spot).
2. **The chat pair no longer breaks off early — the test is GREEN and UN-IGNORED.** On this tree
   (the gate branch's per-tick sight-bit fix included) the pair holds CastleChatGuard1 0x36
   through the intro NIS and the walk legs: the watch spot is the greeting-plates leg's end
   (45.0, 38.3, 24.0), 15 u east of the pair, where their look-for-opponent never fires. The
   pins, all passing: both guards present in 0x36 [M 0x15ba68:26-30 + 0x14c600:147-158], chat
   channel 0x11 armed and up, the start scripts PLAYING (anim.script = 0x06000124/0x06000126
   [M 0x1612e0:57-65; D census 1244/1204 frames]), and after the burst both leave 0x36 with the
   UN-SHOT guard alive [M 0x1612e0:83-98]. The gate branch's t456/t875 red ledger came from its
   own test's close approach (its legs walked to 5.5 u and then 2.7 u from the pair); with the
   watch spot the break happens only at the burst, which is the pin.
3. **The ch 0x11 raiser census — CLOSED against the decompile, and the gate branch's reading was
   wrong.** Its report says #512 "fans out" (out = source 0x11, list = destinations, citing
   Trigger_MultiplexOut 0x1b1a78 case 5) and "no trigger raises 0x11". The decompile: the
   Trigger_Update 0x1b1a78 switch dispatches on the trigger's TYPE — case 5 is the copy-out, and
   it only runs for type-5 triggers; **#512's type is 0xc**, and case 0xc (decomp lines 167-180)
   ORs the u16 input list at +0x26 into `switch_channels[puVar3[8]]` [M]. So #512 (type 0xc,
   out 0x11, list [0x3a, 0x3b] [D census]) IS the raiser: area plate #528 (type 1, out 0x3b, x
   49.7-51.2 z 26.0-30.3 — the truck's west end) or plate #513 (type 1, out 0x3a, z 44.7-46.6 —
   the south approach) fires, and the list-OR raises the chat channel. The sim agrees: ch 0x3a
   never fires while 0x11 rises exactly when the walk crosses #528 — a fan-OUT would have lit
   0x3a too. The port's `pub const OR: u8 = 0xc` (triggers.rs:17) is correct.
4. **The NIS/event-layer sweep is CLEAN** (the gate branch's lever 2, done): 07000006's kind-7
   streams carry 16 op-0x12 event sites [D nf-extract nis-census]; the event-0x0d channel sets
   [M Script_EventHandler 0x1d94c0 case 0xd] write 0x40/0x3f/0x3d/0x3e (the reveal NIS
   0x06000066), 0x83 and 0x02 (the dressing cut-scene 0x06000071) — **no 0x11 write anywhere in
   the event layer**. The chat channel's only raiser on this level is the #512 list-OR.
5. **vision_bit_census green.** CASTLE_CHAT_GUARD1 and INTEROGATE_WALK moved from
   NOT_IMPLEMENTED to REQUIRED — both port ENTER arms already set `vision_on = true`
   [M 0x1612e0:21 case 1's `+0x4F8 |= 0x10000`; 0x162338:20 case 1's same] — exactly what the
   census's ghost check asked for.
6. **fix_npcs_live green, with one test corrected to the decompile.**
   `an_idle_gate_guard_sees_and_attacks_bond_on_the_same_floor` asserted placement 487 stands in
   plain IDLE — a pre-port read. The decompile: 487 is the chat pair's north guard, state 0x36
   from tick 1 via the mode-0x18 start-script special [M 0x14c600:147-158 + 0x15ba68:26-30;
   D census param 1 = 0x06000124]. The test now pins that state and runs the same see-alert-attack
   regression through the chat state's own EnemyLookForOpponent [M 0x1612e0:44-52 + 83-98];
   renamed `an_enabled_gate_guard_sees_and_attacks_bond_on_the_same_floor`.
7. **The d1 real-damage walk re-greened on this tree.** The gate branch's ledger (Bond died
   t3964 at (4.54, 42.03, −33.94), −54.0 from the y-47 balcony grenade guard) re-measured
   IDENTICAL here before the fix. The castle-chat-real-path lever is ported (file-local
   `pre_clear_balcony_guard` — any-height, exposed-only, called from the ramp's upper lane; a
   player clears the balcony guard before cresting). The clean A/B the gate report asked for:
   without the lever the walk dies at t3964 (both trees), with it the whole binary passes.

## How it was verified (the final tree, --test-threads=1)

- `nf-game --test vision_bit_census`: **1 passed, 0 failed**.
- `nf-view --test fix_npcs_live`: **4 passed, 0 failed**.
- `walkreal_07000006_live` whole: **7 passed, 0 failed, 2 ignored** (the chat test runs in the
  count; the d2/00 Agent ignores are master's pre-existing ledgers).
- `interactables_07000006_live` whole: **5 passed, 0 failed**.
- `nf-game --lib` (NF_ISO set): **791 passed, 0 failed, 4 ignored**.
- EVERY walkreal_*_live binary (the drone.rs delta is the merged port + the sight-bit fix, so
  the whole battery ran): 07000001 **7/0/1i**, 07000002 **3/0/2i**, 07000003 **8/0 across its
  tests** (the 142 s full route ran in a dedicated window), 07000004 **10/0**, 07000005 **17/0**,
  07000007 **6/0**, 07000008 **5/0/1i**, 07000009 **9/0**, 0700000a **13/0/2i**, 0700000b **9/0**,
  0700000c **4/0/1i**, 0700000c_barrelrow **1/0**, 0700000c_eastroute **1/0**, 0700000d **1/0/2i**,
  07000011 **4/0**, 07000011_route **1/0**, 07000012 **3/0/1i**, 07000013 **4/0**, 07000014 **7/0/3i**,
  07000015 **8/0/1i**, 07000016 **15/0/3i**, 0700001b **16/0/3i**, 07000041 **15/0**,
  07000041_wall **2/0**, 0700004a **4/0**, 0700004a_satchel **2/0** — plus npc_doors_live
  **6/0/1i** and fix_guard_misc_live **6/0**.
- `nf-audio`: 0/0 (its tests are env-gated); `nf-formats`: 11 passed, 3 failed — the 3 are the
  `disc_golden` set (bigf_header_fields, bigf_tables, directory_tree), the failures the rules
  allow on the slim ISO.

## What is left

- **The full-suite total line: not produced.** `tools/test/suite.py` dies between tool calls in
  this sandbox (started once, suite.out stayed 0 bytes — the same end four sessions recorded).
  The per-binary list above is the substitute and covers every walkreal binary.
- Nothing else from the gate branch's "what is left" list: the alert-source hunt resolved to the
  watch-spot geometry (the pair holds 0x36 until the burst on real input), the census is closed,
  the A/B is done.

## Live checks for the main agent

- Fresh boot, walk the dock west from the truck (cross x≈50 at z 26-30, plate #528): the two
  chat guards at (31.4, 39.5, 22.05)/(30.5, 39.3, 23.08) should be talking (their chat voice
  lines, the 0x06000124/0x06000126 scripts), and `switch_channels[0x11]` should read 1 after the
  plate. Screenshot spot: Bond at (45.0, 38.3, 24.0) facing west down the dock.
- Fire one pistol round near them: both should break off (the alert group fold), the un-shot
  one charging — and `switch_channels[0x1A]` should NOT latch (their start scripts played).

## Usage

not visible to me
