# port-map-player — the player-family map

Branch `s/port-map-player` cut from `origin/master` (22d5836b7). The task: the drone-family
sweep's method applied to every `Player_*` function — one line per decompile function,
ported with the port location or not reached with the reason, to
`docs/port-map/player.md`; any function a real level reaches with no port code is a gap to
fix with a real-level test proved red first, one commit per fix.

## What's done

1. **The map** (`docs/port-map/player.md`): all **90** functions whose decompile symbol
   begins `Player` (the 85 `Player_*` + the 5 `PlayerAnim*` anim-set functions of the
   family). Method as in drones.md: the exact hex address searched in `crates/*/src`, then
   the base name; a citation counts only when attached to the implementing function, and
   every row was verified against its `.c`. Verdicts: **86 ported** (one partial —
   `Player_HandleHasNoAmmo`'s dry-fallback arm; one state-half — `Player_GrappleSetRope`'s
   rope object), **2 absorbed** (`Player_ResetWalkAnim` — its only write, BL+0x954, has no
   reader anywhere in the decompile [M]; `Player_ClearAimInertia` — the port stores no aim
   deltas to clear, GAPS.md G-006), **1 not reached** (`Player_SSScanMode` — dispatch case 5
   is entered only by the debug-fly cheat, `C_CHCHFLY_Handler` 0x215c70). The 17
   `*Player*`-substring symbols of other families (Car_/GT_/MP_/SP_/Trigger_/Camera_/Effect_
   /P_/Menu_/PlrStat_) are recorded as out of family. Seven rows are listed by name only —
   their hex contains a function-key character pair and the push guard trips on it (their
   addresses sit in the `.c` filenames).
2. **The one MISSING the sweep found, fixed**: `Player_UnEquipKey` 0x1a97b8 — the single-use
   key ring. `Player_EquipKey` 0x1a97a0 was ported (the pickup side), but the mirror clear
   had no code AND neither half of the keypad law was wired: `Lock_Unlock` 0x1b0650's
   state-5 arm equips the lock's key at the solve [M]; `Lock_Update` 0x1ae3d8's case 1 holds
   the out channel and its framelist tail leaves the lock in state 2 [M :193-220]; from
   state 2, case 2 re-arms a kind-0 lock when the out channel drops, through `Lock_ReLock`
   0x1b0750 — state 0 + `Player_UnEquipKey` [M :5-10]. The port solved keypads into a
   permanent `unlocked` with no equip and no re-lock.
3. **The fix** (one commit): `Lock::open_done` (the state-2 phase flag) +
   `Lock::relock_if_dropped` (the case-2 arm) in `crates/nf-game/src/lock.rs`; the sim's
   top-of-tick lock pass in `crates/nf-view/src/scene.rs` wires the solve's
   `Player_EquipKey` half (inventory.keys[code] = true), case 1's channel hold + phase flip,
   and the re-lock arm clearing the ring slot; the four `Lock` literals got the new field.
4. **The second MISSING, not fixed here**: `Player_GrappleSetRope` — reached on every SP
   grapple grab (sole game caller `Player_CollisionHandler` 0x1949a8:203), no port code for
   the rope-object half (the point object's +0xF0/+0xFA flags and the transform copy). It is
   purely the visible rope mesh: the port has no rope object at all (gadget.rs:435-437,
   bldata.md:56/59 record the render-side gap), and a visual fix cannot be proved in this
   sandbox (lesson 6) — it is documented in the map with exact live checks for the main
   agent (the `Control_CreateObjEx` hashtable, the flags, the transform source), instead of
   an unverifiable render patch.

## How it was verified

- **Red first**: `decryptor_07000012_live::the_solved_keypad_equips_its_key_and_a_dropped_
  channel_relocks_the_lock` drives 07000012's real load → the Decryptor equip through the
  switch request → the real aim/fire scan hold → the solve; on the pre-fix tree it fails at
  `the solve must equip the lock's key (ring slot 5)`; the drop half (channels.set(10, 0) —
  the game's door-close write, Door_SetState 0x135618's mirror [M]) then asserts the re-arm
  and the ring clear.
- **Green after**: the same binary 2/2, and every lock-adjacent binary re-run whole:
  `decryptor_hold_real_path_live` 3/3, `keypad_unlock_lod_live` 1/1, `gadget_laser_live`
  8/8, the `nf-game` lib tests 686/686 (5 ignored — the lock.rs burnt-open pins and the
  inventory key tests among them). The pre-existing solve-path test in
  `decryptor_07000012_live` passes unchanged (the wiring does not disturb the scan law).
- **Not run**: the full four-crate suite (the window went to the sweep + the fix; the
  changed surfaces are lock.rs/gadget.rs/scene.rs's lock pass, and every test binary that
  touches locks or the decryptor ran whole — the list above). The walk lanes were not
  touched by the change; `crates/nf-view/tests/common/` is untouched.

## What's left

1. `Player_GrappleSetRope`'s rope object — render-side, needs the main agent's render loop
   (the live checks are in the map).
2. The named partial arms inside ported rows, if the driver wants them closed:
   `Player_HandleHasNoAmmo`'s twin/GetBestWeapon dry fallback, `Player_InitAmmoWeapons`'s
   ammo-cheat arm, `Player_MonitorAir`'s HUD air pane, `Player_Enable`'s spherical-velocity
   handoff, `Player_InWater`'s MP skip.
3. The shipped 07000012 lock records' framelist presence is unverified [S in the map] — a
   live check on the open animation would settle whether the port's one-tick state-2 phase
   diverges from the game's animated one.

## Usage

not visible to me

## ntfy failed

Fallback 1 `curl -sS -m 20 -H "Title: NightfirePort" -d "port-map-player: …" https://ntfy.sh/nightfire-yoda-8k3x`
→ `curl: (28) Connection timed out after 20002 milliseconds`, exit 28.
Fallback 2 python urllib → `URLError: <urlopen error [Errno 101] Network is unreachable>`.
Fallback 3 `wget -q -O- --timeout=20 --post-data "port-map-player: …" https://ntfy.sh/nightfire-yoda-8k3x`
→ exit 4 (network failure), no body. All three failed this window (the same shape the
stream-batch-6/7/8 windows recorded).
