//! The world presentation FX the sim previously skipped — the placement object types from
//! docs/systems/levels.md §7 that only drew (or did not exist) before: 0x21 Ripples, 0xe3 Flicker,
//! 0xd4 Cloud drift, 0x4a Apocalypse and 0xf2 Emitter (particles).
//!
//! Game model ([M] = decompile, function + address):
//! - **Ripples** (`Ripples_Create` 0x1acc90 / `Ripples_Update` 0x1ac028): data+0 = param 0 (the s16
//!   x/z spread), data+2 = param 1 (the s16 period); both 0 → spread 1, period 0x1e [M 0x1acc90:20-30].
//!   Every tick: `Rand_Rand(period) == 0` → spawn one `Gas_Init(start, start·2, cel, pos+jitter, …,
//!   mesh 0x02000129, …, rgba 0x7f7f7fff, …)` puff: two `Rand_FRand(2·spread) − spread` draws move the
//!   x/z, one `Rand_FRand(0.05) + 0.05` sets the start scale [M 0x1ac028:34-52].
//! - **Flicker** (`Flicker_Create` 0x1acb00 / `Flicker_Update` 0x1ab960): the tint bytes obj+0x103..105
//!   fade 10 % per tick (min ±1) toward 255,255,255 while the found map light is active and toward 0
//!   when not; `Light_FindMapLight` 0x1cc3d0 is an **8-byte stub returning 0** on this build [M], so
//!   the game always takes the white path and stamps the signal channel (param 1) 0 every tick. The
//!   spark branch (gas puff + `Particles_Create` when the light is off) is unreachable behind the same
//!   stub, and `Particles_Update` 0x1cf528 self-kills through the stubbed `psiParticleUpdate` 0x111290
//!   anyway [M] — the port keeps the white fade + the channel stamp and skips the dead spark path.
//! - **Cloud** (`Cloud_Create` 0x18d620 / `cloud_update` 0x18d768): data = [param1·0.001, param2·0.001,
//!   phase_x, phase_z, param0 (height), param3 (wrap)]. Per tick the phases wrap by fmod and the object
//!   parks at `player + (phase_x, height, phase_z)` — a sky-slot object that drifts with the player
//!   [M 0x18d768:20-31]. No RNG, no channels.
//! - **Apocalypse** (`Apocalypse_Create` 0x18dcc0 / `Apocalypse_Update` 0x18d4f0): data[0] = data[1] =
//!   param1·60 (the remaining/total countdown in ticks), +8 u16 = param 0 (the latch channel), +0xa =
//!   param 2 (rumble), +0xc byte = param 3 (whiteout), +0xd byte = param 4 (flashbang), +0xe byte =
//!   the armed latch. Once `switch_channels[param0]` the latch sets for good; the countdown then runs,
//!   the viewer's whiteout value `glb_viewer+0x200 = remaining·5/total` rises, the rumble/flashbang
//!   params feed their (unported) systems, and at 0 the object self-destructs [M 0x18d4f0].
//! - **Emitter** (`Emitter_CreatePlist` 0x17b848 → `Emitter_Create` 0x17ae00 / `Emitter_Update`
//!   0x17b080 / `Emitter_Draw` 0x17b708): param 0 = the def hash (block 0x30, `Emitter_LoadDefs`
//!   0x17ab40 — `nf_formats::level::EmitterDef`), param 1 = the rate (0 → −1 = emit until disabled,
//!   1 → 0 = one-shot, n → n/60 seconds of emitting), params 2/3/4 = the on/off/out channels (out is
//!   driven 1 while any particle lives), param 5 = the blink period in ticks (ON half of it). Slots
//!   age at `age += dt`, `pos += vel·dt`, `vel.y += dt·gravity·−9.8`; the colour is `ColourGetValue`
//!   0x17a938 over `age/life`; dead slots respawn (batch-capped) while enabled, one-shots only fire
//!   the never-spawned slots once `expired` [M 0x17b080]. Death: 3 all-dead ticks set obj+0xFE bit 0.
//!
//! RNG discipline: the game draws from the shared stream (ripples 1 gate + 3 [M Ripples_Update
//! 0x1ac028:34-52], emitters 4 per spawn [M Emitter_Update 0x17b080:216-247], the flicker spark's
//! `Rand_Rand(5)` [M — behind the stubbed `Light_FindMapLight`, unreachable], leafgens 9 per spawn
//! [M Leaf_Create 0x1abc88:31-79 — 1 discarded 0.02 draw, 2 eject draws, 3 position draws, 3 spin
//! draws], lightning 21 per reflicker [M FractalThing 0x1a94d8:78-87 — 7 mid nodes × 3 axes]) — the
//! port consumes NONE of them (the bit-exact stream must not gain draws — the G-033/fix-world-fx
//! discipline, docs/atlas/emitter.md and docs/models/misc.md) and derives each value from a
//! per-object counter hash instead. That is a deliberate, documented DEVIATION, not a guess about
//! the game: the game's draws are cited per site below; the replacement is deterministic per
//! object and draws from the same uniform bands, so every visible quantity lands in the game's
//! value range with different samples.

use crate::triggers::Channels;
use crate::world::V3;
use nf_formats::level::{EmitterDef, Level};

const REC_FRAME_RATE: f32 = 1.0 / 60.0;

/// The ripples' spawn mesh (`Gas_Init` arg 7 in `Ripples_Update` 0x1ac028:50; the 0x0200xxxx mesh
/// namespace, same as gas.rs's SMOKE_QUAD 0x02000039).
pub const RIPPLE_MESH: u32 = 0x0200_0129;

/// The counter-hash the RNG-discipline note describes (the same bit-mix gas.rs uses).
fn jitter(counter: u32, salt: u32) -> f32 {
    let mut h = counter ^ salt;
    h = h.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h & 0xffff) as f32 / 65536.0
}

/// One 0xe3 placement: the tint ramp and the (always-0) signal channel.
#[derive(Clone, Debug, PartialEq)]
pub struct FlickerFx {
    pub placement: usize,
    pub pos: V3,
    /// param 1: the channel `Flicker_Update` stamps 0 (1 only on the unreachable light-off path).
    pub signal_channel: u16,
    /// obj+0x103..105, started black (the object memory zeroes) and fading white.
    pub rgb: [u8; 3],
}

/// One 0x21 placement.
#[derive(Clone, Debug, PartialEq)]
pub struct RippleFx {
    pub placement: usize,
    pub pos: V3,
    /// param 0 (s16): the x/z jitter half-spread.
    pub spread: f32,
    /// param 1 (s16): the spawn period in ticks (0 with a non-0 spread keeps it).
    pub period: i32,
    counter: u32,
}

/// One 0x2b placement (`LeafGen_Create` 0x1acbd0 — a class-0x31 object linked to room 0;
/// `LeafGen_Update` 0x1acc48: every tick `Rand_Rand(FRAME_RATE_INT >> 1) == 0` → one
/// `Leaf_Create(gen, 1)`). The port runs the whole chain: the gate on the counter-hash (the RNG
/// discipline — the game's leaf draws come out of the shared stream [M Rand_Rand 0x1e3738 =
/// SEED_X/SEED_Y]) and the leaf's own integrator — `LeafFx` below, `Leaf_Update` 0x1abee0
/// verbatim; spawned leaves live in `WorldFx::leaves` and tick every frame after the gate.
#[derive(Clone, Debug, PartialEq)]
pub struct LeafGenFx {
    pub placement: usize,
    pub pos: V3,
    counter: u32,
    /// Ticks since load (a spawn count the tests and the draw code read).
    pub spawned: u32,
    /// The last spawn's position offset — `Rand_FRand(2·spread) − spread` per axis [M
    /// Leaf_Create 0x1abc88:47-55 draws x, z, y in that order]. The spread is the generator's
    /// obj+0x8c, which reads 0.0: `control_create_object` 0x1344e0:17 memsets the object and
    /// `LeafGen_Create` 0x1acbd0:12-18 writes only +0xd8/+0xf0/+0xff — so the game's own jitter
    /// is 0 and leaves spawn AT the generator (the draw still advances the stream).
    pub last_offset: [f32; 2],
    /// The generator's euler yaw (obj+0x54 = the placement's raw ry: parsemap copies the
    /// level_tag rotation words verbatim into obj+0x50.. [M 0x1d04f0:50-53; Leaf_Create reads
    /// +0x54 as the Sinf2 angle]).
    pub yaw: f32,
    /// The generator's obj+0x8c spread — 0.0 on the traced path (the memset above).
    pub spread: f32,
}

/// One falling leaf particle — `Leaf_Update` 0x1abee0 (the integrator `WorldFx::leaves` drives;
/// spawned by `Leaf_Create` 0x1abc88). The game's object carries its position
/// (+0x30/+0x34), rotation (+0x50), the generator-shared velocity (+0x10) and spin (+0x20)
/// vectors, two life limits (instance s16+0x30 counts down 1 per tick; obj u8+0x106 is a tick
/// budget spent at `instance u8+0x32` per tick) and the delete bit (obj+0xFE bit 0) [M 0x1abee0].
///
/// Per tick [M 0x1abee0:15-51] (the y wind draws `Rand_FRand(0.075)` = 0x3d99999a, the x/z drifts
/// `Rand_FRand(0.005)` = 0x3ba3d70a; FRAME_RATE_MUL is 1.0 at the fixed 60 Hz [M GS_SetRefreshRate
/// 0x17d2b8:15]):
/// - `frames_left < 1` → delete; else `frames_left -= 1` (floored at 0 by the caller's `< 1` test);
/// - `budget` (u8) − `tick_cost` ≤ 0 → delete; else `budget -= tick_cost`;
/// - `pos += vel`, then `pos.y -= (Rand_FRand(0.075) − 0.025)` (mostly falling);
/// - `vel.x += (Rand_FRand(0.005) − 0.0025)` and `vel.z += (Rand_FRand(0.005) − 0.0025)`;
/// - `rot += spin`; the game also sets flags2 bit 0x20 (moved).
/// Returns `false` when the leaf dies (the game sets the delete bit).
#[derive(Clone, Debug, PartialEq)]
pub struct LeafFx {
    /// obj+0x30: the position.
    pub pos: V3,
    /// obj+0x50: the rotation (the spin vector adds every tick).
    pub rot: V3,
    /// instance+0x10: the carried velocity (x/z drift; Leaf_Create's jitter slots).
    pub vel: V3,
    /// instance+0x20: the spin added to the rotation every tick.
    pub spin: V3,
    /// instance s16+0x30: the frame countdown.
    pub frames_left: i16,
    /// obj u8+0x106: the tick budget.
    pub budget: u8,
    /// instance u8+0x32: the per-tick budget cost. `Leaf_Create` 0x1abc88 never writes +0x32,
    /// so a game-spawned leaf's cost is 0 and the 256-frame limit always kills first (the
    /// budget guard stays for the mechanism — [M 0x1abee0:22-27]).
    pub tick_cost: u8,
    /// The counter-hash state for the wiring's per-tick draws (the G-033 discipline).
    counter: u32,
}

impl LeafFx {
    /// The shared-stream form (`Leaf_Update` 0x1abee0's three draws in game order: the y wind
    /// `Rand_FRand(0.075)` first, then the x/z drifts `Rand_FRand(0.005)` [M 0x1abee0:30-40]).
    pub fn tick(&mut self, rng: &mut crate::drone::Rng) -> bool {
        let y = rng.frand(0.075);
        let dx = rng.frand(0.005);
        let dz = rng.frand(0.005);
        self.tick_draws(y, dx, dz)
    }

    /// The same integrator with the three draws supplied — the world-FX wiring's counter-hash
    /// form (the G-033 discipline: the game's per-leaf draws ride the shared stream, the port's
    /// placement FX consume none).
    pub fn tick_draws(&mut self, y_wind: f32, drift_x: f32, drift_z: f32) -> bool {
        if self.frames_left < 1 {
            return false;
        }
        let spent = self.budget as i16 - self.tick_cost as i16;
        if spent <= 0 {
            return false; // the game's `iVar3 > 0` guard fails → delete bit [M 0x1abee0:48-50]
        }
        self.budget = spent as u8;
        self.frames_left -= 1;
        for k in 0..3 {
            self.pos[k] += self.vel[k];
        }
        self.pos[1] -= y_wind - 0.025;
        self.vel[0] += drift_x - 0.0025;
        self.vel[2] += drift_z - 0.0025;
        for k in 0..3 {
            self.rot[k] += self.spin[k];
        }
        true
    }
}

/// The per-frame advance `Env_Tree_Update` 0x17c328 (`control_funcs` 0x26f890 class 0x3F)
/// performs: `if (ScriptCam == 0) AnimObjectUpdate(obj)` — the sway script runs unless a script
/// camera is active, and nothing else [M]. `TreeFx.paused` records the create-time pause bit; the
/// anim-script advance itself is the world-object animation runtime's job (draw side).
///
/// One 0xf3 placement (`Env_Tree_Create` 0x17c200: a class-0x3f object at the placement pose,
/// `AnimObjectNew(obj, param0)` [M :15] + `AnimScriptAdd(obj, param1)` [M :23] — the sway
/// animation, with param1 == 0x600008c swapping to 0x600008d on a 50% `Rand_Random() & 1` [M
/// :16-22; the shared stream 0x1e36f0 = SEED_X/SEED_Y]; on every level but 0x7000043 the script
/// pauses (+0x78 bit 0x80000000) and takes a random start phase `Rand_FRand(script length)`
/// truncated to int, stored at script+0x90 [M :24-31]). `Env_Tree_Update` 0x17c328 then only
/// advances the anim runtime: `if (ScriptCam == 0) AnimObjectUpdate(obj)` [M :5-8]. The pose
/// apply for world objects (the sway on the drawn tree) is the render side's job — the fields
/// below are the data it consumes; the runtime functions are the game's own [M AnimObjectUpdate
/// 0x1befd8 / AnimScriptTick 0x1c2630 / AnimSeqTick 0x1c28d0].
#[derive(Clone, Debug, PartialEq)]
pub struct TreeFx {
    pub placement: usize,
    pub pos: V3,
    pub yaw: f32,
    /// param 0 — the model hash: the placement's (index, value) params scatter to
    /// level_tag+0x2c+4·i in the dynamic pass [M parsemap_block_map_data_dynamic 0x1d1b68:70-75]
    /// and `Env_Tree_Create` reads param_3+0x2c for `AnimObjectNew` [M 0x17c200:15].
    /// (`parseentity_fixup_entity` 0x1d00c0 is the STATIC pass only [M 0x1d1998:110, gated on the
    /// 0x8000 flag :92] — never in the tree's dynamic chain [M 0x1d1b68:93-96].) 07000004's two
    /// carry 0x05000003 "Tree" [D param 0 = 83886083, the ISO-gated test's dump].
    pub model: u32,
    /// param 1 — the anim script hash (level_tag+0x30 → `AnimScriptAdd` [M 0x17c200:16-23]);
    /// 0x600008c swaps to 0x600008d on the counter-hashed coin [M :16-22]. 07000004's two carry
    /// 0x0600008c — the coin IS live there [D param 1 = 100663436; the ISO-gated test pins the
    /// two-hash outcome].
    pub anim: u32,
    /// `Env_Tree_Create` 0x17c200:24-31: level == 0x7000043 keeps the script unpaused, every other
    /// level pauses +0x78 bit 0x80000000 and takes a random phase. 07000004 != 0x7000043 → paused.
    pub paused: bool,
    /// The start-phase unit for the paused arm: the game draws `Rand_FRand((f32)(i16)script+0xb0)`
    /// and stores `(int)` of it at script+0x90 [M 0x17c200:26-31] — the port stores the [0,1)
    /// counter-hash unit; the consumer multiplies by the script's length (`Script.length`, the
    /// runtime node's +0xb0 short) and floors.
    pub phase_unit: f32,
}

/// One 0xd4 placement.
#[derive(Clone, Debug, PartialEq)]
pub struct CloudFx {
    pub placement: usize,
    /// param 4: the `View_AddSkyObj` slot the sky pass orders by (levels.md sky_instances).
    pub slot: u32,
    /// param 1/2 × 0.001: the drift per tick.
    pub drift: [f32; 2],
    /// param 0: the height over the player.
    pub height: f32,
    /// param 3: the fmod wrap modulus of both phases.
    pub wrap: f32,
    /// data[2]/data[3]: the drift phases.
    pub phase: [f32; 2],
}

/// The bolt's draw mesh — `hashtable_set_object_to_entity_gfx(obj, 0x200032c)` [M
/// `Lightning_Update` 0x18cf68:30], the same 0x0200xxxx mesh namespace as RIPPLE_MESH.
pub const BOLT_MESH: u32 = 0x0200_032c;

/// One 0x2c placement record at load (pre-pairing) — the fields `Lightning_Create` 0x18cde8
/// copies into the created object's ext [M 0x18cde8:20-24].
struct LightningRec {
    placement: usize,
    /// ext+0x40 = the full word at the placement param area +0x2c (param 0).
    group: u32,
    /// ext+0x44 = the u16 at param area +0x30 (param 1 low half) — the pairing key, compared
    /// as a signed short [M 0x18cde8:40].
    key: i16,
    /// ext+0x46 = the u16 at param area +0x34 (param 2 low half) — stored at create, never
    /// read by `Lightning_Update` [M 0x18cf68] (dead in the game too).
    #[allow(dead_code)]
    extra: u16,
    pos: V3,
}

/// One lightning strike: a paired 0x2c master (`Lightning_Create` 0x18cde8, class 0x33) with
/// its 8-object bolt chain — ext[0] the master, ext[1..7] six fresh class-1 beam objects
/// [M 0x18cde8:44-52], ext[7] the found partner (demoted to class 1 [M 0x18cde8:54]);
/// ext+0x20 the master's position (the start), ext+0x30 the partner's position (the end)
/// [M 0x18cde8:55-68].
#[derive(Clone, Debug, PartialEq)]
pub struct LightningStrike {
    /// The master placement — the SECOND of the pair in creation order: the finder of the
    /// match becomes the master [M 0x18cde8:41-69].
    pub master_placement: usize,
    /// The partner placement — the found earlier class-0x33 object.
    pub partner_placement: usize,
    /// ext+0x40: param 0 — the group word.
    pub group: u32,
    /// ext+0x44: param 1 (u16) — the pairing key.
    pub key: u16,
    /// ext+0x20: the strike's start (the master placement's translation).
    pub start: V3,
    /// ext+0x30: the strike's end (the partner placement's translation).
    pub end: V3,
    /// The 8 chain node positions, node 0 = master … node 7 = partner. At load the six fresh
    /// beams sit at the master's pose (`control_create_object(0, the master's pos/quat)`
    /// [M 0x18cde8:46]); the game's `FractalThing` 0x1a94d8 redistributes nodes 1..7 on the
    /// first drawn reflicker flash — here on the `super_happy && reflicker` tick with the
    /// counter-hash jitter (the G-033 discipline: the game draws 21 `Rand_FRand`s from the
    /// shared stream [M 0x1a94d8:78-87 — 7 mid nodes × 3 axis draws]; the port consumes none
    /// and hashes the same [0, amp) band per node).
    pub nodes: [V3; 8],
    /// `Lightning_Update` 0x18cf68: false = the `View_SetDrawInNoViews` arm (hidden), true =
    /// the `SuperHappyLightningFlag` arm (the BOLT_MESH chain drawn in all views). The game's
    /// per-tick hide arm covers all 8 objects [M 0x18cf68:13-21], the draw arm re-sets the
    /// mesh, the all-views flag and the `PositionBeam` chain master→beams→partner→end
    /// [M 0x18cf68:27-42].
    pub visible: bool,
    counter: u32,
}

impl LightningStrike {
    /// `Lightning_Update` 0x18cf68 for this strike, one tick. `g` is the port's copy of the
    /// lightning globals (`crate::sky::LightningGlobals` — the game reads the globals
    /// `SuperHappyLightningFlag`/`Reflicker` [M 0x18cf68:12/23]; the sim does not drive
    /// `View_DrawSky` yet, so `WorldFx::lightning` rests cleared and the bolt stays hidden —
    /// the sky pass sets them through `WorldFx::lightning_globals_mut`).
    pub fn tick(&mut self, g: &mut crate::sky::LightningGlobals) {
        if !g.super_happy {
            self.visible = false; // the 8 × `View_SetDrawInNoViews` arm [M 0x18cf68:13-21]
            return;
        }
        if g.reflicker {
            g.reflicker = false; // consumed [M 0x18cf68:23-24]
            self.counter = self.counter.wrapping_add(0x9e37_79b9);
            let mut s = self.counter;
            let mut rand = move |amp: f32| { s = s.wrapping_add(0x9e37_79b9); jitter(s, 0x1e) * amp };
            fractal_thing(&mut self.nodes, 0, 8, self.start, self.end, 0, &mut rand);
            self.counter = s;
        }
        self.visible = true;
    }
}

/// `FractalThing(objs, i0, i1, start, end, mode)` 0x1a94d8: the recursive midpoint
/// displacement that shapes the bolt. While `i1 - i0 > 1`: mid = `(i0+i1)/2` (rounded up on a
/// negative sum — the decompile's `- (>> 0x1f)` correction [M 0x1a94d8:66]), the amplitude
/// `Vec_Dist3D(start, end) × 0.5` in mode 0, `sqrt(dist) × 0.16666667` otherwise [M
/// 0x1a94d8:67-76], the node position `(start+end)×0.5 + (Rand_FRand(amp) − amp/2)` per axis
/// (3 draws [M 0x1a94d8:78-87]) written to the mid node, then the two recursions [M
/// 0x1a94d8:92-93]. `rand` returns one `Rand_FRand(amp)` in [0, amp) — the game draws it from
/// the shared stream [M Rand_FRand 0x1e3780 = SEED_X/SEED_Y]; the caller supplies the
/// counter-hash value instead (the G-033 discipline — no shared-stream draws).
pub fn fractal_thing(nodes: &mut [V3], i0: i32, i1: i32, start: V3, end: V3, mode: u8, rand: &mut impl FnMut(f32) -> f32) {
    if i1 - i0 <= 1 { return; }
    let sum = i0 + i1;
    let mid = if sum < 0 { (sum + 1) / 2 } else { sum / 2 };
    let d = ((start[0] - end[0]).powi(2) + (start[1] - end[1]).powi(2) + (start[2] - end[2]).powi(2)).sqrt();
    let amp = if mode == 0 { d * 0.5 } else { d.sqrt() * 0.166_666_67 };
    let half = amp * 0.5;
    let mut m = [0.0f32; 3];
    for k in 0..3 { m[k] = (start[k] + end[k]) * 0.5 + (rand(amp) - half); }
    nodes[mid as usize] = m;
    fractal_thing(nodes, i0, mid, start, m, mode, rand);
    fractal_thing(nodes, mid, i1, m, end, mode, rand);
}

/// One 0x4a placement.
#[derive(Clone, Debug, PartialEq)]
pub struct ApocalypseFx {
    pub placement: usize,
    /// param 0: the latch channel.
    pub channel: u16,
    /// param 2: the rumble strength (the port has no rumble; carried for completeness —
    /// likewise `Input_RumblePause` 0x17f710, `SP_LoadScript` 0x1dbc08:20's stop-all-pads
    /// call before the blocking load, is a no-op here [M 0x17f710:4-9]).
    pub rumble: u16,
    /// param 3 / param 4: the whiteout / flashbang flags.
    pub whiteout: bool,
    pub flashbang: bool,
    /// data[0]/data[1]: the remaining/total countdown in ticks.
    pub remaining: f32,
    pub total: f32,
    /// data+0xe: latched once the channel fires.
    pub armed: bool,
    /// The viewer whiteout value `remaining·5/total` [M 0x18d4f0] the renderer would apply.
    pub fade: f32,
}

/// One emitter particle slot (the game's four parallel arrays collapsed).
#[derive(Clone, Debug, PartialEq)]
pub struct EmitterParticle {
    pub pos: V3,
    pub vel: V3,
    pub age: f32,
    pub life: f32,
    /// The 4-byte ARGB the draw reads (rgb at half range from the keys).
    pub rgba: [u8; 4],
}

/// One 0xf2 placement.
#[derive(Clone, Debug, PartialEq)]
/// `Emitter_Kill` 0x17b960 (8 B) is the emitter object's `control_funcs` kill slot — entry
/// type 0x32 at 0x26faf0 (`Emitter_Create` sets `obj+0xff = 0x32` [M 0x17ae00]) — and it is an
/// empty `return`: emitter state dies with the object [M table dump; `control_delete_object`
/// 0x133178 dispatches the +8 slot]. The port has no destructor table; the FX records here are
/// recreated per level, so the null destructor needs no counterpart.
pub struct EmitterFx {
    pub placement: usize,
    pub pos: V3,
    /// The placement rotation words (+0x18..+0x20), raw — the port's interim spawn-basis source
    /// until the +0x24 quaternion path landed (the spawn now reads `quat`; these stay for the
    /// census cross-checks).
    pub rotation: [f32; 3],
    /// The placement's +0x24..+0x34 quaternion (x, y, z, w) — the game's spawn-basis source for
    /// the 0xf2 path [M 0x1d04f0:474-478].
    pub quat: [f32; 4],
    /// param 0: the def hash (resolved against the level's block-0x30 defs).
    pub def: u32,
    /// param 1: 0 → −1 (emit while enabled), 1 → 0 (one-shot), n → n/60 s countdown.
    pub rate: f32,
    /// params 2/3/4: the on / off / out channels.
    pub ch_on: u16,
    pub ch_off: u16,
    pub ch_out: u16,
    /// param 5: the blink period in ticks (0 = steady).
    pub toggle_period: f32,
    toggle_timer: f32,
    toggle_state: bool,
    /// data+0x4b: the one-shot latch (rate counted out).
    pub expired: bool,
    /// data+0x49: the all-dead tick counter (3 → the object dies).
    dead_ticks: u8,
    pub parts: Vec<EmitterParticle>,
    counter: u32,
    /// G-041 (fix-impact-remainder): the `Effect_Create` 0x1ce688 impact arm's emitters consume
    /// the SHARED stream (`Emitter_Update` 0x17b080's 4 × `Rand_FRandHalf(1.0)` per spawn [M
    /// :216-247]) — unlike the 0xf2 placements, whose counter-hash replacement is the G-033
    /// discipline (a documented deviation: same bands, different samples).
    pub shared_rng: bool,
    /// The object basis `Mat_SetDir/SetNorm/SetUp` 0x1e3418/0x1e3458/0x1e3438 built from the hit
    /// normal (`Emitter_Create` 0x17ae00): rows [Norm = v, Up = normal, Dir = u] — the spawn's
    /// local +y rides the normal. `None` → the 0xf2 placement chain (see the spawn site).
    pub basis: Option<[[f32; 4]; 4]>,
}

impl EmitterFx {
    fn enabled(&self, ch: &Channels) -> bool {
        if self.ch_on != 0 && !ch.on(self.ch_on as u32) { return false; }
        if self.ch_off != 0 && ch.on(self.ch_off as u32) { return false; }
        true
    }
}

/// The frame's particle billboards (`Emitter_Draw` path A: a size-`def.size` rgba sprite per alive
/// slot) — the render handoff, the way `Sim::snow_quads` exposes the snow.
///
/// Replaces `CreatePointerSpriteList` 0x11a160 (unported-front-end-menus-1) [M]: the game converts
/// this same VECTOR array into one batched GIF sprite packet (56 quads per tag, texture bound by
/// psiFont_SetTexture 0x120880; count pre-masked to a multiple of 4 by psiDrawParticleList
/// 0x122d50:107; tint 0x3f,0x3f,0x3f,0x7f; scale 1/(PS2SnowSize·PS2SnowSizeMult)) — reached every
/// frame via DrawDrops 0x18cb90 (rain/snow hashes 0x3000045/0x3000087) and Emitter_Draw 0x17b708
/// (def+4) ← View_DrawObjList 0x1e69a0 ← Game_Draw. Here the quads feed the wgpu snow/particle
/// pipelines instead (gpu.rs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleSprite {
    pub pos: V3,
    pub size: f32,
    pub rgba: [u8; 4],
    /// def+0x3c: 0 = normal alpha, else additive [M 0x17b708:15].
    pub additive: bool,
}

/// The whole FX set for one level (`WorldFx::attach_level`).
#[derive(Default)]
pub struct WorldFx {
    pub flickers: Vec<FlickerFx>,
    pub ripples: Vec<RippleFx>,
    pub leafgens: Vec<LeafGenFx>,
    /// The live falling leaves (`Leaf_Create` 0x1abc88 spawns them, `Leaf_Update` 0x1abee0
    /// integrates them — created mid-walk objects skip their creation frame in the game's
    /// newest-first object pass [M control_add_object_to_list 0x134698 prepends], so a leaf
    /// spawned this tick first updates next tick — the wiring updates before spawning).
    pub leaves: Vec<LeafFx>,
    pub trees: Vec<TreeFx>,
    pub clouds: Vec<CloudFx>,
    pub apocalypses: Vec<ApocalypseFx>,
    pub emitters: Vec<EmitterFx>,
    /// The paired 0x2c strikes, in pairing order (the master's creation order).
    pub lightnings: Vec<LightningStrike>,
    /// The 0x2c placements created with no earlier same-key class-0x33 — their own
    /// `Lightning_Update` no-ops (ext[0] stays 0 [M 0x18cf68:11]); they still serve as
    /// partners of later strikes.
    pub unpaired_lightnings: Vec<usize>,
    /// The port's copy of the lightning globals (`LightningType`/`Reflicker`/
    /// `SuperHappyLightningFlag` [L 0x30d0b8/0x30d0b9]; `crate::sky::LightningGlobals`). The
    /// game's `Lightning_Update` reads the globals each pass; the (future) sky pass mirrors
    /// its `View_DrawSky` tick into here.
    pub lightning: crate::sky::LightningGlobals,
    defs: std::collections::HashMap<u32, EmitterDef>,
}

/// The 0x2c pairing [M `Lightning_Create` 0x18cde8:28-72]: each new bolt walks the object list
/// (newest-first — `control_add_object_to_list` 0x134698 prepends, the worldobj.rs emitter
/// note) for the first OTHER class-0x33 object whose signed key halfword matches, pairs with
/// it (ext[7], demoted to class 1) and becomes the master. A record with no earlier match
/// keeps ext[0] = 0 — its own update no-ops [M 0x18cf68:11] — but it still serves as a later
/// strike's partner. Records: `(placement index, param 0 word, param 1 as i16, param 2 u16,
/// translation)` in creation (= placement) order.
pub fn pair_lightning_recs(recs: &[(usize, u32, i16, u16, V3)]) -> (Vec<LightningStrike>, Vec<usize>) {
    let mut strikes = Vec::new();
    let mut ends = vec![false; recs.len()]; // found as a partner → demoted to class 1
    let mut inert = Vec::new();
    for j in 0..recs.len() {
        let Some(i) = (0..j).rev().find(|&i| !ends[i] && recs[i].2 == recs[j].2) else {
            inert.push(recs[j].0);
            continue;
        };
        ends[i] = true;
        let (a, b) = (&recs[i], &recs[j]);
        strikes.push(LightningStrike {
            master_placement: b.0, partner_placement: a.0,
            group: b.1, key: b.2 as u16,
            start: b.4, end: a.4,
            nodes: [b.4, b.4, b.4, b.4, b.4, b.4, b.4, a.4],
            visible: false, counter: b.0 as u32,
        });
    }
    (strikes, inert)
}

/// `Env_Tree_Create` 0x17c200:16-22: the anim hash with the variant coin — param1 == 0x600008c
/// swaps to 0x600008d on the game's `Rand_Random() & 1` (the shared stream [M 0x1e36f0 =
/// SEED_X/SEED_Y]); the port counter-hashes the bit per placement (the G-033 discipline — no
/// load-time draws). Any other hash passes through untouched [M — the compare is the only gate].
pub fn tree_anim_hash(param1: u32, placement: usize) -> u32 {
    if param1 == 0x0600_008c && jitter(placement as u32, 0x5ee) < 0.5 { 0x0600_008d } else { param1 }
}

impl WorldFx {
    /// Every FX placement of the main map, in placement order (the game creates the dynamic objects
    /// in the same `parsemap_create_dynamic_objects` 0x1d04f0 walk). The emitter defs merge from
    /// every map's block 0x30 (the game's hashtable is global). `level_hash` = the archive hash
    /// (GameState+8; `Env_Tree_Create` 0x17c200:44 compares it for the tree anim's pause arm).
    pub fn attach_level(level: &Level, level_hash: u32) -> WorldFx {
        let mut defs = std::collections::HashMap::new();
        for map in &level.maps {
            for d in &map.emitter_defs { defs.insert(d.hash, d.clone()); }
        }
        let mut fx = WorldFx { defs, ..Default::default() };
        let Some(m) = level.main_map() else { return fx };
        let mut lightning_recs: Vec<LightningRec> = Vec::new();
        for (i, p) in level.maps[m].placements.iter().enumerate() {
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            let kind = p.flags & 0xff;
            let pos = p.translation;
            let _ = &mut fx;
            match kind {
                0x21 => {
                    // `Ripples_Create` 0x1acc90:20-30 — the s16 params, both-0 defaults.
                    let (spread, period) = (param(0) as u16 as i16, param(1) as u16 as i16);
                    let (spread, period) = if period == 0 && spread == 0 { (1.0, 0x1e) } else { (spread as f32, period as i32) };
                    fx.ripples.push(RippleFx { placement: i, pos, spread, period, counter: i as u32 });
                }
                0x2b => {
                    // `LeafGen_Create` 0x1acbd0 — the generator, no placement params read. The
                    // yaw is the placement's raw euler ry (the object's +0x54 [M 0x1d04f0:50-53]);
                    // the spread (obj+0x8c) reads 0.0 — the create memset leaves it unset
                    // [M 0x1344e0:17, 0x1acbd0:12-18].
                    fx.leafgens.push(LeafGenFx { placement: i, pos, counter: i as u32, spawned: 0, last_offset: [0.0; 2], yaw: p.rotation[1], spread: 0.0 });
                }
                0xf3 => {
                    // `Env_Tree_Create` 0x17c200: the model/anim hashes from the placement's params
                    // 0/1 (the dynamic pass's scatter [M 0x1d1b68:70-75]); the anim variant swap
                    // 0x600008c → 0x600008d on the game's 50% coin [M 0x17c200:16-22] —
                    // counter-hashed (no load-time draws, the G-033 discipline); the pause arm —
                    // every level but 0x7000043, with the start-phase unit [M :24-31].
                    let anim = tree_anim_hash(param(1), i);
                    fx.trees.push(TreeFx {
                        placement: i, pos, yaw: p.rotation[1],
                        model: param(0), anim, paused: level_hash != 0x0700_0043,
                        phase_unit: jitter(i as u32, 0x5ef),
                    });
                }
                0xe3 => {
                    // `Flicker_Create` 0x1acb00: data+4 = param 0 (the u16 spark gate — unreachable),
                    // data+0x20 = param 1 (the signal channel). The tint starts black.
                    fx.flickers.push(FlickerFx { placement: i, pos, signal_channel: param(1) as u16, rgb: [0; 3] });
                }
                0xd4 => {
                    // `Cloud_Create` 0x18d620:20-28 (the port's drift rates; the slot is param 4 —
                    // levels.rs sky_instances already draws the static placement).
                    fx.clouds.push(CloudFx {
                        placement: i, slot: param(4),
                        drift: [param(1) as f32 * 0.001, param(2) as f32 * 0.001],
                        height: param(0) as i32 as f32,
                        wrap: param(3) as i32 as f32,
                        phase: [0.0; 2],
                    });
                }
                0x4a => {
                    // `Apocalypse_Create` 0x18dcc0: data[0] = data[1] = param1·60, +8 = param 0,
                    // +0xa = param 2, +0xc = param 3, +0xd = param 4.
                    let total = param(1) as i32 as f32 * 60.0;
                    fx.apocalypses.push(ApocalypseFx {
                        placement: i, channel: param(0) as u16, rumble: param(2) as u16,
                        whiteout: param(3) != 0, flashbang: param(4) != 0,
                        remaining: total, total, armed: false, fade: 0.0,
                    });
                }
                0x2c => {
                    // `Lightning_Create` 0x18cde8:20-24 — the ext fields from the param area;
                    // the pairing runs after the walk (creation order = placement order).
                    lightning_recs.push(LightningRec {
                        placement: i, group: param(0),
                        key: param(1) as u16 as i16, extra: param(2) as u16, pos,
                    });
                }
                0xf2 => {
                    // `Emitter_CreatePlist` 0x17b848: the rate from param 1 (0 → −1, 1 → 0, else /60);
                    // `Emitter_Create` 0x17ae00 inits every slot dead (age 100000, life 0), the
                    // expired byte = (rate == 0), the toggle state = 1, the channels from params 2-4.
                    // An unresolvable def means NO object (`Emitter_Create` 0x17ae00: hashtable_getitem
                    // 0 → returns 0) — the placement is skipped exactly like the game.
                    let Some(d) = fx.defs.get(&param(0)) else { continue; };
                    let rate = match param(1) as i32 { 0 => -1.0, 1 => 0.0, n => n as f32 * REC_FRAME_RATE };
                    fx.emitters.push(EmitterFx {
                        placement: i, pos, rotation: p.rotation, quat: p.quaternion(),
                        def: param(0), rate,
                        ch_on: param(2) as u16, ch_off: param(3) as u16, ch_out: param(4) as u16,
                        toggle_period: param(5) as i32 as f32,
                        toggle_timer: 0.0, toggle_state: true,
                        expired: rate == 0.0, dead_ticks: 0,
                        parts: (0..d.max).map(|_| EmitterParticle {
                            pos: [0.0; 3], vel: [0.0; 3], age: 99999.0, life: 0.0, rgba: [0; 4],
                        }).collect(), counter: i as u32,
                        shared_rng: false, basis: None,
                    });
                }
                _ => {}
            }
        }
        // The 0x2c pairing — the standalone `pair_lightning_recs` (the decompile trace above it).
        let (strikes, inert) = pair_lightning_recs(&lightning_recs.iter().map(|r| (r.placement, r.group, r.key, r.extra, r.pos)).collect::<Vec<_>>());
        fx.lightnings = strikes;
        fx.unpaired_lightnings = inert;
        fx
    }

    /// The lightning globals the (future) sky pass mirrors its `View_DrawSky` tick into —
    /// `Lightning_Update` reads them per pass [M 0x18cf68:12/23].
    pub fn lightning_globals_mut(&mut self) -> &mut crate::sky::LightningGlobals { &mut self.lightning }

    /// The `Effect_Create` 0x1ce688 impact arm's `Emitter_Create(0, 0, EffectInfo+0x84, hit_pos,
    /// normal, 0, 0, 0)` 0x17ae00: a def-hash lookup (a miss → NO object, the placement path's own
    /// rule), a ONE-SHOT emitter (the rate argument is 0 → the +0x4b latch), no channels (+0x4a =
    /// 1 from create, params 0 don't touch it), the basis from the hit normal, every slot dead.
    /// Consumes no RNG here — the burst's draws run on the object's own update ticks.
    pub fn spawn_impact(&mut self, def_hash: u32, pos: V3, normal: V3) {
        let Some(d) = self.defs.get(&def_hash) else { return };
        // `Vec_MakeOrthonormalBasis(normal)` → (u, v); Mat_SetDir(u) → row +0x20 (the spawn's
        // local z), Mat_SetNorm(v) → row +0x00 (local x), Mat_SetUp(normal) → row +0x10 (local
        // y) [M 0x1e3418/0x1e3458/0x1e3438]. The rows the velocity transform reads:
        // vel = Norm_row·x + Up_row·y + Dir_row·z [M 0x17b080 _vmulabc/_vmaddabc/_vmaddbc].
        let (u, v) = crate::effect::orthonormal_basis_pair(normal);
        let basis = [
            [v[0], v[1], v[2], 0.0],
            [normal[0], normal[1], normal[2], 0.0],
            [u[0], u[1], u[2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        self.emitters.push(EmitterFx {
            placement: usize::MAX, pos, rotation: [0.0; 3], quat: [0.0; 4],
            def: def_hash, rate: 0.0, ch_on: 0, ch_off: 0, ch_out: 0,
            toggle_period: 0.0, toggle_timer: 0.0, toggle_state: true,
            expired: true, dead_ticks: 0,
            parts: (0..d.max).map(|_| EmitterParticle {
                pos: [0.0; 3], vel: [0.0; 3], age: 99999.0, life: 0.0, rgba: [0; 4],
            }).collect(),
            counter: 0, shared_rng: true, basis: Some(basis),
        });
    }

    /// The def table (the test's count predictor: the impact burst draws 4 × min(max, batch)).
    pub fn defs(&self) -> &std::collections::HashMap<u32, EmitterDef> { &self.defs }

    /// One 60 Hz tick (`Emitter_Update`/`Ripples_Update`/`cloud_update`/`Apocalypse_Update` run in
    /// the game's object handler; `Flicker_Update` likewise). `frame` = `GameState+0x52`. `rng` is
    /// the SHARED stream — consumed ONLY by the impact emitters (`shared_rng`); the 0xf2
    /// placements keep the counter-hash replacement (the G-033/fix-world-fx discipline — a
    /// documented deviation: the game's 4 draws per spawn [M 0x17b080:216-247] are the stream's,
    /// the port hashes the same bands per object).
    pub fn tick(&mut self, ch: &mut Channels, player_pos: V3, frame: u32, gas: &mut crate::gas::GasPool, rng: &mut crate::drone::Rng) {
        // Flicker [M 0x1ab960]: the stubbed FindMapLight keeps the white target and the 0 stamp.
        for f in &mut self.flickers {
            for k in 0..3 {
                let gap = 255i32 - f.rgb[k] as i32;
                if gap != 0 {
                    let d = gap as f32 * 0.1;
                    let d = if d.abs() < 1.0 { if d < 0.0 { -1.0 } else { 1.0 } } else { d };
                    f.rgb[k] = (f.rgb[k] as i32 + d.round() as i32).clamp(0, 255) as u8;
                }
            }
            if f.signal_channel != 0 { ch.set(f.signal_channel as u32, 0, frame); }
        }
        // Ripples [M 0x1ac028]: the gate (counter-hash for Rand_Rand(period)), then 3 counter-hash
        // draws for the x/z jitter and the start scale.
        for r in &mut self.ripples {
            if r.period <= 0 { continue; }
            r.counter = r.counter.wrapping_add(0x9e37_79b9);
            if jitter(r.counter, 7) * r.period as f32 >= 1.0 { continue; }
            let jx = jitter(r.counter, 1) * (r.spread + r.spread) - r.spread;
            let jz = jitter(r.counter, 2) * (r.spread + r.spread) - r.spread;
            let s = jitter(r.counter, 3) * 0.05 + 0.05;
            gas.init([r.pos[0] + jx, r.pos[1], r.pos[2] + jz], 0.0, [0.0; 3], s, s + s, 6, [0x7f, 0x7f, 0x7f, 0xff], r.period as f32);
        }
        // Leaves first [M Leaf_Update 0x1abee0]: a leaf created this tick must not update until
        // next tick (the newest-first object pass skips it — the emitter note below), so the
        // update runs BEFORE the generators' spawn pass. The three per-tick draws (y wind 0.075,
        // then the x/z drifts 0.005 [M 0x1abee0:30-40]) come out of the per-leaf counter hash
        // (the G-033 discipline).
        let mut alive = Vec::with_capacity(self.leaves.len());
        for mut leaf in self.leaves.drain(..) {
            leaf.counter = leaf.counter.wrapping_add(0x9e37_79b9);
            let y = jitter(leaf.counter, 30) * 0.075;
            let dx = jitter(leaf.counter, 31) * 0.005;
            let dz = jitter(leaf.counter, 32) * 0.005;
            if leaf.tick_draws(y, dx, dz) {
                alive.push(leaf);
            }
        }
        self.leaves = alive;
        // LeafGens [M 0x1acc48]: the gate (counter-hash for Rand_Rand(30)), one `Leaf_Create`
        // spawn — the draw order and bands are the game's [M 0x1abc88:31-79], counter-hashed.
        for l in &mut self.leafgens {
            l.counter = l.counter.wrapping_add(0x9e37_79b9);
            if jitter(l.counter, 11) * 30.0 < 1.0 {
                l.spawned += 1;
                // Leaf_Create 0x1abc88 for count = 1: draw 1 discarded (0.02), draw 2 → the x
                // eject speed +0.01, draw 3 → the z eject speed, then the three position draws
                // `Rand_FRand(2·spread) − spread` (x, z, y order [M :47-55]) — with the
                // generator's spread 0.0 (obj+0x8c never written [M 0x1344e0:17 memset,
                // 0x1acbd0:12-18]) they return 0: the leaf spawns AT the generator. The spin is
                // `Rand_FRandHalf_Vec(0.1)` [M :75, 0x1dfef0 — 3 draws, ±0.05].
                let _discarded = jitter(l.counter, 14) * 0.02;
                let v7 = jitter(l.counter, 15) * 0.02 + 0.01;
                let v8 = jitter(l.counter, 16) * 0.02;
                let (jx, jz, jy) = (
                    jitter(l.counter, 17) * (l.spread + l.spread) - l.spread,
                    jitter(l.counter, 18) * (l.spread + l.spread) - l.spread,
                    jitter(l.counter, 19) * (l.spread + l.spread) - l.spread,
                );
                l.last_offset = [jx, jz];
                // The eject velocity rides the placement's raw euler yaw — obj+0x54 IS the
                // placement ry (parsemap copies the level_tag rotation words verbatim into
                // obj+0x50.. [M 0x1d04f0:50-53]; Leaf_Create's `PS2Sinf2(+0x54 + 1.5707964,
                // +0x54)` → (cos, sin) [M 0x1abc88:31-33]): vel = (v7·sin, 0, (v8+0.01)·cos)
                // [M :70-79]. The rot starts at 0 (control_create_object's rot arg is 0 → no
                // copy [M 0x1344e0:24-31]); the budget 0xff [M :54], 256 frames [M :57].
                let (s, c) = (crate::trig::sinf(l.yaw), crate::trig::cosf(l.yaw));
                self.leaves.push(LeafFx {
                    pos: [l.pos[0] + jx, l.pos[1] + jy, l.pos[2] + jz],
                    rot: [0.0; 3],
                    vel: [v7 * s, 0.0, (v8 + 0.01) * c],
                    spin: [
                        jitter(l.counter, 20) * 0.1 - 0.05,
                        jitter(l.counter, 21) * 0.1 - 0.05,
                        jitter(l.counter, 22) * 0.1 - 0.05,
                    ],
                    frames_left: 256,
                    budget: 0xff,
                    tick_cost: 0,
                    counter: l.counter ^ (l.spawned << 8),
                });
            }
        }
        // Clouds [M 0x18d768]: fmod-wrapped phases, position = player + (phase_x, height, phase_z).
        for c in &mut self.clouds {
            c.phase[0] = (c.phase[0] + c.drift[0]).rem_euclid(c.wrap.max(1e-6));
            c.phase[1] = (c.phase[1] + c.drift[1]).rem_euclid(c.wrap.max(1e-6));
            let _ = player_pos; // the parked position rides the sky pass's own follow (levels.rs)
        }
        // Apocalypse [M 0x18d4f0]: the latch, the countdown, the whiteout value.
        for a in &mut self.apocalypses {
            if !a.armed && ch.on(a.channel as u32) { a.armed = true; }
            if a.armed {
                a.remaining -= 1.0;
                if a.remaining <= 0.0 { a.remaining = 0.0; a.fade = 0.0; continue; }
                if a.whiteout { a.fade = a.remaining * 5.0 / a.total; }
            }
        }
        // Lightning bolts [M 0x18cf68]: the visibility (and the reflicker redistribution) —
        // the game's class-0x33 update reads the globals each pass; the port reads its copy
        // (resting cleared until the sky pass drives it, so the bolts stay hidden).
        let mut lg = self.lightning;
        for l in &mut self.lightnings { l.tick(&mut lg); }
        self.lightning = lg;
        // Emitters [M 0x17b080]: the rate countdown, the channels, the blink, the spawn, the motion.
        for e in &mut self.emitters {
            let Some(def) = self.defs.get(&e.def).cloned() else { continue; };
            // Creation-tick timing (fix-impact-remainder2): the game's object list is NEWEST-FIRST
            // (`control_add_object_to_list` 0x134698 prepends to the 0x2705a0 head; the pass-1 walk
            // in `control_movement_object_handler` 0x1336f8 follows head+0x14 first) and the walk
            // cursor never revisits the head, so an object created mid-walk (the impact emitter,
            // during the bullet's collide phase) is NOT updated its creation frame [M Collide_Update
            // 0x1ea398 vs Control_Update's class walk] and is the FIRST object updated the next
            // frame — ahead of the older drones. The port's `Drones::tick` runs this fx pass BEFORE
            // the bullet collide phase of the same frame, so an emitter created at tick N is first
            // seen HERE at tick N+1 — which IS the game's first `Emitter_Update` (the burst draws
            // then, not one tick later; the creation-frame skip is already implicit in the call
            // order, and gating it again delayed every burst by a tick and shifted every AI roll
            // after a wall hit).
            // The out channel resets every tick and re-sets while any particle lives [M 0x17b080].
            if e.ch_out != 0 { ch.set(e.ch_out as u32, 0, frame); }
            // The blink (param 5): ON half the period, OFF the whole period [M 0x17b080:105-120].
            if e.toggle_period > 0.0 {
                e.toggle_timer += REC_FRAME_RATE;
                let phase = if e.toggle_state { e.toggle_period * 0.5 } else { e.toggle_period };
                if e.toggle_timer > phase { e.toggle_timer = 0.0; e.toggle_state = !e.toggle_state; }
            }
            // The rate countdown [M 0x17b080:121-131].
            if e.rate > 0.0 {
                e.rate -= REC_FRAME_RATE;
                if e.rate < 0.0 { e.expired = true; }
            }
            let enabled = e.enabled(ch) && e.toggle_state;
            // Motion + colour over life; the out channel rides any live particle.
            let mut alive = 0usize;
            for p in &mut e.parts {
                if p.age < p.life {
                    alive += 1;
                    let c = def.colour_at(p.age * (1.0 / p.life));
                    p.rgba = [c[0].clamp(0.0, 255.0) as u8, c[1].clamp(0.0, 255.0) as u8, c[2].clamp(0.0, 255.0) as u8, c[3].clamp(0.0, 255.0) as u8];
                    p.vel[1] += REC_FRAME_RATE * def.gravity * -9.8;
                    for k in 0..3 { p.pos[k] += p.vel[k] * REC_FRAME_RATE; }
                    p.age += REC_FRAME_RATE;
                }
            }
            if alive > 0 && e.ch_out != 0 { ch.set(e.ch_out as u32, 1, frame); }
            // The spawn pass: batch-capped respawns of dead slots while enabled [M 0x17b080:191-260].
            let mut batch = def.batch;
            if def.max > 0 && def.batch > 0 && e.dead_ticks == 0 && enabled {
                for p in e.parts.iter_mut() {
                    if batch == 0 { break; }
                    let dead = p.life < p.age || p.age >= 99999.0;
                    if !dead { continue; }
                    if e.expired && p.age < 99999.0 { continue; } // one-shot: never-spawned slots only
                    p.age = 0.0;
                    batch -= 1;
                    // The spawn draws (yaw, pitch, speed, life) — the game's order [M 0x17b080:
                    // 216-247]. The impact emitters draw the SHARED stream (`Rand_FRandHalf(1.0)`
                    // × 4); the 0xf2 placements keep the counter-hash (no shared-stream draws).
                    let (yaw, pitch, speed, life);
                    if e.shared_rng {
                        yaw = def.yaw_base + def.yaw_var * crate::effect::frand_half(rng, 1.0);
                        pitch = def.pitch_base + def.pitch_var * crate::effect::frand_half(rng, 1.0);
                        speed = def.speed_base + def.speed_var * crate::effect::frand_half(rng, 1.0);
                        life = def.life_base + def.life_var * crate::effect::frand_half(rng, 1.0);
                        if std::env::var_os("NF_RNGLOG").is_some() {
                            eprintln!("[rnglog] emitter_burst +4 (draw #{})", rng.draws);
                        }
                    } else {
                        e.counter = e.counter.wrapping_add(0x9e37_79b9);
                        yaw = def.yaw_base + def.yaw_var * jitter(e.counter, 1);
                        pitch = def.pitch_base + def.pitch_var * jitter(e.counter, 2);
                        speed = def.speed_base + def.speed_var * jitter(e.counter, 3);
                        life = def.life_base + def.life_var * jitter(e.counter, 4);
                    }
                    // G-046: the game's cone is two PS2Sinf2 pairs [M Emitter_Update 0x17b080, disasm
                    // 0x17b504-0x17b53c]: PS2Sinf2(pitch + 1.5707964, pitch) → (cos p, sin p) and
                    // PS2Sinf2(yaw + 1.5707964, yaw) → (cos t, sin t); the stored dir vector is
                    // (sin p·cos t, cos p, sin p·sin t) — polar p from +Y, azimuth t from +X about +Y
                    // — then Vec_MulR32(speed). The port's old form (elevation p, azimuth from +Z)
                    // pointed the cone differently.
                    let (cp, sp) = (crate::trig::cosf(pitch), crate::trig::sinf(pitch));
                    let (ct, st) = (crate::trig::cosf(yaw), crate::trig::sinf(yaw));
                    let dir = [sp * ct * speed, cp * speed, sp * st * speed];
                    // The object basis (obj+0x90): the impact emitters carry the Mat_SetDir/
                    // SetNorm/SetUp rows built from the hit normal. The 0xf2 placements run the
                    // game's own chain [M]: the dispatcher builds the matrix with Quat_QuatToMat
                    // on the placement's +0x24 quaternion (0x1d04f0:474-478), Emitter_CreatePlist
                    // passes ONLY its up row (Mat_GetUp, 0x17b848 — matrix +0x10 [M Mat_GetUp
                    // 0x1e33b8]) and the position (the matrix's +0x30 translation row), and
                    // Emitter_Create rebuilds the basis from that up vector exactly like the
                    // impact path (Vec_MakeOrthonormalBasis + Mat_SetDir/SetNorm/SetUp,
                    // 0x17ae00:104-107) — rows [Norm = v, Up = up, Dir = u]. A pure-yaw quat's
                    // matrix row 1 is (0, 1, 0) [M 0x1e3e80 entries], so every yaw-only
                    // placement spawns on the reconstructed identity basis regardless of yaw.
                    let m = e.basis.unwrap_or_else(|| {
                        let qm = nf_formats::placement::quat_quat_to_mat(e.quat);
                        let up = [qm[1][0], qm[1][1], qm[1][2]];
                        let (u, v) = crate::effect::orthonormal_basis_pair(up);
                        [
                            [v[0], v[1], v[2], 0.0],
                            [up[0], up[1], up[2], 0.0],
                            [u[0], u[1], u[2], 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ]
                    });
                    p.vel = [0, 1, 2].map(|k| m[k][0] * dir[0] + m[k][1] * dir[1] + m[k][2] * dir[2]);
                    p.pos = e.pos;
                    p.life = life;
                    let c0 = def.keys.first().copied().unwrap_or([0.0; 5]);
                    p.rgba = [c0[0].clamp(0.0, 255.0) as u8, c0[1].clamp(0.0, 255.0) as u8, c0[2].clamp(0.0, 255.0) as u8, c0[3].clamp(0.0, 255.0) as u8];
                }
            }
            // The all-dead latch: 3 ticks after everything died, a spent emitter dies [M 0x17b080:300-320].
            let all_dead = e.parts.iter().all(|p| p.age >= p.life);
            if (e.dead_ticks > 0 || e.expired) && all_dead { e.dead_ticks += 1; }
        }
        self.emitters.retain(|e| e.dead_ticks < 3);
        // (no gas.update() here — Drones::tick already runs the pool once per tick at its head)
    }

    /// The frame's particle sprites (`Emitter_Draw` path A inputs: position, def size, the slot rgba).
    pub fn particle_sprites(&self) -> Vec<ParticleSprite> {
        let mut out = Vec::new();
        for e in &self.emitters {
            let Some(def) = self.defs.get(&e.def) else { continue };
            for p in &e.parts {
                if p.age >= p.life || p.rgba[3] == 0 { continue; }
                out.push(ParticleSprite { pos: p.pos, size: def.size, rgba: p.rgba, additive: def.blend != 0 });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_at_lerps_between_keys() {
        let d = EmitterDef {
            hash: 1, texture: 0, model: 0, blend: 0, max: 1, batch: 1, size: 1.0,
            life_base: 1.0, life_var: 0.0, gravity: 0.0, speed_base: 0.0, speed_var: 0.0,
            pitch_base: 0.0, pitch_var: 0.0, yaw_base: 0.0, yaw_var: 0.0,
            keys: vec![[100.0, 0.0, 0.0, 255.0, 0.0], [0.0, 0.0, 0.0, 0.0, 1.0]],
        };
        assert_eq!(d.colour_at(0.0)[0] as i32, 100, "key0 at t=0");
        assert_eq!(d.colour_at(0.5)[0].round() as i32, 50, "the midpoint lerps");
        assert_eq!(d.colour_at(2.0)[3], 0.0, "past the last key = the last key");
    }

    /// `Env_Tree_Create` 0x17c200:15-31 — the hash pass-through outside 0x600008c (the proven
    /// gate [M: the compare is the only swap condition]), the variant coin for 0x600008c
    /// (deterministic per placement, both outcomes possible), and the phase unit's range.
    #[test]
    fn tree_anim_variant_coin_and_phase_unit() {
        // The pass-through [M :16-22 — the swap fires only for 0x600008c]:
        assert_eq!(tree_anim_hash(0x0600_0474, 3), 0x0600_0474, "07000004's tree anim [D] passes through");
        assert_eq!(tree_anim_hash(0x0600_008d, 7), 0x0600_008d, "the variant itself stays");
        // The coin: deterministic per placement; every outcome is one of the two hashes.
        let mut swapped = 0;
        for p in 0..256u32 {
            let a = tree_anim_hash(0x0600_008c, p as usize);
            assert!(a == 0x0600_008c || a == 0x0600_008d, "only the two hashes, got {a:#x}");
            assert_eq!(a, tree_anim_hash(0x0600_008c, p as usize), "deterministic per placement");
            if a == 0x0600_008d { swapped += 1; }
        }
        assert!(swapped > 32 && swapped < 224, "the coin is ~50/50, got {swapped}/256");
        // Sister (can fail): the phase unit is a [0,1) fraction, not the raw hash word.
        for p in 0..64u32 {
            let u = jitter(p, 0x5ef);
            assert!((0.0..1.0).contains(&u), "the phase unit in [0,1), got {u}");
        }
    }

    /// `Flicker_Update` 0x1ab960 with the stubbed `Light_FindMapLight` 0x1cc3d0: the tint ramps
    /// white (10%/tick, min ±1) and the signal channel stamps 0 every tick, never 1.
    #[test]
    fn flicker_fades_white_and_the_signal_channel_stays_clear() {
        let mut fx = WorldFx::default();
        fx.flickers.push(FlickerFx { placement: 0, pos: [0.0; 3], signal_channel: 0x55, rgb: [0; 3] });
        let mut ch = Channels::default();
        ch.set(0x55, 1, 0);
        let mut rng = crate::drone::Rng::new(1);
        for _ in 0..600 { fx.tick(&mut ch, [0.0; 3], 0, &mut crate::gas::GasPool::default(), &mut rng); }
        assert_eq!(fx.flickers[0].rgb, [255; 3], "the tint reaches white");
        assert_eq!(ch.v[0x55], 0, "the stubbed FindMapLight keeps the light-on path");
    }

    /// `LeafGen_Update` 0x1acc48 + `Leaf_Create` 0x1abc88 + `Leaf_Update` 0x1abee0: the gate fires
    /// on the counter-hash, each spawn leaves the offset at 0 (the generator's spread obj+0x8c
    /// reads 0 [M 0x1344e0:17 memset]), the leaf spawns at the generator and falls; the live set
    /// stays bounded (256-frame lives, ~1 spawn per 30 ticks).
    #[test]
    fn leafgens_spawn_leaves_that_fall_and_expire() {
        let mut fx = WorldFx::default();
        fx.leafgens.push(LeafGenFx { placement: 0, pos: [1.0, 5.0, 2.0], counter: 0, spawned: 0, last_offset: [0.0; 2], yaw: 0.5, spread: 0.0 });
        let mut gas = crate::gas::GasPool::default();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        for f in 0..600 { fx.tick(&mut ch, [0.0; 3], f, &mut gas, &mut rng); }
        let l = &fx.leafgens[0];
        assert!(l.spawned >= 5, "600 ticks at the 30-tick gate → several leaf spawns, got {}", l.spawned);
        // The spread-0 rule [M 0x1abc88:47-55 + 0x1344e0:17]: the position jitter is exactly 0.
        assert_eq!(l.last_offset, [0.0, 0.0], "spread 0 → the leaf spawns at the generator");
        assert!(!fx.leaves.is_empty(), "the spawns became live leaves");
        assert!(fx.leaves.len() <= l.spawned as usize, "no phantom leaves");
        // Sister (can fail): the oldest live leaf has fallen well below the generator (the net
        // y-wind fall ≈ 0.0125/tick [M 0x1abee0:30-32] over a 256-frame life).
        let leaf = &fx.leaves[0];
        assert!(leaf.pos[1] < 4.0, "the oldest leaf fell: y = {}", leaf.pos[1]);
        assert!(leaf.frames_left < 256, "the frame countdown ran");
    }

    /// `Leaf_Update` 0x1abee0's two life limits through the wiring path: the 256-frame countdown
    /// kills (tick_cost 0 — `Leaf_Create` never writes instance+0x32), and the update order
    /// leaves a fresh spawn untouched until the next tick (the newest-first pass [M
    /// control_add_object_to_list 0x134698 prepends]).
    #[test]
    fn leaves_skip_their_creation_frame_and_die_at_the_frame_limit() {
        let mut fx = WorldFx::default();
        fx.leafgens.push(LeafGenFx { placement: 0, pos: [0.0, 50.0, 0.0], counter: 0, spawned: 0, last_offset: [0.0; 2], yaw: 0.0, spread: 0.0 });
        let mut gas = crate::gas::GasPool::default();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        // Force a spawn this tick (the gate rolls the counter forward each tick; 1/30 odds).
        let mut spawned_this_run = false;
        for _ in 0..1000 {
            fx.tick(&mut ch, [0.0; 3], 0, &mut gas, &mut rng);
            if fx.leafgens[0].spawned == 1 { spawned_this_run = true; break; }
        }
        assert!(spawned_this_run, "the gate fired within 1000 ticks");
        let spawn_tick_leaves = fx.leaves.len();
        assert_eq!(spawn_tick_leaves, 1, "the spawn landed");
        // Only this leaf's lifetime is under test — stop further spawns.
        let fresh = fx.leaves[0].clone();
        fx.leafgens.clear();
        assert_eq!(fresh.frames_left, 256, "the frame budget [M 0x1abc88:57]");
        assert_eq!(fresh.tick_cost, 0, "instance+0x32 never written [M 0x1abc88]");
        assert_eq!(fresh.pos, [0.0, 50.0, 0.0], "spread 0 → the leaf spawns AT the generator, rot 0 [M 0x1abc88]");
        // The eject bands [M 0x1abc88:70-79]: x = v7·sin(yaw) with yaw 0 → 0; z = (v8+0.01)·cos(0)
        // ∈ [0.01, 0.03).
        assert_eq!(fresh.vel[0], 0.0, "yaw 0 → no x eject");
        assert!((0.01..0.03).contains(&fresh.vel[2]), "the z eject speed in the game's band, got {}", fresh.vel[2]);
        // One more tick: the leaf now updates (the countdown spent one); the net fall is
        // statistical over the y-wind band (mean −0.0125/tick [M 0x1abee0:30-32]), so pin 30
        // ticks rather than one.
        fx.tick(&mut ch, [0.0; 3], 1, &mut gas, &mut rng);
        assert_eq!(fx.leaves[0].frames_left, 255, "the countdown spent one");
        let y30 = {
            for f in 2..31 { fx.tick(&mut ch, [0.0; 3], f, &mut gas, &mut rng); }
            fx.leaves[0].pos[1]
        };
        assert!(y30 < 50.0 - 0.2, "30 ticks of the y wind net-fall: y = {}", y30);
        // 256 more ticks: the frame limit kills the leaf (tick_cost 0 → the budget never does).
        for f in 31..320 { fx.tick(&mut ch, [0.0; 3], f, &mut gas, &mut rng); }
        assert!(fx.leaves.is_empty(), "the 256-frame limit expired the leaf");
    }

    /// `Ripples_Update` 0x1ac028: with the default period 30 the gate fires (counter-hash) roughly
    /// once per period; the puff lands inside the spread and takes the ripple mesh + white rgba.
    #[test]
    fn ripples_spawn_periodically_inside_the_spread() {
        let mut fx = WorldFx::default();
        fx.ripples.push(RippleFx { placement: 0, pos: [10.0, 5.0, 20.0], spread: 2.0, period: 30, counter: 0 });
        let mut gas = crate::gas::GasPool::default();
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        let mut occupied = 0;
        let mut seen = None;
        for f in 0..600 {
            fx.tick(&mut ch, [0.0; 3], f, &mut gas, &mut rng);
            if !gas.puffs.is_empty() { occupied += 1; seen = Some(gas.puffs[0].clone()); }
        }
        // A ring lives its whole period (life = period), so with ~1 spawn per period the pool holds
        // a puff the overwhelming majority of ticks.
        assert!(occupied > 400, "600 ticks at period 30 → a ring alive most ticks, {occupied}");
        let g = seen.unwrap();
        assert!((g.pos[0] - 10.0).abs() <= 2.0 && (g.pos[2] - 20.0).abs() <= 2.0, "the jitter stays inside the spread");
        assert!((g.scale - 0.05).abs() < 0.051 || (g.scale - 0.10).abs() < 0.051, "the start scale is 0.05..0.10");
    }

    /// `Emitter_Update` 0x17b080: a continuous emitter (rate −1, no channels) fills its slots at the
    /// batch rate, integrates them (pos += vel·dt, gravity), colours them over life, drives the out
    /// channel while anything lives, and a one-shot (rate 0 → expired) dies 3 all-dead ticks later.
    #[test]
    fn emitter_spawns_integrates_and_a_one_shot_dies() {
        let def = EmitterDef {
            hash: 0x1234, texture: 0, model: 0, blend: 1, max: 8, batch: 2, size: 0.5,
            life_base: 0.5, life_var: 0.0, gravity: 1.0, speed_base: 1.0, speed_var: 0.0,
            pitch_base: 0.0, pitch_var: 0.0, yaw_base: 0.0, yaw_var: 0.0,
            keys: vec![[128.0, 64.0, 0.0, 255.0, 0.0], [0.0, 0.0, 0.0, 0.0, 1.0]],
        };
        let mut fx = WorldFx { defs: std::iter::once((0x1234, def.clone())).collect(), ..Default::default() };
        fx.emitters.push(EmitterFx {
            placement: 0, pos: [1.0, 2.0, 3.0], rotation: [0.0; 3], quat: [0.0; 4], def: 0x1234, rate: -1.0,
            ch_on: 0, ch_off: 0, ch_out: 0x66, toggle_period: 0.0, toggle_timer: 0.0,
            toggle_state: true, expired: false, dead_ticks: 0, parts: (0..8).map(|_| EmitterParticle {
                pos: [0.0; 3], vel: [0.0; 3], age: 99999.0, life: 0.0, rgba: [0; 4],
            }).collect(), counter: 0,
            shared_rng: false, basis: None,
        });
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        for f in 0..60 { fx.tick(&mut ch, [0.0; 3], f, &mut crate::gas::GasPool::default(), &mut rng); }
        assert_eq!(fx.emitters.len(), 1, "a continuous emitter never dies");
        assert!(fx.emitters[0].parts.iter().filter(|p| p.age < p.life).count() >= 6, "the slots cycle at the batch rate (batch 2/tick, life 0.5 s)");
        assert!(fx.particle_sprites().len() >= 6, "the draw handoff exposes the alive slots");
        assert_eq!(ch.v[0x66], 1, "the out channel is driven while particles live");
        let p0 = &fx.emitters[0].parts[0];
        assert!(p0.pos[1] < 2.0 && p0.vel[1] < 0.0, "gravity pulls the velocity down (speed 1 up, g 1)");
        assert!(p0.rgba != [0; 4], "the colour keys feed the slot rgba");
        // The one-shot: rate 0 → expired at create; the never-spawned slots fire once more
        // [M 0x17b080 the `age < 99999` skip], live their life, and 3 all-dead ticks later the
        // object self-destructs (obj+0xFE bit 0).
        fx.emitters[0].rate = 0.0;
        fx.emitters[0].expired = true;
        let mut died = None;
        for f in 60..200 {
            fx.tick(&mut ch, [0.0; 3], f, &mut crate::gas::GasPool::default(), &mut rng);
            if fx.emitters.is_empty() { died = Some(f); break; }
        }
        assert!(died.is_some(), "the spent one-shot self-destructs (obj+0xFE bit 0)");
        assert_eq!(ch.v[0x66], 0, "the out channel clears when the emitter dies");
    }

    /// `Apocalypse_Update` 0x18d4f0: the channel latches, the countdown runs, the whiteout value
    /// rises, and the object dies at 0.
    #[test]
    fn apocalypse_latches_counts_down_and_fades() {
        let mut fx = WorldFx::default();
        fx.apocalypses.push(ApocalypseFx {
            placement: 0, channel: 0x10, rumble: 1, whiteout: true, flashbang: false,
            remaining: 120.0, total: 120.0, armed: false, fade: 0.0,
        });
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        fx.tick(&mut ch, [0.0; 3], 0, &mut crate::gas::GasPool::default(), &mut rng);
        assert!(!fx.apocalypses[0].armed && fx.apocalypses[0].fade == 0.0, "unlatched: no countdown");
        ch.set(0x10, 1, 1);
        for f in 1..61 { fx.tick(&mut ch, [0.0; 3], f, &mut crate::gas::GasPool::default(), &mut rng); }
        assert!(fx.apocalypses[0].armed, "the latch sets once the channel fires");
        assert!((fx.apocalypses[0].remaining - 60.0).abs() < 1e-4, "60 of 120 ticks remain");
        assert!((fx.apocalypses[0].fade - 60.0 * 5.0 / 120.0).abs() < 1e-4, "fade = remaining·5/total [M 0x18d4f0]");
        for f in 61..130 { fx.tick(&mut ch, [0.0; 3], f, &mut crate::gas::GasPool::default(), &mut rng); }
        assert!(fx.apocalypses[0].remaining == 0.0, "the countdown floors at 0");
    }

    /// `cloud_update` 0x18d768: the phases accumulate the drift and wrap by fmod.
    #[test]
    fn cloud_phases_drift_and_wrap() {
        let mut fx = WorldFx::default();
        fx.clouds.push(CloudFx { placement: 0, slot: 3, drift: [0.002, 0.0], height: 50.0, wrap: 0.1, phase: [0.0; 2] });
        let mut ch = Channels::default();
        let mut rng = crate::drone::Rng::new(1);
        for f in 0..100 { fx.tick(&mut ch, [0.0; 3], f, &mut crate::gas::GasPool::default(), &mut rng); }
        let c = &fx.clouds[0];
        assert!(c.phase[0] >= 0.0 && c.phase[0] < 0.1, "the phase wraps inside the modulus");
        assert!(c.phase[0] > 0.0, "the drift accumulated (100 ticks × 0.002 = 0.2 → wrapped)");
    }

    /// The 0x2c pairing [M `Lightning_Create` 0x18cde8]: the finder (the SECOND of a key pair
    /// in creation order) is the master, the found one the partner; a key with no earlier
    /// match stays inert (its own update no-ops [M 0x18cf68:11]).
    #[test]
    fn lightning_pairing_finder_is_master_and_loners_stay_inert() {
        let (pa, pb, pc) = ([1.0, 0.0, 1.0], [2.0, 0.0, 2.0], [3.0, 0.0, 3.0]);
        let recs = vec![
            (10usize, 0x11u32, 7i16, 0u16, pa),
            (11, 0x11, 7, 0, pb),
            (12, 0x22, 9, 0, pc),
        ];
        let (strikes, inert) = pair_lightning_recs(&recs);
        assert_eq!(strikes.len(), 1, "one key-7 pair; the key-9 record is alone");
        let s = &strikes[0];
        assert_eq!((s.master_placement, s.partner_placement), (11, 10), "the finder (second created) is the master");
        assert_eq!((s.group, s.key), (0x11, 7), "the master's own params ride ext+0x40/+0x44 [M 0x18cde8:20-23]");
        assert_eq!(s.start, pb, "ext+0x20 = the master's position [M 0x18cde8:58-61]");
        assert_eq!(s.end, pa, "ext+0x30 = the partner's position [M 0x18cde8:62-68]");
        assert_eq!(s.nodes, [pb, pb, pb, pb, pb, pb, pb, pa], "the six fresh beams spawn at the master's pose [M 0x18cde8:46]");
        assert!(!s.visible, "the bolt loads hidden (the hide arm owns every tick until the flag rises)");
        assert_eq!(inert, vec![10, 12], "the first key-7 and the lone key-9 keep ext[0] = 0");
    }

    /// Three records on one key chain: the third pairs with the second (the still-0x33 master)
    /// — the newest-first object walk [M `control_add_object_to_list` 0x134698 prepends].
    #[test]
    fn lightning_triple_key_chains_through_the_still_live_master() {
        let p = |x: f32| [x, 0.0, 0.0];
        let recs: Vec<(usize, u32, i16, u16, V3)> = (0..3).map(|i| (i, 0u32, 5i16, 0u16, p(i as f32))).collect();
        let (strikes, inert) = pair_lightning_recs(&recs);
        assert_eq!(strikes.iter().map(|s| (s.master_placement, s.partner_placement)).collect::<Vec<_>>(), vec![(1, 0), (2, 1)]);
        assert_eq!(inert, vec![0], "only the first creation no-ops");
    }

    /// The key compares as a SIGNED short [M 0x18cde8:40 `(short)` cast]: 0xffff (−1) never
    /// pairs with 0x7fff, 0x8000 (−32768) only with itself.
    #[test]
    fn lightning_key_compare_is_signed() {
        let recs = vec![(0usize, 0u32, -1i16, 0u16, [0.0; 3]), (1, 0, 0x7fff, 0, [1.0; 3]), (2, 0, -32768i16, 0, [2.0; 3])];
        let (strikes, inert) = pair_lightning_recs(&recs);
        assert!(strikes.is_empty(), "0xffff/0x7fff/0x8000 are three distinct keys");
        assert_eq!(inert, vec![0, 1, 2]);
        let recs = vec![(0usize, 0u32, -32768i16, 0u16, [0.0; 3]), (1, 0, -32768i16, 0, [1.0; 3])];
        let (strikes, _) = pair_lightning_recs(&recs);
        assert_eq!(strikes.len(), 1, "0x8000 pairs with 0x8000");
    }

    /// `Lightning_Update` 0x18cf68: hidden while `SuperHappyLightningFlag` is clear; the
    /// reflicker consumption + `FractalThing` redistribution on the first drawn flash tick;
    /// plain flashes keep the shape.
    #[test]
    fn lightning_visibility_and_reflicker_redistribution() {
        let (start, end) = ([0.0f32, 0.0, 0.0], [100.0f32, 0.0, 0.0]);
        let mut s = LightningStrike {
            master_placement: 1, partner_placement: 0, group: 0, key: 0,
            start, end, nodes: [start, start, start, start, start, start, start, end],
            visible: false, counter: 0,
        };
        let mut g = crate::sky::LightningGlobals::default();
        s.tick(&mut g);
        assert!(!s.visible, "flag clear → the 8 × SetDrawInNoViews arm [M 0x18cf68:13-21]");
        g.super_happy = true;
        g.reflicker = true;
        s.tick(&mut g);
        assert!(s.visible, "the SuperHappyLightningFlag arm draws the chain [M 0x18cf68:27-42]");
        assert!(!g.reflicker, "the reflicker bit is consumed [M 0x18cf68:24]");
        assert_eq!(s.nodes[0], start, "node 0 (the master) keeps the start — no recursion range covers index 0");
        assert!(s.nodes[1] != start, "the fractal moved the beams");
        let after = s.nodes;
        g.reflicker = false;
        s.tick(&mut g);
        assert!(s.visible && s.nodes == after, "a plain flash tick redraws without reshaping");
        g.reflicker = true;
        s.tick(&mut g);
        assert!(s.nodes != after, "the next reflicker flash redistributes again");
    }

    /// `FractalThing` 0x1a94d8 with a mid-returning rand (offset 0): the mid sequence
    /// (0,8)→4, (0,4)→2, (0,2)→1, (2,4)→3, (4,8)→6, (4,6)→5, (6,8)→7 [M 0x1a94d8:66/92-93]
    /// and every node the exact midpoint of its range — 3 draws per node [M :78-87].
    #[test]
    fn fractal_thing_recursion_shape_and_draw_count() {
        let (start, end) = ([0.0f32, 0.0, 0.0], [8.0f32, 0.0, 0.0]);
        let mut nodes = [[0.0f32; 3]; 8];
        let mut mids = Vec::new();
        fractal_thing(&mut nodes, 0, 8, start, end, 0, &mut |amp| { mids.push(amp); amp * 0.5 });
        assert_eq!(mids.len(), 21, "7 nodes × 3 axis draws each [M 0x1a94d8:78-87]");
        assert!(mids[..3].iter().all(|a| (*a - 4.0).abs() < 1e-4), "the (0,8) range: distance 8 → amp 4 in mode 0");
        assert_eq!(nodes[1], [1.0, 0.0, 0.0], "mid of (start, node2)");
        assert_eq!(nodes[2], [2.0, 0.0, 0.0], "mid of (start, node4)");
        assert_eq!(nodes[3], [3.0, 0.0, 0.0], "mid of (node2, node4)");
        assert_eq!(nodes[4], [4.0, 0.0, 0.0], "mid of (start, end)");
        assert_eq!(nodes[5], [5.0, 0.0, 0.0], "mid of (node4, node6)");
        assert_eq!(nodes[6], [6.0, 0.0, 0.0], "mid of (node4, end)");
        assert_eq!(nodes[7], [7.0, 0.0, 0.0], "mid of (node6, end)");
        assert_eq!(nodes[0], [0.0, 0.0, 0.0], "the start keeps its position");
    }
}


#[cfg(test)]
mod leaf_tests {
    use super::*;
    use crate::drone::Rng;

    // [M 0x1abee0] a live leaf falls, drifts and spins; the two life limits both kill it.
    #[test]
    fn leaf_falls_and_expires_through_either_limit() {
        let mut rng = Rng::new(3);
        let mut leaf = LeafFx {
            pos: [0.0, 10.0, 0.0], rot: [0.0; 3], vel: [0.0, 0.0, 0.0], spin: [0.01, 0.0, 0.0],
            frames_left: 300, budget: 200, tick_cost: 1, counter: 0,
        };
        let y0 = leaf.pos[1];
        for _ in 0..50 {
            assert!(leaf.tick(&mut rng));
        }
        assert!(leaf.pos[1] < y0, "the leaf falls: y = {} < {}", leaf.pos[1], y0);
        assert_ne!(leaf.rot[0], 0.0, "the spin advanced the rotation");
        assert!(leaf.vel[0] != 0.0, "the x drift accumulated");
        // The frame limit: 300 frames minus the 50 lived — still alive at 250 left.
        assert_eq!(leaf.frames_left, 250);
        // Sister (can fail): the frame limit is a hard kill at < 1, not at 0 ticks left.
        let mut dying = LeafFx { frames_left: 1, ..leaf.clone() };
        assert!(dying.tick(&mut rng));
        assert!(!dying.tick(&mut rng), "frames_left hit 0 — the leaf dies");
        // The budget limit: cost 2 with budget 3 dies after one tick.
        let mut poor = LeafFx { frames_left: 300, budget: 3, tick_cost: 2, ..leaf.clone() };
        assert!(poor.tick(&mut rng));
        assert!(!poor.tick(&mut rng), "the tick budget ran out");
    }

    // [M 0x1abee0] the same seed reproduces the same fall (the shared-RNG discipline).
    #[test]
    fn leaf_motion_is_bit_exact_per_seed() {
        let run = |seed: u64| {
            let mut rng = Rng::new(seed as u32);
            let mut leaf = LeafFx {
                pos: [1.0, 5.0, 2.0], rot: [0.0; 3], vel: [0.0; 3], spin: [0.0; 3],
                frames_left: 30, budget: 255, tick_cost: 1, counter: 0,
            };
            for _ in 0..30 { leaf.tick(&mut rng); }
            leaf
        };
        assert_eq!(run(9), run(9));
        // Sister (can fail): a different seed moves the leaf differently.
        assert_ne!(run(9).pos, run(10).pos);
    }
}
