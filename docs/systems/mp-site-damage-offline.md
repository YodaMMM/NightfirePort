# mp-site-damage-offline — the Demolition/Protection site takes Bond's local bullets and explosions

GAPS G-053 item 1. In MP Demolition and Protection the site's HP (the `MPOBJECT` short at +0xF4,
init 2000 [M `MP_CreateObject` 0x185de8:174]) only moved through the port's online shot
(`step_net_combat`'s shared combat ray) — Bond's own local gun pipeline (the `--mp` offline load,
bot matches) never reached `damage_site`, and explosives did not either. Offline the site only
timed out.

Evidence tags: [M] the decompile (`work/decomp/ACTION.ELF/<addr>_*.c`, `:NN` = line) or the
disassembly; [D] disc bytes (the EE weapon table `work/ee_start_05.bin` @ 0x2bf150, 115 × 0x10C);
[L] a live PCSX2 measurement; [S] a port choice. Spec: `docs/systems/mp-match-rules.md` (the
scoring), `docs/fields/weapon_data.md` (the record map).

## The game's two paths onto one HP store

The site's `MPOBJECT` is a short array: `[0]` the type word (3 = demolition site, 8 = protection
site), `[6]` (byte +0xC) the site's SP `obj*`, `[0x28]` (byte +0x50) the last-hitter slot [M
`MP_DemolitionProtectionUpdate` 0x1870a0, `MP_ApplyDamage` 0x18b2c8]. `MP_DemolitionProtectionUpdate`
runs every frame in the `MP_ObjectUpdate` 0x18b198 walk and does, in order:

1. **The bullets' frame sum.** `SP_GetHitDamage(site_sp, HitByList, 0x40, &count, filter=1, 0)`
   [M 0x1870a0:81-83 → 0x1dc5f8] walks the site SP's script stream objects and sums each object's
   `obj+0xD0` HITDATA chain: `Collide_FilterBullets(list, 1)` first, then
   `Collide_GetDamageNObjects(list, HitObjList, &count, 0x40)` [M 0x1ee220] adds every record's
   +8 damage float. The store is the truncating chain
   `(short)(int)((float)(int)hp − sum)` [M :81-83]. The chain was filled by the frame's collision
   pass — one record per bullet contact, `Collide_Update` 0x1ea398 clears every chain at the head
   of the frame, so the sum is per-frame.
   - **The filter** [M `Collide_FilterBullets` 0x1ece30:24-31]: a contact whose hitter is a
     class-5 object whose held weapon's category word (`weapon_data+0x10`, u16) ANDs non-zero
     with the mask is REMOVED from the chain (marked damage 1e8 and sorted out). Mask 1 drops
     category bit 0. Disc census [D]: exactly the melee records carry bit 0 — weapons 1 (the
     fists), 75, 77 and 99–102 (0x63–0x66, the melee/taser specials); every gun is 0x0008, the
     laser 0x0002, rockets/grenades 0x0020 — all kept. Punching the site does nothing.
   - **The hitter** [M :84-91]: when the frame's sum is positive `MPSound_Play(0x14c)` plays and
     the last-hitter slot becomes the first hit object's player (`HitByList[0]` — the decompiler
     dropped the argument; `Control_Plr2Ind`); with no hit objects the old value stands.
2. **The threshold + outcome** [M :125-165]: HP < 1 — the attacking team (1 − the dispatch's
   param_3; 0 Demolition, 1 Protection) +1.0, the last hitter +1.0 personally or −1.0 when he is
   a defender (whose team also pays 1.0 — the own-goal replaces the hitter's +1), no recorded
   hitter (0xffff) scores no one; the site is done (MPOBJECT+0xF6 = 1) and the round enters state
   6 at once. The port's [`MpMatch::damage_site`] implements exactly this accounting
   (`crates/nf-game/src/mp/rules.rs`).

The **explosions** do not go through the chain: `Explode_Propagate` gathers the blast sphere's
objects (`Collide_SphereIntersect`), computes each object's falloff
`damage · (1 − trunc(max(|centre − pos| − obj+0x8C, 0)) / radius)` [M 0x17c518:82-85] and its
case 0x35 calls `MP_ApplyDamage(falloff, mpobj, owner)` DIRECTLY [M :187] — the same truncating
store, the last-hitter slot = the blast owner's player/bot index [M 0x18b2c8:20-25]. A bomb
(weapon 0x3b, the demolition/protection bonus weapon [M 0x129e48:50-56]) carries damage 2000.0
[D] — one blast at the centre is the whole HP bar.

## The port's stand-ins

The port's site has no SP script player (the live load registers the placement's model bounds —
`MpMatch::register_mp_object_bounds` — and `MpMatch::site_sphere` exposes the contact sphere).
The game's two paths map onto it like this:

- **Bullets** (offline, the local pipeline): a player bullet whose flight segment crosses the
  site sphere adds its damage to a per-tick accumulator, with the game's category filter
  (`weapon_data+0x10 & 1` contacts skipped). The driver sums the tick's contacts into ONE
  `damage_site(sum, local slot)` call — the game's per-frame chain read [M :81-91]. The
  sphere-vs-segment test and the "no world-occlusion gate" are the same shape the G-047 SP
  plumbing uses (`Scripts::held_hit_targets` + the bullet pass in `drone.rs`) [S stand-in].
- **Explosions** (offline): each local blast (`Drones::spawn_blast` — grenades, rockets' contact
  blast, mines) adds `blast_falloff(blast, site sphere)` as its own entry, applied per blast —
  the game's case 0x35 [M :187]. `blast_falloff` is the shared exact form
  (`mp/combat.rs`, the online path already uses it for the site).
- **The drain order is the game's frame order**: a frame's explosion stores land before its
  bullet-sum store (the blast fires mid-frame, the MP update reads the chain at the frame's MP
  point) [M the call sites; the port drains the blasts then the bullet sum right after the local
  world tick, before the next tick's `update_objects_live`]. The two stores' truncation is order
  sensitive in principle (`trunc(hp−a−b)` stepwise); the game's order is kept.
- **Online play stays out**: online the shared combat owns the site (the round ray
  `combat.rs:425-431` and the blast sweep :584-587 → `Hurt::Site` → `Sim::mp_hurt` →
  `damage_site`). The local pipeline still flies its visual bullets there, so the offline feed is
  gated on no session (the driver sets the site sphere only when `net` is none) — otherwise every
  online shot would apply twice [S port architecture].
- **Bots' shots** already damage the site the same way (`step_mp_bots`' site round, the
  mp-bot-objectives block) — not this task's change.
- **The hitters**: Bond's local bullets are the local slot [S — the port's local pipeline fires
  no other rounds]; a blast's owner key `usize::MAX` (the established player-throw key) maps to
  the local slot, a drone-index key has no MP slot (the MP bots are not drones) and is skipped
  [S].

`damage_site` itself is untouched: the threshold, the scoring, the own-goal, the state-6 latch
and the 5 s restart all stand as merged (mp-scenarios-online).

## Live checks for the main agent

- `gdbwatch` the site MPOBJECT+0xF4 (HP, i16) while a guard shoots the site in a bot match: the
  value should drop by the weapon's +0x0C per contact tick (the port's bots already do this; the
  game's own bot matches should agree — the `SP_GetHitDamage` filter keeps the bots' guns).
- `fieldprobe` the site SP obj's stream object's +0xD0 chain after one bullet hit: one record,
  +8 = the weapon's damage word, and after the frame's `MP_DemolitionProtectionUpdate` the chain
  is cleared (the next frame's `Collide_Update` head).
- Whether a rocket that dies ON the site leaves BOTH the bullet contact (100) and the blast
  falloff on the frame's books (the port mirrors that; the decompile says the contact registers
  before the handler kills the bullet — one probe shot distinguishes).
