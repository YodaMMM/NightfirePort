
### level-07-party-door-lock (started 02:41 UTC, 2026-10-25)

Task: in the game, after certain points on the party level (07000007) the door back to the main
party room locks; in the port it stays usable. Find the party-room doors [D door placements +
lock/key channel params], what locks them [M switch channels the scripts/triggers/mission
records write at those story points, Door_* lock handling], list the story points that lock (and
any that unlock) with their channels; port any missing writer or lock handling with real-input
tests (before: opens; after: refuses with the game's message/sound; sister: other doors
unaffected).

Plan: branch from origin/master c64b2b80; census the 07 doors; find the lock channels in the
level scripts + Door_* lock handling in the decompile; reproduce through the real input path;
port what's missing; suite; report.

### level-07-party-door-lock (started 02:41 UTC, 2026-10-25)

- (02:41-02:50) Branch from origin/master c64b2b80, WORKLOG started, fast suite daemon running.
- (02:50-03:10) Read the door decompiles: Door_Update 0x1363c8's lock block is the UNLOCK-event
  watcher (lock channel rising edge → the 0x2000003 "Door Unlocked" message + auto-open + the
  field cleared, one-shot); the LOCKED refusal is Door_Activate 0x135be0 (lock channel wired +
  channel off + door closed/closing → the hit sound, return 0). The locked door gives no text
  hint at all (Door_DoMsg 0x135f18's lock gate). docs/fields/door.md's param map (p3 → ext+0x68
  lock) settles the older U-044/switch-channel-writers "param3 beyond the channel fields" note —
  the 07000005 [L] ext reads (200 in +0x68) are decisive.
- (03:00-03:15) census_07000007 (new): 15 doors (7 free p3=0; 8 ch-200), all triggers/SPs decoded.
  The level's ONLY script channel write: pack script 0x06000073 (SP #420, the control-taking
  cut-scene from 07800007.bin, fired by the hall walk-in TouchOnce #412) stream 8, event 0x0D
  [0x82, 0] — ch 130 ← 0 at script tick 4326 (the script's length tick).
- (03:15-03:30) THE MECHANISM: door #251 "Door_Wooden_8" (85.55,43,−35.12) starts OPEN (flags
  0x25: starts-open + keep-open + spline), open channel 130, lock channel 200. The story point
  drops ch 130 → the open-channel fall closes the door (Door_Update's channel arm; Door_SetState's
  mirror bounces ch 130 to 1 until the leaf arrives CLOSED) → then every E press is refused with
  the hit sound (ch 200 off — nothing on the level writes it). No unlock point exists on 07.
- (03:30-03:50) probe_07_route: the plate #412's box IS the hall→north passage (the plate-free
  skirt walks off a ledge and falls) — the story point is unavoidable on the real route; the
  door's pre-story state is pinned standing open through the walk.
- (03:50-03:55) party_door_lock_07000007_live (new, both green): the_party_corridor_door_locks_
  at_the_control_scene (boot open → the real walk → the plate → the scene → ch 130 falls → #251
  arrives CLOSED → 3 E presses refused) + sister_the_free_doors_still_open_after_the_lockout
  (the free hall door #244 still toggles after the story point — a locked door would freeze).
  The port already carried the whole chain (the driver's "stays usable" symptom does not
  reproduce on master c64b2b80 — likely predates the pack-script loading); the deliverable is
  the pins + the doc. No product code changed.
- (03:55) Level doc §4/§8 rewritten (15 doors, not 18; the #251 story lock; the "inert lock"
  note disproven by the pack-script writer). Checkpoint pushed.
