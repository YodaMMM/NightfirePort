//! CPU side of the viewer: a level archive turned into vertex/index buffers grouped by layer and material, plus the
//! per-vertex skinning data that animates characters each frame.
//!
//! Material = (map, texture index) — a batch's texture index refers to the texture list of the map its object comes
//! from (docs/formats/ps2gfx.md). Vertex colour stays on the PS2 scale (0x80 = 1.0); the shader rescales it.
//! Layers follow the game's frame: the sky pass first (`View_DrawSky`, objects centred on the viewer, then fixed sky
//! objects), then the world; characters are drawn like the world but their vertices sit at the end of the buffer and
//! are rewritten every frame (docs/formats/animation.md).

use nf_formats::anim::{self, Mat4, Script, Sequence};
use nf_formats::character::{Skeleton, Skin};
use nf_formats::drone::{self, DroneAnimData};
use nf_formats::level::{Instance, Level};
use nf_formats::placement::transform;

/// weapon-matrix4: the clip view `Player_AmmoInGun` (0x1a9190) reads. [M decompile 001a9190]: a
/// twin record (`weapon_data[w].+5 != 0`) whose ammo type matches its base (`+0x90`) reads the
/// BASE's slot (`iVar2 = weapon_data[w].+2` unless the types differ) — one physical magazine per
/// family, the same slot `Player_EquipWeapon` 0x1a8a58's give path fills and
/// `Player_ReloadAmmoType` 0x1a1988:18-22 reloads (the twin clip-slot unification). The port's
/// `Gun::has_ammo` read the twin's OWN slot — which no game write path ever fills — so a twin
/// with a full shared magazine read as dry and every `Player_WeaponHasAmmo` (0x1a16c0) check
/// refused it: the Shaver twins 0x5a/0x5c (ammo type 30's carried cap is 0 [D ammo_data], the
/// shared clip[0x59] = 500 after the give) were unreachable by the mode press in the weapon
/// matrix where the game reaches them. This is the read-side of the same unification; the
/// runtime boundary sites are fix-reload-hang's (in flight on its own branch).
fn clip_view(w: &nf_formats::weapon::WeaponData, clip: &[i16]) -> Vec<i16> {
    let mut v = clip.to_vec();
    for id in 0..v.len() {
        let base = w.base(id) as usize;
        if base < v.len() && base != id && w.shares_base_clip(id) && w.ammo_type(id) == w.ammo_type(base) {
            v[id] = v[base];
        }
    }
    v
}
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Per-weapon op-1 sound id remap (docs/systems/sound.md §7 line 313): the first-person gun's animation script
/// emits a sound id that the game replaces based on the player's weapon (`+0xDC → +0x62`). The P2K (weapon 2) has no
/// remap — 0x403 plays as-is. Other weapons' fire sounds are remapped so they do not all sound like the P2K [M].
pub(crate) fn remap_op1_sound(id: u32, weapon: u8) -> u32 {
    match (id, weapon) {
        (1, 7) | (1, 9) => 2,
        (0x403, 3) | (0x403, 5) => 2,
        (0xfa, 0x14) => 0x1fb,
        (0x106, 0x1e..=0x23) => 0x284,
        (0x1e1, 0x10) => 0x602,
        (4, 0x16) | (4, 0x17) => 0x604,
        _ => id,
    }
}

/// Test export of `remap_op1_sound` (the `map_analog_stick_for_test` pattern).
pub fn remap_op1_sound_for_test(id: u32, weapon: u8) -> u32 { remap_op1_sound(id, weapon) }

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub rgba: [u8; 4],
    /// The vertex normal (snorm, xyz + 0): world space for static World geometry, object/bone space for
    /// everything the palette poses (the shader rotates those). Used by the dynamic lights (fix-lighting).
    pub nrm: [i8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    /// Sky objects with glist flag bit 2: drawn around the viewer (vertices in object space, rotation only).
    SkyCamera,
    /// Sky objects at their placement translation, drawn in the sky pass (no depth).
    SkyFixed,
    World,
    /// Skinned character meshes; vertices rewritten each frame.
    Character,
    /// Additive effects posed with the characters (muzzle flashes); after Character so the rewritten vertex tail
    /// stays contiguous.
    Effect,
}

impl Layer {
    /// The renderer's per-draw frustum-cull rule (gpu.rs's `cull` closure), spelled out where the
    /// layer semantics live so the live tests can pin it without a GPU.
    ///
    /// The static World bulk always culls (its bounding spheres are exact — fix-render2/fix-render-artifacts).
    /// The EFFECT pools never cull, `NF_CULL` or not: the game culls these objects per frame at their
    /// LIVE matrix (`View_AddObjects` 0x1e62d8 passes obj+0x8c sphere AND obj+0x80 bbox to
    /// `Vision_InView` 0x1e89e0`), but the port's Effect groups are merged pools posed per frame
    /// through attachment matrices — their build-time sphere is the IDENTITY-matrix centre, for the
    /// gas pool the `BodyHit` mesh's obj.centre ≈ the world origin with radius 0.032
    /// [D 07000005 map 10 obj 41]. Culling on that sphere drops every puff the moment the origin
    /// leaves the frustum, no matter where the puffs are posed — the blood-puff-render task's
    /// "the gas matrix branch runs, nothing draws". (A-11's vanish bug was the same unsoundness
    /// with the truck.) Character keeps its build-time placement centre (real positions) so the
    /// NF_CULL bisect override still exercises it.
    pub fn frustum_cullable(self, test_cull: bool) -> bool {
        match self {
            Layer::World => true,
            Layer::Effect => false,
            _ => test_cull,
        }
    }
}

/// How a material is drawn (docs/systems/render.md): the pass (0 sky before the world, fogged; 1 world list 0;
/// 2 characters; 3 world list 1 — glist flag 0x1, alpha list, faded; 4 sky after the fog pass; 5 effects), the GS
/// blend from ALPHA_1 (0 normal (Cs − Cd)·As + Cd, 1 additive Cs·As + Cd, 2 subtractive Cd − Cs·As, 3 keep-destination
/// — the FIX=0 mod batches, 4 additive by destination alpha Cs·Ad + Cd — the env-map overlays), depth write
/// (ZBUF_1 ZMSK clear), the alpha test from TEST_1 (0 none, 1 As ≥ 1, 2 As > 0x7F) and the fog (0 none, 1 depth fog,
/// 2 alpha fade with distance).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Draw { pub pass: u8, pub blend: u8, pub zwrite: bool, pub atest: u8, pub fog: u8 }

impl Draw {
    /// From a leaf's GS registers (ALPHA_1, TEST_1, ZBUF_1 as last written).
    ///
    /// ALPHA_1 fields (ps2tek GS register spec, *sourced*): Output = ((A − B)·C >> 7) + D with
    /// A/B/D ∈ {0: Cs, 1: Cd, 2: 0} and C ∈ {0: As, 1: Ad, 2: FIX} (FIX = bits 8-15 of the register).
    /// *Measured* over every DIRECT A+D write of all 27 on-foot levels: only ALPHA_1 (0x42), TEST_1
    /// (0x47) and ZBUF_1 (0x4e) ever occur, so CLAMP/filtering are global texture-upload state, not
    /// per-object. The disc carries exactly six A/B/C/D shapes:
    /// - 0x44 (0,1,0): (Cs−Cd)·As+Cd → blend 0 (normal) — 179,862 writes
    /// - 0x48 (0,2,0): (Cs−0)·As+Cd → blend 1 (additive) — 28,925 writes
    /// - 0x42 (2,0,0): (0−Cs)·As+Cd → blend 2 (subtractive) — 312 writes
    /// - 0x58 (0,2,1): (Cs−0)·Ad+Cd → blend 4 (additive gated by the framebuffer's alpha — the
    ///   "EnvModTest_*" env-map overlay of 07000041 adds only where the base batch wrote alpha) — 159 writes
    /// - 0x64 (0,1,2, FIX=0): (Cs−Cd)·0+Cd → blend 3 (keep-destination; the GS still depth-tests and
    ///   alpha-tests) — 159 writes
    /// Nothing in the load/draw path patches FIX (psiCreateEntityGfx 0x1171b0 rewrites only texture
    /// REF addresses; DrawThisBox 0x1230d8 only kicks the packet), so the disc values are final [M].
    pub fn from_gs(pass: u8, fog: u8, alpha: u64, test: u64, zbuf: u64) -> Draw {
        let lo = alpha & 0xff;
        let (a, b, c, fix) = (lo & 3, (lo >> 2) & 3, (lo >> 4) & 3, (alpha >> 8) & 0xff);
        let blend = match (a, b, c, fix) {
            (0, 1, 0, _) => 0, // (Cs−Cd)·As+Cd — normal
            (0, 2, 0, _) => 1, // (Cs−0)·As+Cd — additive
            (2, 0, 0, _) => 2, // (0−Cs)·As+Cd — subtractive
            (0, 2, 1, _) => 4, // (Cs−0)·Ad+Cd — additive by destination alpha (07000041 env maps)
            // C = FIX 0: the equation collapses to +D (0x64 → Cd). Paint nothing; the GS still runs
            // the depth test/write and the alpha test on these batches.
            (_, _, 2, 0) => 3,
            _ => 0, // no other shape occurs on the disc
        };
        // TEST_1: bit 0 ATE; bits 1-3 ATST (5=GEQUAL, 6=GREATER on the disc); bits 4-11 AREF.
        // GEQUAL 0 and ALWAYS pass everything — no test (the port used to reject a = 0 texels there,
        // which also skips their depth write, a behaviour the game only has for AFAIL ≠ 0).
        let atest = if test & 1 == 0 { 0 } else {
            let (atst, aref) = ((test >> 1) & 7, (test >> 4) & 0xff);
            match atst {
                0 | 1 => 0,
                5 => match aref { 0 => 0, 1 => 1, _ => 2 },
                6 => if aref >= 0x7f { 2 } else { 1 },
                _ => 1,
            }
        };
        Draw { pass, blend, zwrite: zbuf >> 32 & 1 == 0, atest, fog }
    }
}

/// fix-0c-pylon probe: `Clone` so headless render tests can stage material subsets per render
/// (the pylon-only isolation render); nothing else changes.
#[derive(Clone)]
pub struct Material {
    pub layer: Layer,
    pub draw: Draw,
    /// The object's world centre (draw-order sort).
    pub centre: [f32; 3],
    /// The object's world-space bounding-sphere radius (the glist radius scaled by the instance matrix). Used by the
    /// CPU-side frustum cull (A-11e): a draw is skipped when its sphere is fully outside the view frustum.
    pub radius: f32,
    // unported-misc-5 — the game's render-build layer, folded into the scene's owned structures:
    // `hashtable_create_sections` 0x1ca2a8 / `hashtable_initialise` 0x1ca4b8 (map load:
    // `ResetMap_Load` 0x1d6360, `LoadableLoad` 0x1d69e8, `LoadableReload` 0x1d6ab0) build the cel
    // hashtable the streamer walks; `hashtable_deleteitem` 0x1ca1b0 (`Script_FreeFromHashTable`
    // 0x1daec0, `hashtable_delete_hashtype` 0x1ca520) removes entries. The port's `CelGraph`-keyed
    // draw lists and hash maps carry the same (cel, glist) → data relation without the sectioned
    // table. The viewer/world glue: `build_alloc_viewer` 0x1c9378 (Mem_Malloc 0x240 + the +0x28
    // id + `Camera_Reset` 0x1b9af0) and `build_link_world_to_viewer` 0x1c94d8 (`Camera_Create`
    // 0x1bb0f8's world→viewer link) are this struct's camera + `world` field setup at load;
    // `build_alloc_world` 0x1c93c8 (4-byte world holder, `Camera_CreateCameras` 0x1bb2b8 +
    // `ResetMap_GameInit` 0x1d5080) is the `world` member itself. `build_MakeCelOverlapBox`
    // 0x1c94f8 is EMPTY in the game (8-byte `jr ra`, called by `ResetMap_Load` 0x1d6360) — a no-op
    // the port needs no trace of. `Stack_Init` 0x1cce38 (`Camera_Reset`'s per-viewer stack, and
    // `MenuManager_Create` 0x21b908's) is the plain Vec the camera/menu state grows here and in
    // menu.rs (see the GameFlow note there).
    /// The room cel this static draw sits in — `build_FindCel` (0x1c8cb0) over the placement's world
    /// centre, the same lookup `build_LinkToRoom` (0x1c9680) uses to link the game's objects to their
    /// cel. `None` = no room cel contains it (outdoor/uncel'd geometry — never cel-culled).
    /// `View_AddCels` (0x1e6ba0) frustum-tests each cel's bbox (`Vision_InView`); fix-render3 skips a
    /// World draw whose cel fails that test.
    pub cel: Option<u16>,
    /// The list-0 sort key source. `View_AddCels` (0x1e6ba0, disassembled) pushes the −90000 + 10k
    /// keys on the **+0x20 chain** `Vision_Portal_Recurse` (0x1e86e8) threads per frame from
    /// `VisCelList` (0x34b3b0) — `QuickSort` (0x1d30a8) comparator[3] @0x1d3430 orders it by the
    /// ascending u16 portal-recursion depth at element+4 — and only the **room cels** (placement
    /// flag words 0xc023/0xc047) ride it [L ee_start_05.bin: 19 rooms + 113 inner cels]. The cel
    /// `+0x18` inner nodes — every non-room 0x8000 placement (0x8019/0x8018 marker classes,
    /// `parseentity_fixup_entity` 0x1d00c0) — get **±dist²** keys (list by flags bit 0x1/0x2000).
    /// So `chain` = the room's fallback ordinal (its rank among the room placements in parse
    /// order, used when the per-frame walk cannot run); `u32::MAX` = a distance-keyed entry (the
    /// inner-node statics: the baked decals/walkways/blockers, and the dynamic objects —
    /// `View_AddObjects` 0x1e62d8 pushes `{±dist², obj, 0}`).
    pub chain: u32,
    /// The draw's own runtime cel in the `CelGraph::cells` sense — the placement's pCelList record
    /// identity — for the room-class placements whose primary glist rides the −90000 + 10k chain;
    /// `u32::MAX` = a distance-keyed entry (never looked up in the per-frame chain). The renderer
    /// maps `node` → the cel's position in the depth-sorted `VisCelList` chain each frame
    /// (`gpu::render`); `chain` is the fallback ordinal when the walk cannot run.
    pub node: u32,
    /// `(map, texture)`; `None` = untextured.
    pub key: Option<(usize, u32)>,
    /// The draw's light-selection flags: the low 16 bits of the source object's glist flags word
    /// (`MapObject.flags` = `celglist+0x44`, which `Control_SetGList` 0x134960 copies to `obj+0xF0`;
    /// cels: the same word lands in `cel+0x94` — `lights.rs`' `closest` reads bit 0x80 for kinds 0/2
    /// and bit 0x100 to skip kind 1). fix-lighting2: the port used to pass 0x80 for every draw, which
    /// let 07000005's kind-0 ambient radiators crowd the muzzle light off the world draws — the disc
    /// data has almost no 0x80/0x100 on world cels, 0x8a0/0x28a0 on character/weapon skins, 0x100 on
    /// effect objects [D probe in lighting_live.rs].
    pub lflags: u16,
    /// The skinned owner of a Character/Effect draw: (character index, attachment slot) — the renderer
    /// re-centres the draw's light pick on the owner's live palette root every frame (the game's
    /// obj+0x80 world sphere follows the object). `None` = a static draw (its `centre` stands).
    pub skin: Option<(u32, Option<u32>)>,
    pub first_index: u32,
    pub index_count: u32,
}

/// fix-menu-movies: `Clone` so the front end can keep its own texture list (the scene's entries
/// plus the background-movie frame slot) without re-parsing the archive.
#[derive(Clone)]
pub struct TextureImage {
    pub key: (usize, u32),
    pub width: u32,
    pub height: u32,
    /// Frame 0.
    pub rgba: Vec<u8>,
    /// Animated textures: frames 1.. and the ticks each frame shows (`CacheTexture` 0x11cd58: frame =
    /// (`GameState+0x38` / rate) % frames, `rate` in ticks (`ticks_per_frame`); the counter gains `VIDEO_FRAME_RATE / FRAME_RATE_INT` = 60 / 60 per game
    /// tick [M, L]).
    pub more: Vec<Vec<u8>>,
    pub rate: u32,
}

/// `psiCreateMapTextures` (0x11d200): the header's rate byte is frames per second; 1–59 become `60 / rate` ticks per
/// frame, anything else 1 [M].
pub fn ticks_per_frame(rate: u8) -> u32 { if (1..60).contains(&rate) { 60 / rate as u32 } else { 1 } }

impl TextureImage {
    /// Map `mi`'s texture `t`, every frame.
    pub fn from_texture(mi: usize, t: &nf_formats::texture::Texture) -> TextureImage {
        let pal = t.palette_rgba8();
        let rgba = |f: usize| t.frame(f).iter().flat_map(|&i| pal[i as usize]).collect::<Vec<u8>>();
        TextureImage { key: (mi, t.index as u32), width: t.width as u32, height: t.height as u32, rgba: rgba(0),
            more: (1..t.frames.max(1) as usize).map(rgba).collect(), rate: ticks_per_frame(t.rate) }
    }
}

/// One drawn character mesh: its placement matrix, skin, skeleton and slot → bone list, the animation the game starts
/// it with, and the floor straight below it.
pub struct Character {
    pub matrix: Mat4,
    pub skin: Skin,
    pub skeleton: Skeleton,
    pub slot_bones: Vec<u8>,
    pub start: Option<StartAnim>,
    pub role: drone::SpawnRole,
    /// The spawn-floor seat: the game's own downward floor probe — `build_PointOnFloor` 0x1c95a8
    /// (a (0,−1,0) collision ray) scoped to the containing cel. A player start snaps
    /// y = floor + 1.1 (`Player_StandAtNewPosition` 0x190d88:104-114); a drone keeps its
    /// placement y and settles per tick onto floor + hip (`Drone_FeetOnPoint` 0x140660,
    /// `Drone_CollisionHandler` 0x140968:292-295, 1/8 of the error per tick). The port's
    /// character setup takes the highest World triangle below the placement in the
    /// [py−3.0, py+0.5] band as that floor (the scoped-cel probe's end state, unscoped — the
    /// band matches the player probe's ±3/+0.5 shape).
    pub floor: Option<f32>,
    /// A script's animated object (script hash, stream): drawn while the simulation has it, posed with its root in world
    /// space (docs/systems/script-player.md).
    pub script: Option<(u32, u8)>,
    /// fix-facial2: the object's facial morph TARGET array (the MORPH_DATA block 0x2B payload — the
    /// runtime gfx record's +0x14 base [M `parsemap_block_morph_data` 0x1d23e8]), f32[3] per
    /// (vertex morph index + channel). `SkinIt` 0x10bf30 adds Σ weight[m] × target[morph + m].
    pub morph_targets: Option<Vec<u8>>,
}

/// The script a drone starts with (`NDrone2_DefaultInit` → `DroneAnim_SetAnimD`, or placement parameter 1) and the
/// sequences its opcode-0 commands play: `(first script frame, last script frame, index into the skeleton's list)`.
pub struct StartAnim {
    pub script: Script,
    pub sequences: Vec<(u16, u16, usize)>,
}

/// What the drone setup needs besides the level: ACTION.ELF's tables, the level's scripts, the level id, and
/// `weapon_data` (built by running its static initialiser).
pub struct DroneContext {
    pub tables: DroneAnimData,
    pub scripts: HashMap<u32, Script>,
    pub level_id: u32,
    /// net-match-host: the online match's lockstep seed (0 = the pre-net constant, local play).
    pub mp_seed: u32,
    /// The MP bodies: one skin hashcode per player slot (0..4 humans, 4..8 bots; 0 = none) —
    /// `nf_game::mp::slot_skin` of the match settings.
    pub mp_body_skins: [u32; nf_net::MAX_AGENTS],
    pub weapons: Option<nf_formats::weapon::WeaponData>,
    pub props: Option<nf_formats::drone::PropertyTables>,
    /// The level's tuning (`ReadTuningVars` applied), hit boxes and sound hearing data.
    pub tuning: nf_formats::tuning::Tuning,
    pub hitboxes: Option<nf_formats::hitbox::CollisionBones>,
    pub sfx: Option<nf_formats::sfx::SfxTable>,
    pub surfaces: Option<nf_formats::sfx::SurfaceSounds>,
    pub voices: Option<Vec<nf_formats::sfx::VoiceLine>>,
    /// The HUD's sprite tables and font (docs/systems/hud.md).
    pub hud: Option<nf_formats::hud::HudTables>,
    /// mp-character-data: the `MP_skins` per-character table (ACTION.ELF 0x26d488, 29 × 0x10) [D].
    /// Only the MP body build and the pose consume it.
    pub mp_skins: Option<nf_formats::mpanim::MpSkins>,
    /// mp-character-data: the 16 animation-set descriptors (ACTION.ELF 0x2b91b0..0x2b9430) [D] —
    /// the ids `mp_body_pose` returns.
    pub mp_anims: Option<nf_formats::mpanim::MpAnimSets>,
    /// pickup-medal-gate: the game's `bonus[0]` word (0x2e1b88, the player-0 u64 the save's BUNS
    /// block carries [M LS_LoadBonus 0x1ce488 → Menu_SetBonus(…, 0)]) — the state
    /// `PlrStats_HasGoldMedal` 0x199a40 reads for `Pickup_Create`'s kind-6 gate [M 0x18dda0:93-95].
    /// The front end passes the active profile's word; 0 = the fresh-boot BSS state [L ee_start_05
    /// 0x2e1b88 all zero].
    pub bonus: u64,
}

impl DroneContext {
    /// `Mission_BaseMapHCode()` 0x1830a0 + `PlrStats_HasGoldMedal` 0x199a40 for the build's level:
    /// the `LEVELS` row's `mission` hash for `level_id` — the same lookup `Mission_Init` 0x182270
    /// runs over the game's 24-record mission table (the port's `crate::mission::LEVELS` [L
    /// ee_start_05.bin 0x2a4350], SP arm `MPSettings+0x384 == 0` → `BaseMap` [M 0x1830a0:3-5]) —
    /// then the gold bit over `bonus`. A level with no row (the front end 0x07000048, the MP maps)
    /// plays BaseMap −1 → no RewardsTable row → no medal → the gate stays shut, as the game's.
    pub fn has_gold_medal(&self, level_id: u32) -> bool {
        match nf_game::mission::LEVELS.iter().find(|l| l.level == level_id) {
            Some(l) => nf_game::progress::has_gold_medal(self.bonus, l.mission),
            None => false,
        }
    }
}

/// mp-combat-weapons: one released round as the local fire pipeline produced it — what the
/// session input's shot description carries (`nf_net::Input::shot_weapon/shot_flags/shot_aim`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NetShot {
    /// The fired record (`gun.weapon` at the release — the twin id carries the fire mode).
    pub weapon: u8,
    /// The release's aim ray (free aim, auto-aim, the scope included).
    pub dir: [f32; 3],
    /// Fired through a zoomed scope (`aiming ∧ +0x68 & 0x40`).
    pub scoped: bool,
}

/// The running game simulation (nf-game): drones plus the data they consult, and which character mesh shows which
/// drone. Stepped at the game's fixed 60 Hz by the app; `animate` reads it.
/// fix-weapon-fire-modes: the guided-missile camera ride (`Player_PositionCamera` case 0xc's view
/// state + `Player_Update` substate 10's restore countdown, on the port's Sim). `pos`/`dir` track
/// the live missile (`BLData+0x878`'s object); on its death `restore` counts the 15 ticks of
/// `BLData+0x944` down with the view frozen (the red-static period) [M 0x12e660:26-30, 0x193600:242-256].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MissileCam {
    /// The missile's last position (the camera sits at it [M 0x18f5d8 case 0xc]).
    pub pos: [f32; 3],
    /// The missile's facing (the view matrix = the missile's [M 0x18f5d8 case 0xc]).
    pub dir: [f32; 3],
    /// `BLData+0x944`: the post-impact static countdown [M `Bullet_Delete` 0x12e660:28 = 0xf].
    pub restore: u16,
    /// A live guided bullet still exists (`BLData+0x878 != 0`).
    pub riding: bool,
}

/// mp-perf-16: a debug env flag read once per process (the per-body / per-bot / per-tick reads
/// were a `GetEnvironmentVariableW` round trip each, every tick).
static ENV_MPBODY_DBG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
static ENV_BOTLOG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
fn env_flag(cell: &'static std::sync::OnceLock<bool>, name: &str) -> bool {
    *cell.get_or_init(|| std::env::var_os(name).is_some())
}

pub struct Sim {
    pub drones: nf_game::drone::Drones,
    pub tables: DroneAnimData,
    pub props: nf_formats::drone::PropertyTables,
    pub scripts: HashMap<u32, Script>,
    pub weapons: nf_formats::weapon::WeaponData,
    pub world: nf_game::world::CollisionWorld,
    pub level: u32,
    /// Per character: its drone.
    pub char_drone: Vec<Option<usize>>,
    /// Per character: its script stream when the character is a script object (`Scene::Character::script`).
    pub char_script: Vec<Option<(u32, u8)>>,
    /// Per character: its skin hashcode (for `characters_script_pos`'s entity lookup).
    pub char_script_skin: Vec<Option<u32>>,
    /// Sequence hashcode → (skeleton, index in `Scene::sequences`).
    pub seq_index: HashMap<u32, (u8, usize)>,
    /// The same sequences by hashcode (root motion in the AI).
    pub seqs: HashMap<u32, Sequence>,
    /// Loaded skin assets by hashcode. NOT the game's `SkinCache` — that is a 128-entry × 24 B
    /// pool at 0x246350 zeroed per level by `InitCacheSkin` 0x111ad0 (psiPreGameRun 0x11e2e8),
    /// aged per frame by `AgeSkinCache` 0x111b08 (psiPreDraw 0x121590: each +0x14 age drops
    /// from 3, both Mem_Malloc'd buffers freed at 0; `CacheSkin` 0x1112c0 refreshes age-3 on a
    /// hit) — short-lived DMA-upload buffers for skins drawn this frame [M decomp]. This map is
    /// the port's persistent decode cache, a different job.
    pub skins: HashMap<u32, Skin>,
    pub skeletons: HashMap<u8, Skeleton>,
    pub nav: nf_game::nav::Network,
    pub covers: Vec<nf_game::cover::CoverNode>,
    /// AI points (placements 0xE9): the guards'/guests' run-to points (`Drone_AIPoint` 0x137550).
    pub ai_points: Vec<nf_game::aipoint::AiPoint>,
    pub tuning: nf_formats::tuning::Tuning,
    /// mp-core: the MP match core (`nf_game::mp`) when `--mp` runs an arena — `MP_Update` 0x184bd0
    /// runs per frame in gameplay [M]; `None` on every campaign level.
    pub mp: Option<nf_game::mp::MpMatch>,
    pub hitboxes: nf_formats::hitbox::CollisionBones,
    pub sfx: Option<nf_formats::sfx::SfxTable>,
    pub volumes: Vec<nf_game::aivolume::AiVolume>,
    /// fix-script-objects: the glist bounds (min, max) by hashcode — the script-created AI
    /// volumes' half-extents resolve here (`Drone_AIVolume_Create` 0x1376c0:84-92: |max − min|/2
    /// from the glist's min +0x20 / max +0x30, × the scale triple — the script arm's scales are
    /// {1,1,1} [M 0x1d8a58]).
    pub glist_bounds: HashMap<u32, ([f32; 3], [f32; 3])>,
    pub models: HashMap<u32, nf_game::pickup::Model>,
    /// Bond's gun (07000005's start: P2K, 7 + 41 rounds) and the auto-aim option (`PlayerSetting+1` = 1 in the game
    /// [L]; off by default with a mouse, D-001).
    pub gun: nf_game::gun::Gun,
    /// Bond on foot (docs/systems/player.md).
    pub bond: nf_game::player::Bond,
    /// net-match-host: the other human agents' bonds in an online match, by slot (the local
    /// slot's entry stays `None` — its bond IS `bond`). Port architecture (documented, not a
    /// game-behaviour claim): the PS2 game's four join panels are four `Player_Init` objects in
    /// ONE machine (split-screen [M mp-players.md — every body's `Player_Init` 0x1910f8 runs on
    /// the console]); the port is a network of full machines, so it keeps one full local machine
    /// (the weapons/view path) and ticks the remotes' movement through the same
    /// `Bond::tick_ctx`.
    pub mp_bonds: [Option<nf_game::player::Bond>; nf_net::MAX_AGENTS],
    /// The slot this machine plays (its body is the camera, never drawn) — `swap_local_slot` sets it.
    pub mp_local_slot: usize,
    /// Draw-only: each MP slot's horizontal speed (units/s) over the last tick, from its position
    /// change — the body's idle/walk/run pick (never hashed, never read by the sim).
    pub mp_body_speed: [f32; nf_net::MAX_AGENTS],
    /// Online (700v6): this machine's PRIVATE generator. In an online match the shared bit-exact
    /// stream (`drones.rng`, hashed) is drawn only by the parts every machine runs identically —
    /// the MP rules, the bots and the per-slot MP combat; the local player's own tick (view model,
    /// its bullets' spread / impacts / effects) runs on this one, so one machine's firing can no
    /// longer desync the match. Documented port design (not a game-behaviour guess): the PS2
    /// game's MP is local split-screen only — one machine drew every stream [M mp-combat.md "The
    /// split"]. None = offline.
    pub net_rng: Option<nf_game::drone::Rng>,
    net_rng_shared: Option<nf_game::drone::Rng>,
    /// Online combat (700v6): each human slot's health, and the ticks since its death (0 = alive).
    pub mp_hp: [f32; nf_net::MAX_AGENTS],
    /// The bots' pickup table (`MP_PICKUP` rows the goal picker scores [M BOTSTATE_pickGoal
    /// 0x126b88]): the arena's weapon/ammo/armour pickups, with the per-bot visit stamps the
    /// driver mirrors from `last_visit_stamp` (blocked 704's item 1 — the driver passed none, so
    /// a bot with nobody in sight never left its spawn).
    pub mp_bot_pickups: Vec<nf_game::mp::bots::BotPickup>,
    /// mp-bot-weapons: the bots' `weapon_data` lift (`botweap::BotWeaponTable` — the disc's table
    /// and which first-person skins this arena loaded), built on the first MP tick. Same disc and
    /// arena on every machine → the same table.
    pub mp_bot_weapons: Option<nf_game::mp::botweap::BotWeaponTable>,
    pub mp_dead_ticks: [u32; nf_net::MAX_AGENTS],
    /// mp-world-sync (agent 9): the session's input row for the tick `step_bond` is about to run
    /// (the caller sets it — the live loop and `nf_view::net_tick`; `step_bond` consumes it). With
    /// it, `step_bond` moves the remote bodies BEFORE the local body (the same collision world for
    /// every body — the doors and script objects join it at the drones' tick), the world's contact
    /// passes see every body, and every slot's use key and shot reach the shared world.
    pub net_row: Option<[nf_net::Input; nf_net::MAX_AGENTS]>,
    /// mp-world-sync: `step_bond` already moved the remotes this tick (the callers' later
    /// `step_net_remotes` call is then a no-op).
    net_remotes_done: bool,
    /// mp-world-sync: each human slot's armour (`BLData+0x8B0`) — the armour pickup's shared
    /// state (set to 50 on the take [M Pickup_Handler 0x18e650:220-225], cleared on death).
    pub mp_armour: [f32; nf_net::MAX_AGENTS],
    /// mp-world-sync: the latest online tick's pickup takes (pickup index, slot, bot) — what
    /// `step_net_world` applied (the tests and the dev log read it; never hashed).
    pub net_last_takes: Vec<(usize, usize, bool)>,
    /// door-react-path4: the previous tick's raw use key — `Player_Activate` fires on the action
    /// bits' PRESS EDGE (`Input_Action(player, 9, 1)` | `Input_Action(player, 0xe, 4)` [M
    /// 0x1a5460:395-403]; the action machine 0x17ee88 latches per press), so a held E activates
    /// the world ONCE, not every tick. The port fired the whole use block raw every held tick —
    /// a 1-s hold re-toggled a swing door 60× and the 06 guardhouse door closed itself before
    /// the interact test's assert. The 46-frame auto-repeat arm is real but PRESSED-EDGE-RELEVANT
    /// NOWHERE in the shipped game: it pulses mask 8 (bit 3) every 8th frame from held-frame 48
    /// [M 0x17ee88's counter + disasm 0x17f0d4-0x17f0f0], and the ONLY mask-8 readers are the
    /// menu accept repeat (`Button_Update` 0x1ef948:76) and scan mode (`Player_SSScanMode`
    /// 0x193098:42) — every gameplay reader of the use action 0xE reads mask 4 (the edge) only,
    /// and nothing clears the 0xE byte while held (no `Input_ClearAction(_, 0xE)` caller), so a
    /// held E can never re-trigger the world use. SP path only — the MP row's per-slot use pass
    /// is untouched.
    use_key_prev: bool,
    /// mp-combat-weapons: the online match's shared rounds / grenades / blasts / guided rides
    /// (`nf_game::mp::combat`) — every machine runs every agent's rounds from the shot
    /// descriptions in the session inputs. Shared, hashed.
    pub mp_combat: nf_game::mp::combat::MpCombat,
    /// mp-combat-weapons: the shot the local pipeline released during the latest `step_bond`
    /// (the fired record, the aim ray, through a zoomed scope) — the app puts it into the next
    /// session input. Local only.
    pub net_shot: Option<NetShot>,
    /// Each human slot's held weapon (the session's `Input::held`; the local slot reads `gun.weapon`).
    pub mp_held: [u8; nf_net::MAX_AGENTS],
    /// mp-remote-fx: the other players' and the bots' shot effects/sounds (draw/audio state, not
    /// hashed — `crate::mpfx`).
    pub mp_fx: crate::mpfx::MpFx,
    /// Aim mode's cursor (`BL+0x118 / +0x11C`) and its smoothed copy (`+0x120 / +0x124`), −1…1 across half the screen.
    pub cursor: [f32; 2],
    pub cross: [f32; 2],
    /// The first-person gun's script (`Player_SetWeaponAnimObj`), its time in frames and speed, and the bob phases
    /// (`BL+0x8DC` from the forward speed, `BL+0x8E8` idle time).
    pub view_script: u32,
    /// The camera is Bond's eye (`step_bond`); with the free camera (`step`) the first-person gun is not drawn.
    pub first_person: bool,
    /// The tick of Bond's last shot (the first-person flash).
    pub last_shot: Option<u32>,
    /// Bond fired during the latest `step_bond` (the muzzle light's one tick; `last_shot` is stamped before the
    /// drone tick advances the frame, so it cannot say this alone).
    pub shot_tick: bool,
    /// The fire trigger's last-tick state — the press edge the watch pair 0x54/0x55's mode toggle ORs
    /// in (`Player_Weapon` 0x1a24c8's `Input_Action(slot, 9, 4)`) (fix-weapon-modes).
    pub prev_trigger: bool,
    /// Hurt volumes (placement 0xE4, `Hurt_Create` 0x17e870): the contact damage volumes
    /// (level-02-interactables — 07000002's `electrocute` under the bunker arc).
    pub hurts: Vec<nf_game::hurt::Hurt>,
    /// fix-weapon-anims: the idle-alert substate (`BLData+0x959`, value 1 while the +0xCC fidget
    /// script plays) [M `Player_SetWeaponAnimObj` 0x1a2e38:79-136]. Set when the idle counter
    /// (`drones.aims_idle`, `+0x93a`) crosses the game's 0x14*FRAME_RATE_INT threshold and the
    /// record's idle-alert script (+0xCC) queues; cleared when the fidget stops (back to +0xA0) or
    /// when anything else displaces it (the game resets +0x959 in every non-idle state [M :40-58]).
    pub view_alert: bool,
    /// fix-weapon-anims: last tick's aiming input — the aim raise/lower edge (`Player_Weapon`
    /// 0x1a24c8:117-152 queues +0xB8 at 1.25 on the aim-on edge and +0xBC at 1.25 (or +0xB8 played
    /// REVERSE — the port plays it forward, deviation documented) on the aim-off edge, from the
    /// fully idle machine).
    pub prev_aiming: bool,
    /// fix-weapon-anims + fix-aim-raise2: the aim raise/lower script currently playing
    /// (`Player_Weapon` 0x1a24c8:121-152 — the raise +0xB8 @1.25 is the game's state 0xF, the
    /// lower +0xBC @1.25 (or +0xB8 reversed) is state 0x10). Some(hash): the hash this anim plays;
    /// queued by the view chain's edge arm (the SINGLE queue point), advanced and finished by
    /// `aim_gate` (the SINGLE advance — the raise's finish sets the aim bit [M 0x1a2e38:554-561]);
    /// cleared when it stops or when another script displaces it. Only records with +0xB8 (the
    /// snipers, the AIMS-20 pair, the micro-camera pair [D census]) raise/lower through scripts.
    pub view_aim_anim: Option<u32>,
    /// fix-aim-raise2: true while `view_aim_anim` plays the RAISE — its finish sets the aim bit
    /// [M `Player_SetWeaponAnimObj` case 0xf 0x1a2e38:554-561]; false for the lower — its finish
    /// leaves the bit clear (case 0x10, [M 0x1a2e38:562-567]).
    pub view_aim_raising: bool,
    /// fix-weapon-fire-modes: the guided-missile camera (`Bullet_init` 0x12c5a0:234-254 for a weapon
    /// record with `+0x6C & 4` — only the AT-420 Sentinel 'Guided' 0x2C [L ee_start_05.bin]). While
    /// `Some`, the view rides the projectile: position/direction = the bullet's [M
    /// `Player_PositionCamera` 0x18f5d8 case 0xc — the view matrix IS the missile's], the look input
    /// steers it [M `Bullet_update` 0x12e7b0:177-244], Bond's own arms are skipped (the game's player
    /// substate 10 [M `Player_Update` 0x193600:242-256 — only the countdown runs]) and on the
    /// missile's death the view freezes for the 15 ticks of `BLData+0x944` (the static period) before
    /// the normal view returns [M `Bullet_Delete` 0x12e660:26-30 + `Player_Update` case 10].
    pub missile_cam: Option<super::scene::MissileCam>,
    pub view_time: f32,
    pub view_speed: f32,
    pub bob: [f32; 2],
    /// The aim bit (`BLData+0x60 & 1`, action 0x13 — a pad's L1; not with the mouse, D-002):
    /// for a weapon with an aim-raise script (+0xB8) it sets only on the raise's last tick
    /// [M `Player_SetWeaponAnimObj` case 0xf 0x1a2e38:554-561]; without one it follows the button
    /// at once [M `Player_Weapon` 0x1a24c8:117-119]. fix-aim-raise (was: the raw button).
    pub aiming: bool,
    /// Free-aim (the pad's aim button held with the aim bit up): the crosshair moves in its window and the
    /// bullet rides the cursor offsets [M `Player_Aiming` 0x1a1d28]. The mouse never enters it (PLAN.md
    /// D-002: the mouse aims directly) — the right button drives only the raise/zoom through `aiming`.
    pub free_aim: bool,
    /// The crosshair state (BLData+0x133): 2 normal, 3 = the red "no" mark on a non-enemy target.
    /// Set by the `Check_Target` block in `run` every 6th frame, read by the HUD [M 0x1a7810, 0x19e6d0].
    pub cross_state: u8,
    pub auto_aim: bool,
    /// PC-port option "Rapid fire" (a documented port-only addition, off = the game): while the
    /// trigger is held every weapon fires one round per 60 Hz tick (the record's refire interval
    /// and shots-per-pull are overridden, the round spawns on the press tick instead of the fire
    /// script's release frame) and its clip stays full. A showcase of how many live projectiles
    /// the port's sim and renderer carry; the game has no such option (the PS2 options screens
    /// carry no fire-rate row [M options-census.md]) and its own rules are untouched while it
    /// is off.
    pub rapid_fire: bool,
    /// `BLData+0x8D0`: the current zoom factor (1.0 = no zoom, up to `weapon_data+0x88`). The FOV is divided
    /// by this. `Player_Zoom` (0x1a1a70) interpolates it toward `zoom_target` (`+0x8D4`).
    pub zoom: f32,
    /// `BLData+0x8D4`: the target zoom factor. Right mouse (mouse-first) zooms in; releasing returns to 1.0.
    pub zoom_target: f32,
    /// `Player_Disable` by a script (cut-scene): no input, no first-person view, until `Player_Enable`.
    pub disabled: bool,
    /// Surface sounds (`EffectInfo`) and Bond's footstep distance (`BL+0x924`).
    pub surfaces: Option<nf_formats::sfx::SurfaceSounds>,
    pub voices: Option<Vec<nf_formats::sfx::VoiceLine>>,
    pub step_dist: f32,
    /// The laser-family beam's endpoints this tick (the hand/head end, the hit end) —
    /// `gadget::laser_tick` while a laser weapon is out [M Player_LaserPointer 0x1a65d8]. The
    /// renderer draws the beam mesh (0x2000131/0x2000133) between them.
    pub laser_beam: Option<(nf_game::world::V3, nf_game::world::V3)>,
    /// The stunner's per-tick target test [M Player_WeaponInitBullet 0x1a4bf0:169-200 — BLData+0x95d]:
    /// `Some(true)` = the aim ray met a live guard within weapon_data+0x1c (the shot spawns and the
    /// taser beam draws), `Some(false)` = no target (no bullet, no beam), `None` = not holding fire.
    pub stunner_target: Option<bool>,
    /// The Q-Decryptor's gate pick this tick — the lock index the use gate passed on
    /// [M Player_WeaponFiring 0x1a5460:551-578], `None` = the gate failed (or no decryptor out).
    pub decryptor_target: Option<usize>,
    /// The decryptor shot's scan record, one tick old: (lock index, the upgraded +3 flag). The
    /// game's contact view lags the shot by TWO ticks — the update pass (the shot) runs before
    /// `Collide_Update` writes the contact lists [M 0x1336f8:54-69 then :238], and the fresh
    /// bolt's pick segment exists only from its first `Bullet_update` [M 0x12e7b0:98-110] — so
    /// `Lock_Update` reads a shot's contact two ticks after it. This register is the pipe's
    /// middle; `decryptor_scanned_lag` (two ticks old) is what the top-of-tick pass applies.
    pub decryptor_scanned: Option<(usize, bool)>,
    /// The decryptor scan record TWO ticks old — the game's contact view for this tick (see
    /// `decryptor_scanned`). Field on its own line [WORKLOG lesson 4].
    pub decryptor_scanned_lag: Option<(usize, bool)>,
    /// The decryptor screen's four display chars (`DAT_002a37dd[0..3]`: '*', a digit, or 0 = cleared)
    /// and the view entity's idle countdown (`+0xf6`, reset to FRAME_RATE each scanning tick) [M
    /// Player_WeaponFiring 0x1a5460:155-194]; `pda_lock` is the lock the display follows.
    pub pda_display: [u8; 4],
    pub pda_hold: u16,
    pub pda_lock: Option<usize>,
    /// The laser watch's held-meter charge, 0–100 — the port stands the value in (`Sim::laser_charge`)
    /// because the game's writer of the pane's source is unlocated: the state-0xb pane reads a
    /// 0–100 u16 at the tracked object's instance+0x110 (and an f32 at +0x104)
    /// [M HUD_UpdateAmmoPane 0x19a7f0:160-176], and the exhaustive store scan of ACTION.ELF finds
    /// no writer (gadget.rs's evidence block; live check in the report). The drain matches the
    /// disc measurement [D docs/live/2026-10-14-weapon-measurements-pcsx2.md: ~0.5/frame]; the
    /// idle refill is [S unverified] and kept out of pass/fail.
    pub laser_charge: f32,
    /// The held weapon's laser sight this tick [M `Player_LaserPointer` 0x1a65d8, run per frame from
    /// `Player_CollisionHandler` 0x1949a8:188 while the latch `BLData+0x964` (set from
    /// `weapon_data[w]+0x68 & 0x8` per draw [M `Player_SetWeaponAnim` 0x1a9338:21-26]) is lit — the
    /// SG5's semi twin 0x17 carries the bit, the full-auto 0x16 does not [D]]: a tuple of
    /// (beam start = the first-person model's bone-4 sight datum, beam end = the
    /// head→datum+bone4·Z·100 ray's first hit clamped on it, or the far end when nothing is hit,
    /// dot = `Some((hit + normal·0.01, normal))` when the ray met a surface — the red wall dot;
    /// the draw scales it 1 + dist²/MAX_VIEW_CONE (2000.0 [L symtab 0x30d0fc]) capped at 4).
    /// `None` = sight off (no bit, or no first-person datum yet). `laser_on` mirrors the latch.
    pub laser_sight: Option<(nf_game::world::V3, nf_game::world::V3, Option<(nf_game::world::V3, nf_game::world::V3)>)>,
    /// The level's zip wires (placement type 0x3E, `Wire_Create` 0x1981d8). Bond grabs one by walking
    /// into its world sphere (fix-wire: the real grab + crossing live in `nf_game::player`).
    pub wires: Vec<nf_game::wire::Wire>,
    /// The level's creepwalls (class 0x2B, placement 0x3F) — the crawlable vents (fix-swim-vent).
    pub creepwalls: Vec<nf_game::creepwall::CreepWall>,
    /// The level's third icons (placement kind 0xFA, `ThirdIcon_Create` 0x197070). Each tick the Sim
    /// writes the enabled icon's context into `bond.icon_ctx` (`BLData+0x95F`) — fix-wire2.
    pub icons: Vec<nf_game::wire::Icon>,
    /// The level's hint markers (placement kind 0x33, `Hint_Create` 0x1b3248, class 0x43 —
    /// fix-context-hints). One-shot type-1 (INFO pane) messages with an optional sound, fired when
    /// the player enters the marker's straddle-cel set with its channels satisfied.
    pub hints: Vec<nf_game::player::Hint>,
    /// Per hint marker, the straddled `cells` indices (the game's obj+0xD0 chain occupancy — the
    /// same resolution as `icon_cels`, over the marker's world sphere).
    pub hint_cels: Vec<Vec<usize>>,
    /// Per hint marker: fired once per level load (obj+0xFE bit 0 deletes the game's object
    /// [M Hint_Update 0x1b3380:41-42]).
    pub hint_fired: Vec<bool>,
    /// Per third icon, its instance message timer (data+8 [M ThirdIcon_Update 0x196e80:15-17]):
    /// −1 per tick, reset to 4 while the player is in range; the type-6 "Press Action" hint posts
    /// only when the timer has ticked down to 0.
    pub icon_msg_timers: Vec<u8>,
    /// Each wire's owning cel — the game's own home-cel rule, not a stand-in: every dynamic
    /// object gets its cel at runtime, `build_LinkToRoom` 0x1c9680:40-49 calling
    /// `build_FindCel` 0x1c8cb0 over the object's world-sphere centre (bbox containment plus
    /// the ±256-unit collision-ray tie-break), then `control_link_object_to_cel` 0x1346b8 puts
    /// it on that cel's +8 list. The pick pass is CEL-CULLED: `Collide_Pick` 0x1eb320:125,312-330
    /// tests only the cels the query segment straddles (`Collide_StraddleCels`) and each cel's
    /// own object list — a wire is pickable only in its own cel. (The wire's placement DOES name
    /// its glist — `parsemap_block_map_data_dynamic` 0x1d1b68:77-87 resolves it by index/hash —
    /// but that glist is the wire's own geometry/draw record (obj+0xD8, draw flags obj+0xF0
    /// [M Wire_Create 0x1981d8:24-29]), not its cel: there is no placement→cel parentage in the
    /// stream because the game computes the cel spatially, exactly as this port does.) The port
    /// picks the cel with the faithful `find_cell_raycast`/`world.find_cel` port of
    /// `build_FindCel`; `None` (centre inside no room bbox) keeps the unculled scan. Without the
    /// gate 07000002's courtyard wire (world sphere r 11.5 at (−63.1, 8.1, −34.5)) swallows Bond
    /// the moment he enters the secret bunker (a different cel), making the bunker's touch
    /// plates unreachable. Indices are `CelGraph::cells` (ScopeGraph) indices, not `cels`.
    pub wire_cels: Vec<Option<usize>>,
    /// The runtime cel graph (a build clone) — `find_cell_raycast`'s per-tick input for the third
    /// icons' arm test (fix-08-walls: the arm region is the game's straddle-cel set, `Collide_StraddleCels`
    /// 0x1eac98, resolved over this graph at load into `icon_cels`).
    pub cel_graph: CelGraph,
    /// Per third icon, the straddled `cells` indices — the game's obj+0xd0 chain occupancy set
    /// [M `Collide_Update` 0x1ea398; computed by `icon_straddle_cels` at load]. Empty = the icon
    /// never arms (its `build_FindCel` cel did not resolve — the game's unlinked icon gets no
    /// occupants either [M 0x1c9680:47]).
    pub icon_cels: Vec<Vec<usize>>,
    /// Whether Bond is currently on a wire (substate 6/0xF) — the music event 0xD source.
    pub creeping: bool,
    /// The wire anims' root tracks, resolved from the level archive's kind-5 scripts + the anim
    /// archive's sequences (fix-wire).
    pub wire_anims: nf_game::wire::WireAnims,
    /// The wire switch channel currently held on (0 = none) — set 1 while Bond rides, 0 on drop
    /// [M Player_Wire 0x1976bc/0x1978c4].
    pub wire_channel: u16,
    /// The level's ladders (placement type 0x22, `Ladder_Create` 0x1966c0, class 0x2D) — fix-ladder.
    pub ladders: Vec<nf_game::ladder::Ladder>,
    /// The set of weapon ids that have a view model built in `Scene::view_models` (D-003: the cycle skips
    /// weapons not in this set, because their skin is not in the level archive).
    pub view_model_ids: std::collections::HashSet<u8>,
    /// The level's room cels (`Cel`; `build_FindCel` 0x1c8cb0 walks them, `Lights_CalcAmbientLight` 0x1cb178
    /// reads their ambient) — the source of the 0x800-object ambient ramp.
    pub cels: Vec<Cel>,
    /// The room cel → pCelList cell map (`cel_graph.room_to_cell`), the actor queries' start-cell
    /// lookup (obj+0x20 — fix-safe08b).
    pub room_cell: Vec<Option<usize>>,
    /// Per character (scene index): `obj+0x100..0x102`, the ambient-light bytes of `Lights_CalcAmbientLight`
    /// (0x1cb178), initialised to the 0xff,ff,ff sentinel of `control_init_object` (0x134630) and stepped by
    /// `ramp_char_ambient` each tick.
    pub char_ambient: Vec<[u8; 3]>,
    /// The scene character indices of the first-person view models (`Scene::view_models` values) — they follow
    /// Bond's cel instead of a drone's.
    pub view_model_chars: Vec<usize>,
    /// mp-body-look: the MP bodies (slot, scene character index) — `Scene::mp_bodies`, so the
    /// ambient ramp links each body to the cel it stands in like any other drawn object.
    pub mp_body_chars: Vec<(usize, usize)>,
    /// mp-body-anim: per slot, the body's script list and clocks (`nf_game::mp::body_anim` — the
    /// game's `AnimSetUpdate` 0x1c0938 / `Player_ChangeSubState` 0x190630 list: idle, the
    /// distance-driven walk/run, the strafe overlay, the stance transitions, the death, the aim
    /// ease) [M, mp-players.md §3]. Ticked once per sim tick by `advance_mp_anim` from shared
    /// state; draw-only (never read by the sim, not in the lockstep hash).
    pub mp_anim: Vec<nf_game::mp::body_anim::MpBodyAnim>,
    /// The 16 player anim-set descriptors (ACTION.ELF 0x2b91b0) the body clocks pick from [D].
    pub mp_anim_sets: Option<nf_formats::mpanim::MpAnimSets>,
    /// mp-body-look: per slot, the last nonzero `mp_held` — a timed-out remote is fed `NEUTRAL`
    /// (held 0) every tick, which would blink its body's gun away. Draw-only state.
    pub mp_last_held: [u8; nf_net::MAX_AGENTS],
    /// mp-lobby-settings: which weapon ids' first-person skins this arena loaded — the port of
    /// `Player_CheckWeaponsLoaded` 0x1a97c8's lookup half [M :12-31: walk weapon_data's 114 rows,
    /// every selectable row (+0x04 set) whose +0xDC first-person skin hash misses the loaded-file
    /// hashtable is pruned; the port tests the same hashes against the arena's loaded skin set],
    /// and the respawn's `MP_EquipPlayer` re-reads it (the game prunes at spawn equip [M
    /// 0x1855f8:48]; the per-weapon owned/cleared half lives in `Inventory::owned`, as the
    /// game's mutable `BLData+0x1BA` bytes do).
    pub mp_weapon_loaded: Vec<bool>,
    /// fix-render3: the `ambient_at` call counter (a test guard for the posed-vertices hoist).
    pub ambient_lookups: std::cell::Cell<usize>,
    /// mp-perf-16: the MP bodies' draw memo — the whole-frame bone samples
    /// (`nf_game::anim::PoseSampleCache`) and the walk scripts' distance tables (built once per
    /// walk sequence instead of per body per ticked frame). Draw-only, never read by the sim.
    pub mp_draw_cache: std::cell::RefCell<(nf_game::anim::PoseSampleCache, HashMap<u32, Option<nf_game::anim::DistanceTable>>)>,
    /// light-sun: BLData+0x962 — Bond's head sees the sun this tick (`ramp_char_ambient`).
    pub sun_seen: bool,
    /// The language string table (`USATxt.dat`, the NTSC-U default `FNames[SelLang]`) — the source of the
    /// HUD's label text (`Txt_BindLabel` 0x1cad60). Loaded in `nf_view::load`.
    pub lang: Option<nf_formats::hud::Lang>,
    /// The muzzle smoke puffs (`Gas_Init` 0x1aacc8; fix-lighting: the RNG draw rides the shared
    /// generator in the game's order). The puff's motion here is the port's presentation — the
    /// game's per-particle model is `gases_smoke` 0x1ab2d0: per-frame velocity damping
    /// (case 0: ×FRAME_RATE_DIV·0.95 [M :46]), gravity integration toward `_WldGravity`
    /// [M :57-61], position integration [M :62-65] and a params-driven scale accumulation
    /// [M :66-76] — a particle sim the draw side can adopt later (visual-only).
    pub smoke: Vec<nf_game::light::Smoke>,
    /// The HUD's per-frame state (`fix-hud`): the weapon-name timer (`BL+0x95E`), the health pane's
    /// crouch-icon timer, the pickup message bar (pane 21) and how many `drones.taken` events it has
    /// converted. Stepped per tick in `hud_tick` (the game's HUD update runs per 60 Hz frame).
    pub hud: HudState,
    /// The view aspect the game writes at viewer+0x114 (`Camera_CalcViewAngles` 0x1baf70:28-32:
    /// 16/9 = 0x3fe38e39 with the AV options' widescreen flag set, 4/3 = 0x3faaaaab otherwise; the
    /// host feeds it from `FrontEnd::view_aspect`). screen-adjust-model (field on its own line
    /// [WORKLOG lesson 4]).
    pub view_aspect: f32,
    /// microcam-wheel-zoom: the mouse wheel has stepped the zoom while the aim (right) button is
    /// held — the hold-ramp to the weapon's max (`Player_Zoom` 0x1a1a70's target ×1.025/tick)
    /// yields until the aim falls, which resets the target to 1.0 [M `Player_Weapon` 0x1a24c8's
    /// aim-bit arm] and this latch with it. A documented port-only input:
    /// the game's pad zoom has no wheel — the port's mouse play needs the equivalent — and the
    /// game's plain pad ramp is unchanged when the wheel is unused.
    pub zoom_wheel: bool,
    /// nis-event9-cnis: the C_NIS_Handler route's booted cut-scene script (`FMVScript` [M
    /// `C_NIS_Handler` 0x205f10:71,87]) — the hash `nis_page_boot` started, `None` when the
    /// page hasn't booted one. The msg-0x4b gate re-checks `Script_IsPlaying(FMVScript)`.
    pub fmv_script: Option<u32>,
    /// unported-front-end-menus-3: `Camera_Shear`'s phase (viewer+0x1e0) [M 0x1bba38:26-34] —
    /// advances REC_FRAME_RATE per underwater frame, wrapped into [−π, π].
    pub shear_phase: f32,
    /// unported-front-end-menus-3: the underwater post-FX byte `GlobalBlur` — 0x91 while the
    /// shear runs [M 0x1bba38:49], 0 otherwise (the port's only writer is this trigger). The
    /// draw side reads it for the underwater blur.
    pub global_blur: u8,
    /// unported-front-end-menus-3: `Camera_ApplySwing`'s phase (viewer+0x204) [M 0x1bbf40:10]
    /// — advances 2π/(FRAME_RATE·4) per swaying frame (the 4-second cycle).
    pub swing_phase: f32,
}

/// A status-pane message (`Text_AddMsg` 0x1b0c30's `TXT_MSG` node): the bound text, its remaining ticks,
/// and — for the objective banner — the status label id shown above it (0x2000051 "NEW OBJECTIVE"
/// / 0x2000053 "OBJECTIVE COMPLETE"; `HUD_UpdateStatusPane` 0x19c310:127-144 rebinds it per display
/// by comparing the message text with "OBJECTIVE COMPLETE" — the port carries the id from the
/// event instead) and whether the game's Fail-prefixed format ("%s: %s", 0x30cbb8) rendered it.
/// `life` is the time the node was created with (`Text_AddMsg`'s scaled tick count); `opened`/
/// `second` track the pane's per-display SFX (`ObjectiveSFXTriggered1/2` 0x30ccb4/0x30ccb8 —
/// reset by case 2 each time the message becomes the displayed head [M 0x19c310:148-164]).
#[derive(Clone, Debug)]
pub struct Msg {
    pub text: String,
    pub time: f32,
    pub status: u32,
    pub failed: bool,
    pub life: f32,
    pub opened: bool,
    pub second: bool,
    /// The posting `Txt_BindLabel` id (0 for the queues that post pre-bound text — the INFO
    /// pane's tests pin it; the objective queue carries the objective's own text id since
    /// objectives-0b-start) [M Text_AddMsg 0x1b0c30 stores the bound string; the id is the
    /// port's provenance for the game-label assertions].
    pub label: u32,
    /// A script op-0x1e subtitle (`Text_AddMsg` type 4): the game's `Text_FlushAllSubtitles`
    /// 0x1b15c8 deletes types 4/5 only, so the skip's flush drops these and nothing else
    /// (fix-ladder-water).
    pub script: bool,
    /// The pane's case-3 open chime (PlayMe1st: 1532 new / 468 complete) has fired — the game
    /// plays it one pane frame AFTER case 2's open and only while `ScriptCam == 0` [M
    /// 0x19c310:219-229], so a cut-scene starting in the banner's flash window holds the chime
    /// until the hand-back (objective-banner-timing).
    pub sfx1: bool,
}

impl Msg {
    pub fn new(text: String, time: f32) -> Msg {
        Msg { text, time, status: 0, failed: false, life: time, opened: false, second: false, label: 0, script: false,
            sfx1: false }
    }
}

/// The HUD state the port keeps in Sim (the game keeps it in BLData / the TXTMSG list / pane structs).
#[derive(Clone, Debug, Default)]
pub struct HudState {
    /// The (viewer 0, type 6) pickup message queue, head first (`Text_AddMsg` 0x1b0c30's list head 0x31997c).
    pub msgs: std::collections::VecDeque<Msg>,
    /// How many `drones.taken` events `hud_tick` has converted into messages.
    pub taken_seen: usize,
    /// `BL+0x95E`: re-set to 60 every frame a differing switch request is pending, then −1/frame;
    /// while it runs the ammo pane shows the weapon name alone (`HUD_UpdateAmmoPane` 0x19a7f0).
    pub weapon_label_timer: u8,
    /// Retired by fix-ammo-pane: the timer arms on the pending request (drone+0x63 vs +0x62), not
    /// on a last-seen weapon id. Kept so existing saves/replays keep their layout.
    pub hud_weapon: u8,
    /// The health pane's `CrouchIconTimer` (60 after Bond stands; fades sprite 19 over the last 30)
    /// [M `HUD_UpdateHealthPane` 0x19b9c8:305-328].
    pub crouch_icon_timer: i32,
    /// `BL+0x948`: the 007 Bond-moment flash timer — `PlrStat_DoneBondMoment` 0x199448:23-25
    /// writes 200 on a NEW moment; `HUD_UpdateHealthPane` 0x19b9c8:258-260 takes FRAME_RATE_MUL
    /// (1 at 60 Hz) per frame while ≥ 1 [M]. Drawn as health pane sprite 16 (hud.rs).
    pub bond_moment_flash: i16,
    /// The completed-moment count the flash last saw (None until the first tick: moments
    /// already done when the level loads never flash).
    pub moments_seen: Option<u32>,
    /// The pickup bar's close animation (its remaining width; `HUD_UpdateStatusPane` 0x19c310 state 4
    /// shrinks 16 px per tick — pane+0xC = 0x10 for the pickup pane).
    pub bar_close: f32,
    /// The type-2 objective notice queue (pane 3, `MsgObjectiveStatusPane` 0x319710) and the type-3
    /// mission banner queue (pane 2, `MsgMissionStatusPane` 0x2bd538) — the game keeps one TXTMSG
    /// list per type; `Mission_MonitorObjectives` posts type 2 and `Mission_Update` type 3
    /// [M 0x1827d8/0x182c60, docs/systems/mission.md].
    pub objective_msgs: std::collections::VecDeque<Msg>,
    pub mission_msgs: std::collections::VecDeque<Msg>,
    /// The mission banner's second line: the `FailLabel` global (0x30cba8) — rebound per display
    /// [M 0x19c310:171-173].
    pub fail_label: u32,
    /// The banners' close animations (16 px per tick, the panes' slide field).
    pub objective_close: f32,
    pub mission_close: f32,
    /// The type-1 INFO message queue (pane 4, `MsgInfoStatusPane` 0x319810 — the TOP hint bar):
    /// the Hint markers (placement 0x33) and `ThirdIcon_Update`'s type-6 "Press Action" post
    /// `Text_AddMsg(…, 1, …, 0xb4)` into it [M 0x1b3380:33, 0x196e80:41-42]. One TXTMSG list per
    /// type, like the pickup/objective/mission queues.
    pub info_msgs: std::collections::VecDeque<Msg>,
    /// The INFO bar's close animation (16 px per tick — the pane's slide 0x10 [L 0x319810+0x14]).
    pub info_close: f32,
    /// The raw label ids still awaiting the language bind (drained into `info_msgs` in `hud_tick`;
    /// the tests pin these game label ids directly).
    pub info_labels: std::collections::VecDeque<(u32, f32)>,
    /// The health pane's `ThirdIconTimer` (0x30CCE8): 30 while a context id is set, fading the
    /// context icon out over it on 0xFF [M HUD_UpdateHealthPane 0x19b9c8:273-303].
    pub context_icon_timer: i32,
    /// The last non-0xFF context id the icon showed (the runtime sprite keeps its texture through
    /// the fade — the port re-derives it since the fade path runs after the id is gone).
    pub last_icon_ctx: u8,
    /// door-text-labels: the game's `DoorMsg` global (0x30cd04, .sdata): the aim-ray door hint counter,
    /// `Check_Target` shows the 0x200000b hint only while `DoorMsg < 3` and never resets it — at most
    /// 3 door hints per boot [M 0x1a7b88-0x1a7bc0; the full gp-relative scan finds no other writer].
    /// The game's .sdata survives level loads, so the cap is per BOOT; the port's Sim is per level,
    /// so every level swap carries it over ([`HudState::carry_boot_globals`]).
    pub door_msg: u32,
    /// unported-misc-5: the game's `LoadMsg` global (0x30cd08, .sdata, next to `DoorMsg`): the aim-ray
    /// exit-hint cooldown. `Check_Target` decrements it on EVERY 6th-frame invocation (0x1a7810:78-79),
    /// and the 0x1d `Trigger_Type == 10` arm only fires while it is 0, re-arming it to
    /// `FRAME_RATE_INT * 0x14` = 1200 invocations = 7200 frames [M 0x1a7810:195-196]. Carried
    /// across level swaps by [`HudState::carry_boot_globals`] like the game's .sdata — the same
    /// boot-scoped global the game itself keeps.
    pub load_msg: u32,
    /// The goggles flow controller (`HUD_MonitorNightSight` 0x19e9d8): the view mode (viewer
    /// +0x236: 0 off / 1 night sight / 2 x-ray), the battery (BL+0x920, 0..1800) and the
    /// BL+0x961 latch. Ticked per frame in `hud_tick` — the game calls it from `HUD_Update`
    /// 0x19a088:19 inside `Player_Update`. Consumers read it via `Sim::goggle_mode` /
    /// `Sim::goggle_battery`; docs/systems/goggles.md holds the evidence map.
    pub goggles: nf_game::hud_pane::MonitorNightSight,
    /// The goggles key's press edge this tick, stashed by `step_bond` from
    /// `Input::goggles_action` and consumed by `hud_tick` (which runs inside `run`, later in
    /// the same tick — the game's `Input_Action(p, 0xd, 4)` reads the same edge).
    pub goggles_press: bool,
}

impl HudState {
    /// The per-BOOT text globals the game keeps in .sdata across level loads — `DoorMsg` 0x30cd04
    /// (the door-hint cap of 3) and `LoadMsg` 0x30cd08 (the exit-hint cooldown) [M 0x1a7810:78-79,
    /// 195-196; 0x1a7b98-0x1a7bc8]. The app calls this on every level swap so a new level's Sim
    /// starts from the old one's counts instead of zero (the per-level reset re-showed the door
    /// prompts on every level).
    pub fn carry_boot_globals(&mut self, old: &HudState) {
        self.door_msg = old.door_msg;
        self.load_msg = old.load_msg;
    }
}

impl HudState {
    /// The bar's open width when a message is up (256) — the pane template's width [D hud.md].
    const BAR_W: f32 = 256.0;
    /// The objective banner's open width (480, `ObjectiveStatusSprInfo[2].w` [L 0x319608+0x2c·2]) and
    /// the mission banner's open height (103, `MissionStatusSprInfo[2].h` [L 0x319580+0x2c·2]).
    const OBJECTIVE_W: f32 = 480.0;
    const MISSION_H: f32 = 103.0;
    /// The INFO bar's open width (480, `InfoStatusSprInfo[2].w` [L 0x319730+0x58+0x10] — the same
    /// 480 the CREATE's w carries before `HUD_CalcWidthHeight`).
    const INFO_W: f32 = 480.0;
}

/// mp-death-effects: the local player's death presentation in one struct — what the draw code
/// needs for the blood pane / the death view, offline MP, online MP and SP alike
/// (docs/systems/mp-death-effects.md has the addresses). Read it through
/// [`Sim::local_death_fx`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalDeathFx {
    /// 0 none / 1 the sequence is running / 2 the full cover [M `HUD_UpdateBloodPane`
    /// 0x19a6a8:23-25 — SP only; the MP respawn ends the sequence first].
    pub phase: u8,
    /// The blood pane's grown height in px: 1 px/tick from the enable [M 0x19a6a8:27-30],
    /// capped at the 448 cover [M 0x19f4e8:33].
    pub pane_px: u32,
    /// The port's per-tick fade input for the pane-12 templates: `pane_px / 448` clamped 0..1
    /// (the draw maps it onto layer 9's templates, textures 0x0300005a/0x03000002 [L BloodSprInfo
    /// 0x2bcfd8]). The game writes no alpha on this path — `HUD_UpdateBloodPane` grows the
    /// sprite's height byte only (+0x2e, +1/tick [M 0x19a6a8:27-30]) and the templates keep
    /// their created alpha — so this ramp is the port's documented draw choice for the same
    /// growth, not a game-value claim.
    pub fade_alpha: f32,
    /// The whole HUD hides for the pane — SP only (the `HUD_DisableAll` [M 0x190bb0:24]);
    /// MP keeps the HUD up under the blood [M :39].
    pub hud_hidden: bool,
    /// MP only: the ticks left of the 5 s respawn gate [M 0x190bb0:41-54]; None in SP (SP
    /// waits for the cover / the ✕ instead). The game draws no countdown/prompt — this is
    /// data for the draw, not a game behavior.
    pub respawn_ticks_left: Option<u32>,
    /// The camera holds the frozen first-person eye at the death spot (cam mode 0
    /// [M `Player_CheckForDeath` 0x190810:64]; the pose source is `bond.head()`). Online the
    /// freeze starts on the kill tick itself (`net_dead`), one tick before `drones.death`.
    pub camera_frozen: bool,
}

impl Sim {

    /// fix-laser-beam: stop every LOOPED op-1 sound of `weapon`'s fire + release scripts (the
    /// Laser/Stunner's beam loops 0x12/0x3d [L 0x060004e9/0x0600048a op-1, +0x060004e8/0x06000488
    /// op-4.3]). Called on the weapon switch (the held state 0xb never reaches the release script's
    /// stop) and from the pause path — the game's `MenuManager_Create` runs `SFXPause` +
    /// `SFXPauseAllStreams` [M 0x21b908:120-123], which silences the loop while the menu is up.
    pub fn gadget_loop_ids(&self, weapon: u8) -> Vec<u32> {
        let scripts = [self.weapons.records.get(weapon as usize).map_or(0, |r| u32::from_le_bytes(r[0xb0..0xb4].try_into().unwrap())),
                       self.weapons.records.get(weapon as usize).map_or(0, |r| u32::from_le_bytes(r[0xb4..0xb8].try_into().unwrap()))];
        let mut ids = Vec::new();
        for h in scripts.into_iter().filter(|&h| h != 0) {
            if let Some(sc) = self.scripts.get(&h) {
                for c in &sc.commands {
                    if c.op == 1 && c.args.len() >= 2 {
                        let id = remap_op1_sound(c.args[1] as u32, weapon);
                        if self.sfx.as_ref().map(|t| t.get(id)).is_some_and(|e| e.looped) { ids.push(id); }
                    }
                }
            }
        }
        ids
    }
    /// A script character's live position: the root-bone world translation of its animation. The
    /// script Anim object sits at the origin and the sequence's root carries it (nis.rs; the game's
    /// `AnimSeqTick` 0x1c28d0 takes the root motion as the object's movement) — so the root
    /// translation is the object position `Lights_CalcAmbientLight` (0x1cb178) reads. fix-render2:
    /// the old version looked the skin up through `script_entity`, which matches only `Obj::Entity`
    /// — script Anim characters (the cut-scene actors, the truck) never resolved and kept the white
    /// 0xff sentinel, so every cut-scene drew its characters over-bright (and the 07000005 truck
    /// too bright).
    pub fn characters_script_pos(&self, ci: usize) -> Option<[f32; 3]> {
        let src = self.char_script.get(ci).copied().flatten()?;
        let skin_hash = (*self.char_script_skin.get(ci)?)?;
        let (script, frame) = self.script_anim(src, skin_hash)?;
        let skin = self.skins.get(&skin_hash)?;
        let skeleton = self.skeletons.get(&skin.skeleton)?;
        // fix-cutscene-brightness2: the ramp must sample the ambient where the actor is DRAWN, so
        // the position resolution here is the render path's own — `script_pose_together` (the
        // widest started body window; frame_state's script branch poses with it). The held
        // fix-cutscene-brightness used `script_pose`, whose `script_frame` takes the LAST covering
        // window — on the 07000002 intro that is the zero-root overlay track (char[79]: windows
        // (1,1541,0x04000296)/(1,1540,0x04000224); the last one's root reads [0,0,0] at frame 170
        // while the body track carries [-25.51, 1.13, 30.14] — beside Bond [-27.17, 1.05, 29.80]),
        // a point in no cel. Its scratch-base composition then multiplied the near-origin root by
        // `Player::live_matrix` — which is stream 0's object ∘ the script matrix, i.e. the intro's
        // CAMERA (obj[0] Camera trans == player[1] live trans [-27.25, 1.41, 34.39], live probe) —
        // so the ambient sampled the camera's cel. The premise was wrong too: VECTOR 0x2d6200 is
        // the ORIGIN vector [L ACTION.ELF, nis.rs's Light doc], so `Script_AnimStart` 0x1d8f40's
        // `control_create_object(0, 0x2d6200, 0x2d6200, 0)` creates the anim object AT THE ORIGIN —
        // the object matrix is identity and the sequence's root carries it in world space
        // [M Obj::Anim doc; AnimSeqTick 0x1c28d0 takes the root motion as the object's movement].
        let local = nf_game::anim::script_pose_together(&self.scripts, &self.seqs, skeleton, skin, script, frame)?;
        let world = anim::world_pose(&local, skin);
        Some([world[0][3][0], world[0][3][1], world[0][3][2]])
    }
}

    /// The ambient triple for a 0x800 object at `p` with the zone channel byte `off` (the game's
    /// `switch_channels[cel+0xa2]` value: the read shifts `base` by that many bytes).
pub fn cel_ambient(cels: &[Cel], p: [f32; 3], off: u8) -> Option<[u8; 3]> {
        let c = &cels[find_cel(cels, p)?];
        Some(c.ambient[(off as usize).min(1)])
    }

/// `build_FindCel` (0x1c8cb0): the bbox walk (see `Sim::cel_at`).
pub fn find_cel(cels: &[Cel], p: [f32; 3]) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_vol = f32::INFINITY;
        let mut matches = 0;
        for (i, c) in cels.iter().enumerate() {
            let inside = c.min[0] <= p[0] && p[0] <= c.max[0] && c.min[1] <= p[1] && p[1] <= c.max[1] && c.min[2] <= p[2] && p[2] <= c.max[2];
            if !inside { continue; }
            matches += 1;
            let vol = (c.max[0] - c.min[0]) * (c.max[1] - c.min[1]) * (c.max[2] - c.min[2]);
            if vol < best_vol { best_vol = vol; best = Some(i); }
            if matches >= 0x40 { break; }
        }
        best
}

/// Squared distance from point `p` to the segment `a`-`b` — the game's 0x800 hittest's
/// `dist(segment(h+0x20, h+0x30), obj_centre)` [M Collide_PickObj 0x1eae80:163].
fn seg_point_dist2(a: &[f32; 3], b: &[f32; 3], p: &[f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if ab2 > 1e-9 { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 } else { 0.0 };
    let t = t.clamp(0.0, 1.0);
    let dx = ap[0] - ab[0] * t; let dy = ap[1] - ab[1] * t; let dz = ap[2] - ab[2] * t;
    dx * dx + dy * dy + dz * dz
}

/// Squared distance between segments `a1`-`a2` and `b1`-`b2` (clamped-parameter closest points,
/// the standard Ericson bracketing solve) — the stunner ray vs the guards' 0x800 capsule
/// [M Player_WeaponInitBullet 0x1a4bf0:169-200; NDrone2_Collision 0x140e90:104-121].
fn seg_seg_closest(a1: &[f32; 3], a2: &[f32; 3], b1: &[f32; 3], b2: &[f32; 3]) -> (f32, f32, f32) {
    let sub = |u: &[f32; 3], v: &[f32; 3]| [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
    let dot = |u: &[f32; 3], v: &[f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let (d1, d2, r) = (sub(a2, a1), sub(b2, b1), sub(a1, b1));
    let (a, e) = (dot(&d1, &d1), dot(&d2, &d2));
    let f = dot(&d2, &r);
    let (c, b) = (dot(&d1, &r), dot(&d1, &d2));
    let denom = a * e - b * b;
    let (s, t) = if denom > 1e-9 {
        ((b * f - c * e) / denom, 0.0f32)
    } else { (0.0f32, f / e.max(1e-9)) };
    let t = ((b * s + f) / e.max(1e-9)).clamp(0.0, 1.0);
    let s = if b * t + c < 0.0 { 0.0 } else if b * t + c > a { 1.0 } else { (b * t + c) / a.max(1e-9) };
    let p1 = [a1[0] + d1[0] * s, a1[1] + d1[1] * s, a1[2] + d1[2] * s];
    let p2 = [b1[0] + d2[0] * t, b1[1] + d2[1] * t, b1[2] + d2[2] * t];
    let dd = sub(&p1, &p2);
    (dd[0] * dd[0] + dd[1] * dd[1] + dd[2] * dd[2], s, t)
}

/// `vecutil_Dist2Tri` (the distance the game's `Intersect_Portal` 0x1ea7b8 version-0x101 path
/// thresholds against the straddle sphere's radius): the Euclidean distance from `p` to the
/// triangle (a, b, c) — the standard closest-point-on-triangle construction.
fn dist_point_tri(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let sub = |u: [f32; 3], v: [f32; 3]| [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
    let dot = |u: [f32; 3], v: [f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 { return dot(ap, ap).sqrt(); }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 { return dot(bp, bp).sqrt(); }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 { let v = d1 / (d1 - d3); return dot(ap, [ab[0] * v, ab[1] * v, ab[2] * v]).sqrt(); }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 { return dot(cp, cp).sqrt(); }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 { let w = d2 / (d2 - d6); return dot(ap, [ac[0] * w, ac[1] * w, ac[2] * w]).sqrt(); }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        let bc = sub(c, b);
        return dot(bp, [bc[0] * w, bc[1] * w, bc[2] * w]).sqrt();
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let closest = [a[0] + ab[0] * v + ac[0] * w, a[1] + ab[1] * v + ac[1] * w, a[2] + ab[2] * v + ac[2] * w];
    dot(sub(p, closest), sub(p, closest)).sqrt()
}

/// The icon's straddle-cel set — the port of `Collide_StraddleCels` (0x1eac98) as
/// `Controls_StraddleTest` (0x134778) drives it for the third icons: starting at the object's own
/// cel (the room `build_FindCel` 0x1c8cb0 links at load, via `room_to_cell`), a BFS over the
/// cel graph's portals adds each neighbour whose portal quad the object's world sphere
/// intersects (`Intersect_Portal` 0x1ea7b8, HITTEST version 0x101: the portal AABB first, then
/// `vecutil_Dist2Tri` from the sphere centre to the portal's two triangles < the radius), up to
/// the game's 0x7e-cel cap. `Collide_Update` (0x1ea398) leaves a hitdata node on every object
/// occupying a straddled cel, and `ThirdIcon_Update` (0x196e80) writes the icon's context into
/// the player's +0x95F off that node — so this set IS the icon's arm region. A wall between the
/// icon and the player keeps them in different cels with no portal for the small sphere to walk
/// through (07000008's guardhouse wall: the icon's r-1.164 sphere crosses no portal), which the
/// old flat 12.0-unit arm radius ignored.
///
/// Returns the straddled `cells` indices (always containing the start cell when it resolves).
pub fn icon_straddle_cels(graph: &CelGraph, start_cell: usize, centre: [f32; 3], radius: f32) -> Vec<usize> {
    let mut seen = vec![false; graph.cells.len()];
    let mut out = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    seen[start_cell] = true;
    out.push(start_cell);
    queue.push_back(start_cell);
    while let Some(c) = queue.pop_front() {
        if out.len() >= 0x7e { break; }
        for p in &graph.portals {
            let other = if p.cel_a == c { Some(p.cel_b) } else if p.cel_b == c { Some(p.cel_a) } else { None };
            let Some(other) = other else { continue };
            if seen[other] { continue; }
            // Intersect_Portal 0x101, step 1: the portal quad's AABB against the sphere's AABB.
            let (mut mn, mut mx) = (p.corners[0], p.corners[0]);
            for corner in &p.corners[1..] {
                for k in 0..3 { mn[k] = mn[k].min(corner[k]); mx[k] = mx[k].max(corner[k]); }
            }
            if centre[0] + radius < mn[0] || centre[0] - radius > mx[0]
                || centre[1] + radius < mn[1] || centre[1] - radius > mx[1]
                || centre[2] + radius < mn[2] || centre[2] - radius > mx[2] { continue; }
            // Step 2: Dist2Tri from the sphere centre to the quad's two triangles < radius.
            let (a, b, c2, d) = (p.corners[0], p.corners[1], p.corners[2], p.corners[3]);
            let hit = dist_point_tri(centre, a, b, c2) < radius || dist_point_tri(centre, a, c2, d) < radius;
            if hit {
                seen[other] = true;
                out.push(other);
                queue.push_back(other);
            }
        }
    }
    out
}

/// The `(tint × ambient) >> 8` bytes of `View_SetupRenderModes_Tweak` (0x1e5888) as floats — the VU1
/// `TAmbient` term (`psiSetTweakARGB` 0x124770: each byte / 255). White tint (the port's default) leaves the
/// bytes equal to the ambient bytes.
pub fn mul_bytes(a: [u8; 3]) -> [f32; 3] {
    let tint = [0xffu8; 3];
    [0, 1, 2].map(|k| ((tint[k] as u32 * a[k] as u32) >> 8) as f32 / 255.0)
}

/// The level's room cels: the main map's cel placements whose low 16 flag bits are 0xc023/0xc047
/// (`parseentity_fixup_entity` 0x1d00c0 fills the runtime cel's ambient/zone from the placement's params
/// for exactly these two flag words). See `Cel`. The room half of `parse_cel_graph` — the two MUST
/// agree; `Scene::cels` and the portal walk's `room_to_cell` are built in one pass.
pub fn parse_room_cels(level: &Level) -> Vec<Cel> {
    // The room half of parse_cel_graph (fix-perf-level): one pass builds Scene::cels and the portal
    // graph so the two cannot diverge.
    parse_cel_graph(level).0
}

/// A parsed room cel — the ambient-light source for 0x800 objects (`docs/atlas/cel.md`, `gs-leaf.md`).
/// Built from the main map's cel placements: flags & 0x8000 create a cel, and `parseentity_fixup_entity`
/// (0x1d00c0) fills the runtime cel from the placement's params when the low 16 flag bits are 0xc023/0xc047
/// (the two "room cel" flag words, both carrying the 0x40000 room-cel bit `build_FindCel` walks):
/// ambient RGB = params 0/1/2 low bytes (all zero → 0x3c each), the +1-byte-shifted second triple =
/// params 6/7/8 (0xc047 only), the lighting zone = param 5 (0xc047 only). The bbox is the object's glist
/// min/max copied raw into `cel+0x40..0x60` (`parsemap_block_map_data_static` 0x1d1998 + the
/// `parsemap_block_entity_params` 0x1d1508 record map) — the same box `build_FindCel` (0x1c8cb0) tests.
#[derive(Clone)]
pub struct Cel {
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// `base+0x9a/0x9c/0x9e` for base = cel + `switch_channels[zone]` with offset 0 (triple 0) and 1 (triple 1).
    pub ambient: [[u8; 3]; 2],
    /// `cel+0xa2`: the switch-channel index whose byte value shifts the ambient read by that many bytes
    /// (`base = cel + switch_channels[cel+0xa2]`, `Lights_CalcAmbientLight` 0x1cb178).
    pub zone: u8,
    /// `cel+0x90`: the water surface [M fix-ladder]. `build_alloc_cel` 0x1c93f8 sets it to −500.0 (no
    /// water); `parseentity_fixup_entity` 0x1d00c0 sets `cel+0x90 = cel+0x54` for flag word 0xC023 —
    /// `cel+0x54` is the glist bbox max y (`amemcpy(cel+0x40, glist+0x20, 0x20)` 0x1d1998 copies the bbox
    /// to cel+0x40..0x5F) — and to −500.0 for 0xC047. `Player_InWater` 0x195e50 and the swim substate
    /// clamp read it, and `Player_Collision` skips gravity below it.
    pub water: f32,
    /// `cel+0x98` (s16): the room's reverb environment [M 0x1d00c0, fix-room-sound]. `parseentity_fixup_entity`
    /// writes 0x9898 (s16 −26472, the IOP's underwater sentinel) for flag word 0xC023 and placement param 9
    /// (`level_tag+0x50`, a u16 with no mask or clamp) for 0xC047; a cel without the fixup keeps the 0x98
    /// allocator fill in both bytes, which reads back as the same 0x9898 sentinel [L ee_start_05.bin: the
    /// untouched cel bytes at +0xa4..0xaf are 0x98]. `Sound_UpdateListeners` 0x1dd278 sends it to
    /// `SFXSetEnvironment` from the active viewer's cel every frame.
    pub env: i16,
    /// `cel+0x3c & 0x10000000` [M 0x1d00c0]: the underwater flag — placement param 3 (`level_tag+0x38`,
    /// a full u32) != 0, written for both room-cel classes. `Sound_UpdateListeners` ORs it with `env >= 20`
    /// (a signed s16 compare) into `SFXSetEnvironment`'s underwater argument.
    pub underwater: bool,
}

/// The runtime cel graph for the portal visibility walk (fix-perf-level, G-016's portal half).
///
/// `cells` IS the game's pCelList record array (the port of it, [M cel.md]): one node per main-map
/// placement that creates a cel (`flags & 0x8000`), in placement order, with the object's glist bbox
/// (the box `build_FindCel` 0x1c8cb0 tests). `portals` = map block 0x21 with each endpoint remapped
/// into `cells` (the game matches `runtime_cel+0x38 == pCelList + idx*0x50` [M 0x1d15f8]; a portal
/// whose endpoint placement has no resolvable object is dropped). `room_to_cell` maps the port's room
/// cel index (a `parse_room_cels` / `Scene::cels` index) to its `cells` index.
#[derive(Clone)]
pub struct CelGraph {
    pub cells: Vec<CelNode>,
    pub portals: Vec<PortalEdge>,
    pub room_to_cell: Vec<Option<usize>>,
    /// Portal records dropped because an endpoint cel placement did not resolve.
    pub dropped_portals: usize,
    /// `View_AddCels` (0x1e6ba0)'s `cel+0x18` chain per cell. The chain threads through the cel
    /// records' +0x18 field [M 0x1e6ba0: the head is the VisCelList cel's +0x18, each node's next
    /// is node+0x18]; only room-class cels own chains and only the generic (runtime class 0x8019/
    /// 0x8018) cels ride them as nodes. Built by `build_chains`: each generic hangs on the room
    /// `build_FindCel` (0x1c8cb0) picks for its world-sphere centre — the bbox candidates plus the
    /// collision up/down ray tie-break [L ee_start_05.bin + ee_snapshot.bin: 8 room owners, 113
    /// members, 0 members outside their owner's bbox, 65/65 multi-bbox owners matching the
    /// FindCel tie-break; L live 07000006: the eye cel's chain empty, the big room 0x0179fb30's
    /// chain = 40 generics all inside its bbox]. Node order within a chain is reverse parse
    /// order [L ground-truth doc: the 40-node dump descends by runtime address] — the builder
    /// prepends. fix-portal4 (the old per-map-block parse-order model is refuted by that data).
    pub chains: Vec<Vec<u32>>,
    /// The main map's placements' own `cells` index, by placement ordinal (`None` = the placement
    /// does not create a cel). A cel placement's glist draws when ITS cel is visible
    /// (`View_AddCels` 0x1e6ba0), not when the room its centre lands in is. fix-portal2.
    pub own_cells_main: Vec<Option<u32>>,
    /// Per portal edge, the linked door's `Doors::list` index — `build_LinkDoors2Portals`
    /// (0x1c8ac8, block 0x21's door half) writes the door object into the runtime portal's +0xC
    /// (both halves; `Door_Link2Portal` 0x136788 gates which doors qualify). `None` = no door on
    /// that portal. The vision walk skips a linked portal while `Door_IsClosed` (0x1367b0) reads
    /// closed [M `Vision_Recurse` 0x1e82e0:65-70]. Built by `link_doors_to_portals` at load.
    pub portal_doors: Vec<Option<u32>>,
}

#[derive(Clone, Copy)]
pub struct CelNode {
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// The runtime cel's flags word (`cel+0x3c`), derived per class by `parseentity_fixup_entity`
    /// 0x1d00c0: the 0xc023/0xc047 room classes become **0x804c023/0x804c047** (bit 0x40000 = the
    /// room bit `build_FindCel` 0x1c8cb0 collects; the placement's raw word 0xc047 does NOT carry
    /// it), the generic classes 0x8019. fix-portal.
    pub flags: u32,
    /// The cel's world (centre, radius) sphere — `cel+0x80..0x8c`, the pCelList record's +0x10..0x1c
    /// local sphere transformed by the placement matrix (`parseentity_transform_bounding_box`
    /// 0x1d0fb0 [M]; the port derives it from the placement-transformed bbox corners). This is what
    /// `Vision_InView` (0x1e89e0) tests for the `+0x18` chain pass; `min`/`max` stay raw (the game's
    /// `build_FindCel` 0x1c8cb0 tests the raw glist bbox copied to cel+0x40..0x58). fix-portal2.
    pub sphere: ([f32; 3], f32),
}

/// One block-0x21 portal as two half-edges: `corners` in file order, the FORWARD plane built
/// `Plane_PlaneEq(plane, ref = corner 0, corner 1, corner 2)` [M 0x1d15f8] — its normal points into
/// celA, so the runtime node attached to celB (dest celA) carries it [M 0x1c8928] and the node
/// attached to celA carries its reverse. Side flags per record [M 0x1d15f8]: the celB node walks
/// while `(flags & 2) == 0`, the celA node while `(flags & 1) == 0`.
#[derive(Clone, Copy)]
pub struct PortalEdge {
    pub corners: [[f32; 3]; 4],
    pub n_fwd: [f32; 3],
    pub d_fwd: f32,
    pub cel_a: usize,
    pub cel_b: usize,
    pub flags: u16,
}

/// `Plane_PlaneEq` (0x1df8d0) [M]: n = normalize(cross(v1 − r, v2 − v1)), d = −dot(n, r); the zero
/// vector when the points are collinear (the game stores the zero plane and the walk's dot test then
/// never passes — the same effect here). Value form: `dot(n, p) + d`, zero on the plane.
pub fn plane_eq(r: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 4] {
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let e = sub(v2, v1);
    let cr = cross(sub(v1, r), e);
    let len = (cr[0] * cr[0] + cr[1] * cr[1] + cr[2] * cr[2]).sqrt();
    if len == 0.0 { return [0.0; 4]; }
    let n = [cr[0] / len, cr[1] / len, cr[2] / len];
    [n[0], n[1], n[2], -(n[0] * r[0] + n[1] * r[1] + n[2] * r[2])]
}

/// The level's cel graph, built alongside `parse_room_cels` (which is this function's room half —
/// the two MUST agree; `Scene::cels` is `.0`).
/// The level's cel graph, built alongside `parse_room_cels` (which is this function's room half —
/// the two MUST agree; `Scene::cels` is `.0`).
///
/// `cells` spans the WHOLE parse in archive order — the game's pCelList record array is parse-wide
/// (parsemap_block_map_data_static appends one 0x50-byte record per cel placement as each map is
/// parsed, sub-maps first, main map last), and the main map's portal block indexes THAT array.
/// *Measured* [D]: 07000002's main-map portal records point past the main map's own cel placements
/// (22 of 37 endpoints) — they address sub-map cels too.
/// The level's cel graph, built alongside `parse_room_cels` (which is this function's room half —
/// the two MUST agree; `Scene::cels` is `.0`).
///
/// The pCelList record index: `parsemap_block_map_data_static` (0x1d1998) addresses the record as
/// `pCelList + placement.index * 0x50` for by-index placements (the by-hash path goes through the
/// entity hashtable, untraced — those records are unreachable here and the portals naming them are
/// counted in `dropped_portals`). Records the port cannot resolve are the *sparse* remainder of the
/// game's array; only the named ones exist as nodes.
/// The collision scope graph from the parsed cel graph (fix-safe08b): plain bboxes + the portals as
/// the two triangles `Intersect_Portal` (0x1ea7b8) tests, for `CollisionWorld::build_scoped`.
/// Pub for the census tests to drive the exact build path (`Scene::build`'s sequence) headlessly
/// (fix-missing-objects).
pub fn scope_graph(g: &CelGraph) -> nf_game::world::ScopeGraph {
    nf_game::world::ScopeGraph {
        cells: g.cells.iter().map(|c| (c.min, c.max)).collect(),
        portals: g.portals.iter().map(|p| nf_game::world::ScopePortal {
            tris: [[p.corners[0], p.corners[1], p.corners[2]], [p.corners[0], p.corners[2], p.corners[3]]],
            a: p.cel_a, b: p.cel_b,
            // The game's portal+0x70: the 4 corners' average [M build_alloc_portal 0x1c8928]
            // (fix-08-collision2 — Intersect_Portal's 0x800 path projects against it).
            centre: [
                (p.corners[0][0] + p.corners[1][0] + p.corners[2][0] + p.corners[3][0]) * 0.25,
                (p.corners[0][1] + p.corners[1][1] + p.corners[2][1] + p.corners[3][1]) * 0.25,
                (p.corners[0][2] + p.corners[1][2] + p.corners[2][2] + p.corners[3][2]) * 0.25,
            ],
        }).collect(),
        room: g.cells.iter().map(|c| c.flags & 0x40000 != 0).collect(),
    }
}

/// The runtime cel's `+0x3c` flags word for a placement whose low 16 flag bits are `flags16`
/// [M `parseentity_fixup_entity` 0x1d00c0: the 0xc023 → 0x804c023, 0xc047 → 0x804c047, the
/// 0xd8..0xf3/0x8017..0x8019 generics → 0x8019, 0x803d → 0x28019; the unlisted words leave the
/// `build_alloc_cel` 0x1c93f8 zero]. The port's reads need the room bit (0x40000) only.
fn cel_flags_word(flags16: u16) -> u32 {
    match flags16 {
        0xc023 | 0xc047 => 0x804c000 | flags16 as u32,
        0x803d => 0x28019,
        0xf | 0x20 | 0x4e | 0xd8..=0xf3 | 0x8017..=0x8019 => 0x8019,
        _ => 0,
    }
}

pub fn parse_cel_graph(level: &Level) -> (Vec<Cel>, CelGraph) {
    let main = level.main_map();
    let mut cells: Vec<CelNode> = Vec::new();
    let mut rooms: Vec<Cel> = Vec::new();
    let mut room_to_cell: Vec<Option<usize>> = Vec::new();
    let mut cell_of_record: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut chains: Vec<Vec<u32>> = Vec::new();
    let mut own_cells_main: Vec<Option<u32>> = Vec::new();
    for (mi, map) in level.maps.iter().enumerate() {
        for p in &map.placements {
            if !p.creates_cel() {
                if Some(mi) == main { own_cells_main.push(None); }
                continue;
            }
            let flags16 = (p.flags & 0xffff) as u16;
            let param = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v as u8).unwrap_or(0);
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < map.objects.len() { Some((mi, p.index as usize)) } else { None };
            let is_main = Some(mi) == main;
            let idx = cells.len();
            // The raw param values: the fixup reads param 3 (`level_tag+0x38`) as a full u32 and
            // param 9 (`level_tag+0x50`) as a u16 — neither through the low-byte view of `param`.
            let param32 = |k: u32| p.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
            let node_flags = cel_flags_word(flags16);
            // The world sphere: the object's raw bbox corners through the placement matrix, then the
            // AABB's centre / half-diagonal (`parseentity_transform_bounding_box` 0x1d0fb0).
            // (Merge resolution: this block landed on master with two corrupted lines — the `[m`/`[h`
            // byte pairs dropped from the corner-array and half-diagonal literals; restored here.)
            let sphere = |min: [f32; 3], max: [f32; 3]| {
                let m = nf_formats::placement::object_matrix(p.rotation, p.translation);
                let mut lo = [f32::MAX; 3];
                let mut hi = [f32::MIN; 3];
                for cx in [min[0], max[0]] { for cy in [min[1], max[1]] { for cz in [min[2], max[2]] {
                    let w = nf_formats::placement::transform(&m, [cx, cy, cz]);
                    for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); }
                } } }
                let c3 = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
                let d = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
                (c3, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() * 0.5)
            };
            match obj {
                Some((om, oi)) => {
                    let o = &level.maps[om].objects[oi];
                    let sp = sphere(o.min, o.max);
                    cells.push(CelNode { min: o.min, max: o.max, flags: node_flags, sphere: sp });
                    if is_main && matches!(flags16, 0xc023 | 0xc047) {
                        let mut a0 = [param(0), param(1), param(2)];
                        if a0 == [0, 0, 0] { a0 = [0x3c; 3]; }
                        let a1 = if flags16 == 0xc047 {
                            let b = [param(6), param(7), param(8)];
                            if b == [0, 0, 0] { a0 } else { b }
                        } else { a0 };
                        // `parseentity_fixup_entity` 0x1d00c0: +0x98 = 0x9898 for 0xc023 (the
                        // underwater sentinel), param 9 as a raw u16 for 0xc047; the 0x10000000
                        // flags bit (the underwater flag) = param 3 != 0, both classes.
                        let env = if flags16 == 0xc023 { -0x6768i16 } else { param32(9) as u16 as i16 };
                        rooms.push(Cel { min: o.min, max: o.max, ambient: [a0, a1], zone: if flags16 == 0xc047 { param(5) } else { 0 },
                            water: if flags16 == 0xc023 { o.max[1] } else { -500.0 },
                            env, underwater: param32(3) != 0 });
                        room_to_cell.push(Some(idx));
                    }
                }
                None => { cells.push(CelNode { min: [0.0; 3], max: [0.0; 3], flags: cel_flags_word(flags16), sphere: ([0.0; 3], 0.0) }); }
            }
            chains.push(Vec::new());
            // (The chains are built AFTER the parse by `build_chains` — the game's own builder
            // hangs each generic on the room `build_FindCel` 0x1c8cb0 picks for its centre, a
            // collision raycast tie-break that needs the whole cel set + the scoped world.)
            if is_main { own_cells_main.push(Some(idx as u32)); }
            // The record address is `pCelList + index*0x50` [M 0x1d1998] — by-index placements only.
            if !p.by_hash() { cell_of_record.insert(p.index as usize, idx); }
        }
    }
    let mut portals = Vec::new();
    let mut dropped = 0usize;
    for map in &level.maps {
        for q in &map.portals {
            let (Some(a), Some(b)) = (cell_of_record.get(&(q.cel_a as usize)).copied(), cell_of_record.get(&(q.cel_b as usize)).copied()) else {
                dropped += 1;
                continue;
            };
            // Forward plane: Plane_PlaneEq(plane, ref = c0, v1 = c1, v2 = c2) [M 0x1d15f8 — the
            // parse calls it over the corner array in order, then over the reversed copy for the
            // B side].
            let pl = plane_eq(q.corners[0], q.corners[1], q.corners[2]);
            portals.push(PortalEdge { corners: q.corners, n_fwd: [pl[0], pl[1], pl[2]], d_fwd: pl[3], cel_a: a, cel_b: b, flags: q.flags });
        }
    }
    let portal_doors: Vec<Option<u32>> = vec![None; portals.len()];
    (rooms, CelGraph { cells, portals, room_to_cell, dropped_portals: dropped, chains, own_cells_main, portal_doors })
}

/// `build_LinkDoors2Portals` (0x1c8ac8 — map block 0x21's door half, GAPS G-035). Every door
/// object (class 0x1C) with a runtime record and the portal-link flag — `Door_Link2Portal`
/// 0x136788 = `obj+0xFD >> 3 & 1`, set by `Door_Create` when the placement's param 4 != 0, the
/// port's `Door::straddle` — is linked to the portal quad it sits in: the game straddles the
/// door's gfx sphere (obj+0x80/+0x8C) through the portal graph (`Collide_StraddleCels` 0x1eac98)
/// and tests each straddled cel's portal chain for the quad whose PLANE lies within 0.25 of the
/// sphere centre (`ABS(dot) < 0.25`, 0x1c8c34) and whose polygon contains it
/// (`vecutil_point_on_poly` 0x1ec940 over the node's two triangles, the quad (c0,c1,c2) +
/// (c2,c3,c0)). A match writes the door into `portal+0xC` on BOTH halves (the matched node and
/// its twin at +0x4 — `if (puVar7[3] == 0)`, first door wins) and sets `obj+0xF0 |= 0x8000`.
/// The port's `PortalEdge` is the pair of runtime halves, so one edge field carries the link.
/// The straddle pre-pass is folded into the containment test: a quad containing the centre
/// within 0.25 of its plane is always touched by the door's sphere (dist-to-tri ≤ 0.25 < the
/// door radii measured on our levels, ≥ 1.1), so the game's straddle set provably contains both
/// boundary cels of any quad this test accepts — the filter cannot change the match. Returns the
/// number of doors that linked (a door may match several quads; an edge keeps its first door).
pub fn link_doors_to_portals(graph: &mut CelGraph, doors: &nf_game::door::Doors) -> usize {
    let quad_centre = |c: &[[f32; 3]; 4]| [
        (c[0][0] + c[1][0] + c[2][0] + c[3][0]) * 0.25,
        (c[0][1] + c[1][1] + c[2][1] + c[3][1]) * 0.25,
        (c[0][2] + c[1][2] + c[2][2] + c[3][2]) * 0.25,
    ];
    // `vecutil_point_on_poly` 0x1ec940's barycentric containment over one triangle (the door
    // centre's projection is within 0.25 of the plane, far inside the epsilon).
    let tri_contains = |a: [f32; 3], b: [f32; 3], c: [f32; 3], x: [f32; 3]| {
        let dot = |p: [f32; 3], q: [f32; 3]| p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
        let v0 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let v1 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let v2 = [x[0] - a[0], x[1] - a[1], x[2] - a[2]];
        let (d00, d01, d11) = (dot(v0, v0), dot(v0, v1), dot(v1, v1));
        let (d20, d21) = (dot(v2, v0), dot(v2, v1));
        let den = d00 * d11 - d01 * d01;
        if den.abs() < 1e-12 { return false; }
        let u = (d11 * d20 - d01 * d21) / den;
        let v = (d00 * d21 - d01 * d20) / den;
        u >= -1e-4 && v >= -1e-4 && u + v <= 1.0 + 1e-4
    };
    let mut linked = 0usize;
    for (di, d) in doors.list.iter().enumerate() {
        // The game's `Door_Link2Portal` gate (obj+0xFD bit 3) and its record guard (+0x20 != 0 —
        // every created door has one). Doors created before the cel graph would straddle nothing.
        if !d.straddle { continue; }
        let x = d.sph.0;
        let mut matched = false;
        for (pi, p) in graph.portals.iter().enumerate() {
            // First door wins per edge (`if (puVar7[3] == 0)`).
            if graph.portal_doors[pi].is_some() { continue; }
            let dist = (p.n_fwd[0] * x[0] + p.n_fwd[1] * x[1] + p.n_fwd[2] * x[2] + p.d_fwd).abs();
            if dist >= 0.25 { continue; }
            let c = &p.corners;
            if tri_contains(c[0], c[1], c[2], x) || tri_contains(c[2], c[3], c[0], x) {
                graph.portal_doors[pi] = Some(di as u32);
                matched = true;
            }
        }
        if matched { linked += 1; }
    }
    if std::env::var_os("NF_PORTAL_DEBUG").is_some() {
        for (pi, pd) in graph.portal_doors.iter().enumerate() {
            if let Some(di) = pd {
                let p = &graph.portals[pi];
                eprintln!("[doorlink] portal {pi} (cells {} <-> {}) <- door {di} (placement {})", p.cel_a, p.cel_b, doors.list[*di as usize].placement);
            }
        }
    }
    linked
}

/// The `+0x18` chain builder (fix-portal4, the owner rule fix-portal5). The game hangs each generic
/// cel (runtime class 0x8019/0x8018 — the placement flag words `cel_flags_word` maps there) on a
/// room's chain at LOAD time by `build_link_objects_to_rooms` (0x1c9150): the owner is picked among
/// the ROOM-class cels (short class 0xC047/0xC023) whose radius ≥ the generic's and whose world bbox
/// contains the generic's sphere centre, by `Collide_SphereIntersect` (0x1ec940) — a sphere at the
/// generic's centre with radius 4001 (containing), scoped to the candidate room, i.e. the candidate
/// whose collision sits CLOSEST to the centre; strict `<` keeps the first candidate in parse order.
/// The game's chain-builder rule, ported (fix-portal4's order + fix-portal5's owner rule — the
/// old `find_cell_raycast` stand-in, which hung the 9 courtyard extras on the big room 21 and the
/// wall generics on the wrong rooms, is GONE from this path). Evidence [L ee_start_05.bin +
/// ee_snapshot.bin, both walked +0x18-head/next]: 8 room owners / 113 generic members / 0 members
/// outside the owner's world bbox (+0x40 min vec4, +0x50 max vec4) / no orphan generic (every
/// 0x8019-class cel rides some chain). Live [L ground-truth doc]: the eye cel's chain is empty,
/// the big room 0x0179fb30's chain is exactly the 40 courtyard generics. `world` (the sim's
/// scoped `CollisionWorld`) supplies the closest-hit tie-break (and the radius/bbox gates stand
/// alone); `None` keeps `find_cell_raycast`'s bbox-face approximation (the world-less callers).
/// Nodes are prepended so a chain reads in reverse parse order — the live dump order (the game
/// walks its +0x14 list in parse order, prepending).
pub fn build_chains(graph: &mut CelGraph, world: Option<&nf_game::world::CollisionWorld>) {
    let mut chains = std::mem::take(&mut graph.chains);
    for i in 0..graph.cells.len() {
        let f16 = graph.cells[i].flags & 0xffff;
        if !matches!(f16, 0x8018 | 0x8019) { continue; }
        let node = &graph.cells[i];
        // The port's unresolvable-object cells (zero bbox/sphere) stand for placements the parse
        // could not resolve — the game's records always carry real data, and `View_AddCels`
        // (0x1e6ba0) skips chain nodes whose glist pointer is null; skip them here too.
        if node.min == [0.0; 3] && node.max == [0.0; 3] && node.sphere.1 == 0.0 { continue; }
        let owner = match world {
            Some(w) => link_owner_room(graph, i, w),
            None => find_cell_raycast(graph, node.sphere.0, None),
        };
        if let Some(o) = owner {
            if o < chains.len() && o != i { chains[o].insert(0, i as u32); }
        }
    }
    graph.chains = chains;
}

/// `build_link_objects_to_rooms` (0x1c9150)'s owner pick for the generic cel `generic`: the
/// room-class candidate (short class 0xC047/0xC023, i.e. the port's flags word & 0xffff) with
/// `generic.radius <= room.radius` and the generic's sphere centre inside the room's world bbox,
/// ranked by the room's closest collision point to the centre (`Collide_SphereIntersect` 0x1ec940's
/// scoped closest hit with a containing sphere — `world.closest_point_scoped`); strict `<` keeps
/// the first candidate in parse order [M 0x1c9150 walk]. fix-missing-objects: a candidate whose
/// cel holds no collision reads the game's miss distance 1e8 and STILL participates ("hits whose
/// +0x54 cel is the candidate, or 0/unmapped, intersect; everything else reads 1e8" — fix-portal5
/// §4's disassembly reading), so when no candidate hits, strict `<` keeps the FIRST candidate in
/// parse order — the game leaves a generic unchained only when NO room passes the radius+bbox
/// gates. (The old `continue` on an empty bucket dropped the candidate entirely: with every
/// candidate's bucket empty — the 0700000a office furniture's case — the generic went unchained
/// and `View_AddCels` never drew it.)
fn link_owner_room(graph: &CelGraph, generic: usize, world: &nf_game::world::CollisionWorld) -> Option<usize> {
    let g = &graph.cells[generic];
    let mut best: Option<(usize, f32)> = None;
    for (ri, r) in graph.cells.iter().enumerate() {
        if !matches!(r.flags & 0xffff, 0xc023 | 0xc047) { continue; }
        if g.sphere.1 > r.sphere.1 { continue; }
        if !(0..3).all(|k| r.min[k] <= g.sphere.0[k] && g.sphere.0[k] <= r.max[k]) { continue; }
        let d = world.closest_point_scoped(ri, g.sphere.0).unwrap_or(1.0e8);
        if best.map_or(true, |(_, bd)| d < bd) { best = Some((ri, d)); }
    }
    best.map(|(i, _)| i)
}

/// `Vision_InView` (0x1e89e0) [M]: visible iff the node's (centre, radius) sphere meets each of the
/// four viewer+0x70 side planes. The port reads the game's test as the signed-distance form
/// `n·c + d + r >= 0`. The RAW decompile is `dot((cx,cy,cz,r),(n,d)) < r -> reject` (c.lt.s f0, f12
/// + bc1t -> return 0; the decompile's `< -param_1` is a Ghidra polarity artifact), i.e. the literal
/// `n·c + d·r >= r` — but that form is untestable-absurd with the game's OWN stored planes [L
/// ee_start_05.bin: |n| = 1, n·eye + d = 0, d = 51..117 — the fustrum builder 0x1e7df8 anchors every
/// side plane at the eye through the far-quad corners, so d = -n·eye scales with the eye's distance
/// from the world origin]: at the 05 start (|eye| = 153) the d·r term adds 130..300 of slack per
/// plane (the cull degenerates to draw-everything), and at the 07000007 clock-hall spot (the eye
/// looking away from the origin, d = -69.6 on the left plane) it subtracts 184 and rejects the
/// grandfather clock at r = 2.65 that the real game DRAWS [L ground truth]. The two readings agree
/// exactly when |d| << 1, and the signed-distance form is that common limit — the port ships it and
/// lists the open question (what the game's +0x70 quads hold mid-gameplay) under the report's live
/// checks. fix-portal6 (the fix-portal5 regression: every chain node of a visited room failed this
/// gate at the clock hall, dropping the clock + the wall shields the game shows).
/// The port derives (centre, radius) from the cel's glist bbox (the game's sphere comes from the
/// pCelList record's +0x10..0x1c) — documented as the approximation. fix-portal2.
pub fn vision_in_view(centre: [f32; 3], radius: f32, sides: &[[f32; 4]; 4]) -> bool {
    sides.iter().all(|p| p[0] * centre[0] + p[1] * centre[1] + p[2] * centre[2] + p[3] + radius >= 0.0)
}

/// A cel's (centre, radius) sphere from its bbox — the `Vision_InView` input.
fn cel_sphere(c: &CelNode) -> ([f32; 3], f32) { c.sphere }

/// The game's World-glist visibility: the VisCelList walk PLUS `View_AddCels` (0x1e6ba0)'s `+0x18`
/// chain pass — for every visited cel, each chain node draws when `Vision_InView` passes, skipping
/// 0x8018-class nodes while the viewer's cel is 0xc023-class [M 0x1e6ba0:47-56]. `sides` = the
/// frustum's four side planes (the viewer+0x70 set). Returns the per-`cells` mask; the strict
/// VisCelList mask (what `View_DrawObjList`'s object cull keys on — chain nodes never enter the
/// VisCelList, `Vision_AddCelToDraw` 0x1e8908 is the only inserter) stays
/// `portal_visible_cels_from_cell`. fix-portal2.
pub fn portal_visible_cels_game_from_cell(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], sides: &[[f32; 4]; 4]) -> Option<Vec<bool>> {
    portal_visible_cels_game_from_cell_doors(graph, start, eye, depth0, sides, None)
}

/// The game mask with the door links honoured (`Vision_Recurse` 0x1e82e0's `Door_IsClosed` gate,
/// G-035) — the draw path's walk. `door_closed` = `Doors::closed_flags()` at this tick; `None` =
/// the doors-less walk.
pub fn portal_visible_cels_game_from_cell_doors(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], sides: &[[f32; 4]; 4], door_closed: Option<&[bool]>) -> Option<Vec<bool>> {
    let chain = portal_cel_chain_from_cell_doors(graph, start, eye, depth0, door_closed)?;
    let mut vis = vec![false; graph.cells.len()];
    for &c in &chain { vis[c] = true; }
    let start_class = graph.cells[start].flags & 0xffff;
    for &c in &chain {
        for &n in &graph.chains[c] {
            let n = n as usize;
            if vis[n] { continue; }
            let node = &graph.cells[n];
            if node.flags & 0xffff == 0x8018 && start_class == 0xc023 { continue; }
            let (c3, r) = cel_sphere(node);
            if vision_in_view(c3, r, sides) { vis[n] = true; }
        }
    }
    Some(vis)
}

/// The sweep's LOS ray (fix-portal-sweep): the segment from `eye` to `target` (the closest point
/// of a cel's bbox) against the STATIC collision world, stopped 0.5 units short of the target —
/// the closest bbox point sits on the cel's own collision surface and the collision meshes run up
/// to ~0.3 units in front of the render glist bboxes [D the 07000005 trench probe: node 40's face
/// at 0.27 units], so a full-length ray always "hits" the very cel it is trying to clear. An
/// occluder still blocks whenever its surface is farther than the margin from the target.
/// `d` = the eye→target distance (0 = the eye inside the bbox).
pub fn los_ray_clear(world: &nf_game::world::CollisionWorld, eye: [f32; 3], target: [f32; 3], d: f32) -> bool {
    const LOS_SHORT: f32 = 0.5;
    let dir = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    let shrink = if d > LOS_SHORT { (d - LOS_SHORT) / d } else { 1.0 };
    let to = [eye[0] + dir[0] * shrink, eye[1] + dir[1] * shrink, eye[2] + dir[2] * shrink];
    world.segment_hit_static(eye, to).is_none()
}

/// The near-LOS rescue (fix-portal-sweep): 07000005
/// `--bond 16.67,30.04,58.60,1.755,-0.001` (the snow trench between stone walls) culling the near
/// trench walls with the walk ON. The chain there is sound per the game's own rules — the trench
/// generics sit (per `build_link_objects_to_rooms` 0x1c9150's closest-hit owner pick, the metric
/// being `Intersect_PointGeom`'s point-to-geometry distance [M 0x1ec940→0x1ebaa8 type 0x101]) on
/// the chains of the overlapping rooms 15/24, and `View_AddCels` (0x1e6ba0) only walks the chains
/// of VisCelList rooms, so the game itself draws them only because its eye cel resolves to 15
/// there — `build_FindCel` (0x1c8cb0)'s per-candidate up/down ray tie-break, whose outcome depends
/// on the per-cel object lists the port only approximates (`world.closest_point_scoped`
/// attribution). Where that approximation disagrees, the walk drops geometry the player is
/// touching. This pass restores the game's own floor: a cel the port's walk drops still draws when
/// (1) `Vision_InView` (0x1e89e0, the signed-distance form fix-portal6 ships) keeps its sphere,
/// (2) the closest point of its bbox is within `max_dist` of the eye, and (3) the segment from the
/// eye to that point crosses no collision (the sweep's ray). Only the generic classes
/// `build_link_objects_to_rooms` links ride it (0x8017..0x8019 — the chain nodes; room cels and
/// their objects stay on the strict `VisCelList` mask, so intentionally hidden rooms — the
/// 07000007 ballroom-window building the game hides [L ground-truth doc] — stay hidden).
/// Returns the number of rescued cels.
pub fn near_los_rescue(
    graph: &CelGraph, mask: &mut [bool], eye: [f32; 3], sides: &[[f32; 4]; 4],
    world: &nf_game::world::CollisionWorld, max_dist: f32,
) -> usize {
    let mut rescued = 0usize;
    for (i, c) in graph.cells.iter().enumerate() {
        if mask[i] { continue; }
        if !matches!(c.flags & 0xffff, 0x8017 | 0x8018 | 0x8019) { continue; }
        // The port's unresolvable-object cells stand for placements the parse could not resolve
        // (build_chains skips them the same way); a non-finite or reversed bbox would panic the
        // clamp below and can never pass the game's own tests — skip both.
        if c.min == [0.0; 3] && c.max == [0.0; 3] && c.sphere.1 == 0.0 { continue; }
        if (0..3).any(|k| !c.min[k].is_finite() || !c.max[k].is_finite() || c.min[k] > c.max[k]) { continue; }
        if !vision_in_view(c.sphere.0, c.sphere.1, sides) { continue; }
        let mut tp = [0f32; 3];
        for k in 0..3 { tp[k] = eye[k].clamp(c.min[k], c.max[k]); }
        let d = ((tp[0] - eye[0]).powi(2) + (tp[1] - eye[1]).powi(2) + (tp[2] - eye[2]).powi(2)).sqrt();
        if d > max_dist { continue; }
        if !los_ray_clear(world, eye, tp, d) { continue; }
        mask[i] = true;
        rescued += 1;
    }
    rescued
}

/// One `ClipPortalWithPlane` (0x1e7fa0) pass: Sutherland–Hodgman keeping `value >= 0`, with the game's
/// compaction (a vertex is kept only when its successor is farther than √1e-7). `None` = every vertex
/// inside (the game's return 0 — the polygon passes through unchanged), `Some(v)` = the clipped
/// polygon (possibly empty = fully outside).
fn clip_positive(poly: &[[f32; 3]], plane: [f32; 4]) -> Option<Vec<[f32; 3]>> {
    let val = |p: &[f32; 3]| plane[0] * p[0] + plane[1] * p[1] + plane[2] * p[2] + plane[3];
    if !poly.iter().any(|p| val(p) < 0.0) { return None; }
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 2);
    for i in 0..n {
        let (cur, nxt) = (poly[i], poly[(i + 1) % n]);
        let (cin, nin) = (val(&cur) >= 0.0, val(&nxt) >= 0.0);
        let denom = plane[0] * (cur[0] - nxt[0]) + plane[1] * (cur[1] - nxt[1]) + plane[2] * (cur[2] - nxt[2]);
        match (cin, nin) {
            (true, true) => out.push(nxt),
            (true, false) => {
                if denom != 0.0 {
                    let t = -val(&nxt) / denom;
                    out.push([nxt[0] + t * (cur[0] - nxt[0]), nxt[1] + t * (cur[1] - nxt[1]), nxt[2] + t * (cur[2] - nxt[2])]);
                }
            }
            (false, true) => {
                if denom != 0.0 {
                    let t = -val(&nxt) / denom;
                    out.push([nxt[0] + t * (cur[0] - nxt[0]), nxt[1] + t * (cur[1] - nxt[1]), nxt[2] + t * (cur[2] - nxt[2])]);
                }
                out.push(nxt);
            }
            (false, false) => {}
        }
    }
    // The game's compaction: keep vertex i only when its (wrapped) successor differs by more than
    // √1e-7 (Vec_SqDist3D over the out buffer, 0x1e82e0's copy-back loop).
    let n = out.len();
    let mut comp = Vec::with_capacity(n);
    for i in 0..n {
        let nx = if i + 1 != n { i + 1 } else { 0 };
        let d = [out[i][0] - out[nx][0], out[i][1] - out[nx][1], out[i][2] - out[nx][2]];
        if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] > 1e-7 { comp.push(out[i]); }
    }
    Some(comp)
}

/// The viewer's persistent cel state — `viewer+4` [M `Camera_CheckLocation` 0x1bae90]. The game
/// carries the eye's cel across frames; the port used to re-pick from scratch every frame, which is
/// the 07000006 collapse's enabling bug (the smallest-volume tie-break can flip between overlapping
/// room boxes mid-walk where the game keeps the cel it walked through).
#[derive(Clone, Copy, Default)]
pub struct ViewerCel {
    /// The current cell (`cells` index); `None` = the game's 0 (no cel yet).
    pub cell: Option<usize>,
    /// The position the cel was resolved at (`viewer+0x34`; `Cel_ObjectLeftCel`'s ray origin).
    pub pos: [f32; 3],
}

/// `Cel_ObjectLeftCel` (0x1e0d58) [M]: raycast the move `from -> to` against `cell`'s portal list —
/// the per-portal bbox pre-cull (+0x20/+0x24, the ±0.1 inset box `build_alloc_portal` 0x1c8928
/// fills) then two ray-triangle tests on the quad corners (+0xc/+0x10/+0x14 and +0x14/+0x18/+0xc —
/// corners (0,1,2) and (2,3,0)); a hit returns `portal+0x8` = the destination cel of the node the
/// walker's cel owns. The node attached to celB stores the forward corners (dest celA), the celA
/// node the reversed ones (dest celB) [M 0x1d15f8 / 0x1c8928] — either way the destination is the
/// OTHER endpoint, so the quad to test is the same record's.
fn portal_move_dest(graph: &CelGraph, cell: usize, from: [f32; 3], to: [f32; 3]) -> Option<usize> {
    let dir = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let hit = |tri: [[f32; 3]; 3]| -> bool {
        // Möller–Trumbore over the move segment [0, 1].
        let e1 = [tri[1][0] - tri[0][0], tri[1][1] - tri[0][1], tri[1][2] - tri[0][2]];
        let e2 = [tri[2][0] - tri[0][0], tri[2][1] - tri[0][1], tri[2][2] - tri[0][2]];
        let pv = [dir[1] * e2[2] - dir[2] * e2[1], dir[2] * e2[0] - dir[0] * e2[2], dir[0] * e2[1] - dir[1] * e2[0]];
        let det = e1[0] * pv[0] + e1[1] * pv[1] + e1[2] * pv[2];
        if det.abs() < 1e-12 { return false; }
        let inv = 1.0 / det;
        let tv = [from[0] - tri[0][0], from[1] - tri[0][1], from[2] - tri[0][2]];
        let u = (tv[0] * pv[0] + tv[1] * pv[1] + tv[2] * pv[2]) * inv;
        if !(-1e-9..=1.0 + 1e-9).contains(&u) { return false; }
        let qv = [tv[1] * e1[2] - tv[2] * e1[1], tv[2] * e1[0] - tv[0] * e1[2], tv[0] * e1[1] - tv[1] * e1[0]];
        let v = (dir[0] * qv[0] + dir[1] * qv[1] + dir[2] * qv[2]) * inv;
        if !(-1e-9..=1.0 + 1e-9).contains(&v) || u + v > 1.0 + 2e-9 { return false; }
        let t = (e2[0] * qv[0] + e2[1] * qv[1] + e2[2] * qv[2]) * inv;
        (0.0..=1.0).contains(&t)
    };
    for p in &graph.portals {
        let dest: usize = if p.cel_a == cell { p.cel_b } else if p.cel_b == cell { p.cel_a } else { continue };
        let c = &p.corners;
        if hit([c[0], c[1], c[2]]) || hit([c[2], c[3], c[0]]) { return Some(dest); }
    }
    None
}

/// `build_point_in_bounding_box` (0x1c9500) [M]: the inclusive bbox test over one cel.
fn point_in_cell(graph: &CelGraph, cell: usize, p: [f32; 3]) -> bool {
    graph.cells.get(cell).is_some_and(|c| (0..3).all(|k| c.min[k] <= p[k] && p[k] <= c.max[k]))
}

/// `build_FindCel` (0x1c8cb0) over the graph: collect the containing room cels (the 0x40000 flag
/// [M `*puVar1 & 0x40000`], cap 0x40). One candidate returns directly [M 0x1c8cb0: `if
/// CelListCount_154 != 1` guards the tie-break]. With several, the game's tie-break is a
/// COLLISION raycast per candidate [M 0x1c8cb0: `Collide_RayIntersect(param_1, up_end, cand, …)`
/// then the mirrored down ray — the down ray's length is the up ray's min(hit, len), len starts
/// 256 and shrinks to the best-so-far distance], the candidate with the nearest hit distance
/// wins, and with no hit at all the FIRST candidate stands (0x1c8cb0's tail:
/// `iVar6 = CelList_153; ... return iVar6`). The per-candidate ray is scoped to the CANDIDATE'S
/// OWN chain geometry — `Collide_RayIntersect`'s owner filter (flags arg = 1) exact-tests only
/// hits whose static-geometry cel equals the passed cel — so the port reads the candidate's own
/// scope bucket through `segment_hit_cell` (fix-portal-eyecel; the shared-world scoped query the
/// port stood in with let both candidates see the same floor and the first candidate stand).
/// Without a world the port's bbox-face approximation stands (documented).
pub fn find_cell_raycast(graph: &CelGraph, eye: [f32; 3], world: Option<&nf_game::world::CollisionWorld>) -> Option<usize> {
    let mut cands: Vec<usize> = Vec::new();
    for (i, c) in graph.cells.iter().enumerate() {
        if c.flags & 0x40000 == 0 { continue; }
        if (0..3).all(|k| c.min[k] <= eye[k] && eye[k] <= c.max[k]) {
            cands.push(i);
            if cands.len() >= 0x40 { break; }
        }
    }
    if cands.len() <= 1 { return cands.first().copied(); }
    if let Some(world) = world {
        // [M 0x1c8cb0]: len = the best-so-far distance (256 first); up ray, then the down ray
        // capped at min(up hit, len); the nearest hit distance wins; no hits -> the first.
        let mut best: Option<(usize, f32)> = None;
        for &i in &cands {
            let len = best.map_or(256.0, |(_, d)| d);
            let up = world.segment_hit_cell(i, eye, [eye[0], eye[1] + len, eye[2]]).map(|(t, _)| t * len);
            let dlen = up.map_or(len, |ud| ud.min(len));
            let down = world.segment_hit_cell(i, eye, [eye[0], eye[1] - dlen, eye[2]]).map(|(t, _)| t * dlen);
            match (up, down) {
                (None, None) => continue,
                (a, b) => {
                    let d = a.iter().chain(b.iter()).copied().fold(f32::MAX, f32::min);
                    if best.map_or(true, |(_, bd)| d < bd) { best = Some((i, d)); }
                }
            }
        }
        return Some(best.map_or(cands[0], |(i, _)| i));
    }
    // The no-world approximation: the nearest bbox face (the pre-fix-portal3 behaviour).
    {
        let mut best: Option<(usize, f32)> = None;
        for &i in &cands {
            let c = &graph.cells[i];
            let up = c.max[1] - eye[1];
            let down = eye[1] - c.min[1];
            let d = up.min(down);
            if d < 256.0 && best.map_or(true, |(_, bd)| d < bd) { best = Some((i, d)); }
        }
        Some(best.map_or(cands[0], |(i, _)| i))
    }
}

/// The eye's cel for this frame — `Camera_CheckLocation` (0x1bae90) [M], over the persistent state:
/// 1. `Cel_ObjectLeftCel` (0x1e0d58): the move raycast against the current cel's portals relinks
///    directly to the crossed portal's destination;
/// 2. `build_point_in_bounding_box` (0x1c9500): still inside the current cel's box keeps it;
/// 3. `build_FindCel` (0x1c8cb0): a fresh search;
/// 4. a FindCel miss KEEPS the previous cel, and a first-ever miss takes the world's first cel
///    (`puVar1[1] = *glb_world`) [M 0x1bae90:34-38].
///
/// `world` (the sim's scoped `CollisionWorld`) feeds `build_FindCel`'s collision tie-break
/// [M 0x1c8cb0]; `None` keeps the bbox-face approximation.
pub fn eye_cell(graph: &CelGraph, state: &mut ViewerCel, eye: [f32; 3], world: Option<&nf_game::world::CollisionWorld>) -> Option<usize> {
    if let Some(c) = state.cell {
        if let Some(dest) = portal_move_dest(graph, c, state.pos, eye) {
            state.pos = eye;
            state.cell = Some(dest);
            return Some(dest);
        }
        if point_in_cell(graph, c, eye) {
            state.pos = eye;
            return Some(c);
        }
    }
    let found = find_cell_raycast(graph, eye, world);
    state.pos = eye;
    match found {
        Some(c) => { state.cell = Some(c); Some(c) }
        None => {
            let keep = state.cell.or(Some(0)).filter(|&c| c < graph.cells.len());
            state.cell = keep;
            keep
        }
    }
}

/// The game's portal visibility walk (`Vision_Portal_Recurse` 0x1e86e8 → `Vision_Recurse` 0x1e82e0),
/// producing the per-`cells`-index visibility the renderer gates static World draws with (the
/// `View_AddCels` GfxList set: only portal-visited cels are added).
///
/// `depth0`: the 5-plane view pyramid — `Vision_Calculate_View_Fustrum` (0x1e7df8) builds the eye's
/// four side planes plus the near plane at depth 0; the port passes the frame frustum's first five
/// Gribb-Hartmann planes (the same five faces; convention inside ≥ 0, the clip's keep side).
/// Returns None when the walk cannot run (no graph, the eye's room cel unmapped) — the caller keeps
/// its previous cull. Deviation: the game skips a portal whose linked door `Door_IsClosed` (0x1367b0)
/// [M 0x1e82e0:51]; the port has no per-portal door link yet, so every portal is treated open —
/// conservative (it can only keep cels visible, never hide them).
pub fn portal_visible_cels(graph: &CelGraph, eye_room: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<bool>> {
    let start = graph.room_to_cell.get(eye_room).copied().flatten()?;
    portal_visible_cels_from_cell(graph, start, eye, depth0)
}

/// The walk from an already-resolved runtime cel (the `viewer+4` the game hands
/// `Vision_Portal_Recurse` 0x1e86e8 — `Camera_CheckLocation` 0x1bae90's state, not a per-frame
/// re-pick). fix-portal.
pub fn portal_visible_cels_from_cell(graph: &CelGraph, start_cell: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<bool>> {
    let chain = portal_cel_chain_from_cell(graph, start_cell, eye, depth0)?;
    let mut visible = vec![false; graph.cells.len()];
    for c in chain { visible[c] = true; }
    Some(visible)
}

/// `portal_visible_cels_from_cell` with the door links honoured (`Vision_Recurse` 0x1e82e0's
/// `Door_IsClosed` gate — G-035): `door_closed` = `Doors::closed_flags()` at this tick.
pub fn portal_visible_cels_from_cell_doors(graph: &CelGraph, start_cell: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], door_closed: Option<&[bool]>) -> Option<Vec<bool>> {
    let chain = portal_cel_chain_from_cell_doors(graph, start_cell, eye, depth0, door_closed)?;
    let mut visible = vec![false; graph.cells.len()];
    for c in chain { visible[c] = true; }
    Some(visible)
}

/// The game's `VisCelList` (0x34b3b0) chain order — what `View_AddCels` (0x1e6ba0) walks when it
/// hands out the −90000 + 10·k list-0 keys. `Vision_Portal_Recurse` (0x1e86e8) inserts the viewer's
/// cel at depth 0, `Vision_Recurse` (0x1e82e0) inserts each portal-reached cel at **first reach**
/// with depth = parent+1 (`*(short *)(&DAT_0034b3b4 + uVar9 * 8) = (short)iVar11`), then
/// `QuickSort` (0x1d30a8, comparator id 3) orders the array and it is threaded through `cel+0x20`.
/// Returns the sorted `cells` indices in that order (the viewer's cel first), or None when the walk
/// cannot run (same guards as `portal_visible_cels`, whose visibility mask this also is).
pub fn portal_cel_chain(graph: &CelGraph, eye_room: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<usize>> {
    let start = graph.room_to_cell.get(eye_room).copied().flatten()?;
    portal_cel_chain_from_cell(graph, start, eye, depth0)
}

/// The chain from a resolved cell — see `portal_cel_chain`.
pub fn portal_cel_chain_from_cell(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5]) -> Option<Vec<usize>> {
    portal_cel_chain_from_cell_doors(graph, start, eye, depth0, None)
}

/// The chain from a resolved cell, honouring the door links (`build_LinkDoors2Portals` 0x1c8ac8):
/// `door_closed` is the per-door `Door_IsClosed` (0x1367b0) snapshot (`Doors::closed_flags`), read
/// per portal the same order the game reads it — after the eye-side plane test, before the clip
/// [M `Vision_Recurse` 0x1e82e0:50-70]. `None` = the doors-less walk (the pre-G-035 behaviour).
///
/// This chain is the game's capture pipeline's tail, whose three never-mentioned members run
/// inside `View_CaptureScene` 0x1e6df0 [M its decomp — the only caller of all three]:
/// `Vision_Init_Portal_Recurse` 0x1e89a8 (reset `VisCelCount = 0`, `Mat_GetDir(viewer+0x120)`,
/// `Vision_Calculate_View_Fustrum` — the port's `depth0` plane build + chain start), the
/// `Vision_Portal_Recurse`/`Vision_Recurse` walk below, and `vision_generate_display_list`
/// 0x1e8848 (`QuickSort` comparator 3 over `VisCelList` by recursion depth, then the cels
/// threaded through `cel+0x20` with the sorted order — exactly the sort + order this fn
/// returns; the viewer+0x20 accumulate bit is the game's own draw bookkeeping). The eye
/// position itself is `vision_GetCamPos` 0x1e8990 (copies the 4 words at `ViewPoints`
/// 0x2d67d0 — the current viewer position — into a caller vector) — the port's `eye`
/// parameter, threaded from the frame camera by the renderer [M 001e8990/001e89a8/001e8848
/// decomp; ViewPoints in the symtab 0x2d67d0 size 80].
pub fn portal_cel_chain_from_cell_doors(graph: &CelGraph, start: usize, eye: [f32; 3], depth0: &[[f32; 4]; 5], door_closed: Option<&[bool]>) -> Option<Vec<usize>> {
    let mut visited = vec![false; graph.cells.len()];
    let mut planes: Vec<Vec<[f32; 4]>> = vec![Vec::new(); 48];
    planes[0] = depth0.to_vec();
    let mut budget = 0usize;
    // The VisCelList insertion order: the viewer's cel first (depth 0 — `Vision_Portal_Recurse`
    // inserts it before `Vision_Recurse` runs), then each first-reach cel at depth = parent+1.
    // `Vision_Recurse` [M 0x1e82e0]: the recursion budget (VisionRecurseCount > 100) and the depth
    // cap (45) are the game's own guards; VisCelCount caps at 0x40 cels.
    let mut order: Vec<(usize, u16)> = vec![(start, 0)];
    fn recurse(
        graph: &CelGraph, eye: [f32; 3], cell: usize, depth: usize,
        visited: &mut Vec<bool>, planes: &mut Vec<Vec<[f32; 4]>>, budget: &mut usize, order: &mut Vec<(usize, u16)>,
        door_closed: Option<&[bool]>,
    ) {
        *budget += 1;
        if *budget > 100 || depth > 45 { return; }
        let n0 = planes[depth].len();
        let dbg = std::env::var_os("NF_PORTAL_DEBUG").is_some();
        for (pi, p) in graph.portals.iter().enumerate() {
            let (side_open, dest, fwd) = if cell == p.cel_a {
                ((p.flags & 1) == 0, p.cel_b, false)
            } else if cell == p.cel_b {
                ((p.flags & 2) == 0, p.cel_a, true)
            } else { continue };
            if !side_open {
                if std::env::var_os("NF_PORTAL_DEBUG").is_some() { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: side closed (flags {:x})", p.flags); }
                continue;
            }
            // The side's plane: the node attached to celB carries the forward plane, the celA node
            // its reverse [M 0x1c8928 / 0x1d15f8]. `Vision_Recurse` processes a portal only when the
            // eye is on its negative side [M 0x1e82e0:50].
            let (n, d) = if fwd { (p.n_fwd, p.d_fwd) } else { ([-p.n_fwd[0], -p.n_fwd[1], -p.n_fwd[2]], -p.d_fwd) };
            if n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2] + d >= 0.0 {
                if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: eye on the wrong side of the portal plane"); }
                continue;
            }
            // `Vision_Recurse` 0x1e82e0:65-70: after the plane test, `Door_IsClosed` (0x1367b0)
            // on the portal's linked door (portal+0xC) ends this portal — a shut door's room is
            // not walked into (G-035). The game's walk runs the test for EVERY portal (a NULL
            // door reads open, `Door_IsClosed` 0x1367b0's `param_1 == 0` arm).
            if let Some(closed) = door_closed {
                if let Some(&Some(di)) = graph.portal_doors.get(pi) {
                    if closed.get(di as usize).copied().unwrap_or(false) {
                        if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: linked door {di} is shut"); }
                        continue;
                    }
                }
            }
            // The quad as the runtime node stores it: forward corners on the celB node, reversed
            // (c3,c2,c1,c0) on the celA node [M 0x1c8928's amemcpy sources].
            let mut poly: Vec<[f32; 3]> = if fwd { p.corners.to_vec() } else { vec![p.corners[3], p.corners[2], p.corners[1], p.corners[0]] };
            for k in 0..n0 {
                match clip_positive(&poly, planes[depth][k]) {
                    None => {}
                    Some(clipped) => { poly = clipped; }
                }
                if poly.is_empty() { break; }
            }
            if poly.is_empty() {
                if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: poly empty after clip (n0 {n0})"); }
                continue;
            }
            // The VisCelList insert cap: `if (VisCelCount < 0x40)` guards the dedup + insert
            // [M 0x1e82e0]; the recursion itself continues past the cap.
            if !visited[dest] && order.len() < 0x40 { visited[dest] = true; order.push((dest, (depth + 1) as u16)); }
            if dbg { eprintln!("[pwalk] d{depth} cell {cell} -> {dest}: poly {} (n0 {n0}, budget {budget}, {} new)", poly.len(), !visited[dest] as u8); }
            if poly.len() >= 3 {
                // The next depth's planes: one per clipped-polygon edge, ref = the eye (ViewPoints[0]
                // [M 0x1e82e0:146]), plus the portal's own plane last [M 0x1e82e0:151-155].
                let m = poly.len();
                let mut next: Vec<[f32; 4]> = (0..m).map(|i| plane_eq(eye, poly[i], poly[(i + 1) % m])).collect();
                next.push([n[0], n[1], n[2], d]);
                planes[depth + 1] = next;
                recurse(graph, eye, dest, depth + 1, visited, planes, budget, order, door_closed);
                if *budget > 100 { return; }
            }
        }
    }
    visited[start] = true;
    recurse(graph, eye, start, 0, &mut visited, &mut planes, &mut budget, &mut order, door_closed);
    quicksort_viscel(&mut order);
    Some(order.into_iter().map(|(c, _)| c).collect())
}

/// `QuickSort` (0x1d30a8) with comparator id 3 — jumptable 0x304fc0[3] → 0x1d3430, disassembled:
/// `lhu $a0, 4($a0); lhu $v1, 4($a1)` — the ascending **u16 portal depth** at element+4. Unstable:
/// Hoare partition, pivot = element `count>>1` swapped to the front, the smaller side pushed to the
/// stack (the decomp's byte-size test `(j − (lo+1)) < (hi − i)` at stride 8 is index-space
/// `(j − lo) <= (hi − i)`). The tie order feeds `View_AddCels`' −90000 + 10·k ordinals, so the port
/// sorts exactly like the game over the same insertion order.
pub fn quicksort_viscel(items: &mut [(usize, u16)]) {
    let n = items.len();
    if n < 2 { return; }
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let (mut lo, mut hi) = (0usize, n - 1);
    loop {
        // Pivot: element count>>1 swapped to the front (0x1d31a8's `Swap(((hi−lo)/stride+1)>>1·stride+base, base)`).
        let count = hi - lo + 1;
        items.swap(lo, lo + count / 2);
        let mut i = lo + 1;
        let mut j = hi + 1;
        loop {
            // `while (uVar3 <= uVar5 && Comp(uVar3, param_1) < 1)` — Comp <= 0 stays left.
            while i <= hi && items[i].1 <= items[lo].1 { i += 1; }
            j -= 1;
            // `while (param_1 < uVar2 && Comp(uVar2, param_1) > -1)` — Comp >= 0 stays right.
            while lo < j && items[j].1 >= items[lo].1 { j -= 1; }
            if j < i { break; }
            items.swap(i, j);
            i += 1;
        }
        items.swap(lo, j);
        if j - lo <= hi.saturating_sub(i) {
            // Left side smaller (or equal): push the right, loop on the left [LAB_001d32a8].
            if i < hi { stack.push((i, hi)); }
            if j > lo + 1 {
                hi = j - 1;
            } else {
                // Left is a single element: pop (LAB_001d32d0).
                match stack.pop() { Some((l, h)) => { lo = l; hi = h; } None => return }
            }
        } else {
            // Right side smaller: push the left, loop on the right.
            if lo + 1 < j { stack.push((lo, j - 1)); }
            lo = i;
            if lo >= hi {
                // The do-while's `i < hi` failed: pop.
                match stack.pop() { Some((l, h)) => { lo = l; hi = h; } None => return }
            }
        }
    }
}

/// `Lights_CalcAmbientLight` (0x1cb178): the object's ambient RGB steps toward its cel's ambient through
/// `ClrStep(…, 0.1)` (0x1e8a48: `(target−cur)·rate`, at least ±1 byte per step, disassembled), snapping
/// once on the first cel link when still at the 0xff,ff,ff init sentinel (`control_link_object_to_cel`
/// 0x1346b8). Ported on the Sim's per-character state in `Sim::ramp_char_ambient`.
fn clr_step(cur: u8, target: u8) -> u8 {
    let d = (target as f32 - cur as f32) * 0.1;
    let d = if (target as f32 - cur as f32) == 0.0 { return cur }
        else if d.abs() < 1.0 { if d < 0.0 { -1.0 } else { 1.0 } } else { d };
    (cur as f32 + d).clamp(0.0, 255.0) as u8
}

impl Sim {
    /// `build_FindCel` (0x1c8cb0) over the parsed room cels: inclusive bbox test, one match wins, no match
    /// → None. More than one match: the game raycasts each matched cel's OWN object list — up from the point
    /// (256 max), then down limited by the up hit — and keeps the cel with the nearest hit; no hit → the
    /// first match (0x1c8cb0 disassembly). nf-view has no per-cel object lists, so the smallest-volume cel
    /// wins (*deviation*). fix-render2 re-measured this against every [L] snapshot drone position on
    /// 07000001 (ee_start_07000001.bin: 15 drones in 6 cels, several inside two-cel overlaps) and on
    /// 07000005: the smallest-volume pick reproduces the game's ambient at every one of them, while a
    /// shared-geometry raycast mis-attributes vertically-adjacent geometry (the courtyard arch belongs to
    /// the room cel's list, not the big overlapping box's). Fix-render2's first-guard report ("the first
    /// guard looks too dark") could not be reproduced at any [L] position with this tie-break.
    pub fn cel_at(&self, p: [f32; 3]) -> Option<usize> { find_cel(&self.cels, p) }

    /// The goggles' view mode (viewer `+0x236`): 0 off, 1 night sight, 2 x-ray [M
    /// `HUD_MonitorNightSight` 0x19e9d8]. The renderer's consumers (the night/x-ray pane
    /// data, the crosshair, `AnimObjectDraw`'s night lighting, `BodyGlow_Update`'s mode-2
    /// glow) read this; docs/systems/goggles.md has the consumer list.
    pub fn goggle_mode(&self) -> u16 { self.hud.goggles.mode }

    /// The goggles battery (BL+0x920), 0..1800: −FRAME_RATE_MUL per tick while a mode is up,
    /// +3·FRAME_RATE_MUL capped at 1800 while off (the cap is the EE's `MIN.S` — disasm
    /// 0x19ec68, op 0x29 per ps2tek; FRAME_RATE_MUL = 1 at the fixed 60 Hz) [M 0x19e9d8:154-162].
    /// At ≤ 0 with a mode up the game zeroes it and drops the mode to 0 [M :163-193].
    pub fn goggle_battery(&self) -> f32 { self.hud.goggles.fade }
    /// The ambient triple a 0x800 object at `p` ramps toward: `base = cel + switch_channels[cel+0xa2]`,
    /// read at `base+0x9a/0x9c/0x9e` (`Lights_CalcAmbientLight` 0x1cb178). The port's live switch-channel
    /// table stands in for the game's; values above 1 clamp to 1 (only the 0/1 shift has defined bytes).
    pub fn ambient_at(&self, p: [f32; 3]) -> Option<[u8; 3]> {
        // fix-render3: a test-visible counter for the posed-vertices hoist (the per-vertex version
        // ran this through every pickup/attachment vertex; the hoisted one runs it once per key).
        self.ambient_lookups.set(self.ambient_lookups.get() + 1);
        let c = &self.cels[self.cel_at(p)?];
        let off = *self.drones.channels.v.get(c.zone as usize).unwrap_or(&0);
        Some(c.ambient[(off as usize).min(1)])
    }

    /// One `Lights_CalcAmbientLight` pass over the characters: the view models follow Bond's cel, drone-backed
    /// characters their drone's cel, script characters their entity matrix; characters with none keep the value
    /// snapped at build. The 0x10000 doubled targets (`glb_blokes+0x962` = Bond sees the sun, light-sun) apply to the view models — the
    /// goggles are not in the port yet, so the flag can never set.
    pub fn ramp_char_ambient(&mut self) {
        // light-sun: `HUD_UpdateLensFlarePane` 0x19d770:64-90 sets BLData+0x962 each frame when the
        // `SunVector` is non-zero and the head → sun line of sight is clear; it is
        // the gate of the 0x10000 doubled ambient below (the old note called it night vision — it is
        // the "Bond sees the sun" flag: 1 at the 0700000c start [L PCSX2 BLData+0x962]).
        let sv = self.drones.sun_vector;
        self.sun_seen = sv != [0.0; 3] && {
            let h = self.bond.head();
            // `Collide_LineOfSight(head, head − SunVector, cel, obj, 0, 8)` [M 0x19d770:84-85]: mask
            // 8 | 4 — bit 0x8 is the node-pass skip the bullet ray carries (`segment_bullet`'s class);
            // the raw segment stopped on the 0700000c start's invisible boundary at y 40.
            self.world.segment_surface_bullet(h, [h[0] - sv[0], h[1] - sv[1], h[2] - sv[2]]).is_none()
        };
        // The view model shares Bond's linked cel (obj+0x20 — the same cel on both objects at the
        // 0700000c start [L PCSX2]); `Lights_CalcAmbientLight` reads THAT cel, not a fresh bbox find.
        let bond_room = self.bond.cel.and_then(|cell| self.room_cell.iter().position(|c| *c == Some(cell)));
        // Positions first (the borrow of `self` ends before the mutable ambient walk).
        let positions: Vec<Option<[f32; 3]>> = (0..self.char_ambient.len()).map(|ci| {
            if self.view_model_chars.contains(&ci) {
                Some(self.bond.sphere_centre())
            } else if let Some(&(slot, _)) = self.mp_body_chars.iter().find(|(_, c)| *c == ci) {
                // mp-body-look: an MP body is a drawn object like a drone — `Lights_CalcAmbientLight`
                // 0x1cb178 ramps its obj+0x100..0x102 toward the cel it stands in. Without this the
                // bodies kept the 0xff sentinel: drawn at 1.0 where the arena cel says 0x46 (07000023).
                self.mp_body_state(slot).map(|(p, _, _)| p).or_else(|| self.mp_body_death(slot).map(|(p, _, _)| p))
            } else if let Some(di) = self.char_drone[ci] {
                let d = &self.drones.list[di];
                Some([d.pos[0], d.pos[1], d.pos[2]])
            } else {
                // Script characters: the live entity matrix, when the simulation is showing one.
                self.characters_script_pos(ci)
            }
        }).collect();
        let mut targets: Vec<Option<[u8; 3]>> = positions.iter().map(|p| p.and_then(|p| self.ambient_at(p))).collect();
        for &vm in &self.view_model_chars {
            let Some(t) = targets.get_mut(vm) else { continue };
            if let Some(r) = bond_room {
                let c = &self.cels[r];
                let off = *self.drones.channels.v.get(c.zone as usize).unwrap_or(&0);
                *t = Some(c.ambient[(off as usize).min(1)]);
            }
            // The 0x10000 doubled target [M 0x1cb178:20-35, each byte ×2 clamped to 0xff]: the view
            // model's flags word carries it (obj+0xF0 = 0x1a8a0 at the 0700000c start [L PCSX2]; Bond's
            // own body 0x880 does not).
            if self.sun_seen {
                if let Some(a) = t.as_mut() { for k in 0..3 { a[k] = (a[k] as u32 * 2).min(0xff) as u8; } }
            }
        }
        for (ci, amb) in self.char_ambient.iter_mut().enumerate() {
            let Some(target) = targets[ci] else { continue };
            if *amb == [0xff, 0xff, 0xff] {
                *amb = target; // snap on the first link (`control_link_object_to_cel` 0x1346b8)
            } else {
                for k in 0..3 { amb[k] = clr_step(amb[k], target[k]); }
            }
        }
    }

    /// The multiplier bytes of `View_SetupRenderModes_Tweak` (0x1e5888) for a 0x800 object:
    /// `(tint × ambient) >> 8` per channel, the VU1 `TAmbient` term as floats (`psiSetTweakARGB`
    /// 0x124770 → TAmbient = byte/255). The port's tint is the white default (`Script_SetColour` and the
    /// laser-dot object's 0x7f red are not ported), so the bytes equal the ambient bytes.
    pub fn char_multiplier(&self, ci: usize) -> [f32; 3] {
        mul_bytes(self.char_ambient.get(ci).copied().unwrap_or([0xff; 3]))
    }
}

impl Sim {
    /// fix-aim-raise2: the aim-bit gate rebuilt on master's single raise/lower — ONE queue point
    /// (the view chain's edge arm in `run`, fix-weapon-anims') and ONE advance+finish (here).
    /// `Player_Weapon` 0x1a24c8's aim block plus the raise/lower finish of
    /// `Player_SetWeaponAnimObj` 0x1a2e38 cases 0xf/0x10, hoisted to the top of the tick (the game
    /// ticks `AnimObjectUpdate` → `Player_WeaponRecoil` inside `Player_Update` 0x193600:318/:411,
    /// and the finish must land before this tick's free-aim/zoom/auto-aim consumers). Returns the
    /// aim bit for this tick. Hold-aim weapons only (the port models no toggle option).
    ///
    /// A weapon with an aim-raise script (+0xB8) defers the aim bit: the press clears it again and
    /// the view chain plays the raise at 1.25× [M 0x1a24c8:145-152]; the bit sets when the raise
    /// stops [M 0x1a2e38:554-561]. The zoom target (+0x8D4) leaves 1.0 on the bit's rise and
    /// `Player_AutoAim` 0x1a7c98 keeps scanning until then — both read the bit this returns. A
    /// weapon without +0xB8 aims at once [M 0x1a24c8:36-64].
    fn aim_gate(&mut self, button: bool) -> bool {
        // The advance + finish half (`AnimObjectUpdate`, then case 0xf/0x10: the script stopped →
        // the raise sets the aim bit and returns to state 0 [M 0x1a2e38:554-561], the lower just
        // returns to state 0 [M 0x1a2e38:562-567], idle re-queued [M :63-73]).
        if let Some(cur) = self.view_aim_anim {
            let len = self.scripts.get(&cur).map_or(0.0, |s| s.length as f32);
            if self.view_aim_raising {
                self.view_time = (self.view_time + self.view_speed).min((len - 1.0).max(0.0));
                if len <= 1.0 || self.view_time >= len - 1.0 {
                    self.aiming = true;
                    self.view_script = self.weapons.idle_script(self.gun.weapon as usize);
                    (self.view_time, self.view_speed) = (0.0, 1.0);
                    self.view_aim_anim = None;
                }
            } else {
                if self.view_speed < 0.0 {
                    self.view_time = (self.view_time + self.view_speed).max(0.0);
                } else {
                    self.view_time = (self.view_time + self.view_speed).min((len - 1.0).max(0.0));
                }
                if len <= 1.0 || self.view_time <= 0.0 || self.view_time >= len - 1.0 {
                    self.view_script = self.weapons.idle_script(self.gun.weapon as usize);
                    (self.view_time, self.view_speed) = (0.0, 1.0);
                    self.view_aim_anim = None;
                }
            }
        }
        let raise = self.weapons.raise_script(self.gun.weapon as usize);
        if raise == 0 {
            // No raise script: the aim bit follows the button at once [M 0x1a24c8:36-64].
            self.aiming = button;
            return button;
        }
        // The hold-aim bit dynamics with the raise gate (the game's hold branch 0x1a24c8:36-47:
        // the bit clears with the button up and sets while it is held; case D :145-152 re-clears
        // the input-set bit and queues state 0xF, so the rise waits for the finish above). The
        // queue itself is the view chain's edge arm — one queue point (the raise cannot re-fire
        // mid-play: state 0xF is in the game's own refused set [M 0x1a24c8:66-99]).
        let was = self.aiming;
        if button && !was {
            self.aiming = false; // the rise is deferred until the raise stops [M 0x1a24c8:146]
        } else if !button {
            self.aiming = false; // hold-aim: the bit follows the button down [M 0x1a24c8:46]
        }
        self.aiming
    }

    /// One game tick of the free-camera test harness with Bond at `camera` (eye): his object
    /// position on the collision floor below, aimed at chest height. Port harness — real gameplay
    /// input runs through `step_bond` (the `Player_Update` path); this driver stands in for the
    /// player object only in the no-Bond dev paths (`Drone_View_Bones`' chest aim).
    /// `trigger`: the fire button (action 9) held; `aiming`: aim mode (L1).
    pub fn step(&mut self, camera: [f32; 3], look: [f32; 3], trigger: bool, aiming: bool) {
        // mayhew-end-transition: the exit hold freezes the free-camera path too (the game's whole
        // sim is paused [M 0x1c98c8:44-47]; docs/systems/exit-hold.md) — but NOT Mission_Update:
        // it sits at 0x1c9790:17, outside the GS_IsPaused gate (level-14-every-objective).
        if self.drones.exit_hold.is_some() {
            self.drones.tick_mission_during_hold(self.level);
            self.drones.tick_exit_hold();
            return;
        }
        let floor = self.world.floor_below(camera, 4.0).unwrap_or(camera[1] - 1.6);
        self.first_person = false;
        let ground = [camera[0], floor + 0.4, camera[2]];
        // fix-aim-raise: the harness path goes through the same aim-bit gate as the real input.
        // fix-aim-raise2: run gets the RAW button (the view chain's queue arm reads its edge from
        // it); the gated BIT is `self.aiming`, which the consumers read.
        self.free_aim = self.aim_gate(aiming) && aiming;
        // fix-aims20-modes-lights: the ride runs no player input arms (the game zeroes the input in the
        // substate-10 arm [M 0x193600 case 10]) — no analog deflection reaches the idle counter.
        self.run(ground, ground, camera, look, look, trigger, aiming, false, false, false);
    }

    /// One game tick with Bond walking (`Player_Update`): his input, then the world with him as the player. In aim
    /// mode the mouse turn and look (`input.turn` radians, `input.look` units of π/2) first move the cursor within the
    /// aim box (±0.296 across, ±0.4 up) and turn the view only past its edge (pad aim mode; the mouse has none, D-002); the shot goes through the smoothed
    /// cursor (`Player_GetAimingPoint`).
    /// fix-weapon-fire-modes: the renderer's view override while the guided camera rides the
    /// missile — `Some((pos, dir))` [M `Player_PositionCamera` 0x18f5d8 case 0xc: the view matrix IS
    /// the missile's `obj+0x90`; `View_SetDrawInOtherViewsOnly(BLData+0x878)` keeps the rocket body
    /// out of its own view]. `None` = Bond's own view.
    pub fn view_override(&self) -> Option<([f32; 3], [f32; 3])> {
        self.missile_cam.map(|c| (c.pos, c.dir))
    }

    /// mp-match-end: the host's end-of-match poll — `MP_Update` 0x184bd0 case 4's handover
    /// request (`MpGame::request_debriefing`, set at state 4→5 with the GameState+0x1C =
    /// 0x40000033 navigation [M 0x184bd0:62-78]): taken once, it carries the finished match's
    /// debriefing board (`MpMatch::debriefing`, P_MPDEBRIEFING 0x212198 msg 0x4c) and the
    /// match's own settings (the game's pre-match MPSettings backup [M Menu_RestoreMPSettings
    /// 0x202848] — the restore source, since the front end that started the match is gone).
    pub fn take_mp_debrief_request(&mut self) -> Option<(nf_game::mp::debrief::MpDebriefing, nf_game::mp::MpSettings)> {
        let m = self.mp.as_mut()?;
        if !std::mem::take(&mut m.game.request_debriefing) { return None; }
        Some((m.debriefing(), m.settings.clone()))
    }

    pub fn step_bond(&mut self, mut input: nf_game::player::Input, trigger: bool, reload: bool, use_key: bool) {
        // The game's frame clock — `UpdateT0Count` 0x109468, called from `VBlankInt` 0x124088:76
        // (`PS2DrawFrameTime`), `psiPreDraw`/`psiPostDraw` (`PS2DrawCodeTime`), `psiPreGame_Run` /
        // `psiPostGame_Run` (`PS2GameCodeTime`) and the SFX/DMA probes — accumulates timer-0 deltas
        // into `RunningPS2Count` for profiling. The port takes the timing role with this fixed
        // 60 Hz step (determinism: no wall-clock in Sim); the profiling counters have no
        // counterpart (docs/systems/sound-lifecycle.md).
        nf_game::prof::tick();
        let _prof = nf_game::prof::scope("step_bond (total)");
        // mayhew-end-transition: the exit hold — `Trigger_Activate`'s `GS_PauseGame(1)` +
        // `GameFlow_PushState(300.0f, 7, 0xff)` [M 0x1b2318:90-92]. While it stands the game's
        // clock stops [M 0x1c98c8:44-47], the player input dies [M `psiInput_MapInputs`
        // 0x10cd80:36] and the world sections of `Game_Run` are behind the pause gate
        // [M 0x1c9790:30-38] — but the fade itself advances every frame [M 0x1c9790 tail], and
        // `Mission_Update` runs EVERY frame too: it sits at 0x1c9790:17, outside the pause gate
        // (level-14-every-objective) — the monitor keeps reading the channels during the hold,
        // which is where the exit trigger's fallback channel folds the last objective record
        // [M 0x1827d8 case 3]. The port used to skip the whole sim here; now the hold runs the
        // mission layer (mission first, then the fade tick — the game's per-frame order).
        if self.drones.exit_hold.is_some() {
            self.drones.tick_mission_during_hold(self.level);
            self.drones.tick_exit_hold();
            return;
        }
        // level-1b-drake-fight: the zero-G entry — `C_CHCHZEROG_Handler` 0x216388 [M]: its msg-0x4b
        // arm queries the same control with msg 0x40 and folds
        // `Player_ChangeSubState(glb_players, 9 | 0)` — 9 on a nonzero answer, 0 on zero
        // [M 0x216388:10-17]. The disc's switch-channel-75 (0x4b) writers are the script
        // sub-pulses [D 0x06000971 streams 2/3 (`ev0x0d[0x4b,1]` / `ev0x0d[0x4b,0]`), played by
        // 0x060008a3's stream 6 (SP#143, out ch 15) and by the fly-by NIS 0x060008f0's streams
        // 8/9/11 — the same script whose stream 15 writes the fight gate ch 10 at its t400]. The
        // port binds the control's msg-0x40 value to `switch_channels[75]` itself and fires the
        // handler on the channel's write frame (the game stamps `switch_channels_time[ch]` on
        // every script write [M 0x1d94c0 joined_r0x001d9798]; the port's `Channels::time` is that
        // stamp): a write of 1 this frame = the rising edge → substate 9, with the entry's own
        // writes (`+0x50 = 0`, no rotation [M Player_ChangeSubState 0x190630:49-63 — the port's
        // first zero-g tick builds the body matrix from the walk yaw/pitch]). [S] unverified: no
        // ACTION.ELF sender of the msg-0x4b control message was found (the C_CHCH* family is the
        // menu dispatch, `Handler_HandleMessage` 0x2029e0, and no code reads
        // `switch_channels[75]`), so the channel→control binding is the port's model — live-check
        // it. The handler's zero arm (substate 0) stays unwired: the disc's own pulses end in a
        // falling write two ticks after the rise, which would drop Bond into the pit — the
        // game's observable float is one-way (the walkreal leg-2 note), so no disc evidence
        // reaches the zero arm on this route. The binding is applied on 0700001b only: that is the
        // level whose disc scripts are cited above, and other levels' scripts write ch 0x4b for
        // their own purposes (on 15/16/0b/02 the channel-only binding floated Bond mid-walk).
        if self.level == 0x0700_001b
            && self.bond.substate != nf_game::player::SS_ZEROG_9
            && self.drones.channels.on(0x4b)
            && self.drones.channels.time.get(0x4b) == Some(&self.drones.frame)
        {
            self.bond.vel = [0.0; 3]; // [M 0x190630:49-63 — +0x50 = 0]
            self.bond.substate = nf_game::player::SS_ZEROG_9;
        }
        // goggles: the action-13 press edge rides into this tick's `hud_tick` (the game's
        // `Input_Action(p, 0xd, 4)` reads the same edge inside HUD_Update) [M 0x19e9d8:74].
        self.hud.goggles_press = input.goggles_action;
        // mp-core: `MP_Update` 0x184bd0 runs per frame in the gameplay flow state [M 0x184bd0:17-21];
        // the port's fixed tick stands in for the game's wall-clock delta (the module doc's
        // *Deviation*). Arena loads only — `None` on every campaign level.
        if let Some(m) = self.mp.as_mut() {
            // mp-host-match-logic: the scenario objects' per-frame walk (`MP_ObjectUpdate` 0x18b198
            // — the object system drives it every frame, the master/started gates inside) — the
            // half the live host never ran before this task. `on_hill` is the caller's KOTH volume
            // test [M 0x186ae0:88-93]; the port's live host parses no hill trigger box, so the
            // flags stay false (KOTH scoring stays the headless rigs'; see the report's left
            // items). Order: `update_objects` then `update`, the headless lockstep rig's recorded
            // order (mp_lockstep drive). The RNG is drawn only by the GE/blueprint re-picks.
            // mp-scenarios-online (agent6): the live walk — the hill occupancy from the hill's own
            // box (`MP_KOHUpdate` 0x186ae0:88-93) and the objects' contact tests (the port of
            // `MP_HitBy` 0x1830e0: the slot/team gates [M :32-35], the 4-tick spawn window
            // [M :36-39], the dead gate, then the sphere test the frame's contact list carried
            // [M Collide_PickObj 0x1eae80:221-228]; the documented residuals — the collapsed
            // BLData state gates, the slot-order walk, the measured body sphere — are
            // mp-hitby-contact.md's) run inside, from the match's shared state only.
            m.update_objects_live(1.0 / 60.0, &mut self.drones.rng);
            // the shared bit-exact generator — the game's MP_Update draws the global RNG only on
            // the state-6 restart's site re-pick (MP_RestartScenario 0x183fc0)
            m.update(1.0 / 60.0, false, &mut self.drones.rng);
            // mp-host-match-logic: the humans' live positions ride the match every tick (the game
            // reads the live `obj+0x30` per comparison [M 0x185098:123-129]) — the net path set
            // these per tick already; the local path never did, so the spawn policy and the rules
            // saw the load-time positions.
            // By SLOT: the local body is `bond` at `mp_local_slot`, the others `mp_bonds` — a joiner
            // (local slot 1..3) wrote its own position into slot 0 here, so the match's rules and
            // the spawn picker saw a different world on every client mid-tick (the late join's
            // spawn pick diverged on the other client: the three-window check's desync).
            let local = self.mp_local_slot;
            for slot in 0..nf_net::MAX_AGENTS {
                let b = if slot == local { Some(&self.bond) } else { self.mp_bonds[slot].as_ref() };
                if let Some(b) = b { m.set_slot(slot, b.pos); }
            }
        }
        // mp-scenarios-online (agent6): the round restart's respawns and the match-decided deaths
        // reach the bodies before the bots tick.
        self.step_mp_scenario_sync();
        // mp-bots-in-match: the bots' per-frame driver — the game's `BOT_update` 0x126448 family
        // runs per frame inside the MP flow (the brains + the firing pipeline + respawn/score
        // bookkeeping), exactly like the port's `MP_Update` block above.
        { let _p = nf_game::prof::scope("mp: step_mp_bots"); self.step_mp_bots(); }
        // mp-world-sync (agent 9): with the session's row (online), the remote bodies move HERE —
        // after the shared prologue, before the local body — so every body of the tick moves
        // against the same collision world (the doors and script objects join it at the drones'
        // tick, later in this tick) and the world's contact passes below see every body at its
        // new position on every machine. The old order (remotes after `step_bond`) let the local
        // body collide with the last tick's doors and the remotes with this tick's: a body in a
        // moving door split the machines. The callers' later `step_net_remotes` is a no-op.
        self.net_remotes_done = false;
        let net_row = self.net_row.take().filter(|_| self.net_rng.is_some() || self.net_rng_shared.is_some());
        if let Some(row) = net_row.as_ref() {
            let inputs = std::array::from_fn(|i| crate::mpnet::to_player_input(&row[i]));
            let local = self.mp_local_slot;
            self.step_net_remotes_inner(&inputs, Some(row), local);
            self.net_remotes_done = true;
        }
        // 700v6: online, the rest of the tick (the local player's own pipeline and its effects)
        // draws this machine's private generator; the shared one comes back at the tick's end.
        if let Some(r) = self.net_rng.take() {
            self.net_rng_shared = Some(std::mem::replace(&mut self.drones.rng, r));
        }
        // `Game_Run` 0x1c9790:33 runs `Light_Update` before the object movement each tick (fix-lighting):
        // lifetimes (the muzzle light of the previous tick dies here), the script-channel gate and the
        // flicker state machine. The flicker's RNG re-seeds ride the shared generator before this tick's draws.
        {
            let frame = self.drones.frame + 1; // Light_Update sees the new tick's counter
            let channels = &self.drones.channels;
            let rng = &mut self.drones.rng;
            self.drones.lights.update(&|c| channels.on(c as u32), frame, rng);
            // The script lights (blast scripts, NIS lights) as keyed this frame — intensity 1.0, the
            // create's kind byte (`b2 == 1`) [census site 14].
            self.drones.lights.script = self.drones.scripts.lights().into_iter().map(|(pos, radius, rgb, kind)| nf_game::light::Light {
                pos, life: 0, channel: 0, flicker_period: 0, flicker_base: 0, flicker_count: 0, flicker_mask: 0, sound: 0,
                track: None, radius, intensity: 1.0, falloff: 0.0, rgb, kind: (kind == 1) as u8, active: true,
            }).collect();
        }
        self.shot_tick = false;
        self.net_shot = None;
        // mp-combat-weapons: online (the private generator swapped in above) the shared combat
        // owns the guided ride — see `step_net_combat`.
        let online = self.net_rng_shared.is_some();
        self.drones.net_no_self_hurt = online;
        // fix-weapon-fire-modes: the guided-missile camera ride. While it is up, Bond's own arms do
        // not run — the game's player substate 10 arm is ONLY the restore countdown [M
        // `Player_Update` 0x193600:242-256; `Player_ChangeSubState(10)` cleared the anim set] — and
        // the look input steers the missile instead [M `Bullet_update` 0x12e7b0:177-244, gated on
        // `owner+0xF6 == 10`]. The steering runs here, before the drones' tick moves the bullet
        // along its (now steered) direction.
        if online && self.missile_cam.is_some() {
            // mp-combat-weapons: the shared ride (every machine zeroes this slot's movement on
            // the same ticks — `MpCombat::riding`); the local visual rocket steers alongside.
            const K: f32 = 0.024_543_693;
            if let Some(b) = self.drones.player_bullets.iter_mut().find(|b| b.guided_cam && !b.dead) {
                let yaw_in = (-input.turn / K).clamp(-1.0, 1.0);
                let pitch_in = (-input.look * nf_game::trig::HALF_PI / K).clamp(-1.0, 1.0);
                nf_game::drone::steer_missile(&mut b.dir, yaw_in, pitch_in);
            }
            input = nf_game::player::Input::default();
        } else if let Some(cam) = self.missile_cam.as_mut() {
            let live = self.drones.player_bullets.iter_mut().find(|b| b.guided_cam && !b.dead);
            if let Some(b) = live {
                // `steer_missile` takes the game's analog units (full stick = 1.0 → π/128 per tick). The
                // mouse delivers `turn` in radians (look in π/2 units), so convert, capped at full stick:
                // the raw radians steered at ~3% of the game's rate.
                const K: f32 = 0.024_543_693;
                // Signs: `steer_missile` SUBTRACTS (the game's raw-stick convention) while Bond's view ADDS the
                // port's `turn`/`look` (player.rs: `yaw += turn`, `pitch += look`) — negate, so the mouse turns
                // the rocket the way it turns Bond's view (not reversed).
                let yaw_in = (-input.turn / K).clamp(-1.0, 1.0);
                let pitch_in = (-input.look * std::f32::consts::FRAC_PI_2 / K).clamp(-1.0, 1.0);
                nf_game::drone::steer_missile(&mut b.dir, yaw_in, pitch_in);
                cam.pos = b.pos;
                cam.dir = b.dir;
                cam.riding = true;
            } else {
                if cam.riding { cam.restore = 15; cam.riding = false; } // [M 0x12e660:28 `+0x944 = 0xf`]
                if cam.restore == 0 {
                    // [M 0x193600:252-254: `Player_SetCamMode(BLData, 0)`, the saved substate back]
                    self.missile_cam = None;
                } else {
                    cam.restore -= 1;
                }
            }
            // The substate-10 player runs no weapon/movement arms: the port zeroes the input and
            // holds the trigger [M 0x193600 case 10 — nothing else runs].
            input = nf_game::player::Input::default();
        }
        let trigger = trigger && self.missile_cam.is_none();
        // The smoke puffs tick with the sim — the port's rise+fade presentation; the game's real
        // per-particle model (gases_smoke 0x1ab2d0: damping, gravity, params-driven scale) is
        // described at the `smoke` field (visual-only; the draw side owns the adoption).
        for s in self.smoke.iter_mut() { s.update(); }
        // The laser/stunner beam/burn/target state computes on the SHOT tick now (fix-gadgets-live):
        // see the `release` block in run()'s fire section. The game's chain is Player_RoundToFire
        // 0x1a18e0 (the spend) → Player_WeaponInitBullet 0x1a4bf0 (the +0x95b latch, the stunner's
        // target test) → Player_CollisionHandler 0x1949a8:189-191 → Player_MuzzleFlash 0x1a4298 →
        // Draw_LaserBeam/Draw_TaserBeam — everything rides the refire that actually spent a round,
        // so at clip 0 the beam stops (PCSX2 capture) and nothing burns while switching.
        // fix-laser-watch-look: the laser watch's held meter — a tick where the beam ran drains the
        // 0–100 charge ~31/s [D docs/live/2026-10-14-weapon-measurements-pcsx2.md "The laser watch
        // 0x4e": 100 → 59 over ~1.3 s, ≈0.5/frame]; idle ticks refill it — the game's gadget regen
        // shape, +1 per 4 frames [M Player_Update 0x193600 disasm 0x193f18-58] — that +1/4 is the
        // GADGET-CLIP regen, not this meter, so the refill below stays [S unverified] and OUT of
        // pass/fail: the pane's source value (instance+0x110/+0x104) has no located writer in
        // ACTION.ELF [M HUD_UpdateAmmoPane 0x19a7f0:160-176 the read; gadget.rs's evidence block,
        // live check in the report]. The watch lasers only: the stunner fires in view state
        // 9 (its +0x6C & 0x400 = 0 [L weapon_data 0x4a]), so 0x19a7f0's state-0xb pane — the charge
        // read — never shows for it. Read at the top of the tick against the PREVIOUS tick's beam
        // (the beam computes at the end of the shot tick), so every tick pays exactly once.
        if nf_game::gadget::is_laser_weapon(self.gun.weapon) {
            nf_game::gadget::laser_charge_tick(&mut self.laser_charge, self.laser_beam.is_some());
        }
        self.laser_beam = None;
        self.stunner_target = None;
        self.decryptor_target = None;
        // The decrypt scan's contact and decay [M Lock_Update 0x1ae3d8:296-344]: the lock reads
        // its own hitdata list (+0xD0) for a contact bullet — a contact decodes (+1, or +3 for
        // the 0x57; the state-0 entry consumes its contact and resets the progress [M :179-189]),
        // a state-5 lock with NO contact loses 4/tick (at 0 the scan resets). The game's contact
        // view lags the shot by TWO ticks: the update pass (the fire, the bolt's birth)
        // 0x1336f8:54-69 runs before Collide_Update 0x1ea398 (:238) writes the lists, and the
        // fresh bolt's pick segment exists only from its first Bullet_update [M 0x12e7b0:98-110]
        // — a shot at N lands in the (N+1) collide and Lock_Update reads it at N+2. The port
        // keeps that lag: the fire block below only RECORDS the scan; THIS pass applies the
        // record two ticks old and decays every other scanning lock. (The old stand-in banked
        // the scan on the shot tick and skipped one decay tick — a two-tick phase lead mid-hold
        // and a one-tick-early reset.)
        let scanned_due = self.decryptor_scanned_lag.take();
        self.decryptor_scanned_lag = self.decryptor_scanned.take();
        // fix-decryptor-sounds: the digit/unlock chimes a scanning lock raised this tick (the
        // [M] `Lock_Update` 0x1ae3d8:368/:374 `Sound_Play` family, played here at the lock's
        // position through the port's 3-D path). Collected first: the loop holds the locks
        // mutably and the audio handle lives on the same `drones`.
        let mut lock_sounds: Vec<(u32, [f32; 3])> = Vec::new();
        for (i, l) in self.drones.locks.iter_mut().enumerate() {
            match scanned_due.filter(|(li, _)| *li == i) {
                Some((_, upgraded)) => {
                    let ch = &mut self.drones.channels;
                    l.decrypt_tick(1, upgraded, ch, self.drones.frame);
                }
                None if l.scanning => {
                    let ch = &mut self.drones.channels;
                    l.decrypt_tick(0, false, ch, self.drones.frame);
                }
                None => {}
            }
            // `Control_NextLOD` on the unlock (Lock_Unlock 0x1b0650:23 / case 1's framelist end): the
            // keypad's model becomes its LOD variant and keeps it (see `Lock::lod_next`).
            if l.unlocked || l.burned { l.lod_next = true; }
            if !l.pending_sounds.is_empty() {
                let pos = l.pos;
                for id in l.pending_sounds.drain(..) { lock_sounds.push((id, pos)); }
            }
        }
        for (id, pos) in lock_sounds { self.drones.audio.play3d(id, pos, None); }
        self.pda_tick();
        self.smoke.retain(|s| !s.dead());
        let (trigger, reload) = if self.disabled { input = Default::default(); (false, false) } else { (trigger, reload) };
        // fix-death-restart: a dead Bond gets no input — `Player_Update` 0x193600 runs only the anim
        // and `Player_HandleDeath` for state 2/3 [M 0x193600:317-319] — and the 0x40000042 menu
        // pauses the player (`GS_PausePlayer(1,0)` [M 0x182c60:124]). The fire bit still reaches
        // the death's skip gate while dying (action 25 [M 0x190bb0:30]); the menu eats it.
        let menu_pause = self.drones.fail_menu_open();
        self.drones.bond_fire = trigger && !menu_pause;
        // Online, the slot's dead state (`mp_dead_ticks`, the same on every machine at this point
        // of the tick) freezes the body too: the local death sequence starts a tick or two later
        // (`death_tick` inside the drones' tick), and the other machines' replica of this slot
        // (`step_net_remotes`) stops on the shared state — so must the local body.
        let net_dead = self.net_dead();
        // mission-hold: `Mission_Update` case 7 runs `GS_PausePlayer(1, 0)` EVERY tick of the
        // banner hold — the 240-tick success hold (state 6) and the fail/death holds (states 2/3)
        // alike [M 0x182c60:123-130 — the case body is unconditional in `InternalState` 7]. The
        // player's input dies while the world and the HUD keep running (the banner's own message
        // ticks down through `Text_UpdateMsg`, which no pause gate covers). Without the gate Bond
        // kept walking/shooting under the "Mission Complete" box on 07000004 (level-04-mission-
        // complete-early), where the game freezes him until case 4's load.
        let mission_hold = self.drones.mission.internal == 7;
        let (mut input, trigger, reload, use_key) = if self.drones.death.active || menu_pause || net_dead || mission_hold {
            (Default::default(), false, false, false)
        } else { (input, trigger, reload, use_key) };
        // gunimp: while a mounted gun owns the player (`GunImp_Activate` 0x132f00 — the 0xc
        // substate, cam mode 0xe, `Player_Disable(player, 1)`, `Player_WeaponNone`), the imp reads
        // the raw stick and the fire action; the use key is the dismount action 0xe
        // [M `GunImp_Update` 0x1320d8:119-131]. Bond stands — the port zeroes his input for the
        // rest of the tick (the game's disabled player runs only the imp).
        let (mut input, trigger, reload, use_key) = if self.drones.props.gunimps.iter().any(|g| g.active) {
            self.drones.gunimp_tick(&self.weapons, &self.world, input.look, input.turn, trigger || input.fire, use_key, &mut self.bond);
            // The dismount (`GunImp_Deactivate` 0x132c18): `AnimObj[99] = AnimObj[100]` — the
            // weapon saved by the mount's `Player_WeaponNone` is requested back (the climb
            // dismount's carry).
            if !self.drones.props.gunimps.iter().any(|g| g.active) && self.gun.weapon == 0x47
                && self.gun.saved_weapon != 0x47 && self.gun.saved_weapon != self.gun.weapon {
                self.gun.switch_request = self.gun.saved_weapon;
            }
            (Default::default(), false, false, false)
        } else { (input, trigger, reload, use_key) };
        // fix-aim-raise: the aim BIT, not the raw button — a raise-script weapon defers it to the
        // raise's last tick. fix-aim-raise2: aim_gate owns the bit and the raise/lower advance+
        // finish (one advance); the QUEUE is the view chain's edge arm, which reads the RAW button
        // (run's aiming param) — the gated bit is `self.aiming` for the consumers below.
        // The right mouse button is the scope/aim button for the mouse (lib.rs sets `scope` and `zoom`): it drives the
        // aim bit (the raise, then the zoom) exactly like the pad's aim button; only the pad's button also
        // enters free-aim (the cursor window), since the mouse aims directly (D-002).
        // Right-click must not put an unscoped gun into the aim stance. The mouse button is a SCOPE button: it reaches the aim bit only on a weapon with a scope
        // (weapon_data +0x68 bit 0x40, the pad path's `scoped` [input.rs]); on every other weapon it does nothing.
        let scoped = self.weapons.records.get(self.gun.weapon as usize)
            .and_then(|w| w.get(0x68..0x6c)).map_or(false, |b| u32::from_le_bytes(b.try_into().unwrap()) & 0x40 != 0);
        let aiming = self.aim_gate(input.aiming || (input.scope && scoped));
        self.free_aim = aiming && input.aiming;
        // Weapon switching (`Player_WeaponChange` 0x1a6f80): only when idle (not firing/reloading/holstering). The
        // switch request (`anim+0x63`) is set here; the holster/draw cycle runs in `run` (`Player_SetWeaponAnimObj`).
        // The weapon/gadget split: mouse wheel / Q cycles weapons (mode=0, 1..0x47); G cycles gadgets (mode=1,
        // 0x4A..0x5E); number keys select directly [M disassembly 0x1a707c].
        // The toggle-back: if cycling weapons from a gadget, jump to the last weapon; if cycling gadgets from a
        // weapon, jump to the last gadget [M 0x1a6f80 lines 66-121].
        // If the cycle finds nothing, `Player_WeaponNone` (0x1a9858) → hands (id 0x47), then `Player_GetBestWeapon`
        // (0x1a9680) scans the BestWeapon table for the first owned+has-ammo weapon.
        // `Player_WeaponChange` (0x1a6f80:33-47) refuses EVERY switch while the view-anim state is 9 (a fire
        // script playing) [M the state switch maps 9 → case 8: return] — the port approximates state 9 with its
        // fire-script gate (fix-player2: this is the half of the grenade bug that wedged the weapon after a
        // failed throw: the game plays the 26-frame throw script and blocks switching until it ends).
        // The refused set is {1, 3, 5, 6, 7, 8, 9, 0xb, 0xd, 0xe, 0xf, 0x10} — the reload states 5/6/7 are in
        // it: a reload always completes before a switch can start [M 0x1a6f80:33-47; fix-player3].
        let fire_busy = self.gun.fire_state && self.weapons.records.get(self.gun.weapon as usize).is_some_and(|w| {
            let rd = |o: usize| u32::from_le_bytes(w[o..o + 4].try_into().unwrap());
            let len = |h: u32| self.scripts.get(&h).map_or(0.0, |s| s.length as f32);
            let (fs, fl) = (rd(0xb0), rd(0xb4));
            (self.view_script == fs || (fl != 0 && self.view_script == fl)) && self.view_time < len(self.view_script) - 1.0
        });
        // weapon-matrix4: the gate now ANDs the machine's own state-9 flag (`gun.fire_state`,
        // set by the fired path exactly when `Player_SetFiringAnim` 0x1a3c70:50-54 enters view
        // state 9, cleared at the script end [M 0x1a2e38 case 9:449-456]). The hash predicate
        // alone OVER-REFUSED when a record's idle script IS its fire script — the Shaver/mine
        // family 0x5a: +0xA0 == +0xB0 == 0x060004ea (len 2) [L probe dump weapon_data] — so the
        // machine read fire-busy while merely idle and every switch/reload/regen gate starved
        // forever (the matrix's stuck-machine wedges before 0x38/0x39/0x5b/0x62 on 07000002).
        // The game gates on `anim+0xF4 == 9` [M Player_WeaponChange 0x1a6f80:33-47], a MACHINE
        // state: the idle loop playing the same hash is state 0, and switching works there.
        // The per-tick clip regen [M Player_Update 0x193600 disasm 0x193eb8-0x19405c]: while the view-anim
        // state is not 9 (the port's `fire_busy` gate), six clip slots refill +FRAME_RATE_MUL toward their
        // weapon's `weapon_data[w].clip_size` (+0x92): clip[0x4A] (+0x530) and clip[0x4E] (+0x560) and
        // clip[0x33] (+0x41C) on `frame & 3 == 0`; clip[0x4C] (+0x548) and clip[0x4F] (+0x56C) on
        // `frame & 1 == 0`; clip[0x45] (+0x4F4, the grenade belt) on `frame % 6 == 0`. No ownership gate.
        // The held laser/stunner parks its fire script at the last frame but stays in view state 9 (the regen's
        // gate) until the release [L PCSX2 07000005: the clip drains exactly 1/frame for the whole hold, no +1 per
        // 4 frames]; the port's `fire_busy` reads "script still advancing", so gate on the fire script too.
        let held_fire_script = self.gun.fire_state && self.weapons.records.get(self.gun.weapon as usize)
            .is_some_and(|w| self.view_script == u32::from_le_bytes(w[0xb0..0xb4].try_into().unwrap()));
        if !self.disabled && !fire_busy && !held_fire_script {
            let frame = self.drones.frame;
            let gates: [(usize, bool); 6] = [
                (0x4A, frame & 3 == 0), (0x4E, frame & 3 == 0), (0x33, frame & 3 == 0),
                (0x4C, frame & 1 == 0), (0x4F, frame & 1 == 0), (0x45, frame % 6 == 0),
            ];
            for (w, on) in gates {
                if !on { continue; }
                let cap = self.weapons.records.get(w).map_or(0, |rec| u16::from_le_bytes([rec[0x92], rec[0x93]]) as i16);
                if let Some(c) = self.drones.inventory.clip.get_mut(w) {
                    if *c < cap { *c += 1; }
                    if w == self.gun.weapon as usize { self.gun.clip = (*c).max(0) as u32; }
                }
            }
        }
        // The fire-mode button (pad ○ = action 12; keyboard B) [M `Player_Weapon` 0x1a24c8:204-260]:
        // only with the view anim idle (the game's `anim+0xF4 == 0`: no switch, no reload, no fire
        // script) and not while aiming a scoped weapon (+0x68 & 0x40). Twin-linked records (weapon_data
        // +0x05) switch to their twin through the normal switch request; the rest cycle the mode byte
        // (the Inventory's `fire_mode`, `BL + w*0xC + 0x1BB`) up to the record's mode count (+0x2E).
        // For the watch pair 0x54/0x55 the fire press ORs in (the game reads action 9 mask 4 there) —
        // the port tracks the trigger's press edge in `prev_trigger`. The game enters the block on the
        // OR'd variable alone [M 0x1a24c8: `bVar5 = Input_Action(slot,0xc,4); if (*w - 0x54 < 2)
        // bVar5 |= Input_Action(slot,9,4); if (bVar5 != 0 && anim+0xf4 == 0) ...`] — a fire press with
        // the mode button untouched reaches it for this pair (weapon-matrix5: the old outer
        // `if input.mode_switch` made the OR arm dead code; the harness's fire presses never swapped).
        let watch = matches!(self.gun.weapon, 0x54 | 0x55);
        let fire_new = input.fire && !self.prev_trigger;
        let toggled = watch && fire_new;
        if (input.mode_switch || toggled)
                && self.gun.switch_state == 0 && self.gun.reload_state == 0 && !self.gun.fire_state && !fire_busy
                && !(aiming && self.weapons.flags(self.gun.weapon as usize) & 0x40 != 0)
            {
                if let Some(w) = self.weapons.records.get(self.gun.weapon as usize) {
                    let twin = w[5] as i8;
                    let mode_count = w[0x2e];
                    let mode = self.drones.inventory.fire_mode.get(self.gun.weapon as usize).copied().unwrap_or(0);
                    let carried = self.drones.inventory.carried.clone();
                    let clip = clip_view(&self.weapons, &self.drones.inventory.clip);
                    let weapons = &self.weapons;
                    // fix-samurai-fire: the clip half is `Player_AmmoInGun` 0x1a9190's shared-slot
                    // pick — the Overcharge twin 0x32's dry check reads the base's clip[0x33] (the
                    // give fills the base's slot), else the mode press is forever inert.
                    let slot = |x: u8| nf_game::gun::Gun::ammo_in_gun_slot(x, |y| weapons.base(y as usize) as u8, |y| weapons.ammo_type(y as usize) as u8);
                    let has_ammo = |id: u8| nf_game::gun::Gun::has_ammo(id, &carried, &clip, |x| weapons.ammo_type(x as usize), |x| weapons.flags(x as usize), slot);
                    let sw = nf_game::gun::fire_mode_switch(self.gun.weapon, twin, mode_count, mode, has_ammo);
                    if let Some(new) = sw.new_weapon {
                        // The twin switch writes `obj+0x63` (the switch request) [M 0x1a24c8:246-260].
                        let cur = self.gun.weapon;
                        if cur >= 1 && cur <= 0x47 { self.gun.prev_weapon = cur; }
                        if cur >= 0x4a && cur <= 0x5e { self.gun.prev_gadget = if cur == 0x55 { 0x54 } else { cur }; }
                        self.gun.switch_request = new;
                        // fix-fire-modes3 [M 0x1a24c8:213-228]: a LANDED twin switch resets the zoom —
                        // `BLData+0x8D4 = 1.0` unconditionally, then `+0x8D8 = weapon_data[twin]+0x88`
                        // (the twin's own zoom cap) with `+0x8D4` clamped to it. The port's cap follows
                        // the equipped record per tick already (the zoom block reads +0x88 of the held
                        // weapon), so the reset is `zoom`/`zoom_target` = 1.0.
                        self.zoom = 1.0;
                        self.zoom_target = 1.0;
                        self.zoom_wheel = false;
                    }
                    if let Some(m) = sw.new_mode {
                        if let Some(fm) = self.drones.inventory.fire_mode.get_mut(self.gun.weapon as usize) { *fm = m; }
                    }
                }
        }
        self.prev_trigger = input.fire;
        // The pickup auto-switch, LIVE (fix-auto-switch re-opens the fix-pickups-remainder gate)
        // [M Player_EquipWeapon 0x1a8a58's tail]: the take path writes ONLY the request byte
        // `anim+0x63 = id` (per-player flag DAT_002a38d2 ∧ Player_IsBetterWeapon 0x1a6eb0) — no
        // Player_WeaponChange gate, no prev-weapon memory (+0x930/+0x932 are the cycle path's
        // bookkeeping, written by Player_WeaponChange 0x1a6f80 only), no anim, no view state. The
        // draw machine picks it up on the next idle tick (Player_SetWeaponAnimObj 0x1a2e38 default
        // tail: `if (anim+0x63 != anim+0x62) { anim+0x64 = anim+0x62; Player_WeaponSelect
        // 0x1a2cc0 }`); a later weapon-cycle request overwrites it, as in the game.
        // The 07000001 escort-walk wedge that gated this off does not reproduce on master with the
        // drain live (walkreal_07000001 twice-green this session; the decompile shows why: the full
        // view-state census — Player_WeaponFiring 0x1a5460, Player_Weapon 0x1a24c8, Player_Move
        // 0x191dd0, Player_Update 0x193600 — gates only the fire pipeline on the switch states;
        // walking, walk speed, the capsule and the walk anim are NEVER gated on them, and the
        // holster/draw scripts touch only the first-person view model at BLData+0x7e8, never
        // Bond's body/position). The draw machine below is that default-tail behaviour.
        if let Some(id) = self.drones.weapon_switch_request.take() {
            if id != self.gun.weapon as u16 {
                self.gun.switch_request = id as u8;
            }
        }
        self.drones.weapon_switch_request = None;
        // fix-switch-after-cold-load: the request write is allowed while idle, while the DRAW plays
        // (the port's state 3 ≈ the game's states 3+4) and during the retarget's reverse (the port's
        // state 2): `Player_WeaponChange` 0x1a6f80:24-40 refuses only the view states {1, 3, 5, 6,
        // 7, 8, 9, 0xb, 0xd, 0xe, 0xf, 0x10} — states 0 / 2 / 4 (the draw anim playing) are allowed,
        // and `Player_SetWeaponAnimObj` 0x1a2e38 case 4 polls `+0x63` every tick mid-draw and
        // retargets (case 2's tail applies the LATEST +0x63 at the stop). The pre-fix gate (idle
        // only) dropped a number-key press during the level-start draw forever (switch_to is
        // per-tick). The cycle arms keep the idle gate below (the game's states 0xa/0xc have no
        // port counterpart).
        let idle = self.gun.switch_state == 0;
        let can_request = (idle || self.gun.switch_state == 2 || self.gun.switch_state == 3)
            && self.gun.reload_state == 0 && !fire_busy;
        if can_request {
            let owned = self.drones.inventory.owned.clone();
            let carried = self.drones.inventory.carried.clone();
            let clip = clip_view(&self.weapons, &self.drones.inventory.clip);
            let weapons = &self.weapons;
            let ammo_type_of = |w: u8| weapons.ammo_type(w as usize);
            let flags_of = |w: u8| weapons.flags(w as usize);
            // fix-samurai-fire: `Player_AmmoInGun` 0x1a9190's shared-slot pick for the clip half.
            let slot = |x: u8| nf_game::gun::Gun::ammo_in_gun_slot(x, |y| weapons.base(y as usize) as u8, |y| ammo_type_of(y) as u8);
            let has_ammo = |w: u8| nf_game::gun::Gun::has_ammo(w, &carried, &clip, ammo_type_of, flags_of, slot);
            let selectable = |w: u8| weapons.selectable(w);
            // D-003 REMOVED: the root cause was the missing `Player_Init` override of weapon_data[1]+0xDC
            // (the fists skin). With the override applied in main.rs, the fists skin (0x050000ab) is found
            // in the level archive, the view model is built, and the cycle can land on fists — matching the
            // game. The `has_view_model` filter is no longer needed.
            // fix-silencer-modes: the toggle-back gates on the remembered id's BASE (0x1a6f80:97-111)
            // and restores the remembered TWIN verbatim (line 113); the cycle walk carries the twin
            // step (0x1a6f80:179-198) — `cycle` takes &mut self for the marker clear.
            let base_of = |w: u8| weapons.base(w as usize) as u8;
            let target = if let Some(id) = input.switch_to {
                if id != self.gun.weapon && owned.get(id as usize).copied().unwrap_or(false)
                    && weapons.first_person(id as usize) != 0 { Some(id) } else { None }
            } else if idle && input.weapon_cycle != 0 {
                let t = self.gun.toggle_back(nf_game::gun::CycleMode::Weapons, &owned, has_ammo, selectable, base_of);
                t.or_else(|| self.gun.cycle(input.weapon_cycle, nf_game::gun::CycleMode::Weapons, &owned, has_ammo, selectable))
            } else if idle && input.gadget_cycle != 0 {
                let t = self.gun.toggle_back(nf_game::gun::CycleMode::Gadgets, &owned, has_ammo, selectable, base_of);
                t.or_else(|| self.gun.cycle(input.gadget_cycle, nf_game::gun::CycleMode::Gadgets, &owned, has_ammo, selectable))
            } else { None };
            if let Some(id) = target {
                // Update prev_weapon/prev_gadget [M 0x1a6f80 lines 134-141]: if current is a weapon, save to
                // prev_weapon; if current is a gadget, save to prev_gadget. Special: 0x55 ('U') saves as 0x54 ('T').
                let cur = self.gun.weapon;
                if cur >= 1 && cur <= 0x47 { self.gun.prev_weapon = cur; }
                if cur >= 0x4a && cur <= 0x5e { self.gun.prev_gadget = if cur == 0x55 { 0x54 } else { cur }; }
                self.gun.switch_request = id;
                if std::env::var_os("NF_SOUNDLOG").is_some() {
                    eprintln!("[weapon-switch] tick {}: requesting switch from {:#x} to {:#x}", self.drones.frame, self.gun.weapon, id);
                }
            } else if idle && (input.weapon_cycle != 0 || input.gadget_cycle != 0) {
                // The cycle found nothing: go to hands, then try GetBestWeapon [M Player_WeaponNone + Player_GetBestWeapon].
                const BEST_WEAPON: [u16; 18] = [0x42, 0x32, 0x1a, 0x18, 0x1c, 0x16, 0x15, 0x08, 0x10, 0x0e, 0x0d, 0x0c, 0x12, 0x06, 0x04, 0x02, 0x0a, 0x01];
                self.gun.go_to_hands();
                let base_of = |w: u8| self.weapons.base(w as usize) as u8;
                if let Some(id) = self.gun.get_best_weapon(&BEST_WEAPON, base_of, &owned, has_ammo) {
                    let cur = self.gun.weapon;
                    if cur >= 1 && cur <= 0x47 { self.gun.prev_weapon = cur; }
                    if cur >= 0x4a && cur <= 0x5e { self.gun.prev_gadget = if cur == 0x55 { 0x54 } else { cur }; }
                    self.gun.switch_request = id;
                    if std::env::var_os("NF_SOUNDLOG").is_some() {
                        eprintln!("[weapon-switch] tick {}: hands → GetBestWeapon → {:#x}", self.drones.frame, id);
                    }
                }
            }
        }
        if self.free_aim {
            let want = [self.cursor[0] - input.turn / 0.76966, self.cursor[1] + input.look * std::f32::consts::FRAC_PI_2 / 0.57724];
            let clamped = [want[0].clamp(-0.296, 0.296), want[1].clamp(-0.4, 0.4)];
            input.turn = -(want[0] - clamped[0]) * 0.76966;
            input.look = (want[1] - clamped[1]) * 0.57724 / std::f32::consts::FRAC_PI_2;
            self.cursor = clamped;
            for k in 0..2 { self.cross[k] += (self.cursor[k] - self.cross[k]) * 0.5; }
        } else {
            // `Player_AutoAim` halves the offsets each tick outside aim mode.
            self.cursor = [self.cursor[0] * 0.5, self.cursor[1] * 0.5];
            self.cross = [self.cross[0] * 0.5, self.cross[1] * 0.5];
        }
        // Riding (`Player_FeetOnPoint` → `Control_InheritVelocity` 0x133e88): standing on a moving object (the truck),
        // Bond moves and turns with it — the feet ray reaches cc + 0.6 below him (cc 1.0347 [L]). In the game this runs
        // inside `Player_CollisionHandler` (0x1949a8), which early-returns when `BLData+0x94a == 0` (disabled by a
        // script cut-scene). So riding does NOT run while Bond is disabled — the cut-scene's `Script_Free` →
        // `Player_Enable` places Bond directly. The A-5 change (running riding unconditionally) caused Bond to bounce
        // against the truck during the cut-scene. Fix: gate on `!self.disabled` [M 0x1949a8:24].
        // The relink segment's start: the pos at the END of the previous tick — the game's `obj+0x40`
        // the moved pass copies after `control_handle_cel_change` 0x1336f8:212 (the ride below and the
        // tick's own move both belong to the next relink's segment). fix-swim-entry.
        let prev_tick_pos = self.bond.pos;
        if !self.disabled {
            let p = self.bond.pos;
            if let Some(g) = self.world.ground_motion(p, [p[0], p[1] - 1.6347, p[2]]) {
                let (sn, cs) = g.turn.sin_cos();
                let (x, z) = (p[0] + g.delta[0] - g.pivot[0], p[2] + g.delta[2] - g.pivot[2]);
                self.bond.pos = [g.pivot[0] + x * cs + z * sn, p[1] + g.delta[1], g.pivot[2] - x * sn + z * cs];
                self.bond.yaw += g.turn;
            }
        }
        let before = self.bond.pos;
        // `Player_Update` 0x193600 with the ladder/water context (fix-ladder): the level's ladders
        // (BLData+0x884) and the water surface of the cel Bond is in (cel+0x90; −500 = none, and a cel
        // whose top is far below Bond is no water either). The sink control is unused in play (the
        // viewer's input has no bind; the crouch key sinks in the Sim path below).
        if !self.disabled {
            // The Q-Claw grapple's fire [M Player_SetGrapplePoint 0x195da0]: with the claw (0x50/0x51 —
            // the bank's "Grapple" records, `weapon_data+0x70 & 0x40` [L ee_start_05.bin]; the OLD gate
            // 0x4A/0x4C was the Stunner's — the hardcoded table's shift) out and the fire pressed on
            // foot, the aim ray's grapple point (placement 0x3A) latches and Bond enters the substate 2
            // (the reel). The reach is the claw bolt's own `weapon_data+0x1c` death cap [M Bullet_update
            // 0x12e7b0:158; L: 50.0 for 0x50, 100.0 for 0x51]. The enter tick swallows the fire — it runs
            // BEFORE the bond tick so a release tick's fire cannot re-latch in the same tick.
            let mut trigger = trigger;
            // mp-world-sync: online the claw is the session's held weapon (`Input::held`) — the
            // value every machine's replica of this body latches on (`step_net_remotes`).
            let claw = net_row.as_ref().map_or(self.gun.weapon, |r| r[self.mp_local_slot].held);
            if trigger
                && self.bond.substate == nf_game::player::SS_WALK
                && self.bond.grapple.is_none()
                && nf_game::gadget::is_grapple(claw)
            {
                let head = self.bond.head();
                let dir = self.bond.look();
                if let Some(point) = nf_game::gadget::pick_grapple_point(&self.drones.grapple_points, head, dir, nf_game::gadget::grapple_range(&self.weapons, claw)) {
                    self.bond.grapple = Some(nf_game::gadget::GrappleState { point, phase: 1, pull: 0.0 });
                    self.bond.substate = nf_game::player::SS_GRAPPLE;
                    self.bond.vel = [0.0; 3];
                    trigger = false;
                }
            }
            // The player's runtime cel (obj+0x20) — `control_handle_cel_change` 0x133288's stateful
            // relink on the LAST tick's move segment (the moved pass runs after `Player_Update`
            // [M 0x1336f8:147-236], so the cel `Player_InWater` 0x195e50 reads this tick was linked
            // from `prev→pos` of the tick before): the portal cross (`Cel_ObjectLeftCel` 0x1e0d58)
            // wins over the bbox keep (`build_point_in_bounding_box` 0x1c9500), and the fresh-link
            // fallback is `build_FindCel` 0x1c8cb0 at the world-sphere centre with the raycast
            // tie-break (`build_LinkToRoom` 0x1c9680). fix-swim-entry (was: the stateless
            // smallest-bbox `cel_at(pos)`).
            let bond_cel = if (0..3).any(|k| (self.bond.pos[k] - self.bond.prev_pos[k]).abs() > 3.0) {
                // A placement/teleport: the game's `Player_StandAtNewPosition` 0x190d88 resets
                // `obj+0x40` to the new position (line 46) and relinks directly with one
                // `build_FindCel` (lines 108-117) — no portal walk over the jump. The port's direct
                // `bond.pos` writes (the tests' placements) get the same treatment [S: the 3.0 bound
                // sits above any per-tick move the sim produces — the walk ~0.1, the accelerated fall
                // ~0.3 even seconds in — and below every placement jump].
                self.bond.prev_pos = self.bond.pos;
                self.world.find_cel(self.bond.pos)
            } else {
                self.world.player_cel_relink(self.bond.cel, self.bond.prev_pos, self.bond.pos, self.bond.pos)
            };
            let water = bond_cel.and_then(|cell| self.room_cell.iter().position(|&c| c == Some(cell)))
                .map(|r| self.cels[r].water).filter(|w| *w > -400.0);
            let sink = input.crouch;
            // The pick capsule's straddle set [M Collide_StraddleCels 0x1eac98 runs on the pick's
            // HITTEST; Collide_Pick 0x1eb320:125-134 scopes the candidates to its cels]: the BFS
            // from the player's runtime cel (obj+0x20 — the same stateful one-tick-stale link the
            // game reads) through the portals the current substate's pick arm capsule touches
            // (`Bond::pick_arm`'s segment + radius, anchored at the last integrated pre-push-out
            // spot [S: the game builds the HITTEST at THIS tick's integrated position — one tick
            // apart at most; the portal walk's own permissiveness covers the difference]).
            let (ao, bo, ar) = nf_game::player::pick_arm(self.bond.substate, self.bond.crouched, self.bond.cc);
            let pc = self.bond.pre_collide;
            let straddle_cells = bond_cel
                .map(|c| self.world.debug_straddle(c, [pc[0], pc[1] + ao, pc[2]], [pc[0], pc[1] + bo, pc[2]], ar))
                .unwrap_or_default();
            // The bots' collision capsules for the actor-actor push (`Collide_Jointy`
            // 0x1ec180:229-281 via `LadderCtx::bot_capsules`): the game's bot hittest arms
            // [M NDrone2_Collision 0x140e90:91-121 — top = pos + up·r, bottom = pos + up·(r − h),
            // r 0.4 default / 0.55 kind 0xc, h the anim height word +0xa0 with default 1.0
            // [M NDrone2_DefaultInit 0x14b300:48]] — upright bots (up = (0,1,0)), so the arms are
            // pos + (0, ±0.4, 0) at h 1.0. The radius/shape arms now
            // come from the ported capsule itself (`Drone::bump_capsule` [M NDrone2_Collision
            // 0x140e90:59-124 — 0.3 down / 0.55 DSTATE 0xC / 0.4 default, ends pos+up·r and
            // pos+up·(r − hip), hip the ext+0xa0 anim height (drone-bump-collision-3)]);
            // drone-bump-collision. drone-bump-collision-3: the ON/OFF of a bot's capsule is its
            // kind word ext+0x464, refreshed 0x800/0 on the bot's OWN control tick through
            // `NDrone2_DoCollision` 0x149ec8 (disasm-checked) and zeroed outright while the drone
            // system is frozen [M Drone_CollisionHandler 0x140968:71-75] — NOT a live-distance
            // test (the collision-2 note "0x800 only within 2.0" was the near ARM alone; the
            // drawn-grace arm at 0x149ec8:34-37 and the :42-44 fall-through arm every enabled
            // bot whose anim carries no no-collision flag).
            let bot_capsules: Vec<([f32; 3], [f32; 3], f32)> = {
                // drone-bump-collision-3 [M Player_Update 0x193600:80-88 + :361]: `Player_Collision`
                // — the Jointy resolve's ONLY caller — runs AFTER the ext+0x94a enable gate, and
                // `Player_Disable(player, 0)` 0x195f70:12-14 zeroes that byte: a script-disabled
                // Bond runs NO class-3 query at all and is NEVER pushed. The port's scripted Bond
                // keeps its tick_ctx (zeroed input, the cut-scene rails move him); only the push
                // arm drops out here.
                if self.disabled { Vec::new() } else {
                // The ARming gate [M] — the bot's own kind word ext+0x464: `NDrone2_Collision`
                // 0x140e90:66-72/95-101 rewrites it 0x800/0 EVERY control tick through
                // `NDrone2_DoCollision` 0x149ec8 (disasm-checked), from `NDrone2_ControlSTANDARD`
                // 0x1490f0:230; `DefaultInit` 0x14b300:576 starts it 0x800; `Drone_CollisionHandler`
                // 0x140968:71-75 forces it 0 every tick the drone system is frozen
                // (`Drone_PauseNoDraw` 0x13a710's `Drone_bDisableSystem`, script event 5's
                // `Drone_EnableAll(0)` → `NPCGlobals = 0` [M 0x1383e8 tail]) and `Drone_Control`
                // 0x139268:39-42 gates the re-arm off the same flags. So a bot's capsule takes part
                // in the player push only while its own control pass runs and arms — the port's
                // `kind_464` (drone.rs, refreshed at the per-bot control pass). The corpse
                // exclusion stays the collision-2 [S] door-gate mapping (Jointy itself has no
                // corpse-word evidence either way).
                // The PICK-LEVEL gate the game resolves the class-3 query through: `Collide_Pick`
                // 0x1eb320 walks only the seeker's straddled cels (:125 `Collide_StraddleCels`
                // build; :312-336 the per-cel loop — `iVar9 = *(int *)(iVar12 + 8)` = the cel's
                // object list, `Collide_PickObj(hittest, obj, owner)` per obj with `obj->0xf0 &
                // 0x30000000 == 0`), so a bot whose own cel the player's pick capsule does NOT
                // straddle is never a pick candidate — no contact node, no Jointy resolve, no
                // push [M 0x1eae80 → 0x1ebaa8 → 0x1ec180]. The straddle set here is the same
                // one the third-icon context consumes (the pick capsule = `pick_arm` + the
                // hittest radius, anchored at the integrated pre-push-out spot). The bot's cel
                // is the port's `find_cel(pos)` stand-in for the game's obj+0x20 link (the same
                // stand-in the sight check uses [S noted at drone.rs `same_cel`]; the game's
                // link is relinked by control_handle_cel_change 0x133288, one tick of staleness
                // at most on both sides). drone-bump-collision-2: bots across a cel boundary —
                // the hall-door-gap bot of probe_07_route, the party-door crowds — stop pushing;
                // bots Bond shares a cel with keep the push.
                self.drones.list.iter()
                    .filter(|x| x.kind_464 == 0x800 && !x.corpse)
                    // CORPSES NEVER COLLIDE (master-side citation, kept in the merge): `SetAsDead`
                    // 0x14a2f0:9 sets +0x4F8&0x200 at death, and `NDrone2_DoCollision` 0x149ec8:11
                    // returns 0 (collision OFF) whenever that bit is set — on every level but the
                    // special 0x700004a — and `NDrone2_Collision` 0x140e90:64 gates the same bit
                    // (docs/fields/drone.md, the 0x4F8&0x200 row, census-verified). The game's dead
                    // bot presents no capsule for the player's Jointy query: Bond walks over bodies.
                    // Without this gate the walkreal_07000001 escort wedge reproduces — Bond lands
                    // on a fresh corpse's capsule top (pos+0.4), the push holds him airborne
                    // (grounded never settles), every nav leg dies at the cap, and the escort waits
                    // for a Bond who can no longer reach the [M 0x15eda8] release band.
                    // drone-bump-collision-8 [L audits/live-07-eastdoor.txt via the driver,
                    // fresh-boot 07000007 2026-10-28]: Bond placed ON the E-spot (94.6, 44.4,
                    // -47.4) — 0.48 from d10's shifted centre — STANDS there 120 frames; placed
                    // between the pair the game walks him out NORTH. The 0x31 at-post door
                    // guards' capsules are inert to the player: no Jointy push fires at any
                    // range the tests reach (their own SP query returns 0 against a class-3
                    // opponent [M Collide_Jointy 0x1ec180:229-233], and the player-side resolve
                    // that the armed +0x464 word predicts [M 0x140e90:95-101] is contradicted
                    // by the measurement — the unmodelled DoCollision/Pick arm that zeroes it
                    // for the at-post pair is the live-check item). Until that arm is named,
                    // the at-post CIVILIAN_DOOR_GUARD state does not join the push set — the
                    // port's double 0.95 contour wall (Bond r 0.55 + bot r 0.4 [M 0x194df0:316,
                    // 0x140e90:91-94]) is what pinned probe_07_route / party_door_lock /
                    // cutscene / walkreal at the east door.
                    .filter(|x| x.state != nf_game::drone::state::CIVILIAN_DOOR_GUARD)
                    .filter(|x| straddle_cells.contains(&self.world.find_cel(x.pos).unwrap_or(usize::MAX)))
                    .map(|x| x.bump_capsule())
                    .collect()
                }
            };
            let ctx = nf_game::player::LadderCtx { ladders: &self.ladders, wires: &self.wires, wire_anims: Some(&self.wire_anims), water, sink, creepwalls: &self.creepwalls, level_bump_5fb: false, straddle: &straddle_cells, bot_capsules: &bot_capsules };
            // The third-icon context [M ThirdIcon_Update 0x196e80 + Player_CollisionHandler 0x1949a8:22]:
            // the tick starts with +0x95F = 0xFF (none), then every ENABLED icon (its param-1 channel on,
            // or param 1 == 0 = always [M ThirdIcon_Create 0x197070:27-32]) whose straddle-cel set holds
            // the player's cel writes its type. The game's set is the icon's obj+0xd0 chain: the icon's
            // world sphere straddles cels through the portals it intersects [M Controls_StraddleTest
            // 0x134778 → Collide_StraddleCels 0x1eac98 → Intersect_Portal 0x1ea7b8 v0x101], every object
            // occupying a straddled cel gets a hitdata node [M Collide_Update 0x1ea398:160-233], and the
            // icon writes when a node's +0x58 is the class-3 player [M 0x196e80:36-52]. The port resolves
            // each icon's set once at load (`icon_cels` — `icon_straddle_cels` over `cel_graph` with the
            // icon's own world-sphere radius, e.g. 07000008's "Hand_Over_Hand" r 1.164 [D probe]) and
            // tests the player's FindCel cel [M build_FindCel 0x1c8cb0: room cels + the collision
            // tie-break, `find_cell_raycast`] against it. The old flat 12.0-unit radius armed through
            // walls — on 07000008, walking into the guardhouse wall and jumping threw Bond onto
            // the zipline across it (the icon sits 1.9 from the wall on the far side); the r-1.164 sphere
            // crosses no portal there, so the wall keeps the contexts apart exactly as the cel graph lays
            // them. **Port cull (documented, evidence below — not a guess)**: the port's cel buckets are coarser than the game's cels — the
            //   fix-swim-vent5 probe put 07000005's gate door and the second vent's ctx-6 icons in one
            //   port cel — so pure cel occupancy would arm the gate door against the [L PCSX2 2026-10-13
            //   pin (0xFF there; enforced by fix_swim_vent5_live)]. The arm region is therefore the
            //   INTERSECTION: the straddle set AND the game's own SPHERE test (fix-wallhug-prompt).
            //   Window 12.0 → 6.0 → THE GAME'S SPHERE (fix-context-hints2 → fix-wallhug-prompt): the
            //   driver's 2026-10-14 PCSX2 side-by-side pins 07000005's grapple spot (16.5, 29.6, 58.6)
            //   — 8.17 from the ctx-6 icon[3] (24.20, 31.93, 60.01), same port cel 15 — at NO context
            //   and NO "Press Action" [L PCSX2 2026-10-14], while the vent crawl spot still reads 6
            //   [L PCSX2 2026-10-13]. The game's exact volume is NOT a flat window at all: the icon
            //   writes only into the player object its own obj+0xd0 chain names, and `Collide_Pick`
            //   0x1eb320 admits a candidate object ONLY when its world sphere meets the picker's
            //   sphere — `Vec_SqDist3D(picker+0x60, obj+0x80) < (picker+0x6C + obj+0x8C)²` [M
            //   0x1eb320 LAB_001eb4e4; the picker's +0x6C copies +0x8C = the player's world-sphere
            //   radius 1.0576 (L M_CONTROL+0x8C, ee_start_05 player 0 standing), the icon's +0x8C is
            //   its `Control_BuildWorldSph` 0x133070 radius — r 2.2757 on 05's ctx-6 icons [D probe]],
            //   then `Collide_Intersect` 0x1ebaa8's LAB_001ebf04 re-tests sphere-sphere and the chain
            //   keeps only overlapping pairs. So arm = `dist(bond.sphere_centre(), icon.centre) <
            //   1.0576 + icon.radius`, gated by the straddle-cel set (the cel-cull stand-in). The
            //   measured pins all sit the right side of 3.333: the crawl out-point is 3.28 from
            //   icon[1] (arms), the grapple spot 8.17 from icon[3] (silent), the gate door 24.06
            //   (0xFF). The old [S] 6.0 window measured Bond's FEET, not the sphere centre, and
            //   reached ~1.8× past the game's shell — the prompt stayed up while walking away
            //   (05 trench ledge).
            self.bond.icon_ctx = 0xff;
            let bond_cell = find_cell_raycast(&self.cel_graph, self.bond.pos, Some(&self.world));
            for (ii, icon) in self.icons.iter().enumerate() {
                if icon.on_channel != 0 && !self.drones.channels.on(u32::from(icon.on_channel)) { continue; }
                let Some(cells) = self.icon_cels.get(ii) else { continue };
                if cells.is_empty() { continue; }
                if !bond_cell.is_some_and(|bc| cells.contains(&bc)) { continue; }
                // The arm IS the player's own walk pick [M Collide_Update 0x1ea398:72-84 runs
                // `Collide_Pick` on obj+0xd4 (the BL+0x740 HITTEST) and mirrors every picked
                // object a node about the pick owner :160-233; ThirdIcon_Update 0x196e80:36-52
                // arms only on a node whose +0x58 is the class-3 player]. The pick's 0x800 query
                // test [M Collide_PickObj 0x1eae80:152-166, Collide_Pick 0x1eb320:224-241]:
                // `dist(segment [+0x20,+0x30], obj+0x80) < obj+0x8C·1.5 + HITTEST+0x8C`.
                // EVERY input is now pinned:
                // - icon radius: the placement scale words at +0x34..+0x40 read (1,1,1) on every
                //   0xFA icon of 07000005/08 [D icon_radius_probe], and obj+0xE8 = 1.0 live
                //   [L ee_start_05 icon 0x16dbf10+0xE8 — ThirdIcon_Create 0x197070 never writes
                //   it; control_init_object 0x134630:17 sets 1.0], so obj+0x8C = glist+0x1C raw:
                //   [L] obj+0x8C 2.2757196 == glist 0x170b7b0+0x1C == the record radius [D].
                // - picker: [L ee_start_05 player 0 (M_CONTROL 0x16cb8a0, substate 0)] hittest
                //   BL+0x740: mask +0x94 = 0x800, +0x8C = 0.55, seg A = eye+0.275, seg B =
                //   eye−0.4847 = eye + (0.55 − cc) with cc = parent+0xCC = 1.0347183 — exactly
                //   `pick_arm`'s walk arm anchored at the position (obj+0x30 = the position copy
                //   [M Control_CreateObjEx 0x1343c8:24-27], the port's `pre_collide`).
                // - Player_CollisionHandler 0x1949a8:22 clears BL+0x95F = 0xFF every tick, so the
                //   arm is same-tick — no persistence.
                // The old [S] sphere stand-in (`icon.radius + 2.5` over `sphere_centre`) both
                // over-armed (the ledge band x 130.5..131.0, z −103..−102 seg_dist 3.1..3.6 armed
                // at reach 3.6638) and mis-pinned the grab spot: the old (132.0,−104.0) spot sits
                // seg_dist 3.14 from the icon — outside the game's 2.2957 reach; the ledge CLIMBS
                // toward the anchor (stood_y 46.49 at x 130.4 → 48.72 at x 133.4 [probe
                // ledge_standable_probe]) and arms from ~1.2..2.3 seg_dist across x 131.5..133.4,
                // z −103..−100. fix_08_walls_live's grab spot moves onto that band.
                let (pao, pbo, par) = nf_game::player::pick_arm(self.bond.substate, self.bond.crouched, self.bond.cc);
                let px = self.bond.pre_collide;
                let (ay, by) = (px[1] + pao, px[1] + pbo);
                let seg_len = by - ay;
                let t = if seg_len.abs() > 1e-6 { ((icon.centre[1] - ay) / seg_len).clamp(0.0, 1.0) } else { 0.0 };
                let cy = ay + seg_len * t;
                let dx = icon.centre[0] - px[0];
                let dz = icon.centre[2] - px[2];
                let dy = icon.centre[1] - cy;
                let reach = icon.radius * 1.5 + par;
                let near = dx * dx + dy * dy + dz * dz < reach * reach;
                if near {
                    self.bond.icon_ctx = icon.ctx;
                    // `ThirdIcon_Update` 0x196e80:38-53: the type-6 icon posts the generic hint
                    // "Press Action" (label 0x1000291 [D USATxt.dat entry fixups[1]+0x291]) as a
                    // type-1 INFO message, life 0xB4, while the player is in range AND not
                    // creeping (substate 7) AND the icon's instance timer (data+8) has ticked
                    // down to 0; every in-range write resets it to 4 [M 0x196e80:15-17,52-53].
                    let timer = self.icon_msg_timers.get(ii).copied().unwrap_or(0);
                    if icon.ctx == 6 && timer == 0 && self.bond.substate != nf_game::player::SS_CREEP {
                        self.hud.info_labels.push_back((0x100_0291, 180.0));
                    }
                    if let Some(t) = self.icon_msg_timers.get_mut(ii) { *t = 4; }
                }
            }
            // The icon timers tick every frame regardless of range [M 0x196e80:15-17 — the
            // instance timer decrements at the top of the icon's own update].
            for t in self.icon_msg_timers.iter_mut() { if *t > 0 { *t -= 1; } }
            // The hint markers (fix-context-hints): `Hint_Update` 0x1b3380 fires once per load
            // when the player's cel is in the marker's chain with the channels satisfied — the
            // sound, then the type-1 message with the marker's label, then obj+0xFE bit 0 deletes
            // the marker [M 0x1b3380:24-42]. The chain fill is the player's pick through the
            // marker's SPHERE (r + the player's 1.0576 world sphere [L M_CONTROL+0x8C]) — the
            // cel-set test alone is coarser than the game's cels (fix-08-walls's finding), so
            // the arm region is the intersection: the straddle set AND the sphere volume (the
            // safe's control-panel marker sits 6.7 from the safe spot with r 3.889 — the game
            // leaves it silent there [L — fix-context-hints.md's measured PCSX2 pin]).
            for (hi, h) in self.hints.iter().enumerate() {
                if self.hint_fired.get(hi).copied().unwrap_or(true) { continue; }
                if h.on_channel != 0 && !self.drones.channels.on(h.on_channel) { continue; }
                if h.off_channel != 0 && self.drones.channels.on(h.off_channel) { continue; }
                let Some(cells) = self.hint_cels.get(hi) else { continue; };
                if cells.is_empty() { continue; }
                if !bond_cell.is_some_and(|bc| cells.contains(&bc)) { continue; }
                let c = self.bond.sphere_centre();
                let reach = h.radius + 1.0576;
                let near = (h.centre[0] - c[0]).powi(2) + (h.centre[1] - c[1]).powi(2) + (h.centre[2] - c[2]).powi(2) < reach * reach;
                if !near { continue; }
                self.hint_fired[hi] = true;
                self.hud.info_labels.push_back((h.label, 180.0));
                if h.sound != 0 { self.drones.audio.play3d(h.sound, self.bond.head(), None); }
            }
            // No walk-in or use-key wire grab: the game's
            // grab that is certain is the JUMP grab [M Player_HandleJump 0x1916b0:107-147 — action 7 is the
            // jump (the same press plays the refused-jump sound 0x1db on 07000007), with the icon context
            // +0x95F == 3], ported in `Bond::physics_tick`. E is action 0xE → `Player_Activate` 0x1a5170, which
            // has no class-0x2C arm. A walking-pick grab (`Player_DealWithObjHit` 0x194458:104 → CollWire)
            // exists only if the walking query version reaches `Collide_PickObj`'s world-sphere branch
            // (versions 0x200/0x201 [M 0x1eae80]) — unproven, and the unculled sphere let 07000002's courtyard
            // wire swallow Bond in the bunker; left out until measured. `wire_cels` stays for that.

            let was_on_wire = matches!(self.bond.substate, nf_game::player::SS_WIRE | nf_game::player::SS_ZIPLINE);
            let was_climbing = self.bond.substate == nf_game::player::SS_CLIMB;
            input.fire = trigger;
            // The zero-g auto-stabiliser reads the game's action 0xe — the same pad button the
            // keyboard path splits into the use key (E) and reload (R) [M Player_ZeroG 0x192ab0:
            // 163-166]. OR both edges in; the field is only read in the zero-g substates.
            input.stabilise = input.stabilise || use_key || reload;
            // The queries' start cel (obj+0x20 — fix-safe08b): the game's per-tick relink is the STATEFUL
            // `control_handle_cel_change` 0x133288 (supersedes fix-08-collision2's FindCel-at-pos root —
            // the same `build_FindCel` 0x1c8cb0 two-ray tie-break that fix kept, but as
            // `build_LinkToRoom` 0x1c9680:35-40 runs it: AT THE WORLD-SPHERE CENTRE obj+0x80 [M], behind
            // the move segment's portal cross (`Cel_ObjectLeftCel` 0x1e0d58) and the bbox keep
            // (`build_point_in_bounding_box` 0x1c9500), so the actor's cel always carries the geometry
            // under him and the straddle BFS roots at the floor's cel — the 08 escort's straddle root and
            // the portal-centre crossings ([M 0x1c8928's +0x70]) both ride the same stateful cel).
            self.bond.cel = bond_cel;
            // The melee hit-push `BL+0x910` (fix-bond-damage): `Player_Update` 0x193600 decays it
            // ×0.9 every tick [M disasm 0x1937a4-0x1937d8] and, while Bond is alive, adds it to the
            // move vec's Y before `obj+0x50 += move` [M disasm 0x193ccc-0x193cf0] — a per-tick
            // displacement the floor collision absorbs when it pushes down.
            self.drones.health.hit_push *= 0.9;
            if self.drones.health.health > 0.0 {
                self.bond.pos[1] += self.drones.health.hit_push;
            }
            // death-animation-sp: the dead branch of `Player_Update` 0x193600 runs ONLY
            // `AnimObjectUpdate` + `Player_HandleDeath` [M 0x193600:316-319, state obj+0xf4 2/3] —
            // no substate arm, no `Player_MonitorAir`, no collision pass. Bond's eye (`obj+0x30`)
            // freezes where he died: the death camera is cam mode 0 (`Player_SetCamMode(BL, 0)`
            // [M `Player_CheckForDeath` 0x190810:64]) → `Player_PositionCamera` 0x18f5d8 case 0 →
            // `Camera_SetToPlayer` 0x1bc048 → `Player_GetHeadPos` 0x195aa0 reads `obj+0x30/34/38 +
            // BL+0x908 − BL+0x90c`, and the dead arm zeroes BL+0x90c [M 0x18f5d8:130-132]; nothing
            // in the dead branch writes obj+0x30 (the anim system moves no object position —
            // `AnimObjectUpdate` 0x1befd8 / `AnimSeqTick` 0x1c28d0 / `AnimFrameResolve` 0x1c35f8
            // have no obj+0x30 writes). The pre-fix port kept ticking the walk physics, so a death
            // in the air kept falling (the probe: −3.06 units over ~55 ticks).
            // gunimp-mounted-pose: the mounted player runs no walk either — `Player_Disable`
            // [M 0x132f00] takes the player out of the update and `GunImp_Update` moves the
            // object itself [M 0x1320d8:175-194, the carry above]; cam mode 0xe is the SAME
            // first-person case as mode 0 [M 0x18f5d8:96-97], so the carried pose IS the view.
            if !self.drones.death.active && !net_dead && !self.drones.props.gunimps.iter().any(|g| g.active) {
                { let _pphase = nf_game::prof::scope("phase: player move");
                self.bond.tick_ctx(&self.world, &ctx, &input); }
            }
            // audit-sound-calls: drain the jump launch / head-bump sounds into the Sim's sound
            // pool (`Player_HandleJump` 0x1916b0:184,195-198 via `Bond::sfx_out`).
            for (id, pos) in self.bond.sfx_out.drain(..) {
                self.drones.audio.play3d(id, pos, None);
            }
            // `obj+0x40 ← obj+0x30` — the moved pass's copy AFTER the cel change [M 0x1336f8:212].
            self.bond.prev_pos = prev_tick_pos;
            // `Player_WeaponNone` 0x1a9858 runs in `Player_CollLadder` on the grab [M 0x196118]: the
            // current weapon id is SAVED to `anim+0x64` (the port's `saved_weapon`), the class becomes
            // 'G' (0x47) immediately (no holster cycle — `anim+0xF4 = 0`), the aim bit clears.
            if !was_climbing && self.bond.substate == nf_game::player::SS_CLIMB && self.gun.weapon != 0x47 {
                self.gun.go_to_hands();
            }
            // `Player_WeaponNone` 0x1a9858 also runs in `Player_CollWire` on the grab [M 0x1974e4].
            if !was_on_wire && matches!(self.bond.substate, nf_game::player::SS_WIRE | nf_game::player::SS_ZIPLINE) && self.gun.weapon != 0x47 {
                self.gun.go_to_hands();
            }
            // The dismount carry [M Player_Climb 0x196310:99-102]: `if (AnimObj+0x62 == 'G')
            // AnimObj[99] = AnimObj[100]` — the selected id becomes the saved weapon, and
            // `Player_SetWeaponAnimObj`'s idle path re-runs `Player_WeaponSelect` [M 0x1a2e38:197-201]:
            // the saved weapon is re-equipped on dismount (fix-ladder2: Bond stayed on hands).
            if was_climbing && self.bond.substate == nf_game::player::SS_WALK && self.gun.weapon == 0x47
                && self.gun.saved_weapon != 0x47 && self.gun.saved_weapon != self.gun.weapon {
                self.gun.switch_request = self.gun.saved_weapon;
            }
            // The wire drops write `AnimObj[99] = AnimObj[100]` [M Player_Wire 0x1978bc, Player_Zipline
            // 0x19808c] — the same re-equip carry the climb dismount uses.
            if was_on_wire && self.bond.substate == nf_game::player::SS_WALK && self.gun.weapon == 0x47
                && self.gun.saved_weapon != 0x47 && self.gun.saved_weapon != self.gun.weapon {
                self.gun.switch_request = self.gun.saved_weapon;
            }
            // `Player_Wire`'s per-tick effects outside Bond [M 0x1976b4-0x1976d4]: `Music_Event(0xD, 1)`
            // every tick on the wire (the music jumps to the stealth section) and the wire's switch
            // channel set 1 while Bond is on it, 0 on drop.
            let on_wire = matches!(self.bond.substate, nf_game::player::SS_WIRE | nf_game::player::SS_ZIPLINE);
            // The crawl's per-tick effects [M Player_Creep 0x1969d0:85-88]: `Music_Event(0xD, 1)` and
            // `switch_channels[wall.ch] = 1` every tick while Bond crawls (fix-swim-vent).
            let on_creep = self.bond.substate == nf_game::player::SS_CREEP;
            self.creeping = on_wire || on_creep;
            if on_wire {
                self.drones.music.event(0xd, 1);
                if let Some(ch) = self.bond.wire.and_then(|wi| self.wires.get(wi)).map(|w| w.channel).filter(|c| *c != 0) {
                    self.drones.channels.set(ch as u32, 1, self.drones.frame);
                    self.wire_channel = ch;
                }
            } else if on_creep {
                self.drones.music.event(0xd, 1);
                if let Some(ch) = self.bond.creep.and_then(|wi| self.creepwalls.get(wi)).map(|w| w.ch).filter(|c| *c != 0) {
                    self.drones.channels.set(ch as u32, 1, self.drones.frame);
                    self.wire_channel = ch;
                }
            } else if self.wire_channel != 0 {
                self.drones.channels.set(self.wire_channel as u32, 0, self.drones.frame);
                self.wire_channel = 0;
            }
            // `Player_MonitorAir` 0x191a70's drown damage: `Player_HandlePain(0.5, type 7)` [M 0x191a70:136-143]
            // — the FULL pipeline (difficulty factor, no armour for type 7 [M 0x1902c8:84-104], the pain
            // flash +0x968 and the both-sides bits +0x967 = 0xC), not a bare subtraction.
            // mp-world-sync: online the drown/hurt/fall damage is the shared world pass's
            // (`step_net_world`, every body into its slot's health) — applied here it would reach
            // only this machine's health block and could start the local death sequence alone.
            if let Some(dmg) = self.bond.drown_damage.filter(|_| net_row.is_none()) {
                if self.drones.health.pain_generic(&mut self.drones.rng, dmg, self.drones.difficulty, self.level, 7) {
                    let p = self.bond.pos;
                    self.drones.play_pain_grunt(p);
                }
                // The drown tick's sound rides the same 30-tick cadence [M Player_MonitorAir
                // 0x191a70:135-143: every FRAME_RATE·0.5 ticks, Sound_Play(100, 0x60b) + HandlePain].
                self.drones.audio.play(0x60b);
            }
            // Hurt volumes (0xE4): the collide contact `Player_DealWithObjHit` 0x194458:94-101 — a
            // class-0x27 hit calls `Player_HandlePain(pain + Hurt_GetDamage, …, Hurt_GetType, −1)`,
            // the same full pipeline as the drown tick (difficulty factor, no armour for types 1..7
            // [M 0x1902c8:84-104]). The gate follows `Hurt_Update` 0x17e5d0's disabled bit.
            for h in self.hurts.iter_mut().filter(|_| net_row.is_none()) {
                if let Some((dmg, ty)) = h.update(self.drones.channels.on(h.gate as u32), self.bond.pos) {
                    if self.drones.health.pain_generic(&mut self.drones.rng, dmg, self.drones.difficulty, self.level, ty as u32) {
                        let p = self.bond.pos;
                        self.drones.play_pain_grunt(p);
                    }
                }
            }
            // The drone victims: `NDrone2_DealWithObjHit` 0x13fff0 gives NPCs the same class-0x27
            // treatment — the who masks naming the drone class bit 8 (07000008's kill-floor quads
            // #42-44, who 0x18 = classes 3+2, type 6, 1000 damage [D params]) are delivered for
            // it. The capsule is the trigger walk's drone body ([pos−1.0 … head], r 0.4 — the
            // Toucher law drone.rs builds for `Trigger_Calc`); the delivery is the explosion
            // path's flat subtract (`health -= damage · damage_taken`, `last_damage` stamped —
            // the drones' own update's health ≤ 0 selector picks the death). A one-shot volume
            // fired by any victim retires for all of them (the shared obj state [M 0x17e5d0:41-47]).
            let drone_capsules: Vec<(usize, [f32; 3], [f32; 3])> = self.drones.list.iter().enumerate()
                .filter(|(_, x)| x.enabled)
                .map(|(i, x)| (i, [x.pos[0], x.pos[1] - 1.0, x.pos[2]], x.head()))
                .collect();
            let mut drone_apps: Vec<(usize, f32)> = Vec::new();
            for h in self.hurts.iter_mut() {
                let gate_on = self.drones.channels.on(h.gate as u32);
                for (i, a, b) in &drone_capsules {
                    if let Some((dmg, _ty)) = h.update_touch(gate_on, *a, *b, 0.4, true) {
                        drone_apps.push((*i, dmg));
                    }
                }
            }
            for (i, dmg) in drone_apps {
                let x = &mut self.drones.list[i];
                x.last_damage = dmg;
                x.health -= dmg * x.damage_taken;
            }
            // `Player_MonitorAir` 0x191a70:101-124 [M]: the head crossing cel+0x90 starts the dive
            // loop (`Sound_Stop()` of the old handle, then `Sound_Play3D(100, −1, −1, 0x27, objPos)`
            // saved at BL+0x89C) and, on surfacing, stops it and plays the gasp `Sound_Play(100,
            // 0x60c)`. The port's stop is by id (one voice per id at the alloc).
            {
                let under = self.bond.underwater;
                if under && !self.drones.audio.swim_prev_under {
                    self.drones.audio.play3d(0x27, self.bond.pos, None);
                } else if !under && self.drones.audio.swim_prev_under {
                    self.drones.audio.stop_all_with_id(0x27);
                    self.drones.audio.play(0x60c);
                }
                self.drones.audio.swim_prev_under = under;
            }
            // unported-front-end-menus-3: `Camera_Update` 0x1bb6d8:25-34's shear trigger [M] —
            // while the eye sits below the cel's water surface (cel+0x90; the port's
            // equivalent comparison is `Bond::underwater`, `Player_MonitorAir` 0x191a70's
            // cel+0x90 head test) each frame runs `Camera_Shear` 0x1bba38; the blur byte
            // drops when the eye leaves (the port's only GlobalBlur writer).
            if self.bond.underwater {
                let _ = self.camera_shear();
            } else {
                self.global_blur = 0;
            }
            // unported-front-end-menus-3: `Player_Update` 0x193600:439-441's per-tick sway [M]
            // — the current weapon's swing amount (weapon_data+0x8c, `swing_amount`) × the
            // zoom factor, through `Camera_ApplySwing` 0x1bbf40 (the deltas are the viewer's
            // pitch/yaw/roll offsets; the port's draw side reads `swing_phase`).
            let swing = self.weapons.swing_amount(self.gun.weapon as usize);
            let _ = self.camera_apply_swing(swing, self.zoom);
        }
        // Footsteps (`AnimObjectUpdate` 0x1befd8:163-198 [M], player): `BL+0x924` gathers the distance
        // moved (`+= |anim+0xD8|`); past the stride it wraps (fmod) and a step sounds. The stride is
        // 1.65 — EXCEPT while the CLIMB substate (1) runs, where it is 0.4125 and the step is the fixed
        // `Sound_Play3D(0x286)` at Bond (the rung sound); every other step is `Effect_Create(…, foot|0x42)`
        // → the floor surface's footstep (`EffectInfo+0x16`) at the floor point, AI alertness 0
        // (`Sound_SetAlertness(0)` [M]). fix-ladder2: the port had the 0.4125/0x286 pair on CROUCH — rapid
        // footsteps on metal while crouch-walking. The game's crouch steps run the normal
        // 1.65 surface path; the crouch anim's smaller per-tick displacement slows them naturally.
        // *Suggested*: the distance is Bond's movement per tick (horizontal; 3D while climbing, where the
        // game's `anim+0xD8` follows the climbing anim up the rungs).
        let (dx, dy, dz) = (self.bond.pos[0] - before[0], self.bond.pos[1] - before[1], self.bond.pos[2] - before[2]);
        let moved_h = (dx * dx + dz * dz).sqrt();
        let climbing = self.bond.substate == nf_game::player::SS_CLIMB;
        let moved = if climbing { (moved_h * moved_h + dy * dy).sqrt() } else { moved_h };
        if self.bond.grounded && !self.disabled && self.bond.substate != nf_game::player::SS_SWIM {
            // The swim gate [M]: the game's step is `Effect_Create` off the CACHED ground hit
            // (0x1ce688:63 early-outs unless `(hit+0x50 & 0x3f) != 0`); the swim capsule (0x194df0
            // case 3) hangs under the surface and never touches the pond bed, so swimming plays NO
            // footstep. The port's floor ray still
            // finds the bed mid-pond, so the substate is the gate here.
            self.step_dist += moved;
            let stride = if climbing { 0.4125 } else { 1.65 };
            if self.step_dist > stride {
                self.step_dist %= stride;
                let p = self.bond.pos;
                if climbing { self.drones.audio.play3d(0x286, p, None); }
                else if let (Some(s), Some((t, _, m))) = (self.surfaces.as_ref(), self.world.segment_surface(p, [p[0], p[1] - 1.6347, p[2]], true)) {
                    if let Some(id) = nf_formats::sfx::SurfaceSounds::get(&s.footstep, m) { self.drones.audio.play3d(id, [p[0], p[1] - 1.6347 * t, p[2]], None); }
                }
            }
        }
        // The bob phases: `BL+0x8DC` += the forward speed (`BL+0x28`), `BL+0x8E8` += (π/2) / 60.
        self.bob[0] += self.bond.local[1];
        self.bob[1] += std::f32::consts::FRAC_PI_2 / 60.0;
        // The landing damage: `Player_CollisionHandler` 0x1949a8:117-126 calls `Player_HandlePain(timer - 60,
        // type 6)` — the full pipeline: the difficulty factor, NO armour (types 1..7 bypass it [M
        // 0x1902c8:84-104]), the pain flash +0x968 and the vertical direction bits +0x967 = 2.
        if let Some(dmg) = self.bond.fall_damage.filter(|_| net_row.is_none()) {
            if self.drones.health.pain_generic(&mut self.drones.rng, dmg, self.drones.difficulty, self.level, 6) {
                let p = self.bond.pos;
                self.drones.play_pain_grunt(p);
            }
        }
        let (head, look) = (self.bond.head(), self.bond.look());
        // The shot's direction through the crosshair: forward + 0.76966 cx · right + 0.57724 cy · up.
        let (yaw, p) = (self.bond.yaw, self.bond.pitch * std::f32::consts::FRAC_PI_2);
        let right = [-yaw.cos(), 0.0, yaw.sin()];
        let up = [-p.sin() * yaw.sin(), p.cos(), -p.sin() * yaw.cos()];
        let (tx, ty) = (0.76966 * self.cross[0], 0.57724 * self.cross[1]);
        let d = [0, 1, 2].map(|k| look[k] + tx * right[k] + ty * up[k]);
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let aim = d.map(|v| v / l);
        self.first_person = !self.disabled;
        // The pickup take test's LOS origin: the player's world-sphere centre (`obj+0x80`), not his position
        // [M 0x18e148:212-214] — `Bond::sphere_centre` ports `Control_BuildWorldSph`'s player branch.
        // fix-aim-raise2: the RAW button — run's edges (and the view chain's queue arm) must see
        // the ungated input, or the gated bit's rise at the raise's finish would re-fire the
        // aim-on edge and queue the raise twice.
        // fix-aims20-modes-lights: the analog-deflection half of the idle-counter gate [M 0x17ee88:80-84].
        let stick = input.forward != 0.0 || input.strafe != 0.0 || input.turn != 0.0 || input.look != 0.0;
        // mp-world-sync: online the world's contact passes see every body (`Drones::net_bodies`)
        // and every slot's use key runs in slot order (`net_use_pass`) — the local E no longer
        // reaches the world alone.
        let use_key = match net_row.as_ref() {
            Some(row) => {
                self.drones.net_bodies = Some(self.net_bodies_now());
                self.net_use_pass(row);
                false
            }
            None => use_key,
        };
        self.run(self.bond.pos, self.bond.sphere_centre(), head, look, aim, trigger, input.aiming || input.scope, reload, use_key, stick);
        self.drones.net_bodies = None;
        // `Player_Zoom` (0x1a1a70): the scope zoom. Right mouse (mouse-first) zooms in; releasing returns to 1.0.
        // The zoom interpolates toward the target at the game's rate (1.025× per tick zooming in, 1.15× out).
        // The FOV is divided by the zoom factor. Scoped weapons only (`weapon_data+0x68 & 0x40`).
        let scoped = self.weapons.records.get(self.gun.weapon as usize).is_some_and(|w| u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap()) & 0x40 != 0);
        let max_zoom = self.weapons.zoom(self.gun.weapon as usize);
        if scoped {
            // fix-aim-raise: the zoom starts only when the aim BIT is set — the game writes +0x8D4
            // from the per-weapon table on the bit's rise [M Player_Weapon 0x1a24c8:156-187] and
            // `Player_Zoom` 0x1a1a70:18 reads the zoom button only while the bit is set. During
            // the raise the bit is clear, so the target stays 1.0 until the raise's last tick.
            if input.zoom && self.aiming {
                // microcam-wheel-zoom: a wheel notch while the aim is held steps the zoom target
                // ×2 (up) / ÷2 (down) within the weapon's own range [1.0, weapon_data+0x88 — the
                // max `Player_Weapon` writes to BL+0x8D8]. The ×2 step is port-only design, not a
                // game claim: the PS2 pad has no wheel, and the game's own
                // zoom is continuous — `Player_Zoom` 0x1a1a70 ramps the held target ×1.025/tick
                // [M 0x1a1a70:22-39]. Once the wheel has stepped, the hold-ramp
                // to max yields (`zoom_wheel`) so the notch stays until the aim falls.
                if input.zoom_cycle != 0 {
                    let step = (2.0f32).powi(input.zoom_cycle as i32);
                    self.zoom_target = (self.zoom_target * step).clamp(1.0, max_zoom.max(1.0));
                    self.zoom_wheel = true;
                } else if !self.zoom_wheel {
                    self.zoom_target = max_zoom;
                }
            } else {
                self.zoom_target = 1.0;
                self.zoom_wheel = false;
            }
        } else {
            self.zoom_target = 1.0;
            self.zoom_wheel = false;
        }
        // Interpolate zoom toward target [M 0x1a1a70 lines 22-39].
        // `Player_Zoom` (0x1a1a70): zoom in at 1.025×/tick, zoom out at 1.15×/tick [M lines 22-39].
        // The game's BLData+0x8D4 is the inverse (decreases as zoom increases); the port's `zoom` is the FOV
        // divisor (increases as zoom increases), so the interpolation factor is the complement.
        let rate = if self.zoom < self.zoom_target { 1.0 + 0.025 } else { 1.0 + 0.15 };
        self.zoom = self.zoom + (self.zoom_target - self.zoom) * (1.0 - 1.0 / rate);
        self.zoom = self.zoom.clamp(1.0, max_zoom.max(1.0));
        // Wire / creep: the music event moved up with the substate effects (fix-wire) — the substate is
        // now the only source (the old proximity hack is gone).
        let _ = &self.wires;
        // Script hand-offs (docs/systems/script-player.md): event 4 stops Bond; `Script_Free` after event 6 puts him
        // back at the script's matrix (on the floor below it), facing its +Z.
        for e in std::mem::take(&mut self.drones.player_events) {
            match e {
                nf_game::nis::Request::PlayerDisable => {
                    self.disabled = true;
                    // nis-timer-pause: the game's event-4 arm is Player_Disable +
                    // PlarStat_LogTimerPause(0) [M 0x1d94c0:118-121] — the cut-scene freezes the
                    // mission clock (the accumulator runs only while +0x34 == 0, [M 0x1995a0:9-14]);
                    // Script_Free's re-enable unpauses it [M 0x1da880:44-48]. Without this, both of
                    // 07000007's cut-scenes (SP #419 3000 ticks, SP #420 4326 ticks) keep the
                    // timer row counting.
                    // SP slot 0 [M — the game's call passes slot 0].
                    self.drones.progress.stats[0].log_timer_pause();
                    // goggles: `Player_SetCamMode` 0x195928 clears viewer+0x236 on every camera
                    // -mode change (arms :15/:23/:27/:40) and `Camera_Reset` 0x1b9af0:50 clears
                    // it too — a cut-scene's cam hand-off kills the goggles view. The battery
                    // (BL+0x920) is untouched — only the mode byte drops.
                    self.hud.goggles.mode = 0;
                    self.hud.goggles.night_pane_on = false;
                    self.hud.goggles.xray_pane_on = false;
                }
                // nis-event9-cnis: `Player_Cam2Mode` 0x195fc8 [M — mode 3 → `Player_SetCamMode(BL, 2)`
                // 0x195928 (the cut-scene behind-Bond orbit, `Player_PositionCamera` 0x18f5d8 case 2);
                // anything else → `BL+0x8a8 = 0` + `SetCamMode(0)` + `Player_ChangeSubState(player, 0)`
                // 0x195fc8:11-16 — the walk substate, the exit hand-off]. The game's SetCamMode(2) arm
                // also runs `HUD_Reset(BL)` + `viewer[BL+0x94e]+0x236 = 0` [M 0x195928 default] — HUD
                // state the port does not draw; `BL+0x8a8` has no port model (the game's GunImp writes
                // it, `Player_Move` 0x191dd0:151 reads it in the aim re-heading) so the zero is a no-op
                // here.
                nf_game::nis::Request::Cam2Mode { mode } => {
                    if mode == 3 {
                        self.bond.cam_mode = 2;
                    } else {
                        self.bond.cam_mode = 0;
                        self.bond.substate = nf_game::player::SS_WALK;
                    }
                }
                nf_game::nis::Request::PlayerEnable { at, .. } => {
                    // nis-timer-pause: Script_Free's arm is Player_Enable +
                    // PlarStat_LogTimerUnpause(0) [M 0x1da880:44-48 — +0x2C := the clock, +0x34 := 0].
                    self.drones.progress.stats[0].log_timer_unpause(nf_game::progress::PlrMissionStats::now_100ths(self.drones.frame));
                    if let Some(m) = at {
                        self.bond = nf_game::player::Bond::stand_at(&self.world, [m[3][0], m[3][1], m[3][2]], m[2][0].atan2(m[2][2]));
                    }
                    self.disabled = false;
                }
                _ => {}
            }
        }
        self.first_person = !self.disabled;
        if let Some(shared) = self.net_rng_shared.take() {
            self.net_rng = Some(std::mem::replace(&mut self.drones.rng, shared));
        }
        // mp-world-sync: the shared world's per-body consequences, on the shared stream.
        if let Some(row) = net_row { self.step_net_world(&row); }
        // mp-remote-fx: offline (split/bots) the bots' shot effects flush here; online the flush
        // runs after the per-slot combat (`step_net_combat`).
        if self.net_rng.is_none() { self.flush_mp_fx(); }
    }

    /// drive-ptp: the ACTION→DRIVING handover for a `0x0730xxxx` interlude — the port of
    /// `GameFlow_Main` 0x1c98c8 case 9 (`Boot_GetPTPData` + `psiLaunchDriving`). The DRIVING
    /// program itself is not ported (a separate ELF; only its exit contract is — [M
    /// psiLaunchDriving 0x120af0:12 copies the PTP block and execs]), so the drive's outcome is
    /// the dev choice (`driving_result`). Builds the PTP block from the Sim's
    /// state, runs the DRIVING-side exit, and arms the resume: the level the return
    /// names (`Boot_LoadPTPData` 0x1c8560's unconditional 0x07000048) as a COLD load, with the
    /// return stashed for lib.rs to apply after the reload (the mission status + the 12 stats).
    /// Pub so the headless tests drive the exact path the view's event drain runs.
    pub fn ptp_handover(&mut self, next: u32) {
        // The game NEVER converts an interlude into a drive: the interlude movie ends and
        // `GetLevelToLoadFromMovieCode` 0x1d3560:24-50 returns 0x7000048 (the hub) for EVERY drive
        // interlude; the player then picks the mission (driving hashes included) from the sp_level
        // list, and the state-9 handover's index is `GameState._8_4_ − 0x9000000` [M
        // Boot_GetPTPData 0x1c8198:62]. This call is the port's dev auto-continue (the drive
        // outcome is the dev `driving_result` anyway) — the pairing lives in ptp.rs.
        let index = nf_game::ptp::interlude_drive_index(next);
        let map_id = 0x0900_0000 | index as u32; // GameState._8_4_ [M 0x1c8198:62]
        let block = nf_game::ptp::PtpBlock::build(&nf_game::ptp::BuildInputs {
            map_id,
            game_state_40: self.drones.difficulty, // GameState+0x28 [M 0x1c7f88]
            game_state_52: 0,                      // word 82 — not tracked [M-negative]
            stats: self.drones.driving_stats,
            result: self.drones.driving_result,
        });
        let ret = nf_game::ptp::drive_exit(
            self.drones.driving_result,
            block.drive_index as u8,
            self.drones.driving_stats,
            block.mission_state_1, // the block's +0x144 — the difficulty Boot_LoadPTPData restores [M 0x1c8198:20, 0x1c8560:36]
        );
        eprintln!(
            "[ptp] drive handover: map {map_id:08x} index {} key '{}' → result {} (MissionState {}), resume {:08x}",
            block.drive_index,
            nf_game::ptp::key_for_index(block.drive_index as u8),
            ret.result,
            ret.mission_state(),
            nf_game::ptp::RESUME_LEVEL,
        );
        self.drones.driving_return = Some(ret);
        self.drones.level_exit = Some(nf_game::ptp::RESUME_LEVEL);
        // The return is a fresh ACTION boot (BASE.ELF re-dispatch → GameFlow case 1 →
        // Boot_LoadPTPData) — the port models it as a cold load [M 0x1c8560].
        self.drones.level_exit_cold = true;
    }

    /// The cut-scene camera, when a script runs one: position, yaw, pitch (from the matrix's +Z) and vertical field of
    /// view.
    pub fn script_camera(&self) -> Option<([f32; 3], f32, f32, f32)> {
        let (m, fov) = self.drones.scripts.camera()?;
        let f = m[2];
        let l = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt().max(1e-6);
        Some(([m[3][0], m[3][1], m[3][2]], f[0].atan2(f[2]), (f[1] / l).clamp(-1.0, 1.0).asin(), fov))
    }

    /// The wall-shimmy's third-person tracking camera (`Player_PositionCamera` 0x18f5d8 case 0xb →
    /// `camera_tracking` 0x1ba5e0), Some only while the creep substate (7) runs (third person, not
    /// first). Position, yaw, pitch (radians,
    /// the window's Camera convention) and the FOV factor 1.0 (`viewer+0x108 = 1.0` [M 0x18f5d8:233]).
    ///
    /// The game, per creep tick [M 0x1ba5e0:69-110]:
    /// 1. the carried wall's axis line — `wall+0x30` ± `Vec_Spherical_2_Cartesian(wall+0x8C,
    ///    wall+0x54 + π/2, wall+0x58)` — with BOTH ends' Y raised **+1.0** for the creepwall's '+'
    ///    class (the closest-point line hovers at Bond's head while he crawls) [M 0x1ba5e0:80-88];
    /// 2. the closest point of that line to the player's position (obj+0x30) [M 0x1ba5e0:90];
    /// 3. the camera sits at `closest − Cartesian(radius, yaw, −pitch)` with the radius the entry
    ///    set (`viewer+0x1F0 = 4.0` [M Player_CollCreepWall 0x196758:123]), the yaw the wall's +0x54
    ///    + π [M 0x1ba5e0:104-106] and the pitch the creepwall constant **0x402fede0 = 2.7486268**
    ///    [M 0x1ba5e0:106-110] (a wall-specific ThirdPersonCameras spline only when the wall's
    ///    BLData u16[1] (placement param 1) is set — 07000005's two 0x3F placements carry none
    ///    [D probe], so the computed spherical is the live path);
    /// 4. the look at the closest point, from the actual offset (the tail atan2s [M 0x1ba5e0:135-155]).
    ///
    /// In the port's convention (`Cartesian(r, θ, φ)` = `r·(sinθ·cosφ, −sinφ, cosθ·cosφ)` — the
    /// wall axis [M 0x196758:86-90] pins the φ=0 slice) the offset resolves to
    /// `cam = closest + axis_dir·(4.0·sin(2.7486268)) + up·(4.0·|cos(2.7486268)|)` with
    /// `axis_dir = fwd(wall.yaw + π/2)` — ~1.53 along the wall from the tracked point, ~3.70 above
    /// it, looking down at Bond. The game also draws Bond's body here (`View_SetDrawInAllViews`
    /// [M 0x18f5d8:228]) and hides the view model (`View_SetDrawInNoViews(BLData+0x7e8)` [M :229]) —
    /// the port has no player body mesh; the view-model hide lives in lib.rs's `view_model_entries`.
    pub fn creep_camera(&self) -> Option<([f32; 3], f32, f32, f32)> {
        if self.disabled || self.bond.substate != nf_game::player::SS_CREEP { return None; }
        let w = self.creepwalls.get(self.bond.creep?)?;
        // The tracked point: the closest point of the wall axis to Bond's pos [M 0x1ba5e0:90 —
        // `vecutil_calculate_closest_point_on_line` clamps to the segment ends, as the port's
        // `CreepWall::closest_point` does], with the line's Y +1.0 [M 0x1ba5e0:80-88]. Lifting both
        // ends by a constant does not move the projected t, so the lift applies to the result.
        let closest = w.closest_point(self.bond.pos);
        let closest = [closest[0], closest[1] + 1.0, closest[2]];
        // The spherical offset [M 0x1ba5e0:106-125], resolved in the port's convention (see the doc).
        const CAM_PITCH: f32 = 2.748_626_8; // 0x402fede0 [M 0x1ba5e0:108]
        let (sp, cp) = CAM_PITCH.sin_cos();
        let (along, up) = (4.0 * sp, -4.0 * cp); // 1.5317 along the axis, 3.6952 up
        let axis = [(w.yaw + std::f32::consts::FRAC_PI_2).cos(), 0.0, -(w.yaw + std::f32::consts::FRAC_PI_2).sin()];
        let cam = [closest[0] + axis[0] * along, closest[1] + up, closest[2] + axis[2] * along];
        // The look at the tracked point (the tail atan2s [M 0x1ba5e0:135-155]).
        let d = [closest[0] - cam[0], closest[1] - cam[1], closest[2] - cam[2]];
        let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
        let yaw = d[0].atan2(d[2]);
        let pitch = (d[1] / horiz.max(1e-4)).clamp(-1.0, 1.0).asin();
        Some((cam, yaw, pitch, 1.0))
    }

    /// cable-3p: the wire's third-person tracking camera (`Player_PositionCamera` 0x18f5d8 case 0xb ->
    /// `camera_tracking` 0x1ba5e0, the wire branch), Some while Bond hangs on a wire or rides a zipline
    /// in cam mode 0xB (`Player_CollWire` 0x197130 sets it with viewer+0x1F0 = 4.0; the drops set 0
    /// [M]). Position, yaw, pitch (the window's Camera convention) and the FOV factor 1.0. Per tick
    /// [M 0x1ba5e0]:
    /// 1. the line: obj+0x30 +- `Vec_Spherical_2_Cartesian(obj+0x8C, obj+0x54 + pi/2, obj+0x58)`,
    ///    both ends' Y -1.0 (+1.0 only for wire type 1, which no grab accepts) [:83-108];
    /// 2. the tracked point = the closest point of that line to Bond (obj+0x30), and
    ///    t = Dist2D(start, closest) / Dist2D(start, end) [:109-113];
    /// 3. the pitch pi - t*pi/6, the yaw obj+0x54, +-pi/4 by the move-mode byte (7 -> +, 8 -> -) [:114-148];
    /// 4. the camera at `closest - Cartesian(4.0, yaw, -pitch)` [:149-154];
    /// 5. a wire whose instance[1] (placement param 0) is non-zero takes the position from
    ///    `ThirdPersonCameras[param0]` at t (`ThirdCam_GetPos` 0x1bc590), and that camera's flag byte
    ///    (ThirdCam param 1) picks the view: 0 = the spline matrix as is, else the look-at [:171-182];
    /// 6. otherwise the look at the tracked point [:185-202].
    pub fn wire_camera(&self) -> Option<([f32; 3], f32, f32, f32)> {
        use nf_game::player::{SS_WIRE, SS_ZIPLINE};
        if self.disabled || self.bond.cam_mode != 0xB || !matches!(self.bond.substate, SS_WIRE | SS_ZIPLINE) { return None; }
        let w = self.wires.get(self.bond.wire?)?;
        let off = nf_game::searchlight::spherical(w.radius, w.yaw + std::f32::consts::FRAC_PI_2, w.pitch);
        let lift = if w.wire_type == 1 { 1.0 } else { -1.0 };
        let o = w.origin;
        let a = [o[0] + off[0], o[1] + off[1] + lift, o[2] + off[2]];
        let b = [o[0] - off[0], o[1] - off[1] + lift, o[2] - off[2]];
        let closest = nf_game::wire::Wire::closest_point(a, b, self.bond.pos);
        let d2 = |p: [f32; 3], q: [f32; 3]| ((p[0] - q[0]).powi(2) + (p[2] - q[2]).powi(2)).sqrt();
        let t = d2(a, closest) / d2(a, b).max(1e-6);
        let pitch = std::f32::consts::PI - t * 0.523_598_8;
        let mut yaw = w.yaw;
        if self.bond.mode == 7 { yaw += std::f32::consts::FRAC_PI_4; }
        if self.bond.mode == 8 { yaw -= std::f32::consts::FRAC_PI_4; }
        // `Vec_Spherical_2_Cartesian` with the game's height sign (−r·sin φ — the creep camera's resolved
        // convention, see `creep_camera`): at pitch π − t·π/6 the camera rises above the tracked point,
        // looking down on Bond [L PCSX2 07000005 cable, audits/render-1029-cable p-stand].
        let c = nf_game::searchlight::spherical(4.0, yaw, -pitch);
        let c = [c[0], -c[1], c[2]];
        let mut cam = [closest[0] - c[0], closest[1] - c[1], closest[2] - c[2]];
        if w.param0 != 0 {
            if let Some(tc) = self.drones.worldobjs.thirdcams.iter().find(|tc| tc.cam_id == w.param0) {
                if let Some(m) = tc.get_pos(t) {
                    cam = [m[3][0], m[3][1], m[3][2]];
                    if tc.flag == 0 {
                        // The spline matrix as the view: its forward row.
                        let f = m[2];
                        let l = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt().max(1e-6);
                        return Some((cam, f[0].atan2(f[2]), (f[1] / l).clamp(-1.0, 1.0).asin(), 1.0));
                    }
                }
            }
        }
        let d = [closest[0] - cam[0], closest[1] - cam[1], closest[2] - cam[2]];
        let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
        Some((cam, d[0].atan2(d[2]), d[1].atan2(horiz.max(1e-4)), 1.0))
    }

    /// nis-event9-cnis: NIS event 9's cut-scene camera (`Player_PositionCamera` 0x18f5d8 case 2),
    /// Some while `BL+0x950 == 2` — the mode `Player_Cam2Mode` 0x195fc8's mode-3 arm set. The game,
    /// per tick [M 0x18f5d8 case 2:145-156]:
    /// 1. `camera_track_XZ_pos_behind(1.0, 10.0, obj+0x30, obj+0x50, viewer)` [M 0x1bc260] — the
    ///    camera sits 10 units behind Bond on the XZ plane: `pos − 10·(sin(yaw), cos(yaw))`, the
    ///    yaw read at `obj+0x54` (the second word of the spherical orientation at `obj+0x50` [M
    ///    `Player_Enable` 0x195498:69 fills `obj+0x50..0x5b` from the matrix's direction]) plus the
    ///    BSS debug global `debug_cam_ry`; the lerp factor is 1.0 (0x3f800000), so it snaps;
    /// 2. `camera_track_Y_pos(1.0, 0.0, obj, viewer)` [M 0x1bc1f0] — `cam.y ← obj+0x34 + 0 +
    ///    debug_cam_posy`, Bond's own Y anchor (the debug globals are BSS with no non-debug writer
    ///    in ACTION.ELF's decompile — 0.0 in play);
    /// 3. `Camera_PosYRot`/`Camera_PosXRot(1.0, obj+0x30, viewer)` [M 0x1bc0c0/0x1bc148] — the view
    ///    snaps to look at Bond's position: yaw `atan2(dx, dz)`, pitch `−atan2(dy, √(dx²+dz²))`
    ///    (both lerped by 1.0);
    /// 4. `viewer+0x108 = 1.0` (the FOV factor — the unzoomed value; the zoomed modes 0/0xe feed
    ///    `BL+0x8D0` there [M :129, :154]) and `viewer+4 = 0` [M :155 — the extra rotation the port
    ///    does not model].
    /// The game also draws Bond's body (`View_SetDrawInAllViews`) and hides the view model
    /// (`View_SetDrawInNoViews(BL+0x7e8)`) [M :146-148] — no player body mesh in the port; the
    /// view-model hide rides the cutscene flag in lib.rs.
    pub fn cam2_camera(&self) -> Option<([f32; 3], f32, f32, f32)> {
        if self.bond.cam_mode != 2 { return None; }
        let pos = self.bond.pos;
        let yaw = self.bond.yaw;
        let cam = [pos[0] - 10.0 * yaw.sin(), pos[1], pos[2] - 10.0 * yaw.cos()];
        let d = [pos[0] - cam[0], pos[1] - cam[1], pos[2] - cam[2]];
        let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
        let look_yaw = d[0].atan2(d[2]);
        let look_pitch = -(d[1]).atan2(horiz);
        Some((cam, look_yaw, look_pitch, 1.0))
    }

    /// nis-event9-cnis: the `C_NIS_Handler` route's boot — the mission-driven cut-scene entry the
    /// SP placements don't cover (`C_NIS_Handler` 0x205f10, the IN-GAME NIS page 0x1000003e's
    /// handler [M `Handler_HandleMessage` 0x2029e0:223-227]). The game's msg-0x4b tail [M
    /// 0x205f10:92-112]: when the page's bound FMV script is NOT already playing, resolve the
    /// current level's script hash (the page's list item value, `Menu_SendMessage(msg 0x35)` —
    /// only `List_SendMessage` 0x2196e8:463 answers 0x35), `LoadableLoad` it if the isLoadable
    /// table names a pack, then `Script_Load` + `Player_Disable(glb_players, 0)` + `Script_Play`
    /// + `Camera_PushStates` + `Camera_Enable(7,0,0,0)`; `show_frame_count = 1` [M :88] is the
    /// debug frame counter the port doesn't draw.
    ///
    /// Port mapping: the pack streaming is the load path's own type-0xC pass (`isLoadable`/
    /// `LoadableLoad` — the pack's scripts already sit in `drones.scripts.lib`); `Script_Load` +
    /// `Script_Play(…, 0)` is the once-mode script player (`Scripts::spawn_once`, identity matrix
    /// [M — Script_Load 0x1d8248 takes pos/rot VECTORs and bakes them into the player's matrix
    /// through View_RotTransMatrix 0x1e5168:33-48, but the NIS caller passes the global zero
    /// vector twice (`0x2d6200`, 0x205f10:103-106), so the page-boot player starts at identity]);
    /// `Player_Disable` rides the same
    /// `player_events` hand-off event 4 uses; `Camera_Enable(7,0,0,0)` selects the cut-scene
    /// viewer whose view the script's kind-4 camera stream drives — the port's single
    /// `script_camera()` path. The route's `msg 0x51` bindings (≈70 `Menu_SendMessage(0x10,
    /// control, hash)` pairs [M 0x205f10:7-70]) set runtime M_CONTROL heap pointers' values — the
    /// disc census found NO authored def for page 0x1000003e anywhere on NTSC-U (the census test),
    /// so no level resolves a binding today: the boot takes the resolved hash as its argument.
    pub fn nis_page_boot(&mut self, script_hash: u32) -> bool {
        // The FMVScript gate [M 0x205f10:92 — `Script_IsPlaying(FMVScript) == 0`]: the page
        // re-shown while its cut-scene runs is a no-op.
        if let Some(h) = self.fmv_script {
            if self.drones.scripts.players.iter().any(|p| {
                p.scripts[0] == h && p.state == 2 && p.info[p.cur].as_ref().is_some_and(nf_game::nis::Script::playing)
            }) {
                return false;
            }
            self.fmv_script = None;
        }
        // `hashtable_getitem(hash)` [M 0x205f10:104-106] — the port's lib; a hash neither the
        // level nor its packs carry (the game's `Script_Load` would have nothing to load).
        if !self.drones.scripts.lib.contains_key(&script_hash) { return false; }
        let ident = [[1.0f32, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        self.drones.scripts.spawn_once(self.drones.scripts.players.len(), script_hash, ident);
        self.fmv_script = Some(script_hash);
        // `Player_Disable(glb_players, 0)` [M 0x205f10:102] — the same hand-off event 4 rides.
        self.drones.player_events.push(nf_game::nis::Request::PlayerDisable);
        true
    }

    /// The clip slot the held weapon reads and writes: the BASE's when the record is a twin (+0x05
    /// set) sharing the base's ammo type, else its own [M `Player_ReloadAmmoType` 0x1a1988:18-22 —
    /// `iVar4 = base` routes `clip[0x1B8 + slot·0xC]` to the base for such twins; the give path's
    /// `pickup.rs clip_slot` is the same unification]. One physical magazine per twin family.
    // fix-hud-ammo-odd: pub(crate) — the dev window's ammo pin (dev.rs after_tick) resolves the
    // twin slot with the same pick [M Player_AmmoIndex 0x1a9148]; no behaviour change.
    pub(crate) fn clip_slot(&self, id: usize) -> usize {
        let base = self.weapons.base(id) as usize;
        if self.weapons.shares_base_clip(id) && self.weapons.ammo_type(id) == self.weapons.ammo_type(base) { base } else { id }
    }

    /// The aim ray's trigger pick (`Check_Target` 0x1a7810's class-0x1D arm) [M 0x1a7810:193-208].
    /// **[S unverified as to shape]** — the game's ray hits the trigger's collision through the shared
    /// hitlist; the port's `world.segment` returns no object id, so the pick tests each trigger's
    /// placement box (`lo`/`hi`) and is wall-occluded like `Doors::pick_target`. Spent exits are
    /// skipped: the game latches their discrete type to 99 (`Trigger_Activate` 0x1b2318:113), so
    /// `Trigger_Type` no longer reads 10 — the port's `gone` flag carries that latch. Returns
    /// `(trigger index, hit fraction of the 50-unit ray)` of the nearest touch, or None.
    fn pick_trigger_target(&self, eye: [f32; 3], dir: [f32; 3], wall_t: f32) -> Option<(usize, f32)> {
        let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        if l < 1e-9 { return None; }
        let d = [dir[0] / l, dir[1] / l, dir[2] / l];
        let mut best: Option<(f32, usize)> = None;
        for (i, t) in self.drones.triggers.iter().enumerate() {
            if t.gone || t.tris.is_empty() { continue; } // gone = the type←99 latch [M 0x1b2318:113]; no tris = no collision (01's #18 [D])
            // Ray-vs-AABB slab test [S stand-in shape; the box itself is [M] the placement box
            // `Trigger::from_params` stores from the level data].
            let (mut t0, mut t1) = (0.0f32, wall_t * 50.0);
            let mut hit = true;
            for a in 0..3 {
                let (lo, hi) = (t.lo[a], t.hi[a]);
                let o = eye[a]; let dd = d[a];
                if (lo - hi).abs() < 1e-6 {
                    if (o - lo).abs() > 1e-4 && dd.abs() < 1e-9 { hit = false; break; }
                    continue;
                }
                if dd.abs() < 1e-9 {
                    if o < lo || o > hi { hit = false; break; }
                    continue;
                }
                let (mut ta, mut tb) = ((lo - o) / dd, (hi - o) / dd);
                if ta > tb { std::mem::swap(&mut ta, &mut tb); }
                t0 = t0.max(ta); t1 = t1.min(tb);
                if t0 > t1 { hit = false; break; }
            }
            if !hit || t1 < 0.0 { continue; }
            let entry = t0.max(0.0) / 50.0; // the ray is the 50-unit Check_Target ray [M 0x1a7810:114-116]
            if entry >= wall_t { continue; }
            if best.is_none_or(|(bt, _)| entry < bt) { best = Some((entry, i)); }
        }
        best.map(|(tf, i)| (i, tf))
    }

    fn run(&mut self, pos: [f32; 3], sph: [f32; 3], camera: [f32; 3], look: [f32; 3], aim: [f32; 3], trigger: bool, aiming: bool, reload: bool, use_key: bool, stick: bool) {
        // fix-weapon-anims + fix-aim-raise2: the aim raise/lower edges — the RAW button (the
        // param; the gated BIT is `self.aiming`, which `aim_gate` maintained before run) — before
        // the level is overwritten (the queue arm deep in the view chain reads the EDGE)
        // [M Player_Weapon 0x1a24c8:117-152: the arm fires on the tick the input flips the bit].
        let aim_on_edge = aiming && !self.prev_aiming;
        let aim_off_edge = !aiming && self.prev_aiming;
        self.prev_aiming = aiming;
        // The drones aim at and look for Bond's chest (`Drone_View_Bones`; one point here, 0.45 under the eye).
        let hanging = matches!(self.bond.substate, nf_game::player::SS_WIRE | nf_game::player::SS_ZIPLINE);
        // The game's collision capsule off `pos` (= Bond's obj+0x30): standing (case 0/6) ends
        // pos+up·0.55 / pos+up·(0.275 − cc); crouched (case 3) pos+up·(0.65 − cc) / pos+up·(0.45 − cc)
        // [M Player_Collision 0x194df0]. Kept on `Player` for the toucher work: the trigger
        // toucher keeps the MEASURED reach law (`Toucher::player(pos − 0.6)` — the live
        // [0.557, 0.575] band below the reference, drone.rs's toucher comment [L ee_start_05.bin
        // glb_players→+0xc4 + fix-truck's trace]); the game-accurate capsule was tried and
        // REVERTED — it stalled 07000005's truck chain (whose game route is likely the class-5
        // object contact, trigger-port-check #11, still unproven) and touched the ch-0x70 park
        // plate standing.
        let capsule = if self.bond.crouched {
            ([pos[0], pos[1] + 0.65 - self.bond.cc, pos[2]], [pos[0], pos[1] + 0.45 - self.bond.cc, pos[2]])
        } else {
            ([pos[0], pos[1] + 0.55, pos[2]], [pos[0], pos[1] + 0.275 - self.bond.cc, pos[2]])
        };
        let player = nf_game::drone::Player { pos, target: [camera[0], camera[1] - 0.45, camera[2]], eye: camera, aim: look, sph, hanging, capsule,
            // standins-drone-922: the facing yaw the flash factor reads [M 0x12d890:328-330].
            yaw: self.bond.yaw,
            cel: self.cel_at(self.bond.pos).and_then(|room| self.room_cell.get(room).copied().flatten()) };
        let d = nf_game::drone::Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: self.level, sequences: &self.seqs, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models, surfaces: self.surfaces.as_ref(), voices: self.voices.as_deref(), difficulty: self.drones.difficulty,
            ai_points: &self.ai_points };
        // `Player_WeaponFiring` → `Player_WeaponInitBullet` from the eye along the view (*deviation*: no auto-aim and
        // no aim-point ray yet); the first-person fire script's sound 0x403 feeds the drones' hearing.
        // The gun's clip and spare rounds live in Bond's inventory (BLData `+0x1B8`, `+0x170`), which pickups fill.
        // The clip slot is the game's SHARED one [M Player_ReloadAmmoType 0x1a1988:18-22: a twin record
        // (+0x05 set) whose ammo type matches its base reloads `clip[base]` — one physical magazine per
        // family; pickup.rs `clip_slot` (the give path) already equips into it]. The port read/wrote the
        // twin's own slot here, so every 0x02↔0x03 mode hop read the never-filled clip[0x03] (= 0) — the
        // magazine zeroed and the next tick auto-reloaded, spending carried rounds the game never spends.
        let (wi, t) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
        if let (Some(&c), Some(&n)) = (self.drones.inventory.clip.get(wi), self.drones.inventory.carried.get(t)) { self.gun.clip = c.max(0) as u32; self.gun.spare = n.max(0) as u32; }

        // Weapon switch state machine (`Player_SetWeaponAnimObj` 0x1a2e38 + `Player_WeaponSelect` 0x1a2cc0):
        // the holster/draw cycle. The holster script is `weapon+0xC4` (played at 1.25x), the draw script is
        // `weapon+0xC0` (1.25x), the variant-swap script is `weapon+0xC8` (1.3x for same-base swaps) [M disassembly].
        // State 0 (idle): a switch request starts the holster. State 1 (holstering): the holster script plays; when
        // done, swap the weapon byte and start the draw. State 3 (drawing): the draw script plays; when done, back to idle.
        // State 8 (variant swap): the variant-swap script plays; when done, back to idle.
        // While switching, the trigger and reload are blocked (the game's state 1/3 checks `AnimScriptIsStopped`).
        let log_switch = std::env::var_os("NF_SOUNDLOG").is_some();
        let mut switching = self.gun.switch_state != 0;
        // fix-weapon-anims2: the script-frame watermark the op-1 sound dispatch below measures its
        // [prev_frame+1, new_frame] window against — captured BEFORE the switch machine advances the
        // draw/holster/variant scripts (its tail advance at the end of the `if switching` block), so
        // the window covers the whole tick's advance. Captured after it (where fix-weapon-anims had
        // left it), the switch's advance had already moved view_time before the watermark, the window
        // came up empty (from = to) and NO op-1 fired while a switch played — the Samurai's draw whoosh
        // (0x5c7 at frame 90 of its 107-frame draw script) went silent on every switch [M
        // AnimProcessScriptCmds runs per AnimObjectUpdate on the PLAYING script — holster/draw included,
        // 0x1c1b38; the game's window is (prev frame, new frame] of the same tick].
        let (prev_script, prev_time) = (self.view_script, self.view_time);
        // State 0 → 1/3/8: a switch request starts the holster (`Player_WeaponSelect` 0x1a2cc0).
        // The reload states block it here too (the game's refused view-state set {1,3,5,6,7,8,9,0xb,
        // 0xd..0x10} covers 5/6/7): a pending request simply waits for the reload to finish [M 0x1a6f80:33-47].
        if self.gun.switch_state == 0 && self.gun.reload_state == 0 && self.gun.switch_request != self.gun.weapon {
            // goggles: every switch start saves the outgoing weapon to `anim+0x64` (the port's
            // `saved_weapon`) [M Player_SetWeaponAnimObj 0x1a2e38:197-198 disasm 0x1a31e8-0x1a31fc;
            // Player_WeaponChange 0x1a6f80:142 writes the same at the request]. The goggles arm
            // path reads it for the auto-stow (hud_tick): the goggles return to the weapon they
            // were drawn from, clamped {0x47,0x5d,0x5e} → 1 [M 0x19e9d8:56-66].
            self.gun.saved_weapon = self.gun.weapon;
            let cur_id = self.gun.weapon;
            let req_id = self.gun.switch_request;
            let cur_base = self.weapons.base(cur_id as usize);
            let req_base = self.weapons.base(req_id as usize);
            // fix-weapon-anims2: the switch DISPLACES a playing fire script — `Player_WeaponSelect`
            // 0x1a2cc0 writes the view state +0xF4 = 8/3/1 directly, discarding the fire state 9 (and
            // the +0x95C spawn latch with it — the game's case-9 tail can never run once the state is
            // gone). The port kept gun.fire_state/latch set across the switch: the next weapon's first
            // pull then hit the state-9 suppress (`was_fire_state && flags & 0x100400 && +0x44 != 0`)
            // and queued NO fire script — the sweep's 0x41/0x42/0x43 cells never fired after a cell
            // whose fire script was still playing at its section end. (The fire script's loop sounds
            // already die at the holster arm below [M 0x1a2cc0:33-38].)
            self.gun.fire_state = false;
            self.gun.latch = false;
            // `Player_WeaponSelect` variant-swap path: same base weapon AND same first-person model → play +0xC8
            // at 1.3x, state 8 (a fast swap without holster/draw) [M 0x1a2cc0 lines 13-29].
            // fix-silencer-modes: the twin-variant cycle marker [M Player_WeaponSelect 0x1a2cc0:15-16]
            // is written for EVERY same-base request, before the FP-model gate — (requested id − base
            // id), 0 when the base itself is equipped. `Player_WeaponChange`'s walk (0x1a6f80:180-195)
            // reads it to hand the equipped twin back on the next cycle to this family.
            if cur_base == req_base {
                let slot = cur_base as usize;
                if slot < self.gun.twin_skip.len() {
                    self.gun.twin_skip[slot] = req_id as i8 - self.weapons.base(req_id as usize) as i8;
                }
            }
            if cur_base == req_base && self.weapons.first_person(cur_id as usize) == self.weapons.first_person(req_id as usize) {
                let variant = self.weapons.variant_swap_script(req_id as usize);
                if variant != 0 {
                    if log_switch { eprintln!("[weapon-switch] tick {}: state 0→8 variant-swap {:08x} (weapon {:#x}→{:#x}, same base {:#x})", self.drones.frame, variant, cur_id, req_id, cur_base); }
                    self.view_script = variant;
                    (self.view_time, self.view_speed) = (0.0, 1.3);
                    self.gun.switch_state = 8;
                    switching = true;
                    // The variant swap writes only the weapon byte, the mode byte, the +0xC8 script
                    // at 1.3x and view state 8 [M 0x1a2cc0:18-29] — the game has no separate live
                    // magazine register to carry: the magazine IS the clip array, and which slot the
                    // held weapon reads is `Player_AmmoInGun` 0x1a9190's pick. (The port used to
                    // round-trip gun.clip→clip[old] and clip[new]→gun.clip here and the P2K's 7 live
                    // rounds zeroed — clip[0x03] was a never-equipped slot then; see below.)
                    // fix-aims20-modes-lights: the AIMS-20 pair's ammo types DIFFER (+0x90: 5 vs 12
                    // [L ee_snapshot weapon_data]), so the twins do NOT share the slot
                    // (`Player_AmmoInGun`'s own-slot pick; bldata.md +0x2F0=[0x1A], +0x2FC=[0x1B]) —
                    // the ground truth's swap shows HUD "4-0 Grenade", the grenade slot's count, not
                    // the burst slot's. Carrying gun.clip across synced the burst magazine into
                    // clip[0x1b] (30 "grenades") and the 9mm spare into the type-12 carried — the
                    // mode press then read the twin as loaded with the wrong ammo. Flush the live
                    // cache to the old slot and re-read the new weapon's own: for same-ammo twins
                    // (P2K 0x02↔0x03) `clip_slot` resolves both to the base and the round-trip is a
                    // no-op — the magazine-continues behavior is unchanged.
                    if let Some(c) = self.drones.inventory.clip.get_mut(wi) { *c = self.gun.clip as i16; }
                    if let Some(n) = self.drones.inventory.carried.get_mut(t) { *n = self.gun.spare as i16; }
                    if self.gun.weapon >= 1 && self.gun.weapon <= 0x47 { self.gun.prev_weapon = self.gun.weapon; }
                    if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { self.gun.prev_gadget = if self.gun.weapon == 0x55 { 0x54 } else { self.gun.weapon }; }
                    self.gun.weapon = self.gun.switch_request;
                    // The FP-model rebuild latches the sight laser from the new record [M Player_SetWeaponAnim
                    // 0x1a9338:21-26 — BLData+0x964 = weapon_data[w]+0x68 & 0x8, on every completed draw].
                    self.gun.laser_on = self.weapons.has_laser_sight(self.gun.weapon as usize);
                    let (nwi, nt) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
                    if let (Some(&c), Some(&n)) = (self.drones.inventory.clip.get(nwi), self.drones.inventory.carried.get(nt)) { self.gun.clip = c.max(0) as u32; self.gun.spare = n.max(0) as u32; }
                } else {
                    // Same base but no variant script: instant swap to idle. The game's variant path
                    // touches no clip arrays [M 0x1a2cc0:18-29] — the magazine is the clip array read
                    // through `Player_AmmoInGun` 0x1a9190's slot pick, so flush the cache to the old
                    // slot and re-read the new weapon's own (fix-aims20-modes-lights — see the state-8
                    // arm above: the AIMS pair's slots differ).
                    if log_switch { eprintln!("[weapon-switch] tick {}: instant variant swap (no script) {:#x}→{:#x}", self.drones.frame, cur_id, req_id); }
                    if let Some(c) = self.drones.inventory.clip.get_mut(wi) { *c = self.gun.clip as i16; }
                    if let Some(n) = self.drones.inventory.carried.get_mut(t) { *n = self.gun.spare as i16; }
                    if self.gun.weapon >= 1 && self.gun.weapon <= 0x47 { self.gun.prev_weapon = self.gun.weapon; }
                    if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { self.gun.prev_gadget = if self.gun.weapon == 0x55 { 0x54 } else { self.gun.weapon }; }
                    self.gun.weapon = self.gun.switch_request;
                    // The FP-model rebuild latches the sight laser from the new record [M Player_SetWeaponAnim
                    // 0x1a9338:21-26 — BLData+0x964 = weapon_data[w]+0x68 & 0x8, on every completed draw].
                    self.gun.laser_on = self.weapons.has_laser_sight(self.gun.weapon as usize);
                    let (nwi, nt) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
                    if let (Some(&c), Some(&n)) = (self.drones.inventory.clip.get(nwi), self.drones.inventory.carried.get(nt)) { self.gun.clip = c.max(0) as u32; self.gun.spare = n.max(0) as u32; }
                    let nwi = self.gun.weapon as usize;
                    self.view_script = self.weapons.idle_script(nwi);
                    (self.view_time, self.view_speed) = (0.0, 1.0);
                }
            } else {
                // Different base: holster → draw. `Player_WeaponSelect` checks +0xC4 (holster) [M 0x1a2cc0 line 33].
                // fix-laser-beam: a fire-script LOOP left playing (the Laser/Stunner's op-1 loop) dies
                // here. The game's own stop points are the scripts' op-4.3 `Sound_StopAllWithId(id)`
                // [M AnimProcessScriptCmds 0x1c1b38:387-389] and the hardcoded trio 0x12/0x3d/0xf0 on
                // substate ENTRY [M Player_ChangeSubState 0x190630:83-85, Player_Init 0x1910f8:41-43];
                // the weapon-switch path itself performs NO stop (the census: AnimScriptAdd 0x1c6d40 /
                // AnimScriptDelete 0x1c14f0 / AnimObjectDelete 0x1bee80 carry no sound calls — an
                // anim-script's op-1 channels are not tracked, so in the game the loop outlives the
                // deleted script until a substate transition or an op-4.3). The port stops the ids at
                // the switch with the game's own id-keyed mechanism [M Sound_StopAllWithId
                // 0x1de0c8:10-21] because its deleted script never reaches the op-4.3 it carries.
                for id in self.gadget_loop_ids(cur_id) { self.drones.audio.stop_all_with_id(id); }
                let holster = self.weapons.holster_script(cur_id as usize);
                if holster != 0 {
                    if log_switch { eprintln!("[weapon-switch] tick {}: state 0→1 holster {:08x} (weapon {:#x})", self.drones.frame, holster, cur_id); }
                    self.view_script = holster;
                    (self.view_time, self.view_speed) = (0.0, 1.25);
                    self.gun.switch_state = 1;
                    switching = true;
                    // Sound 0x280 (weapon) or 0x283 (gadget, id > 0x4A) [M 0x1a2e38 case 1].
                    let s = if cur_id >= 0x4a && cur_id <= 0x5e { 0x280 } else { 0x283 };  // gadgets 0x280, weapons 0x283 [M disasm 0x1a3704]
                    self.drones.audio.play(s);
                } else {
                    // No holster script: swap instantly, go to draw (state 3) [M 0x1a2cc0 line 34-35].
                    if log_switch { eprintln!("[weapon-switch] tick {}: instant swap (no holster) {:#x}→{:#x}", self.drones.frame, cur_id, req_id); }
                    // fix-laser-beam: the fire-script loop dies with the weapon (see the holster arm).
                    for id in self.gadget_loop_ids(cur_id) { self.drones.audio.stop_all_with_id(id); }
                    if let Some(c) = self.drones.inventory.clip.get_mut(wi) { *c = self.gun.clip as i16; }
                    if let Some(n) = self.drones.inventory.carried.get_mut(t) { *n = self.gun.spare as i16; }
                    if self.gun.weapon >= 1 && self.gun.weapon <= 0x47 { self.gun.prev_weapon = self.gun.weapon; }
                    if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { self.gun.prev_gadget = if self.gun.weapon == 0x55 { 0x54 } else { self.gun.weapon }; }
                    self.gun.weapon = self.gun.switch_request;
                    // The FP-model rebuild latches the sight laser from the new record [M Player_SetWeaponAnim
                    // 0x1a9338:21-26 — BLData+0x964 = weapon_data[w]+0x68 & 0x8, on every completed draw].
                    self.gun.laser_on = self.weapons.has_laser_sight(self.gun.weapon as usize);
                    let (nwi, nt) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
                    if let (Some(&c), Some(&n)) = (self.drones.inventory.clip.get(nwi), self.drones.inventory.carried.get(nt)) { self.gun.clip = c.max(0) as u32; self.gun.spare = n.max(0) as u32; }
                    // fix-reload-hang2: the script lookups use the weapon's OWN record — the game reads
                    // `weapon_data[cur*0x10c + 0xc0]` for the draw [M Player_SetWeaponAnimObj 0x1a2e38
                    // case 3: iVar15 = the current weapon's own id · 0x10c] — only the clip array is
                    // routed to the base slot [M Player_ReloadAmmoType 0x1a1988:18-22]. Feeding
                    // `nwi` (the unified slot) here started the BASE's draw for twin 0x38 while the
                    // state-3 wait measured 0x38's own script — the wait never finished and the switch
                    // machine wedged (fix_weapon_sounds_live lost every later draw sound).
                    let draw = self.weapons.draw_script(self.gun.weapon as usize);
                    if draw != 0 {
                        if log_switch { eprintln!("[weapon-switch] tick {}: state 0→3 draw {:08x} (weapon {:#x})", self.drones.frame, draw, req_id); }
                        self.view_script = draw;
                        (self.view_time, self.view_speed) = (0.0, 1.25);
                        self.gun.switch_state = 3;
                        switching = true;
                        let s = if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { 0x281 } else { 0x282 };  // gadgets 0x281, weapons 0x282 [M disasm 0x1a385c]
                        self.drones.audio.play(s);
                    } else {
                        // The game's no-draw path plays the own record's idle (+0xa0) [M 0x1a2e38 case 3].
                        self.view_script = self.weapons.idle_script(self.gun.weapon as usize);
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                    }
                }
            }
        }
        if switching {
            let cur = self.gun.weapon as usize;
            let len_of = |h: u32| self.scripts.get(&h).map_or(0.0, |s| s.length as f32);
            match self.gun.switch_state {
                1 => {
                    // Holstering: wait for the holster script (+0xC4) to finish [M case 1].
                    let holster = self.weapons.holster_script(cur);
                    if holster == 0 || self.view_time >= len_of(holster).max(1.0) - 1.0 {
                        // Holster done: write the current weapon's clip/spare back, swap, read the new weapon's.
                        if let Some(c) = self.drones.inventory.clip.get_mut(wi) { *c = self.gun.clip as i16; }
                        if let Some(n) = self.drones.inventory.carried.get_mut(t) { *n = self.gun.spare as i16; }
                        if self.gun.weapon >= 1 && self.gun.weapon <= 0x47 { self.gun.prev_weapon = self.gun.weapon; }
                        if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { self.gun.prev_gadget = if self.gun.weapon == 0x55 { 0x54 } else { self.gun.weapon }; }
                        self.gun.weapon = self.gun.switch_request;
                    // The FP-model rebuild latches the sight laser from the new record [M Player_SetWeaponAnim
                    // 0x1a9338:21-26 — BLData+0x964 = weapon_data[w]+0x68 & 0x8, on every completed draw].
                    self.gun.laser_on = self.weapons.has_laser_sight(self.gun.weapon as usize);
                        let (nwi, nt) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
                        if let (Some(&c), Some(&n)) = (self.drones.inventory.clip.get(nwi), self.drones.inventory.carried.get(nt)) { self.gun.clip = c.max(0) as u32; self.gun.spare = n.max(0) as u32; }
                        // Start the draw script (+0xC0) at 1.25x [M case 2 → case 3]. The script lookup
                        // is the weapon's OWN record (0x1a2e38 case 3), not the clip slot — see the
                        // instant-swap arm above.
                        let draw = self.weapons.draw_script(self.gun.weapon as usize);
                        if draw != 0 {
                            if log_switch { eprintln!("[weapon-switch] tick {}: state 1→3 draw {:08x} (weapon {:#x})", self.drones.frame, draw, self.gun.weapon); }
                            self.view_script = draw;
                            (self.view_time, self.view_speed) = (0.0, 1.25);
                            self.gun.switch_state = 3;
                        } else {
                            if log_switch { eprintln!("[weapon-switch] tick {}: state 1→0 idle (no draw script) {:#x}", self.drones.frame, self.gun.weapon); }
                            // The own record's idle [M 0x1a2e38 case 3], not the clip slot's.
                            self.view_script = self.weapons.idle_script(self.gun.weapon as usize);
                            (self.view_time, self.view_speed) = (0.0, 1.0);
                            self.gun.switch_state = 0;
                        }
                        // Sound 0x281 (weapon) or 0x282 (gadget, id > 0x4A) [M 0x1a2e38 case 3].
                        let s = if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { 0x281 } else { 0x282 };  // gadgets 0x281, weapons 0x282 [M disasm 0x1a385c]
                        self.drones.audio.play(s);
                    }
                }
                8 => {
                    // Variant swap: wait for +0xC8 to finish [M case 8].
                    let variant = self.weapons.variant_swap_script(cur);
                    if variant == 0 || self.view_time >= len_of(variant).max(1.0) - 1.0 {
                        if log_switch { eprintln!("[weapon-switch] tick {}: state 8→0 idle (variant done) {:#x}", self.drones.frame, self.gun.weapon); }
                        self.view_script = self.weapons.idle_script(cur);
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                        self.gun.switch_state = 0;
                    }
                }
                3 => {
                    // Drawing: wait for the draw script (+0xC0) to finish [M case 3].
                    let draw = self.weapons.draw_script(self.gun.weapon as usize);
                    if draw == 0 || self.view_time >= len_of(draw).max(1.0) - 1.0 {
                        // Draw done: back to idle [M case 4]. The game zeroes the refire timer here
                        // (`*(float*)(BLData+0x92C) = 0`, case 4 0x1a2e38:316) — without this, a grenade's
                        // 900-tick refire interval poisons every weapon after switching (fix-player2).
                        self.gun.timer = 0.0;
                        if log_switch { eprintln!("[weapon-switch] tick {}: state 3→0 idle (draw done) {:#x}", self.drones.frame, self.gun.weapon); }
                        self.view_script = self.weapons.idle_script(self.gun.weapon as usize);
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                        self.gun.switch_state = 0;
                    } else if self.gun.switch_request != self.gun.weapon {
                        // fix-switch-after-cold-load [M 0x1a2e38 case 4:309-315 + LAB_001a39c8:492-497]:
                        // a request written while the draw plays REVERSES the draw anim
                        // (`AnimScriptSetPlayDirection(2)`) and drops to the wait state — when the
                        // reversed anim reaches its start, the machine swaps to the request and draws
                        // it (case 2's tail: `+0x62 = +0x63`, state 3). The retarget saves the
                        // being-drawn weapon for the toggle-back (`+0x64 = +0x62`).
                        if log_switch { eprintln!("[weapon-switch] tick {}: state 3→2 retarget {:#x}→{:#x} (reverse the draw)", self.drones.frame, self.gun.weapon, self.gun.switch_request); }
                        self.gun.saved_weapon = self.gun.weapon;
                        self.gun.switch_state = 2;
                        self.view_speed = -self.view_speed;
                    }
                }
                2 => {
                    // fix-switch-after-cold-load: the reversed draw reached its start — the retarget
                    // applies [M 0x1a2e38 case 2/0xe's tail: `+0x62 = +0x63`, state 3; case 3 starts
                    // the new draw at 1.25x]. The same clip/spare flush the holst end does: the live
                    // magazine returns to the being-drawn weapon's slot and the request's is read.
                    if self.view_time <= 0.0 {
                        if let Some(c) = self.drones.inventory.clip.get_mut(wi) { *c = self.gun.clip as i16; }
                        if let Some(n) = self.drones.inventory.carried.get_mut(t) { *n = self.gun.spare as i16; }
                        if self.gun.weapon >= 1 && self.gun.weapon <= 0x47 { self.gun.prev_weapon = self.gun.weapon; }
                        if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { self.gun.prev_gadget = if self.gun.weapon == 0x55 { 0x54 } else { self.gun.weapon }; }
                        self.gun.weapon = self.gun.switch_request;
                    // The FP-model rebuild latches the sight laser from the new record [M Player_SetWeaponAnim
                    // 0x1a9338:21-26 — BLData+0x964 = weapon_data[w]+0x68 & 0x8, on every completed draw].
                    self.gun.laser_on = self.weapons.has_laser_sight(self.gun.weapon as usize);
                        let (nwi, nt) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
                        if let (Some(&c), Some(&n)) = (self.drones.inventory.clip.get(nwi), self.drones.inventory.carried.get(nt)) { self.gun.clip = c.max(0) as u32; self.gun.spare = n.max(0) as u32; }
                        let draw = self.weapons.draw_script(self.gun.weapon as usize);
                        if draw != 0 {
                            if log_switch { eprintln!("[weapon-switch] tick {}: state 2→3 draw {:08x} (weapon {:#x})", self.drones.frame, draw, self.gun.weapon); }
                            self.view_script = draw;
                            (self.view_time, self.view_speed) = (0.0, 1.25);
                            self.gun.switch_state = 3;
                        } else {
                            if log_switch { eprintln!("[weapon-switch] tick {}: state 2→0 idle (no draw script) {:#x}", self.drones.frame, self.gun.weapon); }
                            self.view_script = self.weapons.idle_script(self.gun.weapon as usize);
                            (self.view_time, self.view_speed) = (0.0, 1.0);
                            self.gun.switch_state = 0;
                        }
                        let s = if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { 0x281 } else { 0x282 };  // gadgets 0x281, weapons 0x282 [M disasm 0x1a385c]
                        self.drones.audio.play(s);
                    }
                }
                _ => {}
            }
            // Advance the view script while switching (the retarget's reverse runs it back to 0 —
            // fix-switch-after-cold-load: floored, like the game's reversed script).
            let len = len_of(self.view_script);
            self.view_time = (self.view_time + self.view_speed).min((len - 1.0).max(0.0)).max(0.0);
        }

        let can_fire = !switching;
        if let Some(w) = self.weapons.records.get(self.gun.weapon as usize) {
            let scripts = &self.scripts;
            let mut params = nf_game::gun::GunParams::from_record(w, self.drones.inventory.fire_mode.get(self.gun.weapon as usize).copied().unwrap_or(0) as usize, |h| scripts.get(&h).map(|s| s.length as u32));
            if self.rapid_fire && params.ammo_type != 0 {
                // The PC-port rapid fire (see the field): one round per tick for as long as the
                // trigger is held, spawned on the press tick, from a clip that never empties.
                params.interval = 1.0;
                params.shots_per_pull = i32::MAX;
                params.fire_anim_len = 0;
                if params.clip > 0 {
                    let slot = self.clip_slot(self.gun.weapon as usize);
                    if let Some(c) = self.drones.inventory.clip.get_mut(slot) { *c = (*c).max(params.clip as i16); }
                    self.gun.clip = self.gun.clip.max(params.clip);
                }
            }
            // Auto-aim (on by default in the game's options) scans only outside aim mode
            // (`Player_AutoAim` 0x1a7c98 gates the `Check_AutoAim` scan on the aiming bit, which the
            // game sets only after the aim-raise anim finishes — `Player_SetWeaponAnimObj` case 0xf).
            // While aiming (0x1a7c98's aim branch): no new scan, the lock pointer SURVIVES, and the
            // assist offsets ease toward the manual offsets (the cursor) at 0.5/tick — and the fired
            // bullet still consumes them through `Player_GetAimingPoint` (0x1a4a98), so the residual
            // assist of a target locked before the raise carries into the first aim-mode shots.
            // fix-guard-misc2: the port dropped the lock and the offsets entirely on entering aim
            // mode, so an assist acquired while hip-firing never reached an aim-mode bullet.
            // fix-aim-raise2: the gated BIT (`self.aiming`, maintained by `aim_gate` before run),
            // not the raw button — the scan runs while a raise-script weapon is still raising.
            let dir = if self.auto_aim && !self.free_aim { self.drones.auto_aim(&d, camera, look, self.gun.weapon) } else if self.free_aim {
                // The cursor is screen-space (cross units); the offsets are object-frame angles
                // (a larger yaw turns LEFT, so x flips — the same flip `crosshair()` documents).
                let (ox, oy) = (-self.cursor[0] * 0.76966, self.cursor[1] * 0.57724);
                let off = self.drones.aim_offset;
                self.drones.aim_offset = [off[0] + (ox - off[0]) * 0.5, off[1] + (oy - off[1]) * 0.5];
                let (y0, p0) = (look[0].atan2(look[2]), look[1].clamp(-1.0, 1.0).asin());
                let (y2, p2) = (y0 + self.drones.aim_offset[0], p0 + self.drones.aim_offset[1]);
                [p2.cos() * y2.sin(), p2.sin(), p2.cos() * y2.cos()]
            } else {
                self.drones.aim_target = None;
                self.drones.aim_offset = self.drones.aim_offset.map(|v| v * 0.5);
                aim
            };
            let was_reloading = self.gun.reload_state;
            // (fix-weapon-anims2: the prev-script watermark moved above the switch machine — see there.)
            // The pre-shot fire-script state (`anim+0xF4 == 9`): the game SKIPS `Player_SetFiringAnim`
            // — the running fire script is NOT restarted — when the view state is already 9 and
            // `weapon_data+0x68 & 0x100400` [M 0x1a5460:664-672: `if ((*(short *)(...+0xf4) == 9) &&
            // ((*(uint *)(weapon_data + iVar4 + 0x68) & 0x100400) != 0)) { uVar8 = weapon+0x6c; } else
            // { Player_SetFiringAnim(...) }`]. Without this gate a held gadget whose fire script has a
            // release frame (the Q-Laser's +0x44 = 20) restarts the script every refire tick, the frame
            // never reaches the release, and the shot only spawns after the button comes up.
            let was_fire_state = self.gun.fire_state;
            let rd0 = |o: usize| u32::from_le_bytes(w[o..o + 4].try_into().unwrap());
            // The gun's animation is busy while a fire script plays (gun state 9; idle = 0), and the
            // reload branches additionally gate on the VIEW state being 0 [M 0x1a5460:409 — the R-key
            // branch's `if (0xf4 == 0)` — and :447-450, the auto-reload branch's `if (0xf4 != 0)
            // return`]: a reload can only START from the fully idle view, so the switch states 1/3/8
            // gate here too (`switching` = `switch_state != 0`, and the port's switch_state holds
            // the game's own +0xF4 view-state values 1/3/8 across the holster→draw cycle). Without this the
            // variant-swap tick could start a reload whose +0xA4 script overwrote the +0xC8 variant
            // script, and the state-8 wait (which reads the variant's length) never completed.
            let len_of = |h: u32| self.scripts.get(&h).map_or(0.0, |s| s.length as f32);
            let busy = switching || (self.view_script == rd0(0xb0) || self.view_script == rd0(0xb4)) && self.view_time < len_of(self.view_script) - 1.0;
            // Block the trigger and reload while the holster/draw cycle plays.
            // The hands class 'G' (0x47) cannot fire: its record carries no fire scripts (+0xB0/+0xB4 = 0
            // [D probe_ladder2]; also no idle +0xA0) and the round spawn runs from the state-9 fire-script
            // release path — `Player_SetFiringAnim` 0x1a3c70 queues the script, case 9 0x1a2e38:416-420
            // spawns on its release frame; the press-tick spawn for +0x44 == 0 sits behind that call
            // [M 0x1a5460:681-690]. Fists (0x01) keeps its fire script 0x060006de [D] — punching is
            // unaffected (fix-ladder2: the hands state fired once).
            let has_fire_script = rd0(0xb0) != 0 || rd0(0xb4) != 0;
            // The game's general fire branch has NO fire-script gate — `Player_RoundToFire`
            // (0x1a18e0:9-25) rounds to fire unconditionally for ammo_type 0 and checks only the
            // ammo pool otherwise [M]. The gate here stands in for the hands class 'G' (0x47: no
            // fire scripts and no idle anim — fix-ladder2); every other script-less weapon fires
            // and spawns on the press tick (+0x44 == 0 [M 0x1a5460:688-693]) — the camera gadget
            // 0x54 (idle 0x060005A6, no fire scripts [D probe]) photographs through this path.
            let hands = self.gun.weapon == 0x47;
            // fix-fire-modes3 — the Phoenix Samurai's charge (the base-0x33 records: the overcharge
            // 0x32 and the beam 0x33 both carry base id 0x33 [L ee_start_05.bin @ 0x2bf150]) while the
            // view anim is in the fire state (`anim+0xF4 == 9`) [M Player_WeaponFiring 0x1a5460:186-202]:
            // - the charge drain [M 0x1a5460:197-201]: `clip[0x33] (BLData+0x41C) = clip_size −
            //   clip_size·frame/falen` while the frame is under the held record's fire-anim length
            //   (+0x44). Only the OVERCHARGE 0x32 has a length (280 — the charge script 0x060007c2
            //   [L]); the beam 0x33's +0x44 is 0 so its arm never runs. The drain is what makes the
            //   ammo pane's ammo-type-31 percentage (`HUD_UpdateAmmoPane` 0x19a7f0:
            //   `(min(100−clip, 88)·100)/88`, spent %) climb 0→100 across the charge — the PCSX2
            //   pass's "the HUD number climbs 0 → 100".
            // - the charge glow [M 0x1a5460:193-196]: `Light_Create(radius 5.0, intensity
            //   2·(frame/falen)·0.75, the view model's bone 0, rgb (0,0xff,0xff) — cyan, channel 0,
            //   life 0, kind 1, flicker 0)`, brightening across the charge ("the gun's glow
            //   brightens"). The sim approximates the bone-0 origin with the camera (the muzzle
            //   light's standing deviation). The game's unconditional `Rand_Rand(0)` flicker draw
            //   (Light_Create 0x1cc3d8) lands here — before `update_with`'s spread draws — matching
            //   its position inside `Player_WeaponFiring`, ahead of the trigger read.
            if self.gun.fire_state && u16::from_le_bytes([w[2], w[3]]) == 0x33 {
                let falen = u16::from_le_bytes([w[0x44], w[0x45]]);
                let fire_h = u32::from_le_bytes(w[0xb0..0xb4].try_into().unwrap());
                // The port's script frames are time+1 (the same convention `release_frame_reached`
                // uses): the game reads AnimListBuild's +0x90 after the tick's advance.
                let frame = self.view_time + 1.0;
                if falen > 0 && self.view_script == fire_h && frame < falen as f32 {
                    let clip_size = i32::from(u16::from_le_bytes([w[0x92], w[0x93]]));
                    // The game stores a plain C float→s16 conversion (truncation) [M 0x1a5460:197-201].
                    let v = (clip_size as f32 - clip_size as f32 * frame / falen as f32) as i16;
                    if let Some(c) = self.drones.inventory.clip.get_mut(0x33) { *c = v; }
                    if matches!(self.gun.weapon, 0x32 | 0x33) { self.gun.clip = v.max(0) as u32; }
                    let pos = [camera[0], camera[1], camera[2]];
                    self.drones.lights.create(5.0, 1.5 * frame / falen as f32, pos, [0, 0xff, 0xff], 0, 0, 1, 0, &mut self.drones.rng);
                }
            }
            let (trigger, reload) = if can_fire && (has_fire_script || !hands) { (trigger, reload) } else { (false, reload) };
            // decryptor-hold-real-path: the Q-Decryptor's use gate runs BEFORE `Player_RoundToFire`
            // [M Player_WeaponFiring 0x1a5460:551-590 — on the fire-held branch the aim target
            // (`BLData+0x880`, Check_Target 0x1a7810's 50-unit crosshair ray :114-125) must be a
            // lock (type 0x3a) of record kind 0/2 in state 0/5 (:557-566) within
            // `weapon_data+0x1C` (2.0/4.0 [L ee_start_05.bin @ 0x2c4b58/0x2c4c64]); ANY failure
            // RETURNS before the pull — no burst arm, no spend, no fire anim, no bullet]. The
            // port ran the pull first and refunded the spend on a refused pick; the leftover
            // deviation was the fire script queueing on refused ticks (the game's view stays
            // idle). The pick is computed once here and consumed by the shot block below — the
            // same `camera` the game's Check_Target caches into +0x880 every 6th frame.
            let decryptor_weapon = nf_game::gadget::is_decryptor(self.gun.weapon);
            let decryptor_pick = if decryptor_weapon && trigger {
                let range = self.weapons.records.get(self.gun.weapon as usize).map_or(2.0, |r| f32::from_le_bytes(r[0x1c..0x20].try_into().unwrap()));
                let vdir = self.bond.look();
                nf_game::gadget::pick_decryptor_lock(camera, vdir, self.bond.pos, &self.drones.locks, &self.world, range)
            } else { None };
            let (trigger, reload) = if decryptor_weapon && trigger && decryptor_pick.is_none() { (false, reload) } else { (trigger, reload) };
            let fired = self.gun.update_with(trigger, reload, busy, params);
            // fix-gadgets-live: the AIMS-20's idle timer (`BLData+0x93a`): the game increments it while
            // the view anim is idle and resets on any fire/reload/switch anim [M Player_SetWeaponAnimObj
            // 0x1a2e38:41-57]; it gates the view model's side lights (0x1a5460:264/348).
            // fix-weapon-anims: the game increments +0x93a ONLY while the view state is 0 — a fire
            // script playing (state 9) resets it every tick [M `Player_SetWeaponAnimObj` 0x1a2e38:40-58:
            // "if (+0xf4 == 0 && byte[@DAT_002a3a1c + slot*0x158] == 0) { +0x93a++ } else { +0x959=0;
            // +0x93a=0; +0x93e=0x14*FRAME_RATE_INT }"]. The port counted through the fire script (only
            // the fired tick cleared it), so the AIMS lights and the idle-alert threshold saw a counter
            // that had already crossed 20 when the gun went quiet — the lights snapped on the tick a
            // fire script ended instead of 20 ticks later. fire_state covers states 9/0xb/0xc.
            // fix-weapon-anims: the aim raise/lower is the game's view state 0xF/0x10 — a non-idle
            // state for the +0x93a counter [M 0x1a2e38:40-58]; the port's anim-only raise marks it
            // with view_aim_anim so the counter resets while the scope raises/lowers (else the
            // +0xCC fidget fires mid-raise: the sniper test caught the alert playing at tick ~20
            // of an aim hold).
            // fix-aims20-modes-lights: the missed half of the gate — DAT_002a3a1c is "an analog
            // input is deflected this frame": `Input_Update` 0x17ee88:54 clears it per slot and
            // 0x17ee88:80-84 sets it when any of the pad's analog inputs reads non-zero, and
            // `Player_SetWeaponAnimObj` 0x1a2e38:41-44 RESETS +0x93a on such ticks instead of
            // counting. The port never reset the counter on input, so in play (walking, aiming)
            // it climbed past the 15/45 thresholds and the side lights went dark 0.75 s into any
            // move. The same block owns +0x93e
            // (`aims_raise`, the bone-9 light's gate [M 0x1a5460:275-281]): decremented while the
            // view is idle (0x1a2e38:47-50), refilled to 1200 on every non-idle tick (0x1a2e38:54-57;
            // [L ee_start_05 BL+0x93e = 1200]).
            let view_idle = self.gun.reload_state == 0 && self.gun.switch_state == 0
                && !self.gun.fire_state && self.view_aim_anim.is_none();
            if view_idle {
                self.drones.aims_idle = if stick {
                    0
                } else {
                    self.drones.aims_idle.saturating_add(1)
                };
                self.drones.aims_raise = self.drones.aims_raise.saturating_sub(1);
            } else {
                self.drones.aims_idle = 0;
                self.drones.aims_raise = 1200;
            }
            // Advance the reload state machine [M Player_SetWeaponAnimObj cases 5/6/7].
            let new_reload_state = self.gun.reload_tick(trigger, params);
            let rd = |o: usize| u32::from_le_bytes(w[o..o + 4].try_into().unwrap());
            let flags = rd(0x68);
            let fire_anim_len = if self.rapid_fire && params.ammo_type != 0 { 0 } else { rd(0x44) };
            if fired {
                // `Player_SetFiringAnim` (0x1a3c70), via `Gun::firing_anim_script`: the last-round pick
                // (flag 0x1000 / base 0x11), the fists' Rand(100) left/right alternation (flag 0x80000)
                // and the 0x40000 toggle. The game only queues when the fp model and the script exist
                // (0x1a3c70:51-54); with no script the view stays as it was.
                // fix-gadgets2: while the fire script is ALREADY playing (state 9) and +0x68 carries
                // 0x100400, the game leaves it running — no restart [M 0x1a5460:664-672]. fix-laser-beam:
                // that hold applies only to the RELEASE-FRAME family (+0x44 != 0 — the spawn waits for
                // the script's release frame, so a restart would starve it). The Laser/Stunner (+0x68
                // & 0x400, +0x44 = 0 [L weapon_data]) spawn in the FIRE branch itself [M 0x1a5460:681-690]
                // and every refire tick re-queues the script through AnimScriptAdd — which DELETES the
                // playing script and starts fresh [M 0x1c6d40] — keeping the beam + loop alive while held.
                let script = self.gun.firing_anim_script(flags, u16::from_le_bytes([w[2], w[3]]), self.gun.clip, rd(0xb0), rd(0xb4), &mut self.drones.rng);
                // The hold applies to the laser/stunner family too (+0x68 & 0x400, +0x44 = 0): measured in PCSX2
                // (07000005, held 1.5 s, per frame) the stunner's fire script plays 1→11 once and HOLDS at its last
                // frame while held (the laser 1→16), with the meter draining every frame — no restart. The port's
                // per-refire re-queue kept restarting the push-forward.
                if script != 0 && !(was_fire_state && flags & 0x100400 != 0) {
                    self.view_script = script;
                    // [M `Player_WeaponFiring` 0x1a5460:676-685]: a record with +0x6C & 0x400 and a +0xB4
                    // script enters view state 0xb and the just-queued fire script plays at 1.5×
                    // (`*(script + 0x98) = 0x3fc00000`) — the hand grenades 0x34..0x37 [L +0x6C 0x8c08/0x8c0a].
                    // Measured in PCSX2 (07000015, the frag): script time +1.5 per frame from the press,
                    // frame 26 (the release, +0x44) at +17 frames. At 1.0 the port released at +25 with the
                    // hand still on the pin.
                    let speed = if rd(0x6c) & 0x400 != 0 && rd(0xb4) != 0 { 1.5 } else { 1.0 };
                    (self.view_time, self.view_speed) = (0.0, speed);
                } else if script != 0 && was_fire_state && flags & 0x400 != 0 && fire_anim_len == 0 {
                    // The held laser/stunner fires EVERY tick, so this branch runs every tick and the shared
                    // advance below never would: tick the held script here (`AnimScriptTick` runs on the anim
                    // object regardless [M 0x1c2630]), parking at its last frame (`AnimScriptEnd`).
                    let len = self.scripts.get(&self.view_script).map_or(0.0, |s| s.length as f32);
                    self.view_time = (self.view_time + self.view_speed).min((len - 1.0).max(0.0));
                } else if script != 0 && was_fire_state && flags & 0x100400 != 0 && flags & 0x400 == 0 && fire_anim_len == 0 {
                    // decryptor-hold-real-path: the Q-Decryptor's held tick — the ONLY records with the
                    // state-9 suppress and NO 0x400 hold bit are 0x56/0x57 (flags68 = 0x1820a0 [L
                    // ee_start_05.bin @ weapon_data 0x2c4b58/0x2c4c64]; every other 0x100400 record
                    // carries 0x400). The game skips `Player_SetFiringAnim` [M 0x1a5460:664-672] but the
                    // anim object still ticks every frame [M AnimScriptTick 0x1c2630 — the advance is
                    // anim-pass machinery, not `Player_WeaponFiring`'s] and the case-9 stop check below
                    // re-arms the script when it ends. The port ran the advance and the stop check only on
                    // non-fired ticks — with interval 1 [L +0x40 = 1] the held decryptor froze at frame 0
                    // forever ("does not continue its animation").
                    let len = self.scripts.get(&self.view_script).map_or(0.0, |s| s.length as f32);
                    self.view_time = (self.view_time + self.view_speed).min((len - 1.0).max(0.0));
                }
            } else if self.gun.reload_state != 0 && was_reloading == 0 {
                // Reload just started [M Player_WeaponFiring's shared tail 0x1a6250-0x1a631c]: state 6
                // queues +0xA4 at 1.35× (`AnimScriptAddSpeed`, disasm 0x1a62d0), starting at frame 6 for
                // flag-0x1000 weapons (AnimListBuild +0x90 = 6.0; the port's script frames are time+1, so
                // view_time 5.0). State 5 (a +0xA8 start script) plays at 1.0 [M 0x1a6260-0x1a626c].
                if self.gun.reload_state == 5 {
                    self.view_script = rd(0xa8);
                    (self.view_time, self.view_speed) = (0.0, 1.0);
                } else {
                    self.view_script = rd(0xa4);
                    (self.view_time, self.view_speed) = (if flags & 0x1000 != 0 { 5.0 } else { 0.0 }, 1.35);
                }
            } else if self.gun.reload_loop_restarted {
                // The shell loop continued: restart +0xA4 from frame 0 (matching `AnimScriptAdd(+0xA4)` in
                // the game's case 6) [M 0x1a2e38].
                self.view_script = rd(0xa4);
                (self.view_time, self.view_speed) = (0.0, 1.0);
            } else if self.gun.reload_state != was_reloading {
                // Reload state changed: update the view script.
                match self.gun.reload_state {
                    5 => { self.view_script = rd(0xa8); (self.view_time, self.view_speed) = (0.0, 1.0); }
                    6 => { self.view_script = rd(0xa4); (self.view_time, self.view_speed) = (0.0, 1.0); }
                    7 => { self.view_script = rd(0xac); (self.view_time, self.view_speed) = (0.0, 1.0); }
                    0 => { self.view_script = rd(0xa0); (self.view_time, self.view_speed) = (0.0, 1.0); }
                    _ => {}
                }
            } else if self.view_script == 0 {
                (self.view_script, self.view_time, self.view_speed) = (rd(0xa0), 0.0, 1.0);
            } else {
                let len = self.scripts.get(&self.view_script).map_or(0.0, |s| s.length as f32);
                // fix-weapon-anims: the switch machine above owns the advance while the holster/draw/
                // variant-swap cycle runs (its shared advance at its tail); advancing here too made
                // every holster/draw/variant script play ~2x its rate — the Frinesi holster measured
                // 11-12 ticks instead of the game's ceil((30-1)/1.25) = 24 [D census len 30;
                // M 0x1a2e38 case 1 waits AnimScriptIsStopped at the script's own 1.25 rate].
                // fix-aim-raise2: the aim raise/lower is advanced (and finished) by `aim_gate` — the
                // game ticks it on the same anim object once per tick (AnimObjectUpdate 0x193600:318)
                // and the finish must land before this tick's free-aim/zoom/auto-aim consumers — so
                // the shared advance skips while the aim anim plays (one advance, not two).
                if self.gun.switch_state == 0 && self.view_aim_anim.is_none() {
                    self.view_time = (self.view_time + self.view_speed).min((len - 1.0).max(0.0));
                }
                // Gate the idle-loop reset on switch_state == 0 — fists has draw_script == idle_script
                // (both 0x6000485), so without this gate the reset fires during state 3 (drawing), zeroing
                // view_time before the state-3→0 check can succeed, leaving switch_state stuck at 3 forever.
                if self.gun.switch_state == 0 && self.view_script == rd(0xa0) && self.view_time >= len - 1.0 { self.view_time = 0.0; }
                // decryptor-hold-real-path: the fire script's stop check MOVED below the fire/reload chain
                // (it was the tail of this non-fired arm): the game's case-9 `AnimScriptIsStopped` check
                // runs every frame [M Player_SetWeaponAnimObj case 9 0x1a2e38:449-456], not only on ticks
                // without a shot — the held decryptor fires every tick (interval 1 [L weapon_data
                // 0x2c4b58 +0x40]) and its script must still end → state 9 → 0 → the idle re-queue, so the
                // next fire tick re-queues the fire script and the scan animation loops while held.
            }
            // decryptor-hold-real-path: the fire script's stop check (was the non-fired arm's tail —
            // `Player_SetWeaponAnimObj` case 9's end [M 0x1a2e38:449-456 via AnimScriptIsStopped]; the
            // +0x68 & 0x400 family runs the game's state-0xb transition instead [M 0x1a2e38 case 0xb
            // :499-507 — the fire script stopped → AnimScriptAdd(+0xB4), the RELEASE script whose
            // op-4.3 frame 1 STOPS the loop sound (0x12/0x3d [L 0x060004e8/0x06000488]), at 1.5×
            // [M 0x1a5460:677-679]; the port keeps fire_state through the +0xB4 play so its own
            // finish lands in the idle arm — the game's case 0xc → 0 does the same]).
            // The laser/stunner family holds its fire script's last frame while the trigger is held and plays
            // the release script only on release, at 1.0 [L PCSX2: release → a 27-frame script at +1.0/frame,
            // then idle]. `end_len` is the CURRENT view script's (the game reads the playing script's own
            // stop state); on a just-restarted fire script (view_time 0) the check is a natural no-op.
            let fire_last = rd(0xb4);
            let holding = flags & 0x400 != 0 && fire_last != 0 && self.view_script == rd(0xb0) && trigger;
            let end_len = self.scripts.get(&self.view_script).map_or(0.0, |s| s.length as f32);
            if self.gun.fire_state && !holding && (self.view_script == rd(0xb0) || (fire_last != 0 && self.view_script == fire_last))
                && self.view_time >= end_len - 1.0 {
                if flags & 0x400 != 0 && fire_last != 0 && self.view_script == rd(0xb0) {
                        self.view_script = fire_last;
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                    } else if rd(0x6c) & 0x400 != 0 && fire_last != 0 && self.view_script == rd(0xb0) {
                        // The grenades' state 0xb [M `Player_SetWeaponAnimObj` 0x1a2e38 case 0xb]: the fire
                        // script stopped → `AnimScriptAdd(+0xB4)` (the next grenade coming up) at its own
                        // 1.0 [L PCSX2: the script after the throw runs +1.0 per frame], then idle.
                        self.view_script = fire_last;
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                    } else if rd(0x6c) & 0x400 != 0 && fire_last != 0 && self.view_script == fire_last {
                        // State 0xc's end [M 0x1a2e38 case 0xc :521-551]: +0xB4 stopped → `Player_ReloadAmmoType`
                        // → view state 3 → the DRAW (case 3: `AnimScriptAddSpeed(1.25, draw)` + the 0x281/0x282
                        // draw sound, state 4; case 4's end zeroes the refire timer). Measured in PCSX2 on
                        // 07000015 (the frag, per frame): +0xB0 at 1.5 to +47, +0xB4 at 1.0 to +83, the draw
                        // at 1.25 (state 4) to ~+107, then idle with the clip refilled — the next throw ~+125
                        // with repeated taps. The port went straight to idle and threw again at +84.
                        self.gun.end_fire_state();
                        // remote-mine-detonate [M 0x1a2e38 case 0xc :521-523]: at the +0xB4 script's
                        // stop the game runs `Player_ReloadAmmoType(player, 1)` — the refill lands
                        // HERE, before the draw — then queues the draw (state 3). Without it the clip
                        // sat 0 until the next idle tick's auto-reload branch, and the press that
                        // came during the draw hit an empty clip and did nothing (a long delay
                        // between throws). The dry arm
                        // (Player_HandleHasNoAmmo :524-543) stays with the auto-reload path below.
                        let p = nf_game::gun::GunParams::from_record(w, self.drones.inventory.fire_mode.get(self.gun.weapon as usize).copied().unwrap_or(0) as usize, |h| scripts.get(&h).map(|s| s.length as u32));
                        self.gun.reload_one(p);
                        let draw = self.weapons.draw_script(self.gun.weapon as usize);
                        if draw != 0 {
                            self.view_script = draw;
                            (self.view_time, self.view_speed) = (0.0, 1.25);
                            self.gun.switch_state = 3;
                            self.drones.audio.play(if self.gun.weapon >= 0x4a && self.gun.weapon <= 0x5e { 0x281 } else { 0x282 });
                        } else {
                            self.view_script = rd(0xa0); (self.view_time, self.view_speed) = (0.0, 1.0);
                        }
                    } else {
                        if flags & 0x1000 == 0 { self.view_script = rd(0xa0); (self.view_time, self.view_speed) = (0.0, 1.0); }
                        self.gun.end_fire_state();
                    }
            }
            // fix-weapon-anims: the idle-alert fidget (`Player_SetWeaponAnimObj` 0x1a2e38:79-136).
                // In view state 0 with substate +0x959 == 0, once the idle counter (+0x93a) reaches
                // 0x14*FRAME_RATE_INT = 1200 idle ticks [M 0x1a2e38:83; L bldata.md +0x93E = 1200], a record
                // with an idle-alert script (+0xCC) queues it at 1.0 ONCE and sets substate 1 [M :79-136];
                // when the fidget stops the idle script re-queues, substate clears and the counter
                // resets [M :63-73] — so the fidget repeats every (1200 + alert length) idle ticks. Any
                // non-idle state resets substate + counter [M :40-58]; here that is the displaced-script
                // arm (a fire/reload/switch script replaced the fidget mid-play).
                let idle_hash = rd(0xa0);
                let alert_hash = rd(0xcc);
                if alert_hash != 0 {
                    if self.view_alert && self.view_script != alert_hash {
                        // Displaced (fire/reload/switch queued over the fidget): substate + counter
                        // reset [M 0x1a2e38:40-58].
                        self.view_alert = false;
                        self.drones.aims_idle = 0;
                    } else if self.view_alert && self.view_script == alert_hash
                        && self.view_time >= self.scripts.get(&self.view_script).map_or(0.0, |s| s.length as f32) - 1.0 {
                        // The fidget stopped: idle re-queues, substate 0, counter reset [M :63-73].
                        // (The stop check runs every tick now — the length read is the playing alert
                        // script's own, the game reads the same object's stop state [M 0x1a2e38:63-73].)
                        self.view_script = idle_hash;
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                        self.view_alert = false;
                        self.drones.aims_idle = 0;
                    } else if !self.view_alert && self.view_script == idle_hash
                        && self.gun.switch_state == 0 && self.gun.reload_state == 0 && !self.gun.fire_state
                        && self.drones.aims_idle >= 1200 {
                        // The threshold: 0x14·FRAME_RATE_INT = 1200 idle ticks [M 0x1a2e38:83
                        // `(short)FRAME_RATE_INT * 0x14 <= +0x93a`; FRAME_RATE_INT = 60 [L
                        // docs/fields/bldata.md +0x93E "= n_INT·0x14 = 1200" (the EE snapshot's own
                        // default) and +0x95E "= (char)n (60)"; door.md's n·6 = 360-tick auto-close
                        // corroborates]. fix-weapon-anims' first cut used 20 — 100x early: the fidget kept
                        // restarting every ~20 idle ticks and the idle script never reached its later
                        // frames (the Samurai's idle loop was choked before its own op-1s).
                        self.view_script = alert_hash;
                        (self.view_time, self.view_speed) = (0.0, 1.0);
                        self.view_alert = true;
                    }
                }
                // fix-weapon-anims + fix-aim-raise2: the aim raise/lower QUEUE — the single queue
                // point (`Player_Weapon` 0x1a24c8:121-152: the aim edge from the fully idle machine
                // queues the raise +0xB8 at 1.25 — the game's state 0xF, whose stop sets the aim bit
                // [M 0x1a2e38:554-561] — or the lower +0xBC at 1.25 (state 0x10, its stop leaves the
                // bit clear [M :562-567]); a record without +0xBC plays +0xB8 REVERSE via
                // AnimScriptAddReverse [M 0x1a24c8:138-152] — the port runs the raise backwards with
                // a negative script speed). The ADVANCE + finish live in `aim_gate` (hoisted before
                // this tick's free-aim/zoom/auto-aim consumers read the bit); this arm only queues
                // and marks. Another script displacing it clears the marker below [M AnimScriptAdd
                // deletes the playing scripts 0x1c6d40].
                let raise_hash = rd(0xb8);
                let lower_hash = rd(0xbc);
                if self.view_aim_anim.is_none() && raise_hash != 0 && self.view_script == idle_hash
                    && self.gun.switch_state == 0 && self.gun.reload_state == 0 && !self.gun.fire_state {
                    if aim_on_edge {
                        self.view_script = raise_hash;
                        (self.view_time, self.view_speed) = (0.0, 1.25);
                        self.view_aim_anim = Some(raise_hash);
                        self.view_aim_raising = true;
                    } else if aim_off_edge {
                        if lower_hash != 0 {
                            self.view_script = lower_hash;
                            (self.view_time, self.view_speed) = (0.0, 1.25);
                            self.view_aim_anim = Some(lower_hash);
                        } else {
                            // AnimScriptAddReverse: the raise script played backwards [M 0x1a24c8:138-152].
                            // The start frame is the RAISE script's last frame — `len` here is the
                            // idle's (view_script at the queue moment), which made the reverse
                            // finish instantly (fix_aim_raise's release test caught it).
                            let rlen = self.scripts.get(&raise_hash).map_or(0.0, |s| s.length as f32);
                            self.view_script = raise_hash;
                            (self.view_time, self.view_speed) = ((rlen - 1.0).max(0.0), -1.25);
                            self.view_aim_anim = Some(raise_hash);
                        }
                        self.view_aim_raising = false;
                    }
                }
            // fix-aim-raise: another script displacing the aim raise/lower (a fire/reload/switch
            // queued over it) clears the marker — the raise never finished, so the aim bit stays
            // clear until the next press [M AnimScriptAdd 0x1c6d40 deletes the playing script].
            if let Some(cur) = self.view_aim_anim {
                if self.view_script != cur { self.view_aim_anim = None; }
            }
            // The release frame: the round spawns when the fire script's frame reaches `fire_anim_len` (+0x44)
            // and the +0x95C latch is clear [M Player_SetWeaponAnimObj case 9 0x1a2e38:416-420]; weapons with
            // `fire_anim_len == 0` spawn on the press tick (latched there) [M 0x1a5460:688-693]. Before this,
            // the port spawned every round on the press tick — a grenade pull fired a hitscan bullet instead
            // of winding up and throwing, and the 900-tick refire interval stayed armed afterwards.
            // fix-frag-throw-timing: the release-family weapons (`+0x6C & 0x400` with a `+0xB4` script — the
            // hand grenades) run the view machine through 0xb/0xc [M 0x1a5460:676-685], and case 9's check
            // never fires for them — the state leaves 9 inside the fire tick itself. Their release check is
            // case 0xc's [M 0x1a2e38:505-519, disasm 0x1a3268-0x1a32c4], reached only after the fire script
            // ENDED (case 0xb's `AnimScriptIsStopped` gate → the +0xB4 script queued) — so the playhead it
            // compares against `+0x44` is the +0xB4 script's, which starts at 1.0 [M AnimScriptInit
            // 0x1c1600:37] and advances 1.0/tick [M AnimScriptTick 0x1c2630 mode 0]. The port ran case 9's
            // check during the fire script itself: the frag (fire 70 frames at 1.5×, +0x44 = 26) released at
            // press+17, mid-wind-up, before the animation finished. The gate below (the view is on the +0xB4 script) is the case-0xb → case-0xc phase.
            let release_family = rd(0x6c) & 0x400 != 0 && rd(0xb4) != 0;
            let release = if fire_anim_len == 0 { fired }
                else if release_family { self.view_script == rd(0xb4) && self.gun.release_frame_reached(self.view_time, fire_anim_len) }
                else { self.gun.release_frame_reached(self.view_time, fire_anim_len) };
            if release && !fired { self.gun.latch = true; }
            // The gun scripts' sound commands (`AnimProcessScriptCmds` op 1: `Sound_Play3D(cmd[2], …)` when the
            // script's frame crosses cmd[1]): the shot (0x403 at frame 1 of the P2K's fire scripts) and the reload's
            // three sounds (0x40F, 0x40E, 0x411 at frames 0x2E, 0x59, 0x74). Frame = script time + 1.
            // Per-weapon op-1 id remap (docs/systems/sound.md §7): the script's id is replaced based on the weapon.
            if self.view_script != 0 {
                let from = if self.view_script != prev_script || (fired && self.view_time == 0.0) { f32::NEG_INFINITY } else { prev_time + 1.0 };
                let to = self.view_time + 1.0;
                let wid = self.gun.weapon;
                if let Some(sc) = self.scripts.get(&self.view_script) {
                    for c in sc.commands.iter().filter(|c| c.op == 1 && c.args.len() >= 2) {
                        let f = c.args[0] as f32;
                        if from < f && f <= to {
                            let id = remap_op1_sound(c.args[1] as u32, wid);
                            // fix-laser-beam: a LOOPED op-1 re-trigger keeps the single voice (the held
                            // laser/stunner restarts its fire script every refire tick [M AnimScriptAdd
                            // 0x1c6d40; L weapon_data +0x40 = 1], re-firing the op-1 60×/s).
                            let looped = self.sfx.as_ref().map(|t| t.get(id)).is_some_and(|e| e.looped);
                            if looped { self.drones.audio.play_loop(id, camera); } else { self.drones.audio.play3d(id, camera, None); }
                            // The same `Sound_Play3D` is what the guards hear [M AnimProcessScriptCmds 0x1c1b38 op 1 →
                            // Sound_Play3D 0x1dd548; Sound_Alertness 0x1ddc48 skips looped sounds]: the gunshot alerts
                            // exactly when, and as loudly as, the fire script's own sound. A fire script with no op-1
                            // makes no noise — the held Stunner's 0x0600048a has none [D], and the old hardcoded
                            // "every other weapon is a P2K shot" table fed a 0x403 gunshot (weight 65) every tick of
                            // the hold: the tased guard maxed his alertness, reacted in 0 ticks and shot Bond before
                            // the tase could drop him.
                            if let Some(e) = self.sfx.as_ref().map(|t| t.get(id)) { self.drones.hear(camera, e); }
                            if std::env::var_os("NF_SOUNDLOG").is_some() { eprintln!("tick {}: gun script {:08x} frame {f}: sound {:#x} (remapped from {:#x} for weapon {:#x}, looped {looped})", self.drones.frame, self.view_script, id, c.args[1], wid); }
                        }
                    }
                    // fix-gadgets2 (the sounds kept playing after the stop): `AnimProcessScriptCmds`
                    // op 4 sub-op 3 is `Sound_StopAllWithId(cmd[3])` [M 0x1c1b38:388] — the fire scripts stop
                    // their own loop sounds at a scripted frame. The port played the op-1 starts and dropped
                    // the op-4.3 stops, so every gadget loop (the watch-laser burn, the decryptor decode) leaked
                    // forever — worse while the old restart bug re-fired the script every tick. Same frame
                    // window as op 1; sub-op is args[1], the id args[2] (the port's parse: [frame, sub-op, id]).
                    for c in sc.commands.iter().filter(|c| c.op == 4 && c.args.len() >= 3 && c.args[1] == 3) {
                        let f = c.args[0] as f32;
                        if from < f && f <= to {
                            self.drones.audio.stop_all_with_id(u32::from(c.args[2]));
                            if std::env::var_os("NF_SOUNDLOG").is_some() { eprintln!("tick {}: gun script {:08x} frame {f}: stop sound {:#x}", self.drones.frame, self.view_script, c.args[2]); }
                        }
                    }
                }
            }
            if release {
                'shot: {
                // The Q-Decryptor's use tick [M Player_WeaponFiring 0x1a5460:551-578]: the gate runs
                // BEFORE `Player_RoundToFire` — the aim target (the 50-unit crosshair ray's first
                // hit, `Check_Target` 0x1a7810) must be a lock (type 0x3a) of record kind 0/2 in
                // state 0/5, within the weapon's own `weapon_data+0x1C` (2.0/4.0 [L]); else the
                // function returns and the decryptor does NOTHING. On
                // the pass the shot runs (the scan bullet, class 5 [M Bullet_init 0x12c5a0:87]) and
                // `Lock_Update`'s contact scan decodes (docs/systems/decryptor.md). The port's pick
                // below, then one +1/+3 scan tick, IS the game's per-press count: the scan bullet
                // flies speed 33.333335 u/tick [M record+0x20, ee_start_05 @ 0x2c4b58/0x2c4c64] with
                // range 2.0/4.0 [M record+0x1C] and dies when range < travelled [M Bullet_update
                // 0x12e95c-0x12e984] — exactly ONE moving tick, hence exactly ONE Lock_Update
                // contact tick per press, and the FIRST contact only flips state 0 → 5 (nothing
                // banks [M 0x1ae3d8:163-190]). The spend `update_with` already
                // made is refunded on a FAILED gate (the game never reaches the spend; the refund is
                // the stunner's miss-arm shape [M 0x1a4bf0:193-199]). No bullet spawns: the record
                // carries no beam bits (+0x6C = 0 [L]) and the lock side consumes the pulse.
                if nf_game::gadget::is_decryptor(self.gun.weapon) {
                    let upgraded = self.gun.weapon == 0x57;
                    // decryptor-hold-real-path: the pick was computed BEFORE the pull (the game's
                    // order — the gate at 0x1a5460:551-586 returns before `Player_RoundToFire`
                    // :590, so a fired tick implies a passed gate); the same `camera` and view
                    // dir feed both, so the stashed pick IS this tick's gate result. `fired` on
                    // this tick implies the gate passed — a None here is unreachable; the refund
                    // arm stays for robustness.
                    match decryptor_pick {
                        Some(i) => {
                            // Record only — the contact decodes two ticks from now (the game's
                            // collide-after-update order, the top-of-tick pass above).
                            self.decryptor_scanned = Some((i, upgraded));
                            self.decryptor_target = Some(i);
                        }
                        None => {
                            let clip_size = self.weapons.clip_size(self.gun.weapon as usize).max(1) as u32;
                            let spend = u32::from(self.weapons.records.get(self.gun.weapon as usize).map_or(1u8, |r| r[0x91]).max(1));
                            self.gun.clip = (self.gun.clip + spend).min(clip_size);
                        }
                    }
                    break 'shot;
                }
                // `Player_WeaponInitBullet` 0x1a4bf0's FIRST branch [M disasm 0x1a4c30-0x1a4cd4]: a record
                // whose +0x68 carries 0x20000 — the Remote Mine's detonator 0x38 (+0x68 = 0x00020010
                // [L ee_start_05.bin]; the mine 0x37 = 0x00014090 lacks the bit; 0x39 is the inert
                // sibling) — never shoots: it walks the objects and marks EVERY player-owned bullet
                // of this base (the walk has no break — all placed mines go at once) with the
                // detonate fuse `BU+0xFC = −10.0` [M disasm 0x1a4c54-0x1a4c88: `lui $at, 0xc120;
                // mtc1 $at, $f0; …; swc1 $f0, 0xfc($a1)` — 0xc1200000 = −10.0f; the earlier
                // "+10.0" note dropped the sign]; `Bullet_DoTrails` 0x12d890:268-285 counts it down
                // and blasts at < 0 — the NEXT tick — with `Explode_Create(record+0x08, record+0x08,
                // record+0x0C, …)` — radius 8 / damage 100 [L record 0x37]. The port's fuse model is
                // `range − travelled` = BU+0xFC, so the mark writes `range = travelled − 10.0`.
                // Then `Player_WeaponHasAmmo(base)` arms the switch back to the mine (BL+0x63 =
                // base, `sb v0,99(v1)` [M 0x1a4cb8]) — or, dry, clears the base's owned byte (+0x1BA
                // lane [M bldata.md], `sb zero,442(v0)` [M 0x1a4cd0]) and lets `Player_GetBestWeapon`
                // 0x1a9680 write the request [M 0x1a4ccc]. No bullet, no muzzle latch (+0x95b never
                // sets in this arm), no casing.
                if flags & 0x20000 != 0 {
                    let base = self.weapons.base(self.gun.weapon as usize);
                    for g in self.drones.grenades.iter_mut() {
                        // The game's walk matches type-5 objects whose weapon record's +0x02 equals
                        // the fired record's +0x02 and whose BD+0x30 is the firing player [M
                        // 0x1a4c5c-0x1a4c88]; the port's grenade list is exactly those bullets, the
                        // owner usize::MAX is the player. In-flight, stuck or resting — the walk has
                        // no state filter (every match takes the −10.0 store).
                        if g.state < 2 && self.weapons.base(g.weapon_type as usize) == base && g.owner == usize::MAX {
                            g.range = g.travelled - 10.0; // the absolute BU+0xFC = −10.0 [M 0x1a4c88]
                        }
                    }
                    let base_u8 = base as u8;
                    let carried = self.drones.inventory.carried.clone();
                    let clip = self.drones.inventory.clip.clone();
                    let weapons = &self.weapons;
                    let has = nf_game::gun::Gun::has_ammo(base_u8, &carried, &clip, |t| weapons.ammo_type(t as usize), |t| weapons.flags(t as usize),
                        |x| nf_game::gun::Gun::ammo_in_gun_slot(x, |y| weapons.base(y as usize) as u8, |y| weapons.ammo_type(y as usize) as u8));
                    // mp-combat-weapons: the detonator press rides the session input too (every
                    // machine re-arms this slot's shared mines).
                    self.net_shot = Some(NetShot { weapon: self.gun.weapon, dir, scoped: false });
                    if has {
                        self.gun.switch_request = base_u8; // BL+0x63 = the mine [M 0x1a4cb8]
                    } else {
                        // Dry: the mine's clip and owned byte go, and GetBestWeapon takes the hand [M 0x1a4cbc-0x1a4cd4].
                        if let Some(c) = self.drones.inventory.clip.get_mut(base_u8 as usize) { *c = 0; }
                        if let Some(o) = self.drones.inventory.owned.get_mut(base_u8 as usize) { *o = false; }
                        const BEST_WEAPON: [u16; 18] = [0x42, 0x32, 0x1a, 0x18, 0x1c, 0x16, 0x15, 0x08, 0x10, 0x0e, 0x0d, 0x0c, 0x12, 0x06, 0x04, 0x02, 0x0a, 0x01];
                        let owned = self.drones.inventory.owned.clone();
                        let base_of = |b: u8| weapons.base(b as usize) as u8;
                        let has_ammo_fn = |id: u8| nf_game::gun::Gun::has_ammo(id, &carried, &clip, |x| weapons.ammo_type(x as usize), |x| weapons.flags(x as usize),
                            |x| nf_game::gun::Gun::ammo_in_gun_slot(x, |y| weapons.base(y as usize) as u8, |y| weapons.ammo_type(y as usize) as u8));
                        if let Some(id) = self.gun.get_best_weapon(&BEST_WEAPON, base_of, &owned, has_ammo_fn) {
                            self.gun.switch_request = id; // Player_GetBestWeapon writes BL+0x63 [M 0x1a9680 tail]
                        }
                    }
                    break 'shot;
                }
                // fix-gadgets-live: the laser/stunner's shot-tick state, computed HERE where the
                // refire that spent the round is known (the game's order: Player_RoundToFire 0x1a18e0
                // spends, then Player_WeaponInitBullet 0x1a4bf0 runs on the same tick; its +0x95b
                // latch makes the next tick's Player_CollisionHandler 0x1949a8 call Player_MuzzleFlash
                // 0x1a4298, which re-casts the muzzle→aim ray and draws the beam [M 0x1a4298:107-137]).
                // Gating on `release` is also the CLIP gate: at clip 0 `update_with` never fires and
                // the beam stops (PCSX2 capture). The old top-of-tick block burned and beamed on
                // the raw trigger — including through the draw/switch cycle and with an empty clip.
                if nf_game::gadget::runs_beam(self.gun.weapon) {
                    let is_stunner = matches!(self.gun.weapon, 0x4a | 0x4c);
                    // The stunner's target test [M 0x1a4bf0:169-200]: a Collide_RayIntersect from the
                    // HEAD along the aiming direction; the hit is a live guard when the hit object's
                    // class byte is 2 or 3, and `Vec_Dist3D(head, hit+0x40) < weapon_data+0x1c`
                    // (5/10 [L]) sets +0x95d — Draw_TaserBeam's gate [M 0x1a4298:134]. The ray meets
                    // the guard's own 0x800 walk hittest (a capsule segment; NDrone2_Collision
                    // 0x140e90:104-121), so the port tests ray-vs-capsule over the live drone list:
                    // the drones' body capsule (the Toucher::drone shape — pos−1.0 → head(), r 0.4)
                    // and the head→contact distance inside the weapon's range [M the 0x1a4bf0 gate].
                    let mut target_hit = true;
                    if is_stunner {
                        target_hit = false;
                        let range = self.weapons.records.get(self.gun.weapon as usize).map_or(5.0, |r| f32::from_le_bytes(r[0x1c..0x20].try_into().unwrap()));
                        // The exact shape (a world ray against the guard's 0x800 capsule, the hit
                        // point's head-distance gated) needs the live hit-point measurement — the
                        // capsule port broke fix_stunner_press_live's pinned press (both the ray
                        // length and the 0.4 radius miss where the game and the pinned test hit);
                        // the pinned sphere shape stays until the live check lands (report).
                        let end = [camera[0] + dir[0] * range, camera[1] + dir[1] * range, camera[2] + dir[2] * range];
                        for dr in self.drones.list.iter() {
                            if !dr.present || !dr.enabled || dr.corpse || dr.health <= 0.0 { continue; }
                            let c = [dr.pos[0], dr.pos[1], dr.pos[2]];
                            let ab = [end[0] - camera[0], end[1] - camera[1], end[2] - camera[2]];
                            let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
                            let ap = [c[0] - camera[0], c[1] - camera[1], c[2] - camera[2]];
                            let t = if ab2 > 1e-9 { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 } else { 0.0 };
                            let cl = [camera[0] + ab[0] * t.clamp(0.0, 1.0), camera[1] + ab[1] * t.clamp(0.0, 1.0), camera[2] + ab[2] * t.clamp(0.0, 1.0)];
                            let d2 = (cl[0] - c[0]).powi(2) + (cl[1] - c[1]).powi(2) + (cl[2] - c[2]).powi(2);
                            if d2 <= 1.05 * 1.05 { target_hit = true; break; }
                        }
                    }
                    self.stunner_target = Some(target_hit);
                    if is_stunner && !target_hit {
                        // The game's miss arm [M 0x1a4bf0:193-199]: `clip = min(clip + weapon+0x91,
                        // clip_size)` and return — with RoundToFire's spend the tick is FREE (the
                        // stunner's recharge while aimed off-target). No bullet, no beam.
                        let clip_size = self.weapons.clip_size(self.gun.weapon as usize).max(1) as u32;
                        self.gun.clip = (self.gun.clip + u32::from(self.weapons.records.get(self.gun.weapon as usize).map_or(1u8, |r| r[0x91]).max(1))).min(clip_size);
                    }
                    if target_hit {
                        // The beam's endpoints only — the burn is the bullet pass's contact chain
                        // [M 0x1ae3d8:57-101 via 0x1eae80:92-108; the fire below spawns the real
                        // round through fire_player, whose per-tick segment picks the lock].
                        let (a, b) = nf_game::gadget::laser_tick(camera, dir, &self.world);
                        self.laser_beam = Some((a, b));
                    }
                }
                // The stunner spawns NO bullet without a target [M Player_WeaponInitBullet
                // 0x1a4bf0:187-200 — the 0x4A/0x4C ray test's miss arm returns before Bullet_init;
                // the target test above ran on this same shot tick].
                let spawn_shot = !matches!(self.gun.weapon, 0x4a | 0x4c) || self.stunner_target == Some(true);
                // Grenade-family weapons: `Bullet_init` with the weapon record [M 0x12c5a0] — the throw
                // origin the game uses is the midpoint of the head and the view model's bone 0
                // (`flags68 & 0x10000` arm, `Player_WeaponInitBullet` 0x1a4bf0:230); the port has no view
                // bones and spawns from the eye (*deviation*). Direction: the aim ray with `Bullet_init`'s
                // three spread draws, in the game's RNG order [M 0x12c5a0:166-184]. The spawn itself is
                // silent (`Bullet_init`'s param_7 = 0xffff); the throw's own sound is the fire script's
                // op-1 command (played above).
                // delta-repeater: the gate is the GRAVITY bit AND a grenade-family collision flag — the
                // BOUNCE 0x80 (Bullet_CollisionHandler 0x12ce80's LAB_0012d4bc gate), the STICK 0x100
                // (the stick arm 0x12d1a0..0x12d1fc) or the CONTACT-EXPLODE 0x4 (the explode arm
                // 0x12d068). Gravity alone is NOT the grenade family: `Bullet_update` 0x12e7b0's case 1
                // applies `+0x6C & 8` gravity to ANY bullet, and the Delta Repeater 0x11 (+0x6C = 0x808,
                // +0x70 = 0x2 [L ee_start_05.bin @ 0x2bf150 + 0x11·0x10C]) flies its lobbed arc as a
                // plain bullet — its wall hit is the standard impact effect (flags 0x61) and a DEAD
                // bullet (state 2, no bounce/stick/ricochet bit set) [M 0012ce80]. The old gravity-only
                // gate made the port's bolt a bouncing grenade that bounced off walls.
                // Record sweep [L ee_start_05.bin]: this gate moves exactly record 0x11 off the grenade
                // path; every other gravity record (0x2a/0x2b/0x34–0x37/0x3a–0x3c/0x45) carries 0x4,
                // 0x80 or 0x100 and keeps today's routing.
                let flags6c = u32::from_le_bytes(w[0x6c..0x70].try_into().unwrap());
                let flags70_w = u32::from_le_bytes(w[0x70..0x74].try_into().unwrap());
                if flags6c & 0x8 != 0 && flags70_w & (0x4 | 0x80 | 0x100) != 0 {
                    let f32_at = |o: usize| f32::from_le_bytes(w[o..o + 4].try_into().unwrap());
                    let sdir = nf_game::bullet::spread(dir, f32_at(0x24), flags, &mut self.drones.rng);
                    let speed = f32_at(0x20);
                    // The fuse is the bullet's BU+0xFC: 210 ticks, 300 when weapon +0x70 & 0x10000, the
                    // mine 0x37 → 420 (7777.0 for a player owner — trigger-detonated; the port has no
                    // detonator and keeps 420) — hardcoded in `Bullet_init` (0x12c5a0:103,196-223), NOT
                    // weapon_data+0x1C (0x34's +0x1C is 5000.0 [L ee_start_07000001.bin, audit
                    // fix-player2 claim 8]). The port's `Grenade.range` doubles as the tick fuse.
                    let wt = u16::from_le_bytes([w[2], w[3]]);
                    let flags70 = u32::from_le_bytes(w[0x70..0x74].try_into().unwrap());
                    // fix-weapon-fire-modes [M Bullet_DoTrails 0x12d890:262-281]: only `+0x70 & 0x200`
                    // records tick a fuse (BU+0xFC −= 1 per tick, exploded at < 0) — 210 ticks, 300
                    // with +0x70 & 0x10000, the mine 0x37 → 420. The impact records 0x2A/0x1B carry
                    // NO fuse bit: they range out SILENTLY at +0x1C units [M Bullet_update
                    // 0x12e7b0:158-162 state 4] — the port models that as the tick-equivalent
                    // (+0x1C / speed), and the Sim's Explode arm (the +0x200 gate) removes them
                    // quietly; their first contact blasts through the Sim's Bounce arm.
                    let fuse = if flags70 & 0x200 == 0 {
                        (f32_at(0x1c) / speed).max(1.0)
                    } else if flags70 & 0x10000 != 0 { 300.0 } else { 210.0 };
                    // fix-satchel-tripbomb: `Bullet_init` 0x12c5a0:199-208's family arm overwrites the
                    // defaults — base (weapdef+2) 0x3b → `(id == 0x6c) ? 900 : ((id − base)·5 + 5)·60`
                    // + BU+0x102 = 100 [M]; base 0x69 → 5.0 [:212-214]. [L dump of the 115 records:
                    // every satchel bases to ITSELF and only 0x3b is selectable → the fired 0x3b gets
                    // 300; the tripbomb 0x3a bases to itself → the default 210 (its arm delay comes
                    // from the stick's own `BU+0xFC = 120.0` overwrite [M 0x12ce80:330]).]
                    let base = u16::from_le_bytes([w[2], w[3]]);
                    let wid = u16::from_le_bytes([w[0], w[1]]);
                    let fuse = if base == 0x3b {
                        if wid == 0x6c { 900.0 } else { ((wid - base) as f32 * 5.0 + 5.0) * 60.0 }
                    } else if base == 0x69 { 5.0 } else { fuse };
                    // The mine 0x37 thrown by the PLAYER: BU+0xFC = 7777.0 — it waits for the
                    // detonator and never self-detonates [M Bullet_init 0x12c5a0:196-223 — the
                    // owner obj+0xff == 3 arm; guard-thrown mines keep 420]. The detonator's fire
                    // re-arms the placed mines to 10.0 [M Player_WeaponInitBullet 0x1a4bf0].
                    let fuse = if wt == 0x37 { 7777.0 } else { fuse };
                    // fix-grenade-types: the 0x69 smoke canister's fuse is 5.0 [M Bullet_init
                    // 0x12c5a0:212-215] — it pops almost at once into its long 3600-tick cloud
                    // (the Sim's smoke arm re-arms by type).
                    let fuse = if wt == 0x69 { 5.0 } else { fuse };
                    // mp-combat-weapons: the throw rides the session input (the shared combat
                    // throws the same record from the slot's head on every machine).
                    self.net_shot = Some(NetShot { weapon: self.gun.weapon, dir, scoped: self.aiming && flags & 0x40 != 0 });
                    // The blast damage: `weapon_data+0x0C` [M 0x12cd98 disasm: `lwc1 $f14, 12($v0)` →
                    // Explode_Create's f14 → EXPLODEINFO+0x28 (Explode_CollisionHandler 0x17cde0:13-15);
                    // radius = $f12/$f13 = weapon_data+0x08]. The `addiu $t2, $zero, 0x64` (100) is the
                    // 10th argument, an u8 `Explode_Create` never reads (its prologue — 0x17cbd0 — reads
                    // only a0-a3, f12-f14, t3 and the stack u16). fix-explosions2: the disc labels name
                    // the records [L USATxt.dat] — the frag is 0x34 'Fragmentation Grenade' (radius 8,
                    // damage 100: the real kill through Explode_Propagate), 0x35 'Stun Grenade' (10/0,
                    // the flash branch), 0x36 'Smoke Grenade' (0/0, the occlude/re-arm branch). The old
                    // hardcode's sin was killing with EVERY type and on impact; the per-type records
                    // route each grenade to its own death branch [M 0x12dd60/0x12e104/0x12e1c0].
                    // The launch origin [M `Player_WeaponInitBullet` 0x1a4bf0:212-240]: the head, moved to the
                    // MIDPOINT of the head and the view model's DATUM 0 (`AnimGetBoneWorldTrans(BL+0x7e8, 0, 0)`) when
                    // the record has a first-person model (+0xDC), +0x68 & 0x10000, isn't an excluded base (1, 0x52,
                    // 0x54, 0x58, 0x59, 0x5b) and isn't a scoped weapon held zoomed. The view model's bone 0 is its
                    // object origin (the root's local translation is 0 [D probe, 0x34 frame 26]) = `gun_matrix`,
                    // which sits ~0.43 BEHIND the eye: the grenade starts behind the camera and comes into view
                    // only once it has flown past — the game shows no grenade on the release frame (PCSX2 frozen at
                    // the fire script's frame 26), where the eye origin drew a screen-filling frag.
                    let flags68 = u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap());
                    let fp_model = u32::from_le_bytes(w[0xdc..0xe0].try_into().unwrap()) != 0;
                    let excluded = matches!(base, 1 | 0x52 | 0x54 | 0x58 | 0x59 | 0x5b);
                    let zoomed_scope = self.aiming && flags68 & 0x40 != 0;
                    let origin = if fp_model && !excluded && !zoomed_scope && flags68 & 0x10000 != 0 {
                        // DATUM 0 [M 0x1c3120 — a non-negative id is a datum, not a bone]: the hand's mount.
                        let g = self.gun_matrix();
                        let d0 = self.view_datum(0).unwrap_or([g[3][0], g[3][1], g[3][2]]);
                        [(camera[0] + d0[0]) * 0.5, (camera[1] + d0[1]) * 0.5, (camera[2] + d0[2]) * 0.5]
                    } else { camera };
                    self.drones.grenades.push(nf_game::grenade::Grenade {
                        pos: origin,
                        velocity: [sdir[0] * speed, sdir[1] * speed, sdir[2] * speed],
                        speed, travelled: 0.0, range: fuse, damage: f32_at(0x0c), radius: f32_at(0x08),
                        // The FIRED weapon's id, not the record's +0x02 base: the MGL's 'Delay' twin
                        // 0x2b bases to 0x2a, and the death branches (the impact explode [M
                        // 0x12d068-0x12d0bc], the flash/smoke routing [M 0x12dd60]) read the BULLET'S own
                        // record. The grenade family bases to itself [L ee_start_05.bin +0x02 sweep], so
                        // this only re-points the twins.
                        weapon_type: self.gun.weapon as u16,
                        mesh: self.weapons.projectile_mesh(self.gun.weapon as usize),
                        owner: usize::MAX, state: 0, bounces: 0, smoke_popped: false, resting: false,
                        // fix-oddjob-hat: the stick latch + the homing target (BU+0x40) start clear.
                        stuck: false, seek_target: None,
                        // fix-satchel-tripbomb: the gadget state box exists from the spawn for the
                        // satchel/tripbomb ids — `Bullet_DoTrails`' 0x3b tail (the beeps/digits) runs
                        // in EVERY state [M 0x12d890:418+], so the quarter-second latch (BU+0x100)
                        // must tick from the throw; the stick overwrites it with the surface state.
                        phase: matches!(self.gun.weapon as u16, 0x3a | 0x3b).then(|| {
                            Box::new(nf_game::grenade::StuckGadget { normal: [0.0, 1.0, 0.0], arm: 0.0, armed: false, last_qs: 0, digits: None })
                        }),
                    });
                } else {
                    // The muzzle light runs on every shot tick for the light-gated records — the
                    // stunner's no-target ticks included (its miss arm still latches +0x95b, so
                    // Player_MuzzleFlash runs and Light_Create fires; only Draw_TaserBeam is +0x95d-
                    // gated [M 0x1a4298:107-137]). fix-gadgets-live moved this out of the spawn_shot
                    // gate so the stunner's aim-off-target hold shows its blue arc light.
                    // `Player_MuzzleFlash` 0x1a4298: `Light_Create(radius = weapon_data+0x58,
                    // intensity 2.0, the muzzle — the view model's DATUM 0, or datum 4 for the
                    // +0x68 & 0x40000 dual records while BL+0x958 is clear [M 0x1a4298:66-89, the
                    // same alternation the casing eject reads; the `view_datum` fallback below is a
                    // guard for an unposed model, not the game's rule]), rgb = +0x56/+0x55/+0x54,
                    // channel 0, life 0 → +0x20 = 1 (one
                    // tick), kind 1)` — its `Rand_Rand(0)` draw sits between the accuracy draws and
                    // the bullet's spread draws, as in the game. The smoke: `Gas_Init`'s
                    // `Rand_FRand(2π)` draw, unless weapon_data+0x68 & 0x2000 (no smoke).
                    // The game's gate [M 0x1a42e8]: weapon_data+0xDC != 0 && weapon_data+0x48 != 0.
                    let light_gate = u32::from_le_bytes(w[0xdc..0xe0].try_into().unwrap()) != 0
                        && u32::from_le_bytes(w[0x48..0x4c].try_into().unwrap()) != 0;
                    if light_gate {
                        let radius = f32::from_le_bytes([w[0x58], w[0x59], w[0x5a], w[0x5b]]);
                        // muzzle-light-datum [M Player_MuzzleFlash 0x1a4298:66-89]: the light sits at the view model's
                        // DATUM 0 (the muzzle), or datum 4 for the +0x68 & 0x40000 dual records while BL+0x958 is
                        // clear — the same alternation the casing eject reads. The old eye + aim·0.5 approximation sat
                        // above and behind the gun and lit its camera-facing side; at the muzzle the light is AHEAD of
                        // the gun and the PS2 view model stays dark with a lit edge (render vs PCSX2, 07000008 launch).
                        let dual = u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap()) & 0x40000 != 0 && self.gun.alt_fire_toggle == 0;
                        let pos = self.view_datum(if dual { 4 } else { 0 }).unwrap_or([camera[0] + dir[0] * 0.5, camera[1] + dir[1] * 0.5, camera[2] + dir[2] * 0.5]);
                        self.drones.lights.create(radius, 2.0, pos, [w[0x56], w[0x55], w[0x54]], 0, 0, 1, 0, &mut self.drones.rng);
                        let flags68 = u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap());
                        if flags68 & 0x2000 == 0 {
                            // `Gas_Init`'s `Rand_FRand(2π)` draw (the smoke puff's angle), in the game's
                            // order — every MuzzleFlash tick, the stunner's no-target ones included
                            // (the draw keeps the shared RNG's draw count bit-exact).
                            let _angle = self.drones.rng.frand(6.2831854820251465);
                            // The puff itself: models-misc's gas system (`Gas_Init(0.05, 0.1, …, 0x2000039, 7,
                            // 0x48403080, end 1)` [M 0x1a4298]); Gas_Init's own draws stay un-consumed (G-033).
                            // rocket-trail-gas: the decomp-dropped 11th arg (the life) is 0x80 = 128 ticks
                            // [M 0x1a43f8 `li $v0, 0x80` → 0x1a442c `sw $v0, 0($sp)`] — the old value 8 was a guess —
                            // and the packed END rgba is 1 = bytes (0,0,0,1), so the alpha rate is
                            // (1−0x80)/128 [M 0x1a4440 delay slot `li $t3, 1`].
                            self.drones.gas.init_ex(nf_game::gas::SMOKE_QUAD, pos, 0.0, [0.0; 3], 0.05, 0.1, 7, [0x48, 0x40, 0x30, 0x80], 1, 128.0);
                        }
                    }
                    if spawn_shot {
                        self.last_shot = Some(self.drones.frame);
                        self.shot_tick = true;
                        // mp-combat-weapons: the round's description for the session input.
                        self.net_shot = Some(NetShot { weapon: self.gun.weapon, dir, scoped: self.aiming && flags & 0x40 != 0 });
                        // The hearing sound: the P2K's 0x403 by default, or the remapped op-1 id for other weapons.
                        // The drones' hearing list uses the sfx table's entry for this id.
                        // fix-melee: the melee swings are NOT gunshots — the fists' fire scripts' op-1 is
                        // 0x142 (the whoosh, frames 7/9 of 0x060006de/0x060006dd [D 07000005.bin dump]),
                        // whose SFXOutputData hearing entry weighs 0 (inner 5, outer 30, weight 0.0 [L]) —
                        // a punch alerts nobody until the victim screams or sees Bond. The taser-CC twins
                        // 0x4b/0x4d swing on 0x0600048d, which carries no op-1 at all [D] — equally silent.
                        // The old `_` arm fed both the 0x403 gunshot (weight 65, outer 40 [L]): every
                        // punch alerted the room like a P2K shot.
                        // The shot's hearing rides the fire script's op-1 `Sound_Play3D` above (the view-script
                        // sound loop), not the spawn: a silent placeholder here.
                        let sound = nf_formats::sfx::SfxHearing { id: 0, inner: 0.0, outer: 0.0, weight: 0.0, looped: true, cull: false };
                        // fix-sg5-casings [M Bullet_init_casing_ex 0x12f678:160]: the game ejects the
                        // shell at the view-model gun bone's world position (`AnimGetBoneWorldTrans`
                        // VECTOR out) — hand the posed gun matrix (`Player_PositionGun` 0x1a3b40's
                        // port) to the next fire_player so the casing spawns at the gun, not the eye.
                        // The player arm [M 0x12f678:88-99]: `AnimGetBoneWorldTrans(BL+0x7e8, 1, 0, pos, MATRIX)` —
                        // DATUM 1 (a non-negative id is a datum [M 0x1c3120]); the 0x40000 alternating records with
                        // `BL+0x958 == 0` use datum 5 (the other gun). The view model's origin (`gun_matrix`) sits
                        // ~0.43 behind the eye: shells ejected there spawned at the camera, huge on screen but
                        // unreadable.
                        let alt = u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap()) & 0x40000 != 0 && self.gun.alt_fire_toggle == 0;
                        self.drones.eject_gun = Some(self.view_datum_matrix(if alt { 5 } else { 1 }).unwrap_or_else(|| self.gun_matrix()));
                        // rocket-origin: the bullet's start is the head, moved to the MIDPOINT of the head and the
                        // view model's datum 0 under the same gate the grenade launch uses (a first-person model,
                        // +0x68 & 0x10000, not an excluded base, not a zoomed scope), and the direction is
                        // re-aimed at the aiming point, head + aim·256 [M Player_WeaponInitBullet 0x1a4bf0:226-245
                        // — `Vec_Normalise(aim − start)`; Player_GetAimingPoint 0x1a4a98 — the −256 scale on the
                        // (x, y, −1) view ray]. The port fired every bullet from the eye: a rocket's own mesh
                        // (0x2d's +0x5C projectile) spawned 0.6 units in front of the camera and filled the
                        // screen on the launch frame (render vs PCSX2 07000008, where it leaves the launcher).
                        let (shot_from, shot_dir) = {
                            let flags68 = u32::from_le_bytes(w[0x68..0x6c].try_into().unwrap());
                            let fp_model = u32::from_le_bytes(w[0xdc..0xe0].try_into().unwrap()) != 0;
                            let rbase = u16::from_le_bytes([w[2], w[3]]);
                            let excluded = matches!(rbase, 1 | 0x52 | 0x54 | 0x58 | 0x59 | 0x5b);
                            let zoomed_scope = self.aiming && flags68 & 0x40 != 0;
                            match (fp_model && !excluded && !zoomed_scope && flags68 & 0x10000 != 0).then(|| self.view_datum(0)).flatten() {
                                Some(d0) => {
                                    let o = [(camera[0] + d0[0]) * 0.5, (camera[1] + d0[1]) * 0.5, (camera[2] + d0[2]) * 0.5];
                                    let aim_pt = [camera[0] + dir[0] * 256.0, camera[1] + dir[1] * 256.0, camera[2] + dir[2] * 256.0];
                                    let v = [aim_pt[0] - o[0], aim_pt[1] - o[1], aim_pt[2] - o[2]];
                                    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
                                    (o, [v[0] / l, v[1] / l, v[2] / l])
                                }
                                None => (camera, dir),
                            }
                        };
                        self.drones.fire_player(&d, shot_from, shot_dir, self.gun.weapon, sound);
                        // fix-weapon-fire-modes [M Bullet_init 0x12c5a0:248-253]: a `+0x6C & 4` shot hands
                        // the view to the projectile — `Player_ChangeSubState(owner, 10)`, `BLData+0x878 =
                        // the bullet`, `Player_SetCamMode(BLData, 0xc)`, `BLData+0x944 = 0`.
                        if flags6c & 0x4 != 0 && self.net_rng_shared.is_none() {
                            if let Some(b) = self.drones.player_bullets.last() {
                                self.missile_cam = Some(MissileCam { pos: b.pos, dir: b.dir, restore: 0, riding: true });
                            }
                        }
                    }
                }
                }
            }
        }
        // Write the gun's clip/spare back to the inventory. Recompute wi/t from the CURRENT weapon (not the
        // stale ones from line 410) — the switch state machine may have swapped the weapon mid-tick, and the
        // swap paths already wrote the OLD weapon's values back. Using stale wi/t here corrupts the old
        // weapon's ammo with the new weapon's values [bug fix C-10]. The slot is the shared base clip for
        // twins [M 0x1a1988:18-22, `clip_slot`].
        let (cur_wi, cur_t) = (self.clip_slot(self.gun.weapon as usize), self.weapons.ammo_type(self.gun.weapon as usize));
        if let Some(c) = self.drones.inventory.clip.get_mut(cur_wi) { *c = self.gun.clip as i16; }
        if let Some(n) = self.drones.inventory.carried.get_mut(cur_t) { *n = self.gun.spare as i16; }
        // fix-aim-raise2: the aim BIT is `aim_gate`'s (set before run from the raw button; the
        // param is the ungated input the view chain's queue arm reads its edges from) — run must
        // not clobber it with the raw button.
        self.drones.bond_aiming = self.aiming;
        // The class byte the AI's `ObjectIsArmed` 0x178768 reads: Bond obj+0xdc+0x62. `Player_WeaponSelect`
        // 0x1a2cc0 copies the request (+0x63) into it at the switch's START (the `weapon_data[+0x64].+0xc4
        // == 0` arm — +0x64 is 0x47 from InitWeapon 0x1a7ef8:137-139), so mid-holster the AI already reads
        // the TARGET weapon: at the party Bond reads as the fists (unarmed, class 1) from the first tick,
        // not as the carried P2K for the ~100 switch ticks (the entrance bouncers shot).
        self.drones.bond_weapon = if self.gun.switch_state != 0 && self.gun.switch_request != 0x47 {
            self.gun.switch_request
        } else {
            self.gun.weapon
        };
        // The use key (`Player_Activate` 0x1a5170): the doors its sphere — one unit ahead of the head — meets
        // open, swing or thud locked (docs/systems/world-objects.md "Doors"); the player object updates before
        // the control list's doors, so they see the activation this same tick.
        // door-react-path4: the game calls `Player_Activate` from `Player_WeaponFiring` behind the action
        // bits' PRESS EDGE (`Input_Action(player, 9, 1)` | `Input_Action(player, 0xe, 4)` [M 0x1a5460:395-403];
        // the action machine 0x17ee88 latches per press) — a HELD use key activates the world once per press,
        // not every tick. The port fired the block raw every held tick: a 1-s hold re-toggled the 06
        // guardhouse swing door 60× (activate's open/opening arm [M 0x135be0:121-131] turns OPENING back to
        // CLOSING on the next tick's call) and the door was CLOSED again before the interact assert. The rising
        // edge is the game's whole rule for the use key: `Player_WeaponFiring` 0x1a5460:396 reads only the
        // fresh-press bit (mask 4) of action 0xE [M], and Input_Update 0x17ee88's repeat machinery (the arm at
        // held-frame 46 — asm 0x17f0d4 `sltiu $v0, $v0, 0x2e`; bit 0x8 added every 8th frame thereafter, asm
        // 0x17f0e0-0x17f0f0) has NO consumer — no caller ever masks 0x8, so there is nothing to port. The MP
        // row path (net_use_pass) keeps its own per-slot semantics.
        let use_edge = use_key && !self.use_key_prev;
        self.use_key_prev = use_key;
        let mut crept = false;
        if use_edge {
            // `Player_Activate` 0x1a5170:66-97's creepwall arm comes FIRST: with the third-icon
            // context 6 (BLData+0x95f, the game's ONLY gate [M 0x1a5170:66 `if (BL+0x95f == 6)`])
            // the use key scans every object for class '+' (0x2B) and calls
            // `Player_CollCreepWall` on the NEAREST within the squared distance 1000 (no capsule
            // test) [M] — returning before the head-ray pick ever runs. No proximity fallback: at
            // the 07000005 gate door the 0x1D LOAD_LEVEL trigger sits ~24 units from the second
            // vent's creepwall (squared 579 < 1000), and the game reads +0x95F = 0xFF there
            // [L PCSX2 2026-10-13] — the scan alone must never creep.
            if self.bond.substate == nf_game::player::SS_WALK {
                let mut cbest: Option<(f32, usize)> = None;
                for (i, w) in self.creepwalls.iter().enumerate() {
                    let d = (w.pos[0] - self.bond.pos[0]).powi(2) + (w.pos[1] - self.bond.pos[1]).powi(2) + (w.pos[2] - self.bond.pos[2]).powi(2);
                    if d >= 1000.0 { continue; }
                    if cbest.map_or(true, |(bd, _)| d < bd) { cbest = Some((d, i)); }
                }
                if self.bond.icon_ctx == 6 {
                    if let Some((_, i)) = cbest {
                        let cctx = nf_game::player::LadderCtx { creepwalls: &self.creepwalls, ..Default::default() };
                        self.bond.start_creep(i, &self.world, &cctx);
                        crept = self.bond.substate == nf_game::player::SS_CREEP;
                    }
                }
            }
        }
        if use_edge && !crept {
            // use-pick-head: EVERY arm of `Player_Activate` picks from Bond's HEAD — the game
            // builds the use point at `Player_GetHeadPos` + the viewer direction [M 0x1a5170:100-108,
            // 0x195aa0] — never from the render camera. (run()'s `camera` param is step_bond's
            // `self.bond.head()` binding, so this was already the head at runtime; passing the head
            // EXPLICITLY removes the misnamed-arg trap that read as a camera pick — the free-cam
            // harness `step()` binds the real camera, and its use_key is hard false, so the E path
            // can never regress to it.) The arms: doors + the deferred trigger half + monitors take
            // `head`; the SP/switch/PCQWORM arm takes `sp_head` (garage-lift-calls); the SS/gunimp
            // pick below re-derives the same head. The last argument is Bond's body position
            // (obj+0xC0) — `SP_Activate` 0x1dc338:51-53's facing-dot input (use-pick-sp-standins).
            self.drones.player_use_doors(&d, self.bond.head(), look, self.drones.frame, self.bond.head(), self.bond.pos);
            // `Player_Activate` 0x1a5170's ONE pick for every class [M :100-108]: a sphere of radius
            // exactly 1.0 (asm 0x1a5278 `lui $at, 0x3f80`) centred at head + view dir (`Mat_GetDir`
            // of the slot viewer's matrix, asm 0x1a5250), through Collide_SphereIntersect 0x1ec940
            // → Collide_Pick 0x1eb320's 0x101 candidate test `dist²(centre, obj+0x80) <
            // (1.0 + obj+0x8C)²` [M Collide_PickObj 0x1eae80:219-228]; the class walk then activates
            // EVERY hit, nearest first, no break [M 0x1a5170:113-179]. The port's world sphere is
            // the placement object's gfx sphere (o.centre/o.radius — the Control_BuildWorldSph
            // source, the same data Doors::create loads); the game's glist refine
            // (Intersect_PointGeom 0x1e92d8) is not ported — a mesh veto of a sphere hit is left
            // to the live checks (the report lists them).
            // use-pick-head: the SS/gunimp pick's origin is the SAME head [M 0x1a5170:100-108] —
            // one pick serves every class, so no arm may keep its own origin.
            let head = self.bond.head();
            let pick_c = [head[0] + look[0], head[1] + look[1], head[2] + look[2]];
            let mut ss_hits: Vec<(f32, usize)> = Vec::new();
            for (i, s) in self.drones.sss.list.iter().enumerate() {
                if s.state == 0 || s.mask & 4 == 0 { continue; }
                let p = s.pos();
                let d2 = (p[0] - pick_c[0]).powi(2) + (p[1] - pick_c[1]).powi(2) + (p[2] - pick_c[2]).powi(2);
                let r = 1.0 + s.radius;
                if d2 <= r * r { ss_hits.push((d2.sqrt(), i)); }
            }
            ss_hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            // The same creepwall arm at the second activation site: icon context 6 ONLY
            // [M Player_Activate 0x1a5170:66 `if (BL+0x95f == 6)`; L the 0xFF live pin at the
            // gate door] — the scan's own proximity never creeps.
            let mut crept = false;
            if self.bond.substate == nf_game::player::SS_WALK {
                let mut best: Option<(f32, usize)> = None;
                for (i, w) in self.creepwalls.iter().enumerate() {
                    let d = (w.pos[0] - self.bond.pos[0]).powi(2) + (w.pos[1] - self.bond.pos[1]).powi(2) + (w.pos[2] - self.bond.pos[2]).powi(2);
                    if d >= 1000.0 { continue; }
                    if best.map_or(true, |(bd, _)| d < bd) { best = Some((d, i)); }
                }
                if self.bond.icon_ctx == 6 {
                    if let Some((_, i)) = best {
                        let cctx = nf_game::player::LadderCtx { creepwalls: &self.creepwalls, ..Default::default() };
                        self.bond.start_creep(i, &self.world, &cctx);
                        crept = self.bond.substate == nf_game::player::SS_CREEP;
                    }
                }
            }
            if !crept {
            for (_, i) in ss_hits.iter().copied() {
                let frame = self.drones.frame;
                let st = self.drones.sss.list[i].state;
                let mk = self.drones.sss.list[i].mask;
                let _ = (st, mk);
                let mut ch = std::mem::take(&mut self.drones.channels);
                let hit = self.drones.sss.list[i].activate(&mut ch, frame);
                self.drones.channels = ch;
                if hit { eprintln!("[ss] SS #{i} (placement {}) activated by the use key", self.drones.sss.list[i].placement); }
            }
            // `Player_Activate` 0x1a5170's class walk also reaches class 0x47 — the mounted guns
            // (`GunImp_Create` 0x131f20, `GunImp_Activate` 0x132f00). The SAME one pick serves
            // every class [M 0x1a5170:100-179 — one Collide_SphereIntersect, one hit-list walk]:
            // the sphere at head+look vs the gun's world sphere (1.0 + obj+0x8C, the gfx-sphere
            // radius the props load now carries).
            let mut gbest: Option<(f32, usize)> = None;
            for (i, g) in self.drones.props.gunimps.iter().enumerate() {
                if g.active { continue; }
                let r = 1.0 + g.radius;
                let d2 = (g.pos[0] - pick_c[0]).powi(2) + (g.pos[1] - pick_c[1]).powi(2) + (g.pos[2] - pick_c[2]).powi(2);
                if d2 <= r * r {
                    let dd = d2.sqrt();
                    if gbest.map_or(true, |(bd, _)| dd < bd) { gbest = Some((dd, i)); }
                }
            }
            if let Some((_, i)) = gbest {
                let placement = self.drones.props.gunimps[i].placement;
                self.drones.gunimp_activate(i);
                // `GunImp_Activate` runs `Player_WeaponNone` [M 0x132f00]: the held weapon is saved
                // and Bond goes to hands at once — no first-person gun while mounted.
                if self.gun.weapon != 0x47 { self.gun.go_to_hands(); }
                eprintln!("[gunimp] #{i} (placement {placement}) mounted — GunImp_Activate 0x132f00");
            }
            }
        }
        // `Check_Target` (0x1a7810, every 6th frame — `GameState+0x52 % 6`): Bond's 50-unit view ray (mask
        // 0xC, world only); the first drone under it decides the crosshair state (BLData+0x133): 2 normal,
        // 3 = a non-enemy target — in SP the test is the target's `BLData+0x44 != 1` [M 0x1a7810:109-139,
        // 181-182]. `HUD_UpdateCrossHair` (0x19e6d0) renders state 3 as the red "no" mark (HUDCrossCoords
        // state 3 [L 0x2bec48]). The game also marks the target (+0x2A = 1, +0x2C4 = the clock) for the
        // civilian AI — the port's aimed_at marking in `Drones::tick` covers that arm.
        if self.drones.frame % 6 == 0 {
            let far = [camera[0] + look[0] * 50.0, camera[1] + look[1] * 50.0, camera[2] + look[2] * 50.0];
            let wall = self.world.segment(camera, far).unwrap_or(1.0);
            let hit = self.drones.list.iter().enumerate()
                .filter(|(_, dr)| dr.present && dr.enabled && dr.drawn && !dr.corpse)
                .filter_map(|(i, dr)| {
                    let vol = nf_game::bullet::PlayerVolume { feet: [dr.pos[0], dr.pos[1] - 1.0, dr.pos[2]], height: 1.8, radius: 0.4 };
                    nf_game::bullet::segment_cylinder(camera, far, &vol).filter(|&t| t < wall).map(|t| (i, t))
                }).min_by(|a, b| a.1.total_cmp(&b.1));
            self.cross_state = match hit {
                Some((i, _)) if self.drones.list[i].team != 1 => {
                    // civilian-no-shoot [M 0x1a7810:152-183]: the no-shoot mark is ALSO keyed on the
                    // held weapon id (animData+0x62) and the level. Weapon ids {1, 0x35, 0x36, 0x43,
                    // 0x44} — the fists and the throw family (grenades 0x35/0x36; 0x43/0x44 the other
                    // throwables) — keep the NORMAL crosshair over the same civilian (you may punch or
                    // blast him; the damage path has no team gate either), and levels 0x7000005..0x7000008
                    // (`0x7000008 < level || level < 0x7000005` guards the mark) never mark at all.
                    // Everything else marks +0x132 = 1, +0x133 = 3.
                    let w = self.gun.weapon;
                    let level_exempt = (0x0700_0005..=0x0700_0008).contains(&self.level);
                    if level_exempt || matches!(w, 1 | 0x35 | 0x36 | 0x43 | 0x44) { 2 } else { 3 }
                }
                _ => 2,
            };
            // door-text-labels: the SAME ray's class-0x1C door arm [M 0x1a79e8-0x1a7bc8]. The game's ONE
            // hitlist decides: the nearest hit object takes the hit, so a drone in front of the door
            // (its 0.4 cylinder, the ported drone pick above) leaves the door silent. Gates: hit dist
            // < 5.0 [M 0x1a7b6c-0x1a7b84], level != 0x700004a [M 0x1a7b58-0x1a7b64], the boot-global
            // `DoorMsg` 0x30cd04 < 3 [M 0x1a7b88-0x1a7b90], then `Door_DoMsg` 0x135f18 → on a nonzero
            // label `DoorMsg++` and `Text_AddMsg(BL+0x94e, 0, 1, label, 0, 0xb4)` — 180 ticks [M
            // 0x1a7b98-0x1a7bc8]. The port: the door's own mesh-ray pick (Doors::pick_target —
            // the game's front-face rule [M Intersect_RayGeom 0x10a8c8:168],
            // docs/systems/door-aim-ray.md) and the
            // info_labels queue bound in hud_tick (the game binds at post time, before the pane).
            // unported-misc-5: the trigger pick for the class-0x1D arm below — the game's single hitlist
            // dispatches on the NEAREST hit object only, so the two aim arms must yield to each other too
            // [M 0x1a79e8-0x1a7bc8: the 0x1c/0x1d branches are else-ifs on one `bVar3`].
            let trig_pick = self.pick_trigger_target(camera, look, wall);
            let door_pick = self.drones.doors.pick_target(&self.world, camera, look, wall);
            let trig_in_front_of_door = trig_pick.map_or(false, |(_, t)| door_pick.is_none_or(|(_, dt)| t < dt));
            if let Some((di, dt)) = door_pick {
                let drone_in_front = hit.map_or(false, |(_, t)| t < dt);
                if !drone_in_front && !trig_in_front_of_door && dt * 50.0 < 5.0 && self.level != 0x0700_004a
                    && self.hud.door_msg < 3 {
                    // MPSettings._388_4_ (DECIMAL 388 = 0x184 — the decompile's _NNN_ notation; the
                    // symbol is 476 bytes, so hex 0x388 does not exist) — the SP/MP discriminator:
                    // copied from +0x180 (dec 384) every map load by MP_Start 0x183a50:70, driven by
                    // ResetMap_GameInit 0x1d5080:7 ← GameState._84_4_ (set 1 by the MP-band level
                    // pick 0x1d3800:22-26, 0 by the SP mission pick 0x20b8d8:26); ~100 readers gate
                    // on it (e.g. Door_DoMsg 0x135f18:13, Check_Target 0x1a7810:103). The port's
                    // `mp.is_some()` is that flag's model (an Option present iff the match is MP).
                    let mp = self.mp.is_some();
                    if let Some(label) = self.drones.doors.do_msg(di, mp, &self.drones.channels) {
                        self.hud.door_msg += 1;
                        self.hud.info_labels.push_back((label, 180.0));
                    }
                }
            }
            // unported-misc-5: the SAME ray's class-0x1D trigger arm [M 0x1a7810:193-208]. The hit trigger's
            // `Trigger_Type` (0x1b2fa8 — the discrete's ext+0x38; the port's `Trigger.kind`) must be 10 — a
            // LOAD_LEVEL exit. Gates: hit dist < 5.0 [M 0x1a7810:194], `LoadMsg` 0x30cd08 == 0 (the aim-hint
            // cooldown, decremented on EVERY 6th-frame invocation at 0x1a7810:78-79 and re-armed to
            // `FRAME_RATE_INT * 0x14` = 1200 (invocations, ≈2 minutes at one per 6 frames) here whether or not the hint shows [M 0x1a7810:195-196]), and the
            // level != 0x7000009 — 09's exit panels carry their own hint path [M 0x1a7810:195]. On all gates
            // passing and `!Trigger_Locked` (0x1b2fc8 — for a LOAD_LEVEL exit the key gate off), the game posts
            // `Txt_BindLabel(0x2000009, 0)` for 0xb4 = 180 ticks [M 0x1a7810:201-207]. The port: the AABB pick
            // below [S stand-in — the game's ray hits the trigger's collision via the shared hitlist; the
            // port's `world.segment` returns no object id (the door arm's sphere note is resolved —
            // `Doors::pick_target` now tests each door's own triangles [M 0x10a8c8:168]).]
            self.hud.load_msg = self.hud.load_msg.saturating_sub(1); // [M 0x1a7810:78-79]
            if let Some((ti, tt)) = trig_pick {
                let drone_in_front = hit.map_or(false, |(_, t)| t < tt);
                let door_in_front = door_pick.map_or(false, |(_, dt)| dt < tt);
                let t = &self.drones.triggers[ti];
                if !drone_in_front && !door_in_front && tt * 50.0 < 5.0
                    && t.kind == nf_game::triggers::kind::LOAD_LEVEL
                    && self.level != 0x0700_0009 && self.hud.load_msg == 0 {
                    self.hud.load_msg = 60 * 0x14; // FRAME_RATE_INT * 0x14 = 1200 invocations [M 0x1a7810:196]
                    if !t.locked(&self.drones.channels) {
                        self.hud.info_labels.push_back((0x200_0009, 180.0));
                    }
                }
            }
        }
        // The object handler (`control_movement_object_handler`) runs whenever `GS_IsPaused(-1) == 0` — ScriptCam
        // plays NO role [M Game_Run 0x1c9790:30-35]: `ScriptCam` only picks `Text_Update2Line` vs `Text_Update`
        // (line 17) and the paused branch's `Light_Update`. The port therefore does NOT gate the drone tick on
        // `Sim::disabled`. A cut-scene freezes the guards only when its own script fires event 5
        // (`Drone_EnableAll(0)` → `NPCGlobals = 0` [M 0x1d94c0 case 5, 0x1383e8]), undone by `Script_Free`'s
        // 0x80 flag (event 7) [M 0x1da880:41-43] — ported as `Drones::npc_enabled` (core-loop §7 items 1-2).
        // (The B-10 symptom was the opposite gating bug: freezing the whole tick stopped Trigger_Calc +
        // SP_Update, so PlayerEnable never arrived.)
        // The reserved player shells eject from the view model's datum as posed NOW (`Bullet_init_casing_ex`
        // 0x12f678:88-99 — datum 1, or 5 for the 0x40000 alternating records with `BL+0x958 == 0`).
        self.drones.eject_bone_now = if self.drones.casings.list.iter().any(|c| c.pending && c.state == 3) {
            let w: &[u8] = self.weapons.records.get(self.gun.weapon as usize).map(|r| &r[..]).unwrap_or(&[]);
            let alt = w.get(0x68..0x6c).is_some_and(|b| u32::from_le_bytes(b.try_into().unwrap()) & 0x40000 != 0) && self.gun.alt_fire_toggle == 0;
            Some((self.view_datum_matrix(if alt { 5 } else { 1 }).unwrap_or_else(|| self.gun_matrix()), self.bond.pos[1]))
        } else { None };
        // mp-host-match-logic: the live match's death + respawn half. `mp_death_was_active`
        // brackets the death's start tick — `MP_PlayerKilled` 0x188f70 fires from
        // `Player_CheckForDeath` 0x190810:91 exactly when the health runs out, then the 5 s gate
        // respawns (`Player_HandleDeath` 0x190bb0:41-54 → `MP_ReSpawn` 0x185368).
        // mp-site-damage-offline: the offline local pipeline's site feed — the sphere only on an
        // offline load. The game's rule is ONE feed per site: bullets accumulate in the site
        // object's hit lists (obj+0xD0) and are drained once per frame by
        // MP_DemolitionProtectionUpdate 0x1870a0:81-91 (SP_GetHitDamage 0x1dc5f8), blasts apply
        // instantly through Explode_Propagate's class-0x35 arm 0x17c518:186-188 ->
        // MP_ApplyDamage 0x18b2c8:11-15 (its ONLY caller) — there is no second path in the game
        // [M]. The port's online machines run that one feed in the shared combat; the local sim
        // replaying the same shots would double-apply them, so offline-only is the gate that
        // KEEPS the game's one-feed rule (port architecture, the same predicate
        // `step_mp_death_respawn` runs under).
        self.drones.site_sphere = if self.net_rng.is_none() && self.net_rng_shared.is_none() {
            self.mp.as_ref().and_then(|m| m.site_sphere())
        } else {
            None
        };
        let mp_death_was_active = self.drones.death.active;
        self.drones.tick(&d, &player);
        // mp-site-damage-offline: the local pipeline's site contacts land the game's way — the
        // frame's blasts first (the case-0x35 direct `MP_ApplyDamage` per blast [M 0x17c518:187]),
        // then the frame's bullet sum as ONE store (the game's `SP_GetHitDamage` chain read
        // [M 0x1870a0:81-91]; the first contact's hitter names the last-hitter slot — the port's
        // local rounds are all Bond's). The truncate-per-store order matches the game's frame;
        // `damage_site`'s own gates (scenario, the done flag) stand.
        {
            let blasts = std::mem::take(&mut self.drones.site_blast_hits);
            let bullets = std::mem::replace(&mut self.drones.site_bullet_damage, 0.0);
            if let Some(m) = self.mp.as_mut() {
                let local = self.mp_local_slot;
                for (dmg, owner) in blasts {
                    // The owner key: `usize::MAX` = Bond's throw → the local slot; a drone-index
                    // key is UNREACHABLE here: the site exists only in an MP match (site_sphere
                    // is `self.mp`'s), the port's MP matches create no drones (bots fire through
                    // the botweap Bullet path, never `drones.grenades`), and the only non-MAX
                    // blast owners are SP guards' throws [M callers: drone.throw_grenade via
                    // 15483; grenade.rs's player/placed spawns are usize::MAX]. The GAME's
                    // attribution, for the record: MP_ApplyDamage(falloff, mpobj, attacker obj)
                    // [M 0x17c518:186-188 -> 0x18b2c8:11-15] resolves the attacker through
                    // Control_Plr2Ind 0x134118 — class 2/0x11 (bots ARE drone-class) reads
                    // DCVars+0x75C, class 3/0x12 reads BLData+0x94E, and an unregistered owner
                    // gets −1 = no attribution [D 0x134178-0x1341d0] — the skip arm matches
                    // that −1 case.
                    if owner == usize::MAX { m.damage_site(dmg, local); }
                }
                if bullets > 0.0 { m.damage_site(bullets, local); }
            }
        }
        // death-blood-pane: `Player_CheckForDeath` runs `Player_WeaponNone` 0x1a9858 on the death
        // tick [M 0x190810:61] — the held id becomes 'G' (0x47) at once, no holster cycle — so the
        // first-person gun is gone for the whole sequence (PCSX2: no gun from the 40th dead frame on).
        // MP re-equips at the respawn (`mp_equip_local`).
        if !mp_death_was_active && self.drones.death.active && self.gun.weapon != 0x47 { self.gun.go_to_hands(); }
        // Online, death + respawn belong to the per-slot MP combat (every machine, shared stream).
        if self.net_rng.is_none() && self.net_rng_shared.is_none() { self.step_mp_death_respawn(mp_death_was_active); }
        // fix-script-objects: the script-created AI volumes (`Drone_AIVolume_Create` 0x1376c0 via
        // `Script_CreateEntity` 0x1d8968's 0xF5 arm) join the level's volume list the tick their
        // script creates them (the game's `LList_Add` 0x2706c8 registration), and their matrix
        // refreshes from the live entity every tick (the game keys obj+0x90 per tick [M
        // Script_Interp 0x1d9a08:106-133]). The 07000005 create-time z-negate/+1.5-y lift is NOT
        // applied here: the script arm's object matrix is overwritten by the same tick's interp
        // [M PROVEN — the lift never reaches the box in the game either]: Script_CreateEntity's
        // 0xF5 arm creates the volume with the global zero vector 0x2d6200 for pos AND rot
        // [0x1d8968:132-135], Drone_AIVolume_Create's z-negate/+1.5-y lift lands on that
        // create-time matrix only [0x1376c0:60-64 -> control_create_object 0x1344e0:27
        // View_RotTransMatrix], and Script_EntityStart calls Script_Interp immediately after the
        // create (disasm 0x1d8e38 create, 0x1d8e84 store obj, 0x1d8eb8 jal Script_Interp — the
        // later Control_BuildWorldSph/build_LinkToRoom/Mat_GetNorm consumers read the POST-interp
        // matrix 0x1d8ec0-0x1d8f08); Script_Interp's case 1 rewrites obj+0x90 unconditionally on
        // every stream entry [0x1d9a08:105-111 Mat_Copy/Mat_Scale3f into obj+0x90]. The port's
        // no-lift here IS the game's behaviour.
        if !self.drones.scripts.objects.pending_volumes.is_empty() {
            for v in std::mem::take(&mut self.drones.scripts.objects.pending_volumes) {
                let half = self.glist_bounds.get(&v.hash)
                    .map(|(lo, hi)| [0, 1, 2].map(|k| ((hi[k] - lo[k]) * 0.5).abs()))
                    .unwrap_or([0.0; 3]);
                let kind = match v.params.get(1).copied().unwrap_or(0) { 1 => 2, 2 => 1, 4 => 3, _ => 0 };
                let factor = nf_formats::ps2float::mul(v.params.first().copied().unwrap_or(0) as f32, 0.01);
                self.volumes.push(nf_game::aivolume::AiVolume { m: v.matrix, half, factor, kind, placement: usize::MAX, src: Some((v.script, v.stream)) });
            }
        }
        if self.volumes.iter().any(|v| v.src.is_some()) {
            let mut upd: Vec<(usize, nf_formats::anim::Mat4)> = Vec::new();
            for (i, v) in self.volumes.iter().enumerate() {
                if let Some(src) = v.src {
                    if let Some(m) = self.drones.scripts.entity_matrix(src) { upd.push((i, m)); }
                }
            }
            for (i, m) in upd { self.volumes[i].m = m; }
        }
        // `PlarStat_LogUpdateElapsedTime(0)` 0x1995a0 — `Player_Update` 0x193600:132 calls it once per
        // tick; `now` is the port's frame-derived 1/100 s clock (deterministic; fix-progress).
        self.drones.progress.log_update_elapsed(nf_game::progress::PlrMissionStats::now_100ths(self.drones.frame));
        // door-text-labels: `Door_Update`'s unlock message (`Text_AddMsg(0, 0, 1,
        // Txt_BindLabel(0x2000003, 0), 0, 0xb4)` [M 0x1363c8:101-103]) posts straight into the
        // type-1 INFO list in the game; here `Doors::update` queued it and Sim applies the level
        // gate the game checks inline (`GameState != 0x7000014 && != 0x700004a` [M :101]) — on
        // those two levels the door still unlocks, silently. The activate/clear/busy steps ran in
        // `Doors::update` regardless, exactly like the game.
        if self.level == 0x0700_0014 || self.level == 0x0700_004a {
            self.drones.doors.labels.clear();
        } else {
            for (label, time) in self.drones.doors.labels.drain(..) {
                self.hud.info_labels.push_back((label, time));
            }
        }
        // The HUD's per-frame state: the weapon-name timer, the crouch-icon timer, the pickup messages
        // (`Text_UpdateMsg` 0x1b11d8 runs from the HUD/status-pane update at frame rate).
        self.hud_tick();
        // The held weapon's laser sight [M Player_LaserPointer 0x1a65d8 — per frame from
        // Player_CollisionHandler 0x1949a8:188, after PositionGun/AnimObjectUpdate so the bone pose is
        // current]. Gate: the latch BLData+0x964 (Gun::laser_on, set per draw [M 0x1a9338:21-26] from
        // weapon_data+0x68 & 0x8 — the SG5's semi twin 0x17 carries it [D]); the game's other early
        // outs are the beam/dot sprites existing (the FP model being built — the port's datum lookup).
        // The beam: start = the FP model's bone-4 sight datum (AnimGetBoneWorldTrans +0x7e8, 4, 0),
        // direction = the bone matrix's local +Z (rows are the axes — tracer_matrix) scaled 100
        // (0x1a65d8:104-127), ray from the HEAD (Player_GetHeadPos) with the bit-0x8 query —
        // Collide_RayIntersect 0x1ec690's u32 = 8 drops pass-flagged hits;
        // the end clamps to the hit (0x1a65d8:129-141) and the red dot sits hit + normal·0.01, aligned
        // to the normal (0x1a65d8:189-205). Player_WeaponFiring 0x1a5460:88-98 hides the sprites every
        // frame first — a None here is exactly that hidden state.
        self.laser_sight = None;
        if self.gun.laser_on && !self.disabled {
            if let Some(m) = self.view_datum_matrix(4) {
                let datum = [m[3][0], m[3][1], m[3][2]];
                let z = [m[2][0], m[2][1], m[2][2]];
                let far = [datum[0] + z[0] * 100.0, datum[1] + z[1] * 100.0, datum[2] + z[2] * 100.0];
                let head = self.bond.head();
                let hit = self.world.segment_surface_bullet(head, far);
                let end = hit.map_or(far, |(t, _, _)|
                    [head[0] + (far[0] - head[0]) * t, head[1] + (far[1] - head[1]) * t, head[2] + (far[2] - head[2]) * t]);
                let dot = hit.map(|(_, n, _)|
                    ([end[0] + n[0] * 0.01, end[1] + n[1] * 0.01, end[2] + n[2] * 0.01], n));
                self.laser_sight = Some((datum, end, dot));
            }
        }
        // `Lights_CalcAmbientLight` (0x1cb178) runs per moving object from `control_movement_object_handler`
        // (0x1336f8) — the last step of the tick, before the draw reads obj+0x100..0x102.
        self.ramp_char_ambient();
        // Online the remotes move after this (`step_net_remotes`, `step_net_combat`): their body
        // clocks tick at the end of `step_net_combat`, on this tick's movement.
        if self.net_rng.is_none() && self.net_rng_shared.is_none() { self.advance_mp_anim(); }
    }

    /// mp-body-anim: each drawn MP body's script list, once per tick (`nf_game::mp::body_anim`), fed
    /// from state every machine shares: a human's remote `Bond` (`BLData+0x18`/`+0x10` = `local`,
    /// the stance, the view pitch), a bot's walking body and its brain's opponent, the held weapon,
    /// the slot's life. A slot without a drawn body (the local slot, an empty slot) starts over —
    /// its next body begins at the set init, as the respawn's rebuild does (`MP_ReSpawn` 0x185368
    /// → `PlayerAnimSetInit` [M mp-players.md §3]). Draw-only state.
    fn advance_mp_anim(&mut self) {
        if self.mp.is_none() || self.mp_body_chars.is_empty() { return; }
        for slot in 0..nf_net::MAX_AGENTS {
            if self.mp_held[slot] != 0 { self.mp_last_held[slot] = self.mp_held[slot]; }
            let Some(inp) = self.mp_body_input(slot) else {
                if slot < self.mp_anim.len() { self.mp_anim[slot] = nf_game::mp::body_anim::MpBodyAnim::new(); }
                continue;
            };
            let (Some(sets), Some(a)) = (self.mp_anim_sets.as_ref(), self.mp_anim.get_mut(slot)) else { continue };
            let scripts = &self.scripts;
            let lens = |id: u32| scripts.get(&id).map(|s| (s.length, s.looping));
            a.tick(sets, &lens, &inp);
        }
    }

    /// The body clocks' input for one slot (None = no drawn body): see `advance_mp_anim`.
    pub fn mp_body_input(&self, slot: usize) -> Option<nf_game::mp::body_anim::MpBodyInput> {
        let m = self.mp.as_ref()?;
        self.mp_body_state(slot)?;
        let weapon = self.mp_body_held(slot);
        let class = nf_formats::mpanim::weapon_anim_class(&self.weapons, weapon);
        if slot < m.settings.human_count as usize {
            let b = self.mp_bonds.get(slot)?.as_ref()?;
            // The aim point is head + aim·256 (`Player_GetAimingPoint` 0x1a4a98, the MP player's
            // `AnimObjectSetAiming` target BLData+0x60 [M 0x190d88:122]): body-relative, the view
            // pitch (the body faces the view yaw — no twist).
            Some(nf_game::mp::body_anim::MpBodyInput {
                alive: self.mp_dead_ticks[slot] == 0, weapon, class, crouch: b.crouched,
                fwd: b.local[1], strafe: b.local[0], aim: Some([0.0, b.pitch * std::f32::consts::FRAC_PI_2]),
            })
        } else {
            let bot = m.bots.iter().find(|b| b.slot == slot)?;
            let body = &bot.brain.body;
            // A bot aims at its opponent (the drone tracking's `AnimObjectSetAiming` on the target,
            // `NDrone2_DoTracking` 0x1479d8:53 [M]): the target's head from the bot's head, relative
            // to the drawn body yaw.
            let aim = bot.brain.vars.opponent.and_then(|o| self.mp_slot_head(o)).map(|t| {
                let e = bot.brain.pos;
                let (dx, dy, dz) = (t[0] - e[0], t[1] - e[1], t[2] - e[2]);
                let h = (dx * dx + dz * dz).sqrt().max(1e-3);
                let yaw_to = dx.atan2(dz);
                [nf_game::trig::angle_difference(body.yaw, yaw_to), dy.atan2(h)]
            });
            Some(nf_game::mp::body_anim::MpBodyInput {
                alive: !bot.dead, weapon, class, crouch: body.crouched,
                fwd: body.local[1], strafe: body.local[0], aim,
            })
        }
    }

    /// A slot's head position (a human's `Bond::head`, a bot's `brain.pos`), None = no body.
    fn mp_slot_head(&self, slot: usize) -> Option<[f32; 3]> {
        let m = self.mp.as_ref()?;
        if slot < m.settings.human_count as usize {
            let b = if slot == self.mp_local_slot { Some(&self.bond) } else { self.mp_bonds.get(slot)?.as_ref() }?;
            Some(b.head())
        } else {
            m.bots.iter().find(|b| b.slot == slot).map(|b| b.brain.pos)
        }
    }

    /// The HUD's per-tick state (`fix-hud`). Runs from the tick path (the game's HUD update is per
    /// 60 Hz frame): `HUD_UpdateAmmoPane` 0x19a7f0's BL+0x95E weapon timer, `HUD_UpdateHealthPane`
    /// 0x19b9c8's crouch-icon timer, and the pickup message queue (`Pickup_Handler` 0x18e650 →
    /// `Text_AddMsg` 0x1b0c30; `Text_UpdateMsg` 0x1b11d8 decrements the head's time by FRAME_RATE_MUL
    /// = 1 and deletes it at ≤ 0).
    fn hud_tick(&mut self) {
        // The goggles flow controller first — the game's `HUD_Update` 0x19a088 calls
        // `HUD_MonitorNightSight` 0x19e9d8 right after the crosshair, before the pane walk
        // [M :19]. The guard ladder's inputs, each per its decompile line:
        // - hud_active: BL+0x94a is the player-update countdown (1 = run) [M 0x193600:79-88];
        //   `Player_Update` calls `HUD_Update` [M :76] only on that path, and the port's
        //   `disabled` (cutscene/vehicle/gunimp) stands for the not-running states.
        // - viewer_state: player+0xf4 — 1 plays, 2/3 are the death states [M 0x190810:66].
        // - weapon_class: player+0xff == 0x12 blocks; Bond's class byte is never 0x12 in SP
        //   play (0x12 is the MP body class) — `// [S] unverified` that no SP path sets it.
        // - gadget_sub: player+0xf6 — the port's `bond.substate` (0 or 3..5 pass) [M :37-47].
        // - zoom_pane_on: BL+0x7e8 → +0xf4, the fp view model's anim id — the port's view
        //   machine's switch (1/2/3/8), fire (9) and reload (5/6/7) states [M 0x1a2cc0/0x1a2e38].
        // - weapon_bit0: weapon+0x60 bit 0 — the aim-mode bit [M Player_Weapon 0x1a24c8:47-67
        //   sets it from action 19; Player_WeaponChange refuses while it is up] = `self.aiming`.
        let press = std::mem::take(&mut self.hud.goggles_press);
        let out = self.hud.goggles.tick(&nf_game::hud_pane::MonitorInputs {
            hud_active: !self.disabled,
            viewer_state: if self.drones.death.active { 2 } else { 1 },
            mp: self.mp.is_some(),
            weapon_class: 0,
            gadget_sub: self.bond.substate as i16,
            goggles_equipped: self.gun.weapon == 0x5d,
            zoom_pane_on: self.gun.switch_state != 0 || self.gun.fire_state || self.gun.reload_state != 0,
            toggle_action: press,
            weapon_bit0: self.aiming,
        });
        if let Some(out) = out {
            if let Some(id) = out.item_class_override {
                // The latch-clear press: `weapon+0x63 = 0x5d` — request the goggles item
                // [M 0x19e9d8:82-83, disasm 0x19eb40-0x19eb50]. No owned gate exists in the
                // game's request path; the port's switch machine applies it like any request.
                self.gun.switch_request = id;
            }
            if out.armed {
                // The arm path's `weapon+0x63 = weapon+0x64` restore + the {0x47,0x5d,0x5e} → 1
                // clamp [M 0x19e9d8:56-66, disasm 0x19ead0-0x19eb14; the +0x64 writer is the
                // switch start 0x1a2e38:197-198 / 0x1a6f80:142]: the goggles item auto-stows to
                // the weapon it was drawn from (specials clamp to 1) as soon as its draw ends,
                // while the VIEW mode stays on — the mode is a view state, not an item state.
                self.gun.switch_request =
                    nf_game::hud_pane::MonitorNightSight::clamp_item_class(self.gun.saved_weapon);
            }
        }
        // The weapon-name timer (`HUD_UpdateAmmoPane` 0x19a7f0:212-217): while a switch request
        // differs from the current weapon (drone+0x63 vs +0x62) and the two records' +2 words
        // differ, the timer is RE-SET to FRAME_RATE (60) every pending frame — the name covers the
        // whole draw animation plus the 60-tick decay [M]. fix-ammo-pane: the old arm-on-flip fired
        // once at the completion tick, so the port's name window missed the pending draw entirely
        // and ran 60 ticks from there; the game arms through it.
        let w = self.gun.weapon;
        let req = self.gun.switch_request;
        if req != 0 && req != w {
            let changed = match (self.weapons.records.get(w as usize), self.weapons.records.get(req as usize)) {
                (Some(a), Some(b)) => a[2..4] != b[2..4],
                _ => false,
            };
            if changed { self.hud.weapon_label_timer = 60; }
        }
        self.hud.weapon_label_timer = self.hud.weapon_label_timer.saturating_sub(1);
        // The crouch icon: the timer holds 60 while crouched, then fades after standing
        // [M 0x19b9c8:305-328].
        if self.bond.crouched { self.hud.crouch_icon_timer = 60; }
        else if self.hud.crouch_icon_timer > 0 { self.hud.crouch_icon_timer -= 1; }
        // The 007 Bond-moment flash: −1 per frame while ≥ 1 [M 0x19b9c8:258-260]; a newly done
        // moment (the BondMoments[id − 0x65] entry going 0 → 1 in Trigger_Calc's
        // PlrStat_DoneBondMoment) sets 200 [M 0x199448:23-25].
        if self.hud.bond_moment_flash > 0 { self.hud.bond_moment_flash -= 1; }
        let moments = self.drones.progress.bond_moments.iter().filter(|&&m| m != 0).count() as u32;
        if self.hud.moments_seen.is_some_and(|n| moments > n) { self.hud.bond_moment_flash = 200; }
        self.hud.moments_seen = Some(moments);
        // The context icon's ThirdIconTimer (0x30CCE8): held at 30 while a context id is set
        // (the game rewrites it on every shown frame [M 0x19b9c8:298]), counting down on 0xFF
        // for the 30-tick fade-out [M 0x19b9c8:275-282]. Runs with the health pane's update —
        // outside the cut-scene message freeze (that gate is Text_UpdateMsg's, the panes still
        // step [M 0x1b11d8]).
        if self.bond.icon_ctx != 0xff { self.hud.context_icon_timer = 30; self.hud.last_icon_ctx = self.bond.icon_ctx; }
        else if self.hud.context_icon_timer > 0 { self.hud.context_icon_timer -= 1; }
        // Script op-0x1e text commands (`Script_TextStart` 0x1db2b8 → `Text_AddMsg(-1, 0, 4,
        // Txt_BindLabel(label), 0, time)` [M]): the raw labels drain to HUD messages with the same
        // bank binding as the pickup texts (fix-script-commands).
        // The skip's subtitle flush [M Script_FFwd 0x1dac78:28-29 `Text_FlushAllSubtitles()`]: the
        // script-sourced lines (type 4/5 [M 0x1b15c8]) die; the pickup/INFO queues stay.
        if std::mem::take(&mut self.drones.text_flush) {
            self.hud.msgs.retain(|m| !m.script);
        }
        let lang = self.lang.take();
        let mut new_msgs: std::collections::VecDeque<Msg> = Default::default();
        if let Some(lang) = lang.as_ref() {
            for (label, time) in self.drones.script_msgs.drain(..) {
                let text = lang.bind(label).unwrap_or("").trim().to_string();
                if !text.is_empty() { new_msgs.push_back(Msg { script: true, ..Msg::new(text, time as f32) }); }
            }
        }
        // The INFO pane's label drain (fix-context-hints): the game binds at POST time
        // (`Txt_BindLabel` runs inside ThirdIcon_Update 0x196e80:41 / Hint_Update 0x1b3380:31),
        // before the pane could ever draw, so this sits before the cut-scene freeze like the
        // script-op drain above; the pane display waits on the viewer gate (below).
        for (label, time) in self.hud.info_labels.drain(..) {
            let text = lang.as_ref().and_then(|l| l.bind(label)).unwrap_or("").trim().to_string();

            if !text.is_empty() {
                let mut m = Msg::new(text, time);
                m.label = label;
                self.hud.info_msgs.push_back(m);
            }
        }
        // mp-hud-real: the kill messages addressed to this machine's slot (`MP_PlayerKilled`
        // 0x188f70:100-107 → `Text_AddMsg(killer, 0, 1, label 0x200004d % the victim's name, 0,
        // 0xb4)` [M]) — the MP info pane draws them (`HUD_MPUpdateStatusPane` 0x19ca88).
        let me = self.mp_local_slot;
        if let Some(m) = self.mp.as_mut() {
            if m.kill_msgs.iter().any(|&(to, _)| to == me) {
                let mine: Vec<usize> = m.kill_msgs.iter().filter(|&&(to, _)| to == me).map(|&(_, v)| v).collect();
                m.kill_msgs.retain(|&(to, _)| to != me);
                for victim in mine {
                    let name = crate::hud::mp_name(m, lang.as_ref(), victim);
                    let text = lang.as_ref().map(|l| l.format(nf_game::mp::hud::KILL_MSG_LABEL, &[name])).unwrap_or_default();
                    if !text.is_empty() {
                        let mut msg = Msg::new(text, nf_game::mp::hud::KILL_MSG_TICKS as f32);
                        msg.label = nf_game::mp::hud::KILL_MSG_LABEL;
                        self.hud.info_msgs.push_back(msg);
                    }
                }
            }
        }
        for (_, t) in self.drones.taken.iter().skip(self.hud.taken_seen) {
            let Some(lang) = lang.as_ref() else { break };
            let text = match t {
                nf_game::pickup::Taken::Weapon { weapon } => {
                    let name = lang.bind(self.weapons.name_label(*weapon as usize)).unwrap_or("");
                    lang.format(0x200_000D, &[name.to_string()])
                }
                nf_game::pickup::Taken::Ammo { weapon, rounds } => {
                    let at = self.weapons.ammo_type(*weapon as usize);
                    let name = lang.bind(self.weapons.ammo_name_label(at)).unwrap_or("");
                    lang.format(0x200_000E, &[rounds.to_string(), name.to_string()])
                }
                nf_game::pickup::Taken::Key { .. } => lang.bind(0x200_0010).unwrap_or("").to_string(),
                nf_game::pickup::Taken::Armour => lang.bind(0x200_000F).unwrap_or("").to_string(),
                nf_game::pickup::Taken::Bonus => lang.bind(0x200_004A).unwrap_or("").to_string(),
                // Case 5 [M 0x18e650:229-233 + the shared tail]: message 0x2000013.
                nf_game::pickup::Taken::Message => lang.bind(0x200_0013).unwrap_or("").to_string(),
                // Case 4 [M 0x18e650]: the pickup's own label (PICKUPINFO+0x34) bound and shown when
                // non-zero — 0700000b's id-0x47 item carries 0x2000036 [D the placement params]; the
                // 07000008 briefcase's label is 0 → no message.
                nf_game::pickup::Taken::Label { label } => if *label != 0 { lang.bind(*label).unwrap_or("").to_string() } else { String::new() },
            };
            if !text.is_empty() { new_msgs.push_back(Msg::new(text, 180.0)); }
        }
        self.hud.taken_seen = self.drones.taken.len();
        for m in new_msgs { self.hud.msgs.push_back(m); }
        self.lang = lang;
        // The mission layer's events (`Mission_Update` 0x182c60 / `Mission_MonitorObjectives`
        // 0x1827d8, docs/systems/mission.md): type-2 objective notices, the FailLabel, the type-3
        // mission banner and the level-end / restart decisions. `Text_AddMsg` only QUEUES the
        // message — every sound and every drawn pixel belong to the pane update
        // (`HUD_UpdateStatusPane` 0x19c310 cases 2/3), which cannot open while a script camera
        // holds the viewer's +0x2a gate [M 0x1b11d8:16-18]. The port therefore drains the events
        // BEFORE the frozen check (a banner queued mid-cut-scene waits in the queue, its timer
        // untouched) and plays the SFX when the message first displays, below.
        // fix-objective-banner: the drain used to sit behind the frozen check and play the SFX as
        // the event landed, so on 07000007 the Kimiko trigger's ch 0x48 announced (and played
        // 1532+469) on the trigger tick — one tick before the scene's `Script_CameraStart` set the
        // freeze — so the objective sound played before the Kimiko cut-scene.
        let mut events = std::mem::take(&mut self.drones.mission_events);
        for ev in events.drain(..) {
            match ev {
                nf_game::mission::Event::Objective { label, is_new: _, ticks, failed } => {
                    let complete = label == 0x0200_0053;
                    let mut text = match self.lang.as_ref().and_then(|l| l.bind(label)) {
                        Some(t) => t.to_string(),
                        None => continue,
                    };
                    if failed {
                        // The game's "%s: %s" (0x30cbb8) with 0x170 "Fail" [M 0x1827d8:72-74].
                        let fail = self.lang.as_ref().and_then(|l| l.bind(0x170)).unwrap_or("Fail").to_string();
                        text = format!("{fail}: {text}");
                    }
                    let status = if complete { 0x0200_0053 } else { 0x0200_0051 };
                    // (fix-objective-messages: `Text_AddMsg` 0x1b0c30 also wraps the stored string
                    // with `Font_WordWrapString` at TextMsgFormats[2]'s 480 [L 0x2bf108+0x14] and
                    // scales the life by the wrapped/orig length [M 0x1b0c30's +0x18 store]. The
                    // port wraps at draw time — the font lives in the scene's HudTables, which the
                    // Sim-side drain doesn't hold; the life scaling is not ported, see the report.)
                    self.hud.objective_msgs.push_back(Msg { text, time: ticks as f32, status, failed, life: ticks as f32, opened: false, second: false, label, script: false,
                        sfx1: false }); // objectives-0b-start: the label rides the Msg (the port's provenance for the game-label pins, `Text_AddMsg` 0x1b0c30 stores the bound string's id)
                }
                nf_game::mission::Event::FailLabel(l) => {
                    self.hud.fail_label = l;
                }
                nf_game::mission::Event::MissionComplete { next: _ } => {
                    // The type-3 "Mission Complete" (0x2000006), 240 ticks [M 0x182c60:85-122].
                    // The level end itself arrives as `Event::LevelEnd` after the banner (case 4).
                    if let Some(t) = self.lang.as_ref().and_then(|l| l.bind(0x0200_0006)) {
                        self.hud.mission_msgs.push_back(Msg::new(t.to_string(), 240.0));
                    }
                }
                nf_game::mission::Event::MissionFailed => {
                    // The type-3 "Mission Fail" (0x2000007), 300 ticks [M 0x182c60:45-57].
                    if let Some(t) = self.lang.as_ref().and_then(|l| l.bind(0x0200_0007)) {
                        self.hud.mission_msgs.push_back(Msg::new(t.to_string(), 300.0));
                    }
                }
                nf_game::mission::Event::PlayerDead => {} // the red fade is render-side; the menu arrives below
                nf_game::mission::Event::LevelEnd { next } => {
                    // `ResetMap_LevelToLoad(LevelToEndTo)` [M 0x182c60:73]. The 0x0700xxxx codes
                    // are the port's own levels; the 0x0730xxxx codes are the driving interludes
                    // (the handover below).
                    if next & 0xffff_0000 == 0x0700_0000 && next != 0x0700_0048 {
                        self.drones.level_exit = Some(next);
                    } else {
                        // The mission's own end: the front end (no outro) or an 0x0730 outro code
                        // (the outro movie plays, then its phase-2 resolve routes on — the driving
                        // interludes ride the same code [M 0x1d3560]). The host runs the end
                        // sequence (`mission_ended` → outro → results); the driving return is
                        // still stashed for the hub. Missing glue before 2026-10-15: a finished
                        // mission only logged "not ported" — no outro, no results.
                        if nf_game::ptp::is_driving_interlude(next) {
                            self.ptp_handover(next);
                            // The handover's hub resume (0x7000048) is the end sequence's job now:
                            // the outro plays first, then the front end (no driving to run).
                            self.drones.level_exit = None;
                        }
                        eprintln!("[mission] complete → LevelToEndTo {next:08x}: the end sequence");
                        self.drones.mission_end = Some(next);
                    }
                }
                nf_game::mission::Event::MenuOpen => {
                    // The 0x40000042 fail/death menu [M 0x182c60:78]. Until 180-fix-menus draws
                    // it, the port's auto-restart answers Restart immediately (the flag keeps
                    // the view free to answer instead via `fail_choose_restart`/`_quit`).
                    if self.drones.auto_restart { self.drones.fail_choose_restart(); }
                }
            }
        }
        // `Text_UpdateMsg` 0x1b11d8: time −= 1 per frame, delete at ≤ 0 — but the viewer's +0x2a HUD
        // flag returns early while a cut-scene camera runs [M 0x1b11d8:16-18], freezing every message
        // until the NIS ends. The port: no countdown and no bar close while frozen.
        if self.drones.hud_frozen {
            return;
        }
        // The pickup/script pane (viewer 0, TXTMSG type 6/4): the head ticks alone, exactly like
        // every other status pane. `Text_UpdateMsg` 0x1b11d8:48-73 decrements ONLY the pane's head
        // node (`*param_3` — the out-param the pane state machine 0x19c310 re-looks-up on expiry),
        // so a message queued behind another keeps its FULL life until it reaches the front; the
        // port used to tick every member in parallel, which expired the queued texts before they
        // ever showed — they flashed past one per frame once they surfaced (07000005).
        if let Some(m) = self.hud.msgs.front_mut() {
            m.time -= 1.0;
            if m.time <= 0.0 { self.hud.msgs.pop_front(); }
        }
        if self.hud.msgs.is_empty() {
            if self.hud.bar_close > 0.0 { self.hud.bar_close = (self.hud.bar_close - 16.0).max(0.0); }
        } else {
            self.hud.bar_close = HudState::BAR_W;
        }
        // The INFO pane (pane 4): the head ticks alone, the close shrinks 16 px/tick — the same
        // `HUD_UpdateStatusPane` 0x19c310 machine as the pickup bar (the CREATE's slide is 0x10
        // [L MsgInfoStatusPane 0x319810+0x14]).
        if let Some(m) = self.hud.info_msgs.front_mut() {
            m.time -= 1.0;
            if m.time <= 0.0 { self.hud.info_msgs.pop_front(); }
        }
        if self.hud.info_msgs.is_empty() {
            if self.hud.info_close > 0.0 { self.hud.info_close = (self.hud.info_close - 16.0).max(0.0); }
        } else {
            self.hud.info_close = HudState::INFO_W;
        }
        // `Text_UpdateMsg` ticks only the displayed head [M 0x1b11d8:48-73 — the out-param
        // `*param_3` is the head node; the queue behind waits]; when it expires the next message
        // becomes the head and the pane re-opens for it (case 0 → 1). The port used to tick every
        // queue member in parallel, which expired the banners behind the front one before they
        // ever showed.
        if let Some(m) = self.hud.objective_msgs.front_mut() {
            m.time -= 1.0;
            if m.time <= 0.0 { self.hud.objective_msgs.pop_front(); }
        }
        if self.hud.objective_msgs.is_empty() {
            if self.hud.objective_close > 0.0 { self.hud.objective_close = (self.hud.objective_close - 16.0).max(0.0); }
        } else {
            self.hud.objective_close = HudState::OBJECTIVE_W;
        }
        if let Some(m) = self.hud.mission_msgs.front_mut() {
            m.time -= 1.0;
            if m.time <= 0.0 { self.hud.mission_msgs.pop_front(); }
        }
        if self.hud.mission_msgs.is_empty() {
            if self.hud.mission_close > 0.0 { self.hud.mission_close = (self.hud.mission_close - 32.0).max(0.0); }
        } else {
            self.hud.mission_close = HudState::MISSION_H;
        }
        // The banner SFX, where the game plays them: the pane update. Case 2 schedules the pair per
        // display (new objective: PlayMe1st 1532, PlayMe2nd 469; "OBJECTIVE COMPLETE": PlayMe1st 468
        // and ObjectiveSFXTriggered2 preset so the 2nd never plays) [M 0x19c310:148-164] but plays
        // NOTHING itself; case 3 — the NEXT pane frame — fires PlayMe1st while the message is in its
        // flash window (remaining + 60 > life), ONLY while `ScriptCam == 0` [M 0x19c310:219-229], and
        // PlayMe2nd (469) once the steady phase begins (remaining + 60 ≤ life; no camera check — the
        // frozen timer cannot newly reach it mid-scene) [M 0x19c310:232-235]. The port's pane cannot
        // OPEN while `hud_frozen` (the viewer+0x2a gate above [M 0x1b11d8:16-18]) — the same window a
        // script camera holds `ScriptCam != 0` [M 0x1d9140:6 sets it, 0x1d9860 clears it] — so a
        // banner queued mid-scene opens after the hand-back and its chime fires then; and a banner
        // that opened the tick before a scene starts waits in its frozen flash window with the chime
        // unplayed, exactly the game's held Triggered1 (objective-banner-timing).
        if let Some(m) = self.hud.objective_msgs.front_mut() {
            let cam = self.bond.head();
            if !m.opened {
                // Case 2: the open — bind the header, schedule the pair, no sound yet [M 0x19c310:126-176].
                m.opened = true;
            } else if !m.sfx1 && m.time + 60.0 > m.life {
                // Case 3's flash branch: PlayMe1st once (1532 new / 468 complete) [M 0x19c310:219-229].
                m.sfx1 = true;
                let id = if m.status == 0x0200_0053 { 468 } else { 1532 };
                self.drones.audio.play3d(id, cam, None);
            }
            if m.sfx1 && !m.second && m.status != 0x0200_0053 && m.time <= m.life - 60.0 {
                // The steady branch: PlayMe2nd (469) once [M 0x19c310:232-235] — only after the
                // 1st fired (the game's Triggered2 arm sits in the steady branch, which the flash
                // branch's ScriptCam hold can delay the 1st past; the timer freeze keeps the
                // ordering 1532 then 469 exactly as the pane plays them).
                m.second = true;
                self.drones.audio.play3d(469, cam, None);
            }
        }
    }

    /// The crosshair offset (`BL+0x120 / +0x124`: −1…1 across half the screen) from the auto-aim offset angles: tan of the
    /// angle over the half-screen tangents of the game's projection [L] (0.76966 across, 0.57724 up); a larger yaw
    /// turns left (+X is screen-left in the object frame), so x flips.
    pub fn crosshair(&self) -> (f32, f32) {
        if self.free_aim { return (self.cross[0], self.cross[1]); }
        (-self.drones.aim_offset[0].tan() / 0.76966, self.drones.aim_offset[1].tan() / 0.57724)
    }

    /// The drawn instance of a level-placed pickup (`View_DrawObjectGlist` 0x1e6780 draws `obj+0xD8` at
    /// `obj+0x90`): its matrix while it lies there, `None` once taken or hidden. fix-player2.
    pub fn placed_pickup_pose(&self, k: usize) -> Option<nf_formats::anim::Mat4> {
        self.drones.pickups.get(k).filter(|p| p.from.is_none() && !p.gone && !p.hidden).map(|p| p.m)
    }

    /// The crosshair state (`HUD_UpdateCrossHair` 0x19e6d0 reads BLData+0x133): 2 normal, 3 = the red
    /// "no" mark on a non-enemy target (`Check_Target` 0x1a7810:181-182). fix-player2.
    pub fn crosshair_state(&self) -> u8 { self.cross_state }

    /// One tick of the decryptor screen (`Player_WeaponFiring` 0x1a5460:155-194 reading the display
    /// buffer `Lock_Update` 0x1ae3d8 fills [M :179-189, :346-369]): a scan start shows "****"; the
    /// solved digits show the code; the next unsolved one re-rolls on every odd progress value; while
    /// a scan runs the idle countdown re-arms to FRAME_RATE (60), and when it runs out the four datums
    /// clear. *Deviation (cosmetic)*: the code digits (`Locks_Init` 0x1b0548, `'0' + rand % 9`) and
    /// the re-rolls (`Rand_Rand(10)`) come from the game's shared RNG stream, whose state is play
    /// history; the port shows the same rule with a hash of (code index, position, progress) and
    /// draws nothing from the sim's RNG (the decode never compares digits [M]).
    fn pda_tick(&mut self) {
        let hash = |a: u32, b: u32, c: u32| { let mut h = a.wrapping_mul(0x9e37_79b9) ^ b.wrapping_mul(0x85eb_ca6b) ^ c.wrapping_mul(0xc2b2_ae35); h ^= h >> 15; h = h.wrapping_mul(0x2c1b_3c6d); h ^ (h >> 12) };
        if let Some(li) = self.drones.locks.iter().position(|l| l.scanning) { self.pda_lock = Some(li); }
        let scan = self.pda_lock.and_then(|li| self.drones.locks.get(li))
            .map(|l| (l.scanning, (l.scan_solved as usize).min(4), l.scan_progress, l.code as u32));
        match scan {
            Some((scanning, solved, progress, code)) if scanning || (solved == 4 && self.pda_hold > 0) => {
                for i in 0..4 {
                    self.pda_display[i] = if i < solved { b'0' + (hash(code, i as u32, 0) % 9) as u8 }
                        else if scanning && i == solved && progress > 0 { b'0' + (hash(code, i as u32, progress | 1) % 10) as u8 }
                        else { b'*' };
                }
                if scanning { self.pda_hold = 60; } else { self.pda_hold -= 1; }
            }
            _ => {
                self.pda_hold = self.pda_hold.saturating_sub(1);
                if self.pda_hold == 0 { self.pda_display = [0; 4]; }
            }
        }
    }

    /// The decryptor's scan state (`Player_WeaponFiring` 0x1a5460:155-194): the scanning lock's
    /// index, progress (`obj+0xf6`, the +1/+3 accumulator) and solved digit count (`DAT_002a37e3`) —
    /// `None` when no scan runs. The screen itself is `pda_display` (see `pda_tick`).
    pub fn decryptor_scan(&self) -> Option<(usize, u32, u8)> {
        self.drones.locks.iter().position(|l| l.scanning)
            .map(|i| (i, self.drones.locks[i].scan_progress, self.drones.locks[i].scan_solved))
    }

    /// The held weapon's laser sight this tick — the getter the draw code reads. The tuple is
    /// (beam start = the FP model's bone-4 sight datum, beam end = the aim-trace hit or the far
    /// end, dot = Some((pos, normal)) when the trace met a surface) [M Player_LaserPointer
    /// 0x1a65d8; the field doc carries the cites]. `None` = the sight is off (the sprites stay
    /// hidden, `Player_WeaponFiring` 0x1a5460:88-98).
    pub fn laser_sight(&self) -> Option<(nf_game::world::V3, nf_game::world::V3, Option<(nf_game::world::V3, nf_game::world::V3)>)> {
        self.laser_sight
    }

    /// Characters whose drone is present but whose animation cannot be shown (drawn in the bind pose): (character,
    /// drone, script, its first sequence, in `seq_index`, that sequence's skeleton vs the mesh's).
    pub fn unposed(&self, characters: &[Character]) -> Vec<String> {
        let mut out = Vec::new();
        for (ci, c) in characters.iter().enumerate() {
            let Some(di) = self.char_drone.get(ci).copied().flatten() else { continue };
            let dr = &self.drones.list[di];
            if !dr.present || self.frame_of(di, c.skin.skeleton).is_some() { continue; }
            let q = dr.anim.script.and_then(|h| self.scripts.get(&h)).and_then(|s| s.sequences().next().map(|x| x.2));
            out.push(format!("char {ci} drone {di} (placement {}) state {:#x} script {:x?} seq {:x?} indexed {:?} mesh skeleton {}", dr.placement, dr.state, dr.anim.script, q,
                q.and_then(|q| self.seq_index.get(&q)), c.skin.skeleton));
        }
        out
    }

    /// The falling drops as camera-facing quads (`DrawDrops` → `psiDrawParticleList(0.2, …, 0x808080, 0x03000045)`:
    /// GS sprites, colour (63, 63, 63, 127) × texture, additive, depth-tested without writing). The GAME'S SIZE RULE
    /// for the render pass: DrawDrops passes colours = NULL, so psiDrawParticleList takes the NON-colour
    /// creator, which never reads the 0.2 [M disasm 0x18cc1c-30 mtc1 0.2/`move $t0, $zero`/jal; 0x122d50 `beqz $s4 →
    /// CreatePointerSpriteList 0x11a160`] — the flake scale there is `1/(PS2SnowSize · PS2SnowSizeMult)` [M 0x11a160:81],
    /// PS2SnowSize level-set (28988/12378/19746 for 0x07000005/6/8 [M 0x11e2e8:32-46]) and PS2SnowMult = 448/viewport-H
    /// [M 0x10b2b0:100-112] — a screen-space size, not the 0.2 (VU1 sprite microcode owns the final px factor; backlog
    /// L-706). The port's 0.1-half-extent (0.2 full width) quads stand in for that screen-space rule (visual only).
    pub fn snow_quads(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Vertex> {
        let Some(env) = self.drones.env.as_ref() else { return Vec::new() };
        let h = 0.1;
        let (r, u) = (right.map(|v| v * h), up.map(|v| v * h));
        let mut out = Vec::with_capacity(env.drops.len() * 6);
        for (p, &alive) in env.drops.iter().zip(&env.alive) {
            if !alive { continue; }
            let c = |sr: f32, su: f32, uv: [f32; 2]| Vertex { pos: [p[0] + r[0] * sr + u[0] * su, p[1] + r[1] * sr + u[1] * su, p[2] + r[2] * sr + u[2] * su], uv, rgba: [63, 63, 63, 127], nrm: [0; 4] };
            let (a, b, cc, d) = (c(-1.0, -1.0, [0.0, 1.0]), c(1.0, -1.0, [1.0, 1.0]), c(1.0, 1.0, [1.0, 0.0]), c(-1.0, 1.0, [0.0, 0.0]));
            out.extend_from_slice(&[a, b, cc, a, cc, d]);
        }
        out
    }

    /// The emitter particles as camera-facing quads (`Emitter_Draw` 0x17b708 path A →
    /// `psiDrawParticleList` 0x122d50: a size-`def.size` rgba sprite per alive slot). The game sizes
    /// the sprite in screen space (`4·size / (PS2ParticleSize · PS2SnowSizeMult)`) — the port
    /// takes the def size as the world-space half-extent, the way the snow quads do. The GAME'S SCALAR for the
    /// render pass, EXACT [M 0x119910:81 `pDMA[4] = (uint)((1.0/(PS2ParticleSize·PS2SnowSizeMult))·4.0·size)`
    /// — a VU1 sprite scale word next to the ST=1.0 constants]: PS2ParticleSize = Ps2Far1·0.002386635 [M 0x11e2e8:67],
    /// PS2SnowSizeMult = (1/viewer+0x108)·(1/(viewport-H·0.002232143)) = 448/H [M 0x10b2b0:100-112] — ∝ 1/far and
    /// ∝ viewport height, i.e. a FIXED SCREEN-SPACE size, not a world extent; the ±size/2 world-space reading is the
    /// port's approximation (the half-vs-full factor lives in the VU1 sprite microcode, backlog L-706). The
    /// white rgb in the game's sprite comes from the bound def texture (the font-slot hash, def+0x04)
    /// modulated by the slot rgba [M 0x119910:19]; the port has no per-def texture, so the colour is
    /// the slot rgba itself — listed in the report's render follow-ups.
    pub fn particle_quads(&self, right: [f32; 3], up: [f32; 3]) -> Vec<Vertex> {
        let sprites = self.drones.fx.particle_sprites();
        let mut out = Vec::with_capacity(sprites.len() * 6);
        for s in sprites {
            let (r, u) = (right.map(|v| v * s.size), up.map(|v| v * s.size));
            let c = |sr: f32, su: f32, uv: [f32; 2]| Vertex {
                pos: [s.pos[0] + r[0] * sr + u[0] * su, s.pos[1] + r[1] * sr + u[1] * su, s.pos[2] + r[2] * sr + u[2] * su],
                uv, rgba: s.rgba, nrm: [0; 4],
            };
            let (a, b, cc, d) = (c(-1.0, -1.0, [0.0, 1.0]), c(1.0, -1.0, [1.0, 1.0]), c(1.0, 1.0, [1.0, 0.0]), c(-1.0, 1.0, [0.0, 0.0]));
            out.extend_from_slice(&[a, b, cc, a, cc, d]);
        }
        out
    }

    /// The live animated script object from `src` (script hash, stream) on `skin`: its animation script and frame.
    /// A hidden SP's stream objects read as absent — the Script_HideObj cascade [M 0x1db808].
    pub fn script_anim(&self, src: (u32, u8), skin: u32) -> Option<(u32, f32)> {
        let hidden = self.drones.scripts.hidden_stream_objects();
        self.drones.scripts.objects.list.iter().enumerate().rev().find_map(|(i, o)| {
            if hidden.contains(&i) { return None; }
            match o {
                Some(nf_game::nis::Obj::Anim { skin: k, script, frame, src: o, .. }) if *o == src && *k == skin => Some((*script, *frame)),
                _ => None,
            }
        })
    }


    /// The base matrix of the live animated script object from `src` on `skin` (its owning SP's matrix at
    /// the start - see `Obj::Anim::base`); the same newest-first walk as `script_anim`.
    pub fn script_anim_base(&self, src: (u32, u8), skin: u32) -> Option<Mat4> {
        let hidden = self.drones.scripts.hidden_stream_objects();
        self.drones.scripts.objects.list.iter().enumerate().rev().find_map(|(i, o)| {
            if hidden.contains(&i) { return None; }
            match o {
                Some(nf_game::nis::Obj::Anim { skin: k, src: o, base, .. }) if *o == src && *k == skin => Some(*base),
                _ => None,
            }
        })
    }

    /// The world matrix of the live script entity `hash` from `src`.
    pub fn script_entity(&self, src: (u32, u8), hash: u32) -> Option<Mat4> { self.script_entity_k(src, hash, 0) }

    /// The k-th live script entity `hash` from `src` (newest first, the order `script_entity` reads).
    /// A hidden SP's stream objects read as absent — the Script_HideObj cascade [M 0x1db808].
    pub fn script_entity_k(&self, src: (u32, u8), hash: u32, k: usize) -> Option<Mat4> {
        let hidden = self.drones.scripts.hidden_stream_objects();
        self.drones.scripts.objects.list.iter().enumerate().rev().filter_map(|(i, o)| {
            if hidden.contains(&i) { return None; }
            match o {
                Some(nf_game::nis::Obj::Entity { hash: h, matrix, src: o, .. }) if *o == src && *h == hash => Some(*matrix),
                _ => None,
            }
        }).nth(k)
    }
    /// The game data the simulation consults, borrowed from this Sim.
    pub fn data(&self) -> nf_game::drone::Data<'_> {
        nf_game::drone::Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: self.level, sequences: &self.seqs, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models , surfaces: self.surfaces.as_ref(), voices: self.voices.as_deref(), difficulty: self.drones.difficulty,
            ai_points: &[],
        }
    }

    /// (sequence index, frame) the first-person gun's script shows.
    /// The world position of the held weapon's view-model DATUM `id` at this tick's pose
    /// (`AnimGetBoneWorldTrans(BLData+0x7e8, id, 0)` [M 0x1c3120: a non-negative id is a datum —
    /// `QuatTransToMat(datum.q, datum.t) · bone world`]): the first-person skin (+0xDC), the view script's
    /// sequence at the current frame (root translation zeroed, as the draw does), through `gun_matrix`.
    pub fn view_datum(&self, id: u32) -> Option<[f32; 3]> {
        self.view_datum_matrix(id).map(|m| [m[3][0], m[3][1], m[3][2]])
    }

    /// `view_datum`'s full world matrix (rotation rows + translation) — the `AnimGetBoneWorldTrans` MATRIX out.
    pub fn view_datum_matrix(&self, id: u32) -> Option<Mat4> {
        let w = self.weapons.records.get(self.gun.weapon as usize)?;
        let skin = self.skins.get(&u32::from_le_bytes(w[0xdc..0xe0].try_into().ok()?))?;
        let skel = self.skeletons.get(&skin.skeleton)?;
        let dt = skin.datums.iter().find(|d| d.id == id && d.bone >= 0)?;
        let script = self.scripts.get(&self.view_script)?;
        let t = (1.0 + self.view_time) as u16;
        let (a, _, q) = script.sequences().find(|(a, b, _)| *a <= t && t <= *b).or_else(|| script.sequences().last())?;
        let seq = self.seqs.get(&q).filter(|s| s.skeleton == skin.skeleton)?;
        let mut local = anim::local_pose(seq, skel, skin, (t.saturating_sub(a) + 1) as u32);
        if let Some(r) = local.first_mut() { r.0 = [0.0; 3]; }
        let world = anim::world_pose(&local, skin);
        Some(anim::mul(&anim::mul(&anim::quat_trans_to_mat(dt.q, dt.t), world.get(dt.bone as usize)?), &self.gun_matrix()))
    }

    pub fn view_frame(&self, skeleton: u8) -> Option<(usize, u32)> {
        let script = self.scripts.get(&self.view_script)?;
        let t = (1.0 + self.view_time) as u16;
        let (a, _, q) = script.sequences().find(|(a, b, _)| *a <= t && t <= *b).or_else(|| script.sequences().last())?;
        let &(_, i) = self.seq_index.get(&q).filter(|(sk, _)| *sk == skeleton)?;
        Some((i, (t.saturating_sub(a) + 1) as u32))
    }

    /// The first-person gun's matrix (`Player_SetWeaponAnimObj` + `Player_PositionGun` 0x1a3b40): the gun turns toward
    /// the crosshair — pitch `cy·(π/6)`, yaw aspect·`cx·(π/6)` (G+0x50 / +0x54, zoom 1) — about the view-space point
    /// (0, −0.2, 0.5): position = eye + (offset `+0xE0` + bob + (0, 0.2, −0.5))·R_gun + (0, −0.2, 0.5)·R_view. The
    /// rotation order and signs, THE GAME'S EXACT RULE for render comparison
    /// [M disasm 0x1a3b40-0x1a3c20]: the gun object's rotation is `objmat · RotX(−(BLData+0x8a8)·π/2) ·
    /// RotMatrixZYX(euler)` — RotMatrixZYX 0x1e25a8 (R = Rz·Ry·Rx) on GUN+0x50..0x5b (GUN =
    /// *(BLData+0x7e8)), where Player_WeaponRecoil writes euler X = −(BLData+0x124)·(π/6)/zoom
    /// (the VERTICAL crosshair term), euler Y = −aspect·(BLData+0x120)·(π/6)/zoom + recoil(+0x8e4),
    /// euler Z = 0 [M 0x1a3f64-0x1a3fbc; +0x120/+0x124 are the horizontal/vertical aim terms,
    /// Player_GetAimingPoint 0x1a4a98:53-55]; the pivot (0, −0.2, 0.5) is rotated by (RotX∘objmat)
    /// and added to the head + gunRot·(GUN+0x30) [0x1a3bec-0x1a3c20]. The port builds the same
    /// orientation by turning the view basis (`fwd` by yaw about up, then pitch about the turned
    /// right) instead of the euler product — the turned-forward construction stands in for the
    /// matrix product (visual only; the sign/order check is a render check).
    pub fn gun_matrix(&self) -> [[f32; 4]; 4] {
        let w = &self.weapons.records[self.gun.weapon as usize];
        let f = |o: usize| f32::from_le_bytes(w[o..o + 4].try_into().unwrap());
        let (p, t) = (self.bob[0], self.bob[1]);
        let l = [f(0xe0) + 0.01 * p.sin() + 0.001 * t.sin(), f(0xe4) + 0.01 * p.cos().abs() + 0.001 * (1.7 * t).cos(), f(0xe8) + 0.02 * (0.84329 * p).sin()];
        let (y, pt) = (self.bond.yaw, self.bond.pitch * std::f32::consts::FRAC_PI_2);
        let left = [y.cos(), 0.0, -y.sin()];
        let up = [-pt.sin() * y.sin(), pt.cos(), -pt.sin() * y.cos()];
        let fwd = self.bond.look();
        // The gun's axes: forward turned by the crosshair angles (right = −left), then left / up rebuilt.
        let (cx, cy) = self.crosshair();
        // G-044 [M Player_WeaponRecoil 0x1a3de8:52-57]: both tilt terms divide by `glb_viewer+0x108`
        // (the zoom — `Player_PositionCamera` 0x18f5d8:129 feeds it BL+0x8D0, the port's `self.zoom`),
        // and the x term scales by `viewer+0x114` = 4/3 on a 4:3 screen, 16/9 with the AV options'
        // widescreen flag [L ee_start_05 viewer 0x1fc6c10+0x114 = 0x3faaaaab; `Camera_CalcViewAngles`
        // 0x1baf70:28-32 writes 0x3fe38e39 = 16/9 there when DAT_002a37c4 is set — screen-adjust-model
        // feeds it through `self.view_aspect`]. A zoomed view used to tilt the gun as if unzoomed.
        let (gy, gp) = (
            self.view_aspect * cx * std::f32::consts::FRAC_PI_6 / self.zoom,
            cy * std::f32::consts::FRAC_PI_6 / self.zoom,
        );
        let gf = [0, 1, 2].map(|k| fwd[k] * gp.cos() * gy.cos() - left[k] * gp.cos() * gy.sin() + up[k] * gp.sin());
        let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
        let norm = |v: [f32; 3]| { let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9); v.map(|x| x / n) };
        let gl = norm(cross(up, gf));
        let gu = cross(gf, gl);
        let h = self.bond.head();
        let o = [l[0], l[1] + 0.2, l[2] - 0.5];
        let pos = [0, 1, 2].map(|k| h[k] + o[0] * gl[k] + o[1] * gu[k] + o[2] * gf[k] - 0.2 * up[k] + 0.5 * fwd[k]);
        [[gl[0], gl[1], gl[2], 0.0], [gu[0], gu[1], gu[2], 0.0], [gf[0], gf[1], gf[2], 0.0], [pos[0], pos[1], pos[2], 1.0]]
    }

    /// (sequence index, frame) the drone's playing script shows.
    fn frame_of(&self, drone: usize, skeleton: u8) -> Option<(usize, u32)> {
        let dr = &self.drones.list[drone];
        let script = self.scripts.get(&dr.anim.script?)?;
        let t = (1 + dr.anim.time) as u16;
        let (a, _, q) = script.sequences().find(|(a, b, _)| *a <= t && t <= *b).or_else(|| script.sequences().next())?;
        let &(sk, i) = self.seq_index.get(&q).filter(|(sk, _)| *sk == skeleton)?;
        let _ = sk;
        Some((i, (t.saturating_sub(a) + 1) as u32))
    }
}

/// `FlashMod` (Check_Target 0x1a7810:95-97): `FlashMod = (int)GameState+0x76 · 0.01`, then
/// `((PS2Sinf(FlashMod·2π + π/2) + 1)·0.5 + 1)·0.5` — a 1 Hz sine shimmer in [0.5, 1.0], quantized
/// to the integer 100ths clock (the sine's phase steps in 0.01 s units). GameState+0x76 is
/// `psiGetTimeIn100ths()` = `((VBlankCount − RealTimeCount)·10) / 6` (INTEGER division, [M
/// 0x1094a0:8]), written every in-game flow pass [M GameFlow_Main 0x1c98c8:84]. The port feeds it
/// its tick count through the SAME integer formula (`frame·10/6`, 100ths at the fixed 60 Hz) —
/// the rate and the quantization are the game's; the phase carries the boot→level offset (the
/// game's clock runs from `psiInitTimeIn100ths` [M 0x109490:6], the port's from the level load —
/// a cosmetic shimmer phase, render-side only, no sim state).
pub fn flash_mod(frame: u32) -> f32 {
    let t = (frame * 10 / 6) as f32 * 0.01; // [M 0x1a7810:95: (int)clock100 · 0.01, clock100 = 0x1094a0:8]
    let s = (t * std::f32::consts::TAU + std::f32::consts::FRAC_PI_2).sin();
    ((s + 1.0) * 0.5 + 1.0) * 0.5
}

/// The game clock the AIMS blink and FlashMod read, in the game's integer 100ths
/// (`psiGetTimeIn100ths` 0x1094a0:8 — `((VBlankCount − RealTimeCount)·10)/6` — over the port's
/// fixed 60 Hz ticks; the phase offset vs the game's boot-relative clock is cosmetic, the rate
/// and quantization are exact [M GameFlow_Main 0x1c98c8:84 writes it every pass]).
fn clock_100ths(frame: u32) -> u32 { frame * 10 / 6 }

/// A rigid mesh hung on a character's datum (`AnimObjectDraw`): world = `QuatTransToMat(q, t)` · bone world.
/// Draw slots per gas mesh (the sim's pool holds `GasPool::CAP` = 128; a puff past this index doesn't draw).
pub const GAS_DRAW_SLOTS: usize = 64;
/// "DroneScan" [D 07000005.bin]: the Micro-Camera's per-drone target bracket mesh, and its draw slots.
pub const DRONE_SCAN: u32 = 0x0200_0832;
pub const DRONE_SCAN_SLOTS: usize = 16;

pub struct Attachment {
    pub character: u32,
    pub bone: usize,
    pub local: Mat4,
    /// `Some(rgb)`: the muzzle flash of `DroneWeap_FireWeapon` (mesh `weapon_data+0x4C`, light colour +0x56/+0x55/+0x54),
    /// drawn only on firing frames; `None`: the weapon itself.
    pub flash: Option<[u8; 3]>,
    /// `Some(slot)`: a bullet tracer (`GUN_BULLET_TRAIL` 0x02000123) showing bullet `slot` of the simulation.
    pub tracer: Option<usize>,
    /// `Some((slot, mesh))`: a projectile mesh slot (`Bullet_update` state 0 sets the weapon's `+0x5C` mesh on the
    /// bullet object, `View_SetDrawInAllViews`, disasm 0x12e8f0-0x12e900 [M]). The slot indexes the simulation's
    /// projectile lists (0..GRENADE_SLOTS grenades, then the drone and player bullets); the mesh selects which of
    /// the slot's per-mesh instances shows this frame. u16: GRENADE_SLOTS is the game's 512-entry per-view draw
    /// list (fix-hat-details), so the grenade range alone passes a u8.
    pub projectile: Option<(u16, u32)>,
    /// The character's gun once dropped (`DroneWeap_DropWeapon`): drawn at the pickup's matrix while it lies there.
    pub dropped: bool,
    /// A script entity (script hash, stream, entity hash): drawn at the simulation object's matrix while it exists.
    /// `(script, stream, entity hash, instance k)` — k picks the k-th live entity with that
    /// src/hash (several ScriptPlayers can run one script: 07000014's vent lids, nine SPs on
    /// 0x06000A3B).
    pub script: Option<(u32, u8, u32, u8)>,
    /// `Some(bone)`: one of the AIMS-20's four side lights (`Player_WeaponFiring` 0x1a5460:256-360
    /// attaches mesh 0x2000132 to the view model's bones 8..0xB every tick — a row of lit red
    /// lights). fix-gadgets-live; the blink/off arms live in `frame_state`.
    pub aims_light: Option<u8>,
    /// `Some(slot)`: a gas/smoke puff (`Gas_Init` 0x1aacc8) showing `sim.drones.gas.puffs[slot]` —
    /// a billboarded `smoke_1_dust` quad scaled by the puff's scale, faded by its rgba.
    pub gas: Option<(u16, u32)>,
    /// fix-impacts-render: an impact-effect draw slot — kind 0 a bullet decal (`Effect_Bullet`
    /// 0x1ce8a8's 200-node FIFO, `sim.drones.effects.decals[slot]`), kind 1 a shell casing
    /// (`Bullet_InitCasingLists` 0x12fd48's 32-object pool, `sim.drones.casings.list[slot]`),
    /// kind 2 a debris particle (`Debris_Create` 0x1aaa00, `sim.drones.effects.debris[slot]`).
    /// The u32 is the slot's mesh hash (one instance per distinct `EffectInfo`/`ammo_data` hash
    /// the level archive resolves, the `hashtable_set_sub_object_to_entity_gfx` mesh); the slot
    /// draws only while the live effect carries THAT mesh — free slots collapse far below the world.
    pub effect: Option<(u8, usize, u32)>,
    /// fix-silencer-visual follow-up: the silenced twins' +0x7C tube. `Player_WeaponFiring` 0x1a5460:96-110 clears
    /// datum 0 while the view anim is in state 8 (the swap/holster) — the swap anim screws its own part-bone 43
    /// piece on — so the tube hides during the swap (otherwise it looks already attached while being screwed on).
    pub silencer: bool,
    /// delta-repeater: one of the Delta Repeater's three loaded-bolt slots — the datum id 4/5/6
    /// the quiver arm attaches the bolt mesh (`weapon_data+0x5C` = 0x0200816) to, per the bolt
    /// count [M Player_WeaponFiring 0x1a5460:331-381]. The visibility rule lives in frame_state.
    pub quiver_bolt: Option<u8>,
    /// The fp skin's extra piece that duplicates the record's +0x7C suppressor (bone 43): drawn
    /// ONLY during the variant swap (state 8), where the game has cleared datum 0 and the swap
    /// anim's own piece is what gets screwed on [M 0x1a5460:96-110]; hidden otherwise so the mesh
    /// never draws twice (silencer-white-artifact's doubled sheen).
    pub swap_piece: bool,
    /// The wristwatch hand this attachment draws (0 hours, 1 minutes, 2 seconds) — `Player_SetWatch`
    /// 0x1a69d0 [M]: on the fp skin 0x0500004B it creates meshes 0x2000876/877/878 and each tick sets
    /// their rotation to `RotMatrixX(−ang − π/2)` concatenated onto view-model datum 2's world
    /// transform (`AnimGetBoneWorldTrans(fp, 2, …)` disasm 0x1a6b2c-0x1a6b44; `Concat` 0x1e4a90),
    /// at the datum's position (`Mat_SetTrans(matrix row 3)`), ang = fmod(t·rate, 2π) with t = the
    /// mission's elapsed centiseconds + the level's clock base [M :89-137; nf_game::watch].
    pub watch_hand: Option<u8>,
    /// `Some((datum, digit))`: the decryptor screen's digit mesh `PdaDigits[digit]` on view-model
    /// datum 0..3 — shown while the sim's `pda_display[3 − datum]` reads that digit
    /// (`Player_WeaponFiring` 0x1a5460:155-194 [M]).
    pub pda_digit: Option<(u8, u8)>,
}

/// Viewer switches that change what the game state would be.
#[derive(Clone, Copy, Default)]
pub struct ViewOpts {
    /// Also draw drones a spawner holds back at level start (docs/systems/drone-animation.md "Spawners").
    pub reinforcements: bool,
    /// Make every armed drone fire (muzzle flashes); the game's AI decides this, nf-view has none yet.
    pub firing: bool,
    /// The camera's screen axes this frame (corona billboards; set by the caller from its camera).
    pub right: [f32; 3],
    pub up: [f32; 3],
}

/// Bind-pose data of one character vertex (same order as the Character-layer vertices).
#[derive(Clone, Copy)]
pub struct CharVertex {
    pub character: u32,
    pub pos: [f32; 3],
    pub weight: f32,
    pub slots: [u16; 2],
    /// Rigid vertex of an attached mesh (index into `Scene::attachments`) instead of a skinned one.
    pub attachment: Option<u32>,
    /// A searchlight part's vertex (index into `Sim::drones.searchlights.parts`), posed by the simulation
    /// each frame (`Searchlight_Update`): the mount/head/beam/spot matrices, coronas as camera billboards.
    pub searchlight: Option<u16>,
    /// A door's vertex (index into `Sim::drones.doors.list`), posed by the simulation each frame
    /// (`Door_Interp`'s matrix at `obj+0x90`).
    pub door: Option<u16>,
    /// A level-placed pickup's vertex (index into `Sim::drones.pickups`), posed by the simulation each
    /// frame at the pickup's matrix (`View_DrawObjectGlist` 0x1e6780 draws `obj+0xD8` at `obj+0x90`);
    /// collapsed when taken/hidden. fix-player2 (the invisible armour).
    pub pickup: Option<u16>,
    /// fix-facial2: the vertex's morph base index — the skinning word's upper half-word (the short at
    /// +0x1E, −1 = none), which `SkinIt` 0x10bf30 indexes the morph targets with:
    /// `target + (morph + channel) * 12`.
    pub morph: i16,
}

/// Per-vertex GPU skinning data (`Scene::skin_layout`); world vertices carry `a = u32::MAX` (static).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinAttr { pub a: u32, pub b: u32, pub w: f32, pub col: u32,
    /// fix-gpu-facial-morphs (`SkinIt` 0x10bf30:90-116): the vertex's morph-target element base in
    /// the concatenated per-character target buffer (char target base + morph·3), −1 = no morph
    /// (non-character verts, no targets, facial count 0, or a negative morph index — the game's
    /// `*(short*)(vert+6) != -1` gate).
    pub mtb: i32,
    /// This character's facial-weight block base, in vec4s of the weight buffer (`mwb + m/4`,
    /// lane `m%4` — `AnimObjectDraw` 0x1c4490:147-152 marshals the weights from animobj+0xb0).
    pub mwb: u32,
    /// The character's facial channel count (the skin's +0x12 facial count — `Morph_Count`).
    pub cnt: u32,
    /// The character's target block end (exclusive element index) — the per-channel bound the
    /// game's count-bounded loop gives (`f32_at`'s `o + 12 > len` skip in `morph_vertex`).
    pub tend: u32 }

/// The palette / colour-table layout of `Scene::skin_layout`.
pub struct SkinLayout { pub char_base: Vec<u32>, pub attach_base: u32, pub sl_base: u32, pub door_base: u32, pub pick_base: u32, pub n_mats: u32, pub col_att: u32, pub col_pick: u32, pub col_sl: u32, pub col_one: u32,
    /// fix-gpu-facial-morphs: the per-character facial-weight block stride, in vec4s (the widest
    /// facial count across the characters, rounded up; 0 = the scene has no morph bearer).
    pub wt_stride: u32,
    /// Total morph-target f32 elements across the characters in character order (the GPU target
    /// buffer's size; equals `morph_target_blob().len() / 4`).
    pub n_tgt: u32 }

/// One frame's pose state (`Scene::frame_state`).
pub struct FrameState {
    pub posed: Vec<Option<(Vec<Mat4>, Vec<Mat4>, Mat4)>>,
    pub attach: Vec<Mat4>,
    pub hidden: Vec<bool>,
}

/// Dev-panel NPC pool size per skin.
pub const NPC_POOL: usize = 3;

/// One hidden pool NPC (`Scene::npc_pool`): its skin, the placement parameters it was copied from, and the
/// characters (meshes) that draw it.
#[derive(Clone)]
pub struct NpcSlot { pub key: [u32; 3], pub skin: u32, pub params: Vec<(u32, u32)>, pub chars: Vec<usize> }

/// One MP body (mp-character-data): the third-person character one MP slot drives — a human
/// (slots 0..3) or a bot (4..7). The data is the game's own tables: `Player_Init` 0x1910f8 builds
/// the body's anim object from `MP_skins[MPSettings+slot*0x30+0x24]` [M 0x1910f8:108-117] and the
/// body runs the descriptor set `PlayerAnimSetInitNormal`/`InitCrouch` pick per weapon class
/// [M 0x18fcd8/0x18fea0]. Parked hidden (the meshes sit at the sentinel translation, below the
/// world) until the draw moves them — `frame_state_for`'s hidden pass skips a parked body.
pub struct MpBody {
    /// The MP slot (0..3 human, 4..7 bot).
    pub slot: usize,
    /// `MPSettings+slot*0x30+0x24`: the MP character index (the `MP_skins` row).
    pub char_index: u32,
    /// `MP_skins[char_index]+0x00`: the skin (anim object) hash [M 0x1913d4].
    pub skin: u32,
    /// `MP_skins[char_index]+0x04`: the skin's DATA file hash [M 0x120ca0].
    pub file: u32,
    /// `MP_skins[char_index]+0x08`: the sleeve variant (→ `BLData+0x965`, the first-person arms
    /// when this slot is the local view) [M 0x1913e0].
    pub sleeve: u32,
    /// The skin's skeleton id — `Scene::sequences`' bucket key and `mp_body_pose`'s draw filter.
    pub skeleton: u8,
    /// The parked characters (one per main mesh, in the skin's `main` order) — hidden until the
    /// draw moves their matrices.
    pub chars: Vec<usize>,
    /// The body's sequence set: every anim SCRIPT id the descriptors/transitions/death can put on
    /// the body (`MpAnimSets::all_body_scripts`) resolved through the arena's kind-5 scripts →
    /// `(first script frame, last script frame, index into Scene::sequences[skeleton])` — the
    /// `StartAnim` resolution. Scripts the arena doesn't carry are absent.
    pub scripts: HashMap<u32, Vec<(u16, u16, usize)>>,
}

pub struct Scene {
    /// Hidden NPC copies the dev panel can bring to life (`dev::spawn_npc`).
    pub npc_pool: Vec<NpcSlot>,
    /// The MP bodies (mp-character-data), one per human slot and per bot — parked hidden until the
    /// draw moves them (mp-character-data's loader API; the pose comes from
    /// `nf_formats::mpanim::mp_body_pose` over `sim.mp`'s live slots).
    pub mp_char_bodies: Vec<MpBody>,
    /// mp-character-data: the animation-set descriptors (ACTION.ELF 0x2b91b0) the drawn MP bodies
    /// pick their script from each frame (`nf_formats::mpanim::mp_body_pose`).
    pub mp_anim_sets: Option<nf_formats::mpanim::MpAnimSets>,
    /// The level's room cels (`parse_room_cels`), kept for the static-draw cel cull
    /// (`View_AddCels` 0x1e6ba0 frustum-tests each cel's bbox; `cel_sphere`/`cel_index_at`).
    pub cels: Vec<Cel>,
    /// The runtime cel graph + portals (map block 0x21) — `portal_visible_cels`' input (fix-perf-level,
    /// G-016's portal half). `.room_to_cell` maps a `cels` index into `cells`.
    pub cel_graph: CelGraph,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub materials: Vec<Material>,
    pub textures: Vec<TextureImage>,
    /// The first-person gun (`Player_SetWeaponAnim`: `AnimObjectNew(G, weapon+0xDC)`): one character per owned
    /// weapon id, built at scene build time. The active one is selected each frame in `animate` from
    /// `sim.gun.weapon`.
    pub view_models: HashMap<u8, usize>,
    /// The MP bodies: (player slot, character index) — see `Scene::build`'s MP-body block.
    pub mp_bodies: Vec<(usize, usize)>,
    /// cable-3p: Bond's own third-person body character (single player) - drawn only while a
    /// third-person camera mode runs (`Sim::wire_camera`).
    pub bond_body: Option<usize>,
    /// The MP bodies' weapon attachments: attachment index → (slot, weapon id).
    pub mp_body_weapons: HashMap<u32, (usize, u8)>,
    /// The snow flake texture (`DrawDrops`: 0x03000045).
    pub snow_texture: Option<TextureImage>,
    /// Streamed effects: `STREAMS.LUT` entries and `STREAMS.BIN` (set by the loader).
    pub streams: Option<(Vec<[u32; 4]>, Vec<u8>)>,
    /// The level's music (track, SMF, SSD) and the music tables (set by the loader).
    pub music_files: Option<(i32, nf_formats::music::Smf, Vec<u8>)>,
    pub music_tables: Option<(Vec<nf_formats::music::MapMusic>, Vec<nf_formats::music::MusicVars>)>,
    /// The level's sound bank (`Sound_Ready` → `SFXLoadSoundBank`): slot, bank, ADPCM (set by the loader).
    pub sound_bank: Option<(usize, nf_formats::soundbank::Bank, Vec<u8>)>,
    /// Spare banks (fix-weapon-sounds): the other SBINFO slots' `SB_<slot>.{SHF,SFX,SBF}`, loaded so a
    /// weapon given on a level whose own bank lacks its sounds still plays them from a bank that holds
    /// them (`load_spare_bank`). The level's own bank stays the primary lookup [M `Sound_Ready`].
    pub sound_spares: Vec<(usize, nf_formats::soundbank::Bank, Vec<u8>)>,
    /// The level's fog (`psiPreGameRun` 0x11e2e8: colour, `Ps2Far1`; `None` = `FogOn` 0).
    pub fog: Option<([u8; 3], f32)>,
    /// The HUD's textures by hash (docs/systems/hud.md), from any map of the archive, and its tables from ACTION.ELF.
    pub hud_textures: Vec<(u32, TextureImage)>,
    /// break-shards draw: a kind-2 breakable's face draw word (its batch texture) → the index into
    /// `textures` of `(the pane object's map, word)`, so a shard binds the pane's own texture.
    pub shard_tex: HashMap<u32, usize>,
    /// The pause/fail menus' burning woman (`psiDecompressWoman` 0x1202a8): decoded into 0x030000db's HUD texture
    /// while a paused page shows it. Set by the loader from the level archive's kind-16 "woman" entry.
    pub woman: Option<crate::woman::Woman>,
    pub hud: Option<nf_formats::hud::HudTables>,
    /// Script-entity meshes whose glist flags (`celglist+0x44` → `obj+0xF0` via `Control_SetGList`
    /// 0x134960) carry the billboard bit 0x8: `View_DrawObjectGlist` 0x1e6780:43-65 draws them
    /// camera-facing — the drawn rotation is the VIEWER's, with the interp's translation and the
    /// interp scale (`obj+0xE8`, the key scale × the script scale, e.g. the blast radius). The
    /// explosion script 0x0600052's puffs are such meshes; drawn with the raw interp matrix they
    /// sit in their authored plane and read edge-on as a very thin wall.
    /// Collected at build time (the parsed level is not kept on the Scene).
    pub script_billboards: std::collections::HashSet<u32>,
    pub bounds: ([f32; 3], [f32; 3]),
    /// Camera start: the first placement whose object is named "Player1" (translation, yaw), if any.
    pub start: Option<([f32; 3], f32)>,
    pub stats: String,
    pub characters: Vec<Character>,
    /// Index of the first Character-layer vertex; `char_verts` describe that tail of `vertices`.
    pub char_vertex_start: usize,
    pub char_verts: Vec<CharVertex>,
    /// Sequences of the level's animation archive (`07F000NN.bin`), by skeleton id.
    pub sequences: HashMap<u8, Vec<Sequence>>,
    /// Weapons and props hung on character datums; their vertices are Character-layer vertices with `attachment`.
    pub attachments: Vec<Attachment>,
    /// The guard AI, when the drone context was available.
    pub sim: Option<Sim>,
    /// How many ladder (0x22) placements got a World-layer instance (fix-ladder2's draw sweep).
    /// The game draws each ladder exactly once through the dynamic-object path [M View_DrawObjList
    /// 0x1e69a0 → View_DrawObjectGlist 0x1e6780]; 0 means the ladders are invisible.
    /// The 0x2E locks' placement matrices (`Lock::create` order): where the keypads' posed slots
    /// draw the placed object or its LOD variant.
    pub lock_matrices: Vec<Mat4>,
    /// The 0x3B gated dynamic objects' placement matrices (`WorldObjects::create` order — every
    /// 0x3b placement, placement order): where the posed slots draw them, collapsed once the sim
    /// deletes the object (`DynamicObject_Update` 0x18d3b8:26 mode-0 armed-B → obj+0xFE bit 0 →
    /// the delete pass; 0700001b's control-room glass dies with ch 10, Drake's gate).
    pub dynobj_matrices: Vec<Mat4>,
    /// The rotors (0x71, `WorldObjects::create`'s order — every 0x71 placement): the placement
    /// rotation and translation. `rotor_init` 0x18f4b8 creates the object at the placement ROT
    /// (`control_create_object(0x20, pos, rot)`) and `rotor_update` 0x18f568 adds ROT·FRAME_RATE_MUL
    /// to obj+0x50 each spinning tick, so the drawn rotation is the placement ROT + the sim's angle.
    pub rotor_placements: Vec<([f32; 3], [f32; 3])>,
    /// The 0x34 placements the build gave posed slots (the camera slots follow them; the sim's
    /// gunimp list may be emptied by the MP Gun Emplacements setting, the slot layout is not).
    pub gunimp_placements: usize,
    pub ladders_drawn: usize,
    /// How many grapple-point (0x3A) placements got a World-layer instance (fix-grapple-hook's
    /// draw sweep). The game draws each point's pulley mesh exactly once through the dynamic-object
    /// path [M View_DrawObjList 0x1e69a0 → View_DrawObjectGlist 0x1e6780]; 0 on a level with 0x3A
    /// placements means the hooks are invisible.
    pub grapple_hooks_drawn: usize,
    /// Models-misc: how many static dynamic entities got a World-layer instance, per placement kind
    /// (`misc_static_sweep` in `build`). Zero for a kind the census says the level carries means the
    /// model is missing (see docs/models/misc.md for the per-kind creator evidence).
    pub misc_drawn: BTreeMap<u8, usize>,
    /// The drawn 0xe1 destroy leaves' (world centre, destroy placement) pairs — the per-frame
    /// dead gate's input (`destroyed-glist swap's draw side`: a dead leaf's materials stop
    /// drawing [M Destroy_Smash 0x12bb60's SetGList swap]; the C5 leaves swap to an undrawable
    /// slot [D lod_word −1] and the doorway ends open [L]). Consumed by lib.rs's per-frame
    /// `Renderer::destroy_dead` feed; the centre matches the material's cull sphere by bits.
    pub destroy_centres: Vec<([f32; 3], usize)>,
    /// Per level object hashcode: whether its glist flags carry 0x800 (the character-lighting bit
    /// `Control_SetGList` 0x134960 merges into obj+0xF0). fix-render2: decides the ambient dim for
    /// the drawn script entities (the truck's nine objects carry 0x8880 [L ee_start_05.bin
    /// 0x0137C8A0…, script-player.md]); pickups are gated by their kind instead (Pickup_Create
    /// 0x18dda0 forces 0x840, clearing it for kinds 3 and 6).
    pub models_0x800: HashMap<u32, bool>,
    /// Bug 3 diagnostic: objects with collision, for the "name what I'm touching" overlay (`C` key). Each entry is
    /// the object's name and its world-space AABB (lo, hi), built from the instance matrix + the object's bounds.
    pub colliders: Vec<(String, [f32; 3], [f32; 3])>,
    /// cutscene-door-dup: the main-map door placements whose object's glist flags (celglist+0x44 =
    /// MapObject.flags) carry 0x40000 — `Control_SetGList` 0x134960 clears obj+0xF8's bit 0x10 for
    /// such objects, and `Script_CameraStart` 0x1d9140 renders ONLY camera 4 while a scene runs
    /// (viewer+0x28 = 4 → object_display_mask 0x10 [M View_CaptureScene 0x1e6df0 / build_alloc_viewer
    /// 0x1c9378]), so both draw walks (View_AddObjects 0x1e62d8:33, View_AddForcedObjects
    /// 0x1e5fb8:79) skip them: the placed doors are invisible during a cut-scene. Keyed by
    /// placement index; the palette pass collapses their slots while the script camera renders.
    pub door_scene_hidden: std::collections::HashSet<usize>,
    /// Searchlight LOD slots: slot `parts.len() + k` draws the LOD (broken) mesh of part `sl_lod[k]`,
    /// shown iff that part's `lod_next` (Control_NextLOD) is set.
    pub sl_lod: Vec<u16>,
}

/// `psiPreGameRun` (0x11e2e8, read): per level the fog colour and `Ps2Far1`; `FogOn` only on these levels.
pub fn fog_for(level: u32) -> Option<([u8; 3], f32)> {
    Some(match level {
        0x0700_0002 => ([0x24, 0x2e, 0x4c], 9_987_520.0),
        0x0700_0004 => ([0x11, 0x15, 0x21], 9_379_776.0),
        0x0700_0005 => ([0x3a, 0x4d, 0x68], 19_118_528.0),
        0x0700_0006 => ([0x2e, 0x3c, 0x52], 8_059_232.0),
        0x0700_0008 => ([0x3b, 0x50, 0x69], 13_191_350.0),
        0x0700_000c => ([0xd1, 0xd5, 0xd6], 20_475_840.0),
        0x0700_000d => ([0x92, 0x85, 0x69], 19_707_840.0),
        _ => return None,
    })
}

/// Grouped by layer, then (world layers) the object — the game sorts cels per object (`View_AddCels`) — then texture
/// and draw state. The value keeps the object's world centre and scaled radius for the per-frame sort + frustum cull.
pub type GroupKey = (Layer, [u32; 3], u32, u32, Option<(usize, u32)>, Draw, u16);
/// The group key's slot-2 tag for a per-character group (see `add_instance`'s `ikey`).
const SKIN_TAG: u32 = 0x5c1d_0000;
type Groups = BTreeMap<GroupKey, (Vec<Vertex>, Vec<u32>, Vec<CharVertex>, [f32; 3], f32, Option<u16>, u32)>;

/// ── Pool-slot replay (level-load-time) ───────────────────────────────────────────────────────────
/// The decal/casing/debris/gas/projectile/tracer pools call `add_instance` once per pool slot with
/// IDENTICAL geometry (same object, same identity matrix, same layer and character pattern — only
/// the attachment index and the pushed `Attachment` differ). Slots 1..N therefore replay slot 0's
/// group pushes bit-exactly instead of re-transforming every vertex: `pool_record` snapshots what
/// the slot-0 call grew, `pool_replay` re-pushes it. Every replayed value is the same the original
/// per-slot call would have written: the vertex/CharVertex data is a pure function of the identical
/// (object, matrix, mult) inputs; indices are re-based on the current group length (the original
/// `base` was the group length at push time, and replays append the same vertex count each time);
/// centre/cel/node are constants of the pool; the radius update is a max, and the record's r is
/// exactly max(glist radius, all batch bounds) of the same geometry, so
/// `r_n = max(r_{n-1}, r_record)` equals the original `r.max(glist).max(batch_d2.sqrt())` chain.
type PoolRecord = Vec<(GroupKey, Vec<Vertex>, Vec<u32>, Vec<CharVertex>, [f32; 3], f32, Option<u16>, u32)>;

/// The group lens before a slot-0 `add_instance` call (for `pool_record`'s diff).
fn pool_snapshot(groups: &Groups) -> std::collections::BTreeMap<GroupKey, (usize, usize, usize)> {
    groups.iter().map(|(k, v)| (k.clone(), (v.0.len(), v.1.len(), v.2.len()))).collect()
}
/// After the slot-0 call: what it pushed, per touched group (indices re-based to vertex offsets,
/// i.e. relative to the base the slot-0 call used = the recorded pre-call vertex count).
fn pool_record(groups: &Groups, before: &std::collections::BTreeMap<GroupKey, (usize, usize, usize)>) -> PoolRecord {
    groups.iter().filter_map(|(k, (vs, is, cv, centre, r, cel, node))| {
        let &(l0, l1, l2) = before.get(k).unwrap_or(&(0, 0, 0)); // absent from `before` = created by slot 0 (base 0)
        let base = l0 as u32;
        let vsn = &vs[l0.min(vs.len())..];
        let isn: Vec<u32> = is[l1.min(is.len())..].iter().map(|i| i - base).collect();
        let cvn = &cv[l2.min(cv.len())..];
        if vsn.is_empty() && isn.is_empty() && cvn.is_empty() { return None; } // untouched by slot 0
        Some((k.clone(), vsn.to_vec(), isn, cvn.to_vec(), *centre, *r, *cel, *node))
    }).collect()
}
/// Replays a slot's contribution for pool slots 1..N. `attachment` = the slot's own attachment
/// index (the only per-slot difference in the CharVertex stream).
fn pool_replay(groups: &mut Groups, rec: &PoolRecord, attachment: Option<u32>) {
    for (k, vs, is, cv, centre, r, cel, node) in rec {
        let e = groups.entry(k.clone()).or_default();
        let base = e.0.len() as u32;
        e.0.extend_from_slice(vs);
        e.1.extend(is.iter().map(|i| i + base));
        e.2.extend(cv.iter().map(|c| { let mut c = *c; if c.attachment.is_some() { c.attachment = attachment; } c }));
        e.3 = *centre;
        e.4 = e.4.max(*r);
        e.5 = *cel;
        e.6 = *node;
    }
}

/// fix-safe08: the pickups the level's scripts spawn through `Script_CreateEntity` 0x1d8968's
/// obj_type-0xF0 arm (`Pickup_Create` 0x18dda0 at the stream's posed position, rest = 1 [M disasm
/// 0x1d8bcc-0x1d8c20]). Each such (script, stream) gets a RESERVED pickup slot: built inert
/// (`gone`, so the draw loop and the take test skip it) and activated by the sim when the script
/// actually spawns it. The slot keeps the renderer's per-index draw (`placed_pickup_pose`) working
/// for a pickup that does not exist at load. 07000008's SafePickUps script 0x060009E2 spawns the
/// safe's briefcase (kind 4, switch 0x4A) and the rocket launcher (id 0x2C) at the safe's shelves
/// this way [D the level-bin script; the parked 0xF0 placements sit 10 units east, inside the east
/// wall]. Scans `level.nis` sorted by hashcode (deterministic); pack scripts (07x00008) are not in
/// the level archive and fall back to the runtime's no-slot push (takeable, by-hash mesh).
fn script_pickup_reservations(level: &Level, weapons: &nf_formats::weapon::WeaponData, models: &std::collections::HashMap<u32, nf_game::pickup::Model>, level_id: u32, has_gold_medal: bool) -> (Vec<nf_game::pickup::Pickup>, Vec<((u32, u8), usize)>) {
    let mut out = Vec::new();
    let mut slots = Vec::new();
    let mut hashes: Vec<u32> = level.nis.keys().copied().collect();
    hashes.sort_unstable();
    for hash in hashes {
        let nis = &level.nis[&hash];
        for (si, st) in nis.streams.iter().enumerate() {
            let cmds = st.decode();
            let Some(nf_formats::nis::Cmd::Entity { hash: eh, obj_type: 0xf0, params, .. }) = cmds.iter().find(|c| matches!(c, nf_formats::nis::Cmd::Entity { obj_type: 0xf0, .. })) else { continue };
            let p = |k: usize| params.get(k).copied().unwrap_or(0);
            let target = level.by_hash.get(eh).copied();
            let Some((om, oi)) = target else { continue };
            let obj_ = &level.maps[om].objects[oi];
            let Some(&m0) = models.get(&obj_.hashcode) else { continue };
            let key = nis.keys.get(st.first_key as usize).map(|k| k.pos).unwrap_or([0.0; 4]);
            let Some(mut pk) = nf_game::pickup::Pickup::create_placed(
                [key[0], key[1], key[2]], [0.0; 3], p(0) as u16, p(1) as u16,
                nf_game::pickup::Pickup::create_rounds_override(level_id, p(1) as u16, p(2) as u16 as i16),
                p(3) as u16, p(4) as u16, p(5) as u16, 0, obj_.hashcode,
                nf_game::pickup::Model { centre: obj_.centre, radius: obj_.radius }, weapons, models, has_gold_medal) else { continue };
            pk.label = 0; // [M 0x1d8c1c delay slot `sw $zero, 0x20($sp)`]: the SCRIPT arm passes label 0 —
                          // unlike the placements, Script_CreateEntity never reads record+0x44 (param 6).
                          // pickups-g029: the old p(6) stood in for the placement rule.
            pk.gone = true; // reserved: inert until the script spawns it
            pk.obj = Some((om, oi));
            slots.push(((hash, si as u8), out.len()));
            out.push(pk);
        }
    }
    (out, slots)
}

/// fix-facial2: `SkinIt` 0x10bf30's morph term — `pos += Σ_m weight[m] × target(base + (morph + m) × 12)`,
/// the 12 bytes read as three f32 (the decompile's `auVar14._0_12_ = *(undefined1 (*) [12])lVar1` over the
/// base `parsemap_block_morph_data` 0x1d23e8 stores at the gfx record's +0x14). Out-of-range channels (a
/// target array shorter than the mesh's index space) contribute nothing, as the game's count-bounded loop.
fn morph_vertex(mut pos: [f32; 3], morph: i16, weights: &[f32], targets: &[u8]) -> [f32; 3] {
    let f32_at = |i: usize| -> Option<[f32; 3]> {
        let o = i * 12;
        if o + 12 > targets.len() { return None; }
        Some([
            f32::from_le_bytes(targets[o..o + 4].try_into().unwrap()),
            f32::from_le_bytes(targets[o + 4..o + 8].try_into().unwrap()),
            f32::from_le_bytes(targets[o + 8..o + 12].try_into().unwrap()),
        ])
    };
    for (m, w) in weights.iter().enumerate() {
        if let Some(d) = f32_at(morph as usize + m) {
            for k in 0..3 { pos[k] += w * d[k]; }
        }
    }
    pos
}

fn add_instance(level: &Level, inst: &Instance, layer: Layer, character: Option<(u32, Option<u32>)>, groups: &mut Groups, bounds: Option<&mut ([f32; 3], [f32; 3])>, searchlight: Option<u16>, door: Option<u16>, pickup: Option<u16>, mult: [f32; 3], cels: &[Cel], chain: u32, node: u32) {
    let obj = &level.maps[inst.map].objects[inst.object];
    let mut m = inst.matrix;
    if layer == Layer::SkyCamera { m[3] = [0.0, 0.0, 0.0, 1.0]; }
    let mut bounds = bounds;
    // The pass and fog by layer: sky slots with placement parameter 4 = 1 go before the world (fogged), others after
    // the fog pass. World: the static cel primaries (chain set) draw in list 0 regardless of glist flags —
    // `View_AddCels` (0x1e6ba0) pushes the primary into the list-0 ring without consulting flags bit 0x1 (only the
    // +0x18 chain records and `View_AddObjects`' objects pick their list by flags). Dynamic World objects
    // (chain == u32::MAX) keep the flag-0x1 alpha list (pass 3, back-to-front, `View_AddObjects` 0x1e62d8).
    let param4 = inst.placement.params.iter().find(|(i, _)| *i == 4).map(|&(_, v)| v).unwrap_or(0);
    let (pass, fog) = match layer {
        Layer::SkyCamera | Layer::SkyFixed => if param4 == 1 { (0, 1) } else { (4, 0) },
        Layer::World => if chain == u32::MAX && obj.flags & 1 != 0 { (3, 2) } else { (1, 1) },
        Layer::Character => (2, 1),
        // `View_AddObjects` 0x1e62d8:57-59 routes an object with glist flag 0x2000 to sort list 2, which
        // `Game_Draw` 0x1e6f50:162-175 draws after `psiClearZ` — over the whole world. Only the Micro-Camera's
        // DroneScan (0x02000832, flags 0x2128 [D]) takes it here; the other 0x2000 effect meshes are unverified.
        Layer::Effect => if obj.hashcode == DRONE_SCAN && obj.flags & 0x2000 != 0 { (6, 0) } else { (5, 0) },
    };
    // view-model-last: the first-person gun and its flash are `Game_Draw` 0x1e6f50:162-175's list 2, drawn
    // after `psiClearZ` — over everything the world and its effects drew [M]. Pass 7 draws after every
    // other pass with its depth squashed toward the eye (gpu.rs `VMZ`) — the port's architecture for the game's
    // `psiClearZ` before list 2 (the renderer cannot clear mid-frame), measured against PCSX2 08 side by side,
    // 2026-10-17: the gun still sorts its own parts, and a rocket spawned between the eye and the tube no longer
    // paints over it.
    let pass = if node == VIEW_MODEL_NODE && matches!(layer, Layer::Character | Layer::Effect) { 7 } else { pass };
    // The object's world centre (glist sphere centre through the instance matrix) for the draw-order sort; world layers
    // keep each object apart. The world-space bounding-sphere radius (glist radius × the instance matrix's max axis
    // scale) feeds the CPU-side frustum cull (A-11e) — `transform` uses row-vector convention, so the columns of the
    // upper-left 3×3 are the basis vectors and their lengths are the per-axis scales. `object_matrix` is a pure
    // rotation+translation (column lengths = 1), so most world placements keep `obj.radius` unchanged.
    let centre = transform(&inst.matrix, obj.centre);
    // CompAlphaObj 0x1d3458 [M] — QuickSort comparator id 4, View_DrawObjList 0x1e69a0:24's
    // per-object draw key (f32 at +0 = the world-centre distance, ascending); this pass/layer
    // + centre model replaces the game's alpha-object ordering.
    let col_len = |c: usize| (inst.matrix[0][c] * inst.matrix[0][c] + inst.matrix[1][c] * inst.matrix[1][c] + inst.matrix[2][c] * inst.matrix[2][c]).sqrt();
    let radius = obj.radius * col_len(0).max(col_len(1)).max(col_len(2));
    // The light-selection flags: the glist flags word's low 16 bits — `Control_SetGList` 0x134960
    // copies `celglist+0x44` to `obj+0xF0` and `View_DrawObjList` 0x1e69a0 / `View_DrawCelGlist`
    // 0x1e6538 / `View_SetupRenderModes` 0x1e78b0 (the anim-object path) pass its u16 to
    // `Lights_CalcClosestLights`. `MapObject.flags` is that word [D level.rs ENTITY_PARAMS +4].
    let lflags = (obj.flags & 0xffff) as u16;
    // World layers key each object apart (centre). Characters and their attachments key per character
    // (and per attachment slot): the game picks the ≤2 lights PER OBJECT at the object's own world sphere
    // [M View_DrawObjList 0x1e69a0:73-81 → Lights_CalcClosestLights(obj+0x80, obj+0xF0)]; the shared
    // `[0; 3]` key merged every character with the same texture into ONE draw lit at ONE centre (the
    // last instance's — the bind-time origin for script actors), so cut-scene faces got no point lights
    // (flat-lit cut-scene characters). The tag SKIN_TAG in
    // slot 2 lets the material build recover the (character, attachment) pair.
    let ikey = if matches!(layer, Layer::World) { centre.map(f32::to_bits) }
        // Keyed per CHARACTER only: an attachment (the held gun) lights with its owner, and the pools that
        // share a character index across many slots (projectiles/debris) keep merging — keying per slot
        // multiplied the draw count (07000002: 4540 → 5927 materials) for no lighting gain.
        else if let (Layer::Character, Some((ci, _))) = (layer, character) { [ci, 0, SKIN_TAG] }
        else { [0; 3] };
    // GS state: each leaf's `DIRECT` writes, inherited by the leaves that send none. The GS applies each DIRECT
    // write immediately at its position in the packet, so a write between batch 2 and batch 3 affects batch 3
    // onward — we apply the writes per-batch, not per-leaf (the old per-leaf approach made the sniper's gun body
    // transparent because the leaf's final state was additive for the trailing reflection batches).
    let (mut alpha, mut test, mut zbuf) = (0x44u64, 0x5001bu64, 0x3100_0000u64);
    for (li, leaf) in obj.gfx.leaves.iter().enumerate() {
    let mut gs_idx = 0usize;
    for (bi, b) in leaf.batches.iter().enumerate() {
        // Apply DIRECT writes that take effect before/at this batch.
        while gs_idx < leaf.gs.len() && leaf.gs[gs_idx].2 <= bi {
            let (r, v, _) = leaf.gs[gs_idx];
            match r { 0x42 => alpha = v, 0x47 => test = v, 0x4e => zbuf = v, _ => {} }
            gs_idx += 1;
        }
        let draw = Draw::from_gs(pass, fog, alpha, test, zbuf);
        let lkey = if matches!(layer, Layer::World) { (li as u32) << 16 | bi as u32 } else if node == VIEW_MODEL_NODE { 1 } else { 0 };
        let key = b.texture.filter(|&t| (t as usize) < level.maps[inst.map].textures.len()).map(|t| (inst.map, t));
        // fix-render3: the room cel for the static World layer (build_LinkToRoom 0x1c9680 →
        // build_FindCel 0x1c8cb0 on the object's world centre; View_AddCels 0x1e6ba0 frustum-tests
        // each cel's bbox per viewer). Everything posed per frame keeps None.
        let cel = if layer == Layer::World { find_cel(cels, centre).map(|i| i as u16) } else { None };
        let (vs, is, cv, c, r, gcel, gnode) = groups.entry((layer, ikey, lkey, chain, key, draw, lflags)).or_default();
        *gcel = cel;
        *gnode = node;
        *c = centre;
        let base = vs.len() as u32;
        // fix-render-artifacts: the A-11e sphere cull must be SOUND. fix-render2 assumed the placement-matrix
        // glist sphere is exact, but the glist radius can undershoot the drawn geometry [D 0700000c.bin: the
        // floor decal at glist centre (28.54, 0.33, −182.49) carries r 0.6 while its drawn verts span a
        // 1.1×0.9×1.0 box — half-diagonal 0.87]. A too-small sphere drops draws whose geometry IS in the
        // frustum (pale fog-coloured patches: the skipped quad uncovers the fog-coloured clear).
        // The game's own cull consults the bbox as well as the sphere (`View_AddObjects` 0x1e62d8 passes
        // obj+0x8c sphere AND obj+0x80 bbox to `Vision_InView` 0x1e89e0), so the group's real bound belongs
        // in the test: every batch expands its group's radius to cover its transformed verts.
        let mut batch_d2 = 0.0f32;
        for v in &b.verts {
            let p = transform(&m, v.pos);
            let d2 = (p[0] - centre[0]).powi(2) + (p[1] - centre[1]).powi(2) + (p[2] - centre[2]).powi(2);
            if d2.is_finite() && d2 > batch_d2 { batch_d2 = d2; }
            if let Some((lo, hi)) = bounds.as_deref_mut() {
                if p.iter().all(|c| c.is_finite()) {
                    for k in 0..3 { lo[k] = lo[k].min(p[k]); hi[k] = hi[k].max(p[k]); }
                }
            }
            let uv = if v.uv.iter().all(|c| c.is_finite()) { v.uv } else { [0.0, 0.0] };
            // The per-instance colour multiplier: the game's VU1 `TAmbient` term (`psiSetTweakARGB` 0x124770,
            // `View_SetupRenderModes_Tweak` 0x1e5888) multiplies the vertex colour of 0x800 objects by
            // tint × obj+0x100..0x102 >> 8 / 255. Applied here on the vertex bytes (the GS modulate is the
            // same multiply); alpha byte obj+0x106 is passed as-is (white tint → untouched).
            let rgba = if mult == [1.0; 3] { v.rgba } else {
                let rgb = [0, 1, 2].map(|k| ((v.rgba[k] as f32 * mult[k]).round() as i32).clamp(0, 255) as u8);
                [rgb[0], rgb[1], rgb[2], v.rgba[3]]
            };
            // The normal: posed vertices keep the mesh's own (the shader rotates it with the palette); static
            // World vertices get it in world space through the instance matrix here.
            let posed = character.is_some() || searchlight.is_some() || door.is_some() || pickup.is_some();
            let n0 = v.normal.map(|c| c as f32);
            let n = if posed { n0 } else { [0, 1, 2].map(|k| n0[0] * m[0][k] + n0[1] * m[1][k] + n0[2] * m[2][k]) };
            let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let nrm = if nl > 1e-6 { [(n[0] / nl * 127.0) as i8, (n[1] / nl * 127.0) as i8, (n[2] / nl * 127.0) as i8, 0] } else { [0; 4] };
            vs.push(Vertex { pos: p, uv, rgba, nrm });
            if let Some((c, attachment)) = character { cv.push(CharVertex { character: c, pos: v.pos, weight: v.weight, slots: v.slots, attachment, searchlight: None, door: None, pickup: None, morph: (v.w >> 16) as i16 }); }
            else if let Some(sl) = searchlight { cv.push(CharVertex { character: 0, pos: v.pos, weight: v.weight, slots: v.slots, attachment: None, searchlight: Some(sl), door: None, pickup: None, morph: -1 }); }
            else if let Some(dr) = door { cv.push(CharVertex { character: 0, pos: v.pos, weight: v.weight, slots: v.slots, attachment: None, searchlight: None, door: Some(dr), pickup: None, morph: -1 }); }
            else if let Some(pk) = pickup { cv.push(CharVertex { character: 0, pos: v.pos, weight: v.weight, slots: v.slots, attachment: None, searchlight: None, door: None, pickup: Some(pk), morph: -1 }); }
        }
        // The batch's bound joins the group's (max across the group's batches — the entry's radius persists).
        *r = radius.max(*r).max(batch_d2.sqrt());
        for t in b.triangles() { is.extend(t.map(|k| base + k as u32)); }
    }
    // Apply trailing DIRECT writes (after the leaf's last batch) so the state carries to the next leaf.
    while gs_idx < leaf.gs.len() {
        let (r, v, _) = leaf.gs[gs_idx];
        match r { 0x42 => alpha = v, 0x47 => test = v, 0x4e => zbuf = v, _ => {} }
        gs_idx += 1;
    }
    }
}

/// Height of the highest World triangle straight below `p` (vertical ray from 0.5 above it), within 3 units.
/// In 07000005 the drone placements sit 1.0845 above this floor (10 of 16; the rest stand on steps or slopes).
fn floor_below(vertices: &[Vertex], indices: &[u32], materials: &[Material], p: [f32; 3]) -> Option<f32> {
    let [px, py, pz] = p;
    let mut best: Option<f32> = None;
    for m in materials.iter().filter(|m| m.layer == Layer::World) {
        for t in indices[m.first_index as usize..(m.first_index + m.index_count) as usize].chunks(3) {
            let [a, b, d] = [0, 1, 2].map(|k| vertices[t[k] as usize].pos);
            if a[1].min(b[1]).min(d[1]) > py + 0.5 || a[1].max(b[1]).max(d[1]) < py - 3.0 { continue; }
            // Barycentric test in the xz plane, then the plane height at (px, pz).
            let det = (b[2] - d[2]) * (a[0] - d[0]) + (d[0] - b[0]) * (a[2] - d[2]);
            if det.abs() < 1e-9 { continue; }
            let l1 = ((b[2] - d[2]) * (px - d[0]) + (d[0] - b[0]) * (pz - d[2])) / det;
            let l2 = ((d[2] - a[2]) * (px - d[0]) + (a[0] - d[0]) * (pz - d[2])) / det;
            let l3 = 1.0 - l1 - l2;
            if l1 < 0.0 || l2 < 0.0 || l3 < 0.0 { continue; }
            let y = l1 * a[1] + l2 * b[1] + l3 * d[1];
            if y <= py + 0.5 && y >= py - 3.0 && best.is_none_or(|bb| y > bb) { best = Some(y); }
        }
    }
    best
}

/// `DroneWeap_FireWeapon` (0x178f68): the flash object sits at the weapon datum's world position; its rotation is
/// the spherical angles of the datum's −x axis (`Vec_Cartesian_2_Spherical`: atan2(y, |xz|), atan2(x, z), length)
/// plus `Rand_FRand(2π)` on x, `Rand_FRandHalf(0.2) + π/2` on y and `Rand_FRandHalf(0.2)` on z; matrix as for
/// placements (`RotMatrix`). `seed` stands in for the game's random state.
/// `Draw_MuzzleFlash` for Bond: local +Z along the muzzle datum's +Z, a random roll, scale 0.9 + rand(0.2).
fn player_flash_matrix(datum: &Mat4, seed: u32) -> Mat4 {
    let mut s = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let mut rand = || { s ^= s << 13; s ^= s >> 17; s ^= s << 5; (s >> 8) as f32 / (1u32 << 24) as f32 };
    let n = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9); v.map(|x| x / l) };
    let z = n([datum[2][0], datum[2][1], datum[2][2]]);
    let (x0, y0) = (n([datum[0][0], datum[0][1], datum[0][2]]), n([datum[1][0], datum[1][1], datum[1][2]]));
    let (a, sc) = (rand() * std::f32::consts::TAU, 0.9 + rand() * 0.2);
    let (sa, ca) = (a.sin(), a.cos());
    let x = [0, 1, 2].map(|k| (x0[k] * ca + y0[k] * sa) * sc);
    let y = [0, 1, 2].map(|k| (y0[k] * ca - x0[k] * sa) * sc);
    [[x[0], x[1], x[2], 0.0], [y[0], y[1], y[2], 0.0], [z[0] * sc, z[1] * sc, z[2] * sc, 0.0], [datum[3][0], datum[3][1], datum[3][2], 1.0]]
}

fn muzzle_flash_matrix(datum: &Mat4, seed: u32) -> Mat4 {
    let mut s = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    let mut rand = || { s ^= s << 13; s ^= s >> 17; s ^= s << 5; (s >> 8) as f32 / (1u32 << 24) as f32 };
    let d = [-datum[0][0], -datum[0][1], -datum[0][2]];
    let horiz = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let sph = [d[1].atan2(horiz), d[0].atan2(d[2]), (horiz * horiz + d[1] * d[1]).sqrt()];
    let rot = [
        sph[0] + rand() * std::f32::consts::TAU,
        sph[1] + (rand() - 0.5) * 0.2 + std::f32::consts::FRAC_PI_2,
        sph[2] + (rand() - 0.5) * 0.2,
    ];
    nf_formats::placement::object_matrix(rot, [datum[3][0], datum[3][1], datum[3][2]])
}

/// Bullet tracer matrix: local +Z along the bullet (`Mat_Align2Dir`), scaled by its speed (one tick of travel).
fn tracer_matrix(b: &nf_game::bullet::Bullet) -> Mat4 {
    let d = b.dir;
    // R6 class sweep (2026-10-22): `Mat_Align2Dir(dir, (0,1,0), (1,0,0))` [M Bullet_init 0x12c5a0:185] —
    // the placement-euler `object_matrix([pitch, yaw, 0])` this used is a different rotation order
    // (z row (sin y, −sin x·cos y, cos x·cos y)) and bent every steep, angled tracer off its line.
    let mut m = nf_game::searchlight::align2dir(d, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
    m[3] = [b.pos[0], b.pos[1], b.pos[2], 1.0];
    for k in 0..3 { m[2][k] *= b.speed; }
    m
}

/// fix-impacts-render: a shell casing's draw matrix. `Bullet_casing_update` 0x12faa0 adds the eject spin to the
/// object's euler every tick (`rot(+0x50..+0x5b) += spin` [M]); `Bullet_init_casing_ex` 0x12f678 starts the euler
/// at the spherical angles of the eject direction (`vecutil_cartesian_to_spherical_acc`, obj+0x58 = 0 [M]). The
/// port reconstructs the eject direction by undoing the tick gravity (`vel[1] += GRAVITY·REC²·age`). The GAME'S RULE,
/// corrected from the decompile: the initial euler is the WORLD-space vector `(0,0,1)` rotated through the WEAPON
/// BONE's world matrix (palette root = the weapon obj's obj+0x90; AnimGetBoneWorldTrans 0x1c3120:131-133), converted
/// `vecutil_cartesian_to_spherical_acc` 0x1e3a70:9-11 — elevation = atan2(y, √(x²+z²)), azimuth = atan2(x, z) — and
/// stored as casing+0x50 = −elevation, +0x54 = azimuth, +0x58 = 0 [M 0x12f678:161-172]; the spin then adds to ALL
/// THREE euler fields every tick [M 0x12faa0:105-113 VADD]. The port's reconstruction (undo gravity → the same
/// atan2 pair, spin per-tick on the same fields) matches the game's convention field-for-field; what it stands in
/// for is the SOURCE vector — the game eulers the bone's forward, the port the eject velocity (the port's eject
/// origin/basis is already a documented stand-in, docs/atlas/effect.md), so the initial attitude can differ by the
/// weapon's eject-offset angle. Euler→matrix order for the render check: the game's object euler path
/// is the RotMatrixZYX family [M RotMatrixZYX 0x1e25a8], the port's `object_matrix` z-row (sin ry, −sin rx·cos ry,
/// cos rx·cos ry) is that same Z·Y·X shape.
fn casing_matrix(c: &nf_game::casing::Casing) -> Mat4 {
    // casings-persist: a rested casing has no velocity left —
    // the in-flight reconstruction degenerates to pitch −π/2 (every shell planted nose-down), so a
    // rest draws with its own settled attitude: roughly horizontal (the eject-reconstruction maps
    // the euler's forward onto the shell's long axis), deterministically yawed by position so a
    // pile of kept shells doesn't all lie identically. No RNG (determinism); the look is a
    // render check.
    if c.resting {
        let yaw = c.pos[0] * 0.7 + c.pos[2] * 1.3;
        return nf_formats::placement::object_matrix([0.12, yaw, 0.08], c.pos);
    }
    let g = 9.8 * (1.0 / 60.0) * (1.0 / 60.0);
    let v0 = [c.vel[0], c.vel[1] + g * c.age, c.vel[2]];
    let horiz = (v0[0] * v0[0] + v0[2] * v0[2]).sqrt();
    let pitch0 = -v0[1].atan2(horiz);
    let yaw0 = v0[0].atan2(v0[2]);
    let eul = [pitch0 + c.spin[0] * c.age, yaw0 + c.spin[1] * c.age, c.spin[2] * c.age];
    nf_formats::placement::object_matrix(eul, c.pos)
}

/// view-model-unlit: the `node` the first-person view model's draws carry. The game draws the view model
/// in `Game_Draw` 0x1e6f50's list-2 pass, after `psiLight_SetLights(0, …)` zeroed the light slots for
/// `DrawDrops`, and an animated object's draw (`obj+0xFA & 8` → `AnimObjectDraw` 0x1c4490) never
/// re-uploads lights [M View_DrawObjList 0x1e69a0:75-98] — so the gun takes NO point lights. The renderer
/// zeroes them for these draws; `node` is otherwise read only for World cel culling.
pub const VIEW_MODEL_NODE: u32 = u32::MAX - 1;
/// Posed slots per gun emplacement: the base object, then one per `gunimp::TURRET_ARTS` gun child.
const GUNIMP_SLOTS: usize = 1 + nf_game::gunimp::TURRET_ARTS.len();

/// Tracer slots (bullets drawn at once).
const TRACER_SLOTS: usize = 32;
/// mp-remote-fx: the MP trail slots start here (after the gadget/laser beam slots TRACER_SLOTS..+6).
const MP_TRACER_BASE: usize = TRACER_SLOTS + 16;

/// A unit-axis matrix (rows = axes) whose z row points along `dir` at `pos`, x kept horizontal
/// (up × z; world x when `dir` is vertical). sg5-laser-draw: the yaw/pitch euler the tripbomb beam
/// uses is only right for level beams — `object_matrix`'s z row is (sin ry, −sin rx·cos ry,
/// cos rx·cos ry), so an atan2 yaw with a tilted beam pointed it off its end.
fn z_basis(dir: [f32; 3], pos: [f32; 3]) -> [[f32; 4]; 4] {
    let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-9);
    let z = [dir[0] / l, dir[1] / l, dir[2] / l];
    // up × z = (z.z, 0, −z.x)
    let (hx, hz) = (z[2], -z[0]);
    let hl = (hx * hx + hz * hz).sqrt();
    let x = if hl < 1e-6 { [1.0, 0.0, 0.0] } else { [hx / hl, 0.0, hz / hl] };
    let y = [z[1] * x[2] - z[2] * x[1], z[2] * x[0] - z[0] * x[2], z[0] * x[1] - z[1] * x[0]];
    [[x[0], x[1], x[2], 0.0], [y[0], y[1], y[2], 0.0], [z[0], z[1], z[2], 0.0], [pos[0], pos[1], pos[2], 1.0]]
}
/// Projectile mesh slots (`models-weapons-items`): 0..GRENADE_SLOTS are the simulation's grenades
/// (`Sim::drones.grenades`), GRENADE_SLOTS..+TRACER_SLOTS the drone bullets (`Sim::drones.bullets`, the same
/// indices the tracer attachments use), and the last PLAYER_BULLET_SLOTS Bond's (`Sim::drones.player_bullets`).
/// Each (slot, mesh) pair has its own Effect attachment; the live projectile's `+0x5C` mesh picks the one that
/// shows (`Bullet_update` disasm 0x12e8f0 [M]).
/// fix-hat-details: GRENADE_SLOTS is the game's per-view draw-list capacity — `View_AddObjects` 0x1e62d8 adds
/// every in-view object with a glist to a RING: `entry = viewer+8+ring·4 + cursor·0xc; cursor = (cursor+1) &
/// (viewer+0x1a+ring·2 − 1)` — and a flying bullet rides ring 0 (`Bullet_update` state 0 clears obj+0xf0's
/// 0x10/0x2000 [M 0x12e7b0]; ring 2's 128 is the +0xf0 & 0x2000 billboards). [L ee_snapshot.bin] viewer[0]
/// @0x1fc6c10 and viewer[5] @0x1fc69b0 both carry sizes 512/512/128 — 512 entries per ring. The cel lists are
/// newest-first [M `control_link_object_to_cel` 0x1346b8 prepends], so a full ring reuses the OLDEST-ADDED
/// entries: the game keeps the NEWEST 512 objects per view. The pre-fix pool showed only the FIRST 8 grenades
/// — with the stuck hats holding slots for their 1500-tick life [M 0x12e7b0:363-371], every later throw flew
/// INVISIBLE; the slot
/// mapping below is therefore newest-first, so an overflow drops the oldest exactly like the ring.
const GRENADE_SLOTS: usize = 512;
/// The projectile-slot base (the grenade range's size) — the drone bullet slots start here
/// (`GRENADE_SLOTS + bullet_index`, the same indices the tracer attachments use). Public for
/// the draw-list tests.
pub const PROJECTILE_SLOT_BASE: usize = GRENADE_SLOTS;
/// Bond's live projectiles drawn at once (rapid fire keeps 100+ rockets in the air; the game draws
/// every bullet object — the old 8 hid the rest).
const PLAYER_BULLET_SLOTS: usize = 32;
/// rocket-swoosh: the ribbon segments drawn at once per swoosh mesh (0x2000918 / 0x2000481). The slot
/// index rides `Attachment::projectile` at `SWOOSH_SLOT_BASE + k`.
const SWOOSH_SLOTS: usize = 128;
const SWOOSH_SLOT_BASE: u16 = 0x8000;
/// fix-impacts-render: the impact-effect draw pools. Decals: the game's 200-node FIFO
/// (`Effect_InitLists` 0x1cf268: `DList_Init(0x3496e0, 0x60, 200)` [M]). Casings: the 32-object
/// round-robin pool (`Bullet_InitCasingLists` 0x12fd48 [M]). Debris: unbounded in the game (a
/// class-0x40 object per particle, ≤ 6 per impact, ≤ ~54 ticks) — the port caps the draw slots.
const DECAL_SLOTS: usize = 200;
const DEBRIS_SLOTS: usize = 128;

/// The drone's height state after `DroneAnim_PreTransCallback` (0x13b490): `Drone+0xA0` = hip height − 0.02, at least
/// 0.2 (it starts at 1.0 and keeps its value while the hip height is 0).
/// mp-body-anim: the upper-body aim, `AnimObjectAimAt` 0x1bfe58 as `psiBuildMatrixPalette` 0x111498
/// runs it on bone 2 (object space, before the object matrix). The game's geometry [M]: axis A =
/// unit(bone 2 − its parent) (the spine), the gun direction G = the weapon datum's −X row, axis B =
/// unit(A × G); the pitch is the rotation about B that brings the gun line onto the aim point, the
/// twist (drone-class objects only — the bots) the rotation about A; clamps: a class-3 player
/// [−π/2, 1.2708] on the pitch (positive = down about B) and no twist, a drone ±0.9 on both; the
/// rotation `q_A(twist) ⊗ q_B(pitch)` premultiplies bone 2's world rotation, so bone 2 and its whole
/// subtree turn about bone 2's position. The game's geometry, EXACT [M AnimObjectAimAt 0x1bfe58]: the target is a
/// REAL POINT — a stored VECTOR* (animobj+4) or the target object's bone world position [+210-227], not a far aim
/// direction; the correction angle comes from a line-through-sphere construction — sphere centred at the aim bone
/// with radius = |aim bone − projected target|, line = the weapon axis through `weaponPos + (−weaponX)·100`
/// [0x42c80000 = 100, +244-277], angle = acos(unit(target−bone), unit(hit−bone)) signed by the scalar triple
/// product [293-296], clamped ±0.9 (drone) / [−π/2, 1.2708] (class 3) [302-317], then rate-limited through
/// Ang_RotateToBearing into the +0x84/+0x8c channels [394-411]. With the sphere passing through the target, that
/// angle IS the angular error between the weapon axis and the target direction — the "turn the gun onto the target
/// direction" the port computes with `signed(axis, g, target)`; the port's `aim` is the eased body-relative
/// (twist, pitch) of the target direction, pitch positive up. Returns the applied (pitch about B, twist about A).
pub fn mp_body_aim(world: &mut [Mat4], skin: &nf_formats::character::Skin, gun: Option<(usize, Mat4)>, aim: [f32; 2], drone: bool) -> Option<(f32, f32)> {
    if world.len() <= 2 { return None; }
    let (gbone, glocal) = gun.filter(|(b, _)| *b < world.len())?;
    let parent = skin.parent(2)?;
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let unit = |a: [f32; 3]| { let l = dot(a, a).sqrt(); if l > 1e-6 { Some([a[0] / l, a[1] / l, a[2] / l]) } else { None } };
    let t = |m: &Mat4| [m[3][0], m[3][1], m[3][2]];
    let axis_a = unit(sub(t(&world[2]), t(&world[parent])))?;
    let gw = nf_formats::anim::mul(&glocal, &world[gbone]);
    let g = unit([-gw[0][0], -gw[0][1], -gw[0][2]])?;
    let axis_b = unit(cross(axis_a, g))?;
    // The target direction in object space (forward +Z, left +X — the object matrix's frame).
    let (st, ct) = aim[0].sin_cos();
    let (sp, cp) = aim[1].sin_cos();
    let target = [st * cp, sp, ct * cp];
    // The signed rotation about `axis` taking `from` onto `to`, both projected off the axis.
    let signed = |axis: [f32; 3], from: [f32; 3], to: [f32; 3]| -> f32 {
        let kf = dot(from, axis);
        let kt = dot(to, axis);
        let pf = sub(from, [axis[0] * kf, axis[1] * kf, axis[2] * kf]);
        let pt = sub(to, [axis[0] * kt, axis[1] * kt, axis[2] * kt]);
        dot(cross(pf, pt), axis).atan2(dot(pf, pt))
    };
    let (pitch, twist) = if drone {
        (signed(axis_b, g, target).clamp(-0.9, 0.9), signed(axis_a, g, target).clamp(-0.9, 0.9))
    } else {
        (signed(axis_b, g, target).clamp(-std::f32::consts::FRAC_PI_2, f32::from_bits(0x3fa2_a974)), 0.0)
    };
    // Row-vector rotation (v' = v·Rᵀ for the column rotation R about `k` by θ, right-handed).
    let rot_row = |k: [f32; 3], th: f32| -> Mat4 {
        let (sn, cs) = th.sin_cos();
        let v = 1.0 - cs;
        let r = [
            [cs + k[0] * k[0] * v, k[0] * k[1] * v - k[2] * sn, k[0] * k[2] * v + k[1] * sn],
            [k[1] * k[0] * v + k[2] * sn, cs + k[1] * k[1] * v, k[1] * k[2] * v - k[0] * sn],
            [k[2] * k[0] * v - k[1] * sn, k[2] * k[1] * v + k[0] * sn, cs + k[2] * k[2] * v],
        ];
        [[r[0][0], r[1][0], r[2][0], 0.0], [r[0][1], r[1][1], r[2][1], 0.0], [r[0][2], r[1][2], r[2][2], 0.0], [0.0, 0.0, 0.0, 1.0]]
    };
    // q_A ⊗ q_B: the pitch applies first, then the twist.
    let rot = nf_formats::anim::mul(&rot_row(axis_b, pitch), &rot_row(axis_a, twist));
    let pivot = t(&world[2]);
    let below = |b: usize| { let mut p = Some(b); while let Some(x) = p { if x == 2 { return true; } p = skin.parent(x); } false };
    for b in 0..world.len() {
        if !below(b) { continue; }
        let mut w = world[b];
        for k in 0..3 { w[3][k] -= pivot[k]; }
        let mut r = nf_formats::anim::mul(&w, &rot);
        for k in 0..3 { r[3][k] += pivot[k]; }
        world[b] = r;
    }
    Some((pitch, twist))
}

fn drone_height(hip: f32) -> f32 {
    if hip == 0.0 { 1.0 } else { (hip - 0.02).max(0.2) }
}

impl Scene {
    /// break-shards draw: the live glass shards as world triangles grouped by texture — `(index into
    /// textures, vertices)`. `DrawAllShards` 0x12bef8 draws every shard with `lifetime > 0` through
    /// `psiDrawShard` 0x122ae8: the shard's matrix (rotation from `rot`, translation `pos`, scale 1
    /// [M 0x122ae8: rows' w zeroed, +0x3c = 1.0]), the face's texture (`GetTextureFromDMA` of the draw
    /// word), and the alpha `min(lifetime·FRAME_RATE·10, 255)` through `psiSetTweakA` [M 0x12bef8:50-56].
    /// The tweak scales the face's own vertex alpha (255 → the pane as it was) [S: the tweak's exact
    /// use in the shard microcode is unread]. A shard still in its peel delay draws at its spawn spot.
    pub fn shard_draws(&self) -> Vec<(usize, Vec<Vertex>)> {
        let Some(sim) = self.sim.as_ref() else { return Vec::new() };
        let sh = &sim.drones.breaks.shatter;
        let mut groups: Vec<(usize, Vec<Vertex>)> = Vec::new();
        for s in sh.shards.iter().filter(|s| s.lifetime > 0.0) {
            let Some(&ti) = self.shard_tex.get(&s.material) else { continue };
            let a = (sh.shard_alpha(s) / 255.0).clamp(0.0, 1.0);
            let m = nf_formats::placement::object_matrix(s.rot, s.pos);
            let g = match groups.iter().position(|(t, _)| *t == ti) { Some(k) => k, None => { groups.push((ti, Vec::new())); groups.len() - 1 } };
            for v in &s.verts {
                let p = v.pos;
                let w = [0, 1, 2].map(|c| m[0][c] * p[0] + m[1][c] * p[1] + m[2][c] * p[2] + m[3][c]);
                groups[g].1.push(Vertex { pos: w, uv: v.uv, rgba: [v.rgba[0].clamp(0.0, 255.0) as u8, v.rgba[1].clamp(0.0, 255.0) as u8, v.rgba[2].clamp(0.0, 255.0) as u8, (v.rgba[3] * a).clamp(0.0, 255.0) as u8], nrm: [0; 4] });
            }
        }
        groups
    }

    /// The script player's overlay (fix-script-commands): the fade, the letterbox borders and the
    /// op-0x14 sprites — see `hud::script_overlay`. Appended to the HUD frame OUTSIDE the cut-scene
    /// gate (the game draws these over the blanked HUD). fix-grenade-types: the flashbang whiteout
    /// rides the same overlay — `Player_Update`'s fade is full alpha while the remaining time sits
    /// above its half, then fades with it [M 0x193600:328-352], at `BL+0x963`'s alpha [M 0x196040].
    pub fn script_overlay_hud(&self, aspect: f32) -> crate::gpu::HudFrame {
        let Some(sim) = self.sim.as_ref() else { return Default::default() };
        let sprites: Vec<(u32, [f32; 4])> = sim.drones.scripts.objects.list.iter()
            .filter_map(|o| match o { Some(nf_game::nis::Obj::Sprite { hash, rect, .. }) => Some((*hash, *rect)), _ => None })
            .collect();
        let h = &sim.drones.health;
        let flash = if h.flashbang <= 0.0 || h.flash_total <= 0.0 { 0.0 }
            else if h.flashbang > h.flash_total * 0.5 { h.flash_alpha }
            else { h.flash_alpha * (h.flashbang / (h.flash_total * 0.5)) };
        // level-exit-fade: the exit hold rides the same overlay as the script fade. In the game
        // the fade object is PER VIEWER (`Camera_SetFade` 0x1bbdd8 indexes `(&glb_viewer)[arg0]`;
        // `Game_Draw` 0x1e6f50 draws one `psiFadeView(R+0x230, G+0x231, B+0x232, A+0x233)` per
        // active viewer — pass 1 :188-209, pass 2 :316-356): the exit push arms VIEWER 6's object
        // [M 0x1c9f38 disasm 0x1c9f7c: a0=6], the script op-0x13 fade arms viewer 4's [M 0x1db3d0]
        // and the entry fade the player viewer's — separate objects that draw independently. A
        // parked script fade (a fade-in ends at alpha byte 0 [M 0x1bada0 zeroes +0x233 when the
        // fade is idle]) draws nothing and cannot cover the exit pane, so the port's single
        // overlay chain puts the EXIT hold first, then the script fade, then the entry fade. The
        // old script-first order let 07000001's parked intro/meet fade-in (alpha 0, never cleared
        // — the hand-back only clears a viewer-0 fade) shadow the exit fade: the level-end fade
        // never drew and the exit looked like a bare freeze + cut (the reported symptom). The
        // push aborts only when the TARGET viewer's own object is a running mode-1 fade
        // [M 0x1bbdd8 disasm 0x1bbe08-0x1bbe18: the refuse gate is mode(+0x234) != 0 AND
        // active(+0x235) != 0] — viewer 6's object is idle unless a previous exit fade runs, so
        // the exit fade can never be refused or covered by a script fade.
        let fade = sim.drones.exit_fade_view().or_else(|| sim.drones.view_fade).or_else(|| sim.drones.spawn_fade_view());
        crate::hud::script_overlay(fade, sim.drones.letterbox, &sprites, &self.hud_textures, aspect, flash)
    }

    /// One level-placed pickup (type 0xF0) — the ONE builder both `Scene::build` sides call (the draw
    /// list and the sim's `Drones::pickups` are index-paired, `Scene::n`/`placed_pickup_pose`). The
    /// game's `Pickup_Create` 0x18dda0 via `parsemap_create_dynamic_objects` 0x1d04f0 case 0xF0 [M]:
    /// the placement's params give kind/id/rounds/switch/sound/respawn (read by INDEX,
    /// `level_tag+0x2C+4k` [M 0x1d04f0:457-462; pickup-port-check §3], absent reads 0), and the
    /// placement's resolved object is the drawn model for kinds 0..6 ([M 0x1d04f0:459-462] —
    /// fix-objects: the game draws the glist pointer, not a hashcode re-lookup, so the resolved
    /// (map, object) is kept; an object with hashcode 0xFFFFFFFF (07000005's `oOBJECT_AmmoPickUp_9mm`
    /// [D]) has no by-hash entry but is still drawn through its own object).
    ///
    /// mp-pickups-visible (docs/systems/mp-pickups-visible.md): kinds 7..11 are `Pickup_CreateFromSet`
    /// 0x18f1b0 [M] — a kind-0 pickup of `PickupMatrix[row][kind-7]` (the row = `MPSettings+0x1B4`,
    /// `mp_row`) with the WEAPON's own world model `weapon_data[base]+0x80`, and the placement's object
    /// is NEVER drawn (`obj` stays None — the arenas' set placements are `PICKUP` objects with
    /// hashcode 0xFFFFFFFF [D 07000023/24], which is why they drew nothing before). Without a match
    /// the row is 0 (`MPSettings+0x1B4` is 0 in SP [M]). The placement's object must still resolve
    /// (the game's null-glist gate 0x18dda0:88) and so must the weapon's model (`create_from_set_row`'s
    /// lookup — the arenas ship their set weapons' meshes [D 07000023/24]). In a match every created
    /// pickup spins (`+0x38 = 1` for all kinds [M 0x18dda0:138-144]; `Pickup_Update` 0x18e148:193-201
    /// applies it per tick); in SP only kind 6 does. `None` where the game refuses (id −1 [M 0x18de0c],
    /// an unresolvable model).
    fn placed_pickup_from_placement(
    level: &Level,
    p: &nf_formats::placement::Placement,
    main_map: usize,
    mp_row: Option<&[u16; 5]>,
    weapons: &nf_formats::weapon::WeaponData,
    level_id: u32,
    models: &std::collections::HashMap<u32, nf_game::pickup::Model>,
    has_gold_medal: bool,
) -> Option<nf_game::pickup::Pickup> {
    let param = |idx: u32| -> u32 { p.params.iter().find(|(i, _)| *i == idx).map(|(_, v)| *v).unwrap_or(0) };
    let type_ = param(0) as u16;
    let id = param(1) as u16;
    // `Pickup_Create`'s level-conditional rounds override [M 0x18dda0: GameState+0xc ==
    // 0x7000008 && id == 0x2c → rounds = 4]: the vault's rocket placement says 5 [D], the game creates 4.
    let rounds = nf_game::pickup::Pickup::create_rounds_override(level_id, id, param(2) as i16);
    let switch_channel = param(3) as u16;
    let sound = param(4) as u16;
    let respawn = param(5) & 0xFFFF;
    let lifetime = 0u8; // always 0 for level-placed pickups (rest=1 in the game)
    let set_kind = (7..=11).contains(&type_) && id != 0xFFFF;
    // The placement's resolved object (the celglist handed to Pickup_Create, [M 0x1d04f0:459-462]).
    let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
        else if (p.index as usize) < level.maps[main_map].objects.len() { Some((main_map, p.index as usize)) } else { None };
    let obj = target.and_then(|(om, oi)| {
        let obj_ = &level.maps[om].objects[oi];
        models.get(&obj_.hashcode).map(|&m| (obj_.hashcode,
            nf_game::pickup::Model { centre: obj_.centre, radius: obj_.radius }, (om, oi)))
    });
    let mut p2 = match (set_kind, mp_row) {
        (true, row) => {
            // The game's null-glist gate comes first [M 0x18dda0:88]: the placement's object must
            // resolve even though it is never drawn for a set kind.
            let _placement_glist = target?;
            match row {
                // mp-lobby-settings / mp-pickups-visible: the match's weapon-set row [M 0x18f1b0:56].
                Some(row) => nf_game::pickup::Pickup::create_from_set_row(p.translation, p.rotation, type_, row, weapons, models)?,
                // SP: `MPSettings+0x1B4` is 0, `Pickup_CreateFromSet` reads row 0 [M].
                None => nf_game::pickup::Pickup::create_from_set_row(p.translation, p.rotation, type_, &nf_game::pickup::PICKUP_MATRIX_ROW0, weapons, models)?,
            }
        }
        (false, _) => {
            let (model, bounds, obj) = obj?;
            let mut p2 = nf_game::pickup::Pickup::create_placed(p.translation, p.rotation, type_, id, rounds, switch_channel, sound, respawn as u16, lifetime, model, bounds, weapons, models, has_gold_medal)?;
            p2.obj = Some(obj);
            p2
        }
    };
    p2.label = param(6); // the kind-4 label (`Pickup_Create` param_13 → +0x34; 0700000b's item: 0x2000036 [D])
    // In a match every pickup spins (`+0x38 = 1` for all kinds [M 0x18dda0:138-144]); SP keeps
    // `create_placed`'s kind-6-only spin.
    if mp_row.is_some() { p2.spin = true; }
    Some(p2)
}

    /// `skeletons`: every skeleton on the disc by id; `sequences`: the level's animation archive, grouped by skeleton.
    /// `mp`: the MP match settings on an MP load (`--mp`/net) — the body slots the scene parks
    /// (mp-character-data); `None` on every campaign load (no bodies).
pub fn build(level: &Level, cels_only: bool, skeletons: &HashMap<u8, Skeleton>, sequences: HashMap<u8, Vec<Sequence>>, drones: Option<&DroneContext>, difficulty: u32, mp: Option<&nf_game::mp::MpSettings>) -> Scene {
        let mut t = std::time::Instant::now();
        // cross-process-determinism: the buckets are walked in ASCENDING skeleton id, not HashMap
        // order — a hashcode present under two buckets resolves to the highest-id bucket's entry
        // in every process (a std HashMap's order is per-process random; last-wins over it is a
        // latent cross-process divergence that reaches anim/root-motion resolution). Disc scan:
        // no cross-bucket duplicate hashcode on any shipped archive today, so this only pins the
        // resolution rule.
        let mut sk_keys: Vec<u8> = sequences.keys().copied().collect();
        sk_keys.sort_unstable();
        let by_hash: HashMap<u32, (u8, usize)> = sk_keys.iter()
            .flat_map(|&sk| sequences[&sk].iter().enumerate().map(move |(i, s)| (s.hashcode, (sk, i)))).collect();
        let (mut instances, missing) = level.scene_instances(cels_only);
        // Sky objects are drawn by the sky pass only (with --all they would also appear as dynamic entities).
        // Ladders (0x22) are dynamic objects too (`parsemap_create_dynamic_objects` 0x1d04f0 case 0x22):
        // their placements never carry the 0x8000 CREATE_CEL flag [M 0x1d1998 vs 0x1d1b68 — the static and
        // dynamic paths are mutually exclusive per placement], so they must not ride the instance loop;
        // the ladder sweep below draws them exactly once (fix-ladder2: the ladders were invisible because
        // `instances(cels_only=true)` skips every placement without 0x8000).
        instances.retain(|i| !matches!(i.placement.flags & 0xff, 0x2a | 0x22 | 0xd4));
        // The breakables (0x20, `Break_Create` 0x12c230) leave the static draw: they ride the posed
        // door/SS slot space below so a killed one stops drawing (`Break_Kill` 0x12b970 sets
        // obj+0xFE bit 0 and the delete pass removes the object [M]). `own_cells` drops them too.
        instances.retain(|i| !(i.placement.flags & 0xff == 0x20 && !i.placement.creates_cel()));
        // (pi, inst): the placement's OWN index rides each instance — the drone link keys on it
        // (992-0c: 0c ships three bit-identical-XZ drone placement pairs [D #263/#461, #261/#462,
        // #310/#463], so a translation key gave both of a pair's characters to one drone and left
        // the other, active, with nothing drawn).
        let (char_insts_pi, chars_missing) = level.character_instances();
        let mut char_instances: Vec<(Option<usize>, Instance)> =
            char_insts_pi.into_iter().map(|(pi, i)| (Some(pi), i)).collect();
        // ghost-guards-990: the game's create gate — `Drone_Create` 0x136be0 (first branch) makes
        // NOTHING for a drone placement whose level tag (parameter 2) exceeds the load's
        // difficulty (GameState+0x40): no object, nothing drawn. `character_instances` built every
        // 0x05-skin placement's meshes regardless, so a skipped placement kept characters the sim
        // list has no drone for: `char_drone` (keyed on the sim drones' placement translations)
        // missed them and `char_hidden`'s Placed arm drew them at their placement matrix — T-posed,
        // full-ambient (their `char_ambient` never leaves the 0xff sentinel) — the 07000011
        // stairwell's T-posed guards in the doorway and in the air over the well. The same gate
        // guards the captain re-add below (a gated captain gets no object either).
        let gated = |p: &nf_formats::placement::Placement| {
            p.flags & 0xff == 0x0f
                && p.params.iter().find(|(j, _)| *j == 2).map_or(false, |&(_, v)| v > difficulty)
        };
        char_instances.retain(|(_, i)| !gated(&i.placement));
        // drone-skins-g021 (corrected by cite-14-skinless-drones): the drone's object is built from
        // the hash `NDrone2_DefaultInit` 0x14b300:563-565 hands `AnimObjectNew` — Drone+0xe0 after
        // the 07000014 param-0 rewrite (:82-95: 0x050000b1 → 0x0500008c on GameState+12 ==
        // 0x07000014, BEFORE the kind switch) and the captain swap (:536-555; the sim's `dr.skin`
        // mirrors both). `character_instances` keyed the raw param, so a placement whose final
        // +0xe0 hash differs from it needs the rebuild here: 07000014's 0x050000b1 placements
        // (the skin is carried only by 07000015/16 [D], which the story loads after 14 — the raw
        // request would draw nothing) and the captains (kind 6 → 0x05000093, kind 7 → 0x050000b2,
        // both in the own archive). A final hash the table lacks draws nothing — the game's
        // `AnimObjectNew` 0x1be910:44-46 miss. Raw-present placements keep their pre-existing
        // draw path untouched.
        let mut drone_skins: HashMap<[u32; 3], u32> = HashMap::new();
        if let (Some(d), Some(m)) = (drones, level.main_map()) {
            // `captain_skin` is pure (params + property tables + difficulty): the drones do not
            // exist yet at this point in the build (b9b creates them after the characters).
            if let Some(props) = d.props.as_ref() {
                for (pi, p) in level.maps[m].placements.iter().enumerate() {
                    if p.flags & 0xff != 0x0f || gated(p) { continue; } // ghost-guards-990: no create, no captain object [M 0x136be0]
                    let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
                    let raw = param(0);
                    if raw >> 24 != 5 { continue; }
                    // The 07000014 rewrite [M 0x14b300:82-95]: with GameState+12 == 0x07000014 the
                    // param-0 skin 0x050000b1 is rewritten to 0x0500008c in Drone+0xe0 BEFORE any
                    // read — the kind switch, the captain block and the `AnimObjectNew` request
                    // ([M 0x14b300:563-565]) all see the rewritten value, captain or not. The raw
                    // 0x050000b1 never reaches the skin table (carried only by 07000016 entry 139
                    // @0x8594a0 / 07000015 entry 102 @0x609e60 [D], which the story loads after 14);
                    // 0x0500008c is in 07000014's own archive [D], so the placement resolves.
                    let request = if raw == 0x0500_00b1 && d.level_id == 0x0700_0014 { 0x0500_008c } else { raw };
                    let swapped = nf_game::drone::captain_skin(request, &param, props, d.level_id, difficulty);
                    let final_skin = swapped.unwrap_or(request);
                    if final_skin == raw { continue; } // the ordinary path's instances are already keyed param0
                    // The game builds the object from the final +0xe0 hash (`NDrone2_DefaultInit`
                    // 0x14b300:563-565 — the rewritten param or the captain swap), so the draw
                    // resolves the final hash and the raw skin's meshes give way to it. When the
                    // raw skin is not in the table (07000014's 0x050000b1 placements) no instance
                    // was built at all and the final skin's mains are ADDED; a final skin the
                    // table lacks (the game's `AnimObjectNew` 0x1be910:44-46 miss) draws nothing.
                    let bits = p.translation.map(f32::to_bits);
                    char_instances.retain(|(_, i)| i.placement.translation.map(f32::to_bits) != bits);
                    if let Some(sk) = level.skins.get(&final_skin) {
                        for h in sk.main.iter().flatten() {
                            if let Some(&(map, object)) = level.by_hash.get(h) {
                                char_instances.push((Some(pi), Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() }));
                            }
                        }
                    }
                    drone_skins.insert(bits, final_skin);
                }
            }
        }
        // Dev NPC pool (the external dev panel's "spawn NPC"): for the first drone placement of each skin, NPC_POOL
        // hidden copies of all its meshes, far below the world. They are ordinary characters with their held weapon,
        // so a spawned drone is drawn exactly like a placed one once `Sim::char_drone` points them at it.
        let pool_sentinel = |t: [f32; 3]| t[1] <= -999_000.0;
        {
            let mut first: Vec<(u32, [u32; 3])> = Vec::new();
            for (_, inst) in &char_instances {
                if inst.placement.flags & 0xff != 0x0f { continue; }
                let skin = inst.placement.params.iter().find(|(i, _)| *i == 0).map_or(0, |&(_, v)| v);
                if skin == 0 || !level.skins.contains_key(&skin) || first.iter().any(|(s, _)| *s == skin) { continue; }
                first.push((skin, inst.placement.translation.map(f32::to_bits)));
            }
            let mut extra = Vec::new();
            for (slot, (_, t)) in first.iter().enumerate() {
                for k in 0..NPC_POOL {
                    let tr = [-1.0e6 - (slot * NPC_POOL + k) as f32, -1.0e6, -1.0e6];
                    for inst in char_instances.iter().filter(|(_, i)| i.placement.translation.map(f32::to_bits) == *t).map(|(_, i)| i) {
                        let mut placement = inst.placement.clone();
                        placement.translation = tr;
                        let mut matrix = inst.matrix;
                        matrix[3] = [tr[0], tr[1], tr[2], 1.0];
                        extra.push(Instance { map: inst.map, object: inst.object, matrix, placement });
                    }
                }
            }
            // The pool copies never link at load (the dev panel binds them at runtime): no placement index.
            char_instances.extend(extra.into_iter().map(|i| (None, i)));
        }
        let mut npc_pool: Vec<NpcSlot> = Vec::new();
        let sky = level.sky_instances();
        crate::phase_mark("b1 instances gather (scene/char/npc/sky)", t); t = std::time::Instant::now();
        let mut groups: Groups = BTreeMap::new();
        let mut bounds = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        // Level-placed pickups (`Pickup_Create` 0x18dda0 via `parsemap_create_dynamic_objects` type 0xF0),
        // built BEFORE the draw groups so the draw loop can instance them (fix-player2: the placed armour
        // had no drawn instance at all — `View_DrawObjectGlist` 0x1e6780 draws `obj+0xD8` at `obj+0x90`).
        let placed_weapons = drones.and_then(|d| d.weapons.clone()).unwrap_or_else(|| nf_formats::weapon::WeaponData { records: Vec::new(), ammo_cap: Vec::new(), ammo_raw: Vec::new() });
            let hoist_models = nf_game::pickup::models(level);
            let mm = level.main_map().unwrap_or(0);
            // mp-pickups-visible: the same weapon-set row the sim build takes (the draw slot k must
            // pair the sim's pickups[k], and kinds 7..11 must model the match's weapons) [M 0x18f1b0].
            let mp_row = mp.map(nf_game::mp::equip::weapon_row);
            let placed_pickups: Vec<nf_game::pickup::Pickup> = level.maps[mm].placements.iter()
                .filter(|p| p.flags & 0xff == 0xf0)
                .filter_map(|p| Self::placed_pickup_from_placement(level, p, mm, mp_row.as_ref(), &placed_weapons, drones.map(|d| d.level_id).unwrap_or(0), &hoist_models, drones.map_or(false, |d| d.has_gold_medal(d.level_id))))
                .collect();
            // fix-safe08: the scripts' obj_type-0xF0 Entity spawns reserve slots after the placed
            // pickups, in the SAME order the sim build appends them (the draw slot k must equal the
            // sim's pickups[k]).
            let (reserved, _render_slots) = script_pickup_reservations(level, &placed_weapons, &hoist_models, drones.map(|d| d.level_id).unwrap_or(0), drones.map_or(false, |d| d.has_gold_medal(d.level_id)));
            let reserved_len = reserved.len();
            let mut placed_pickups = placed_pickups;
            placed_pickups.extend(reserved);

        // The spawn comes from the placement records even when entities are not drawn (Player1 is not a 0x8000 cel).
        crate::phase_mark("b2 placed pickups", t); t = std::time::Instant::now();
        let start = level.instances(false).0.iter()
            .find(|i| level.maps[i.map].objects[i.object].name.trim() == "Player1")
            .map(|i| (i.placement.translation, i.placement.rotation[1]));
        let mut hidden = 0;
        let mut ladders_drawn = 0usize; // the sweep below counts the drawn ladder instances (fix-ladder2)
        let mut grapple_hooks_drawn = 0usize; // the sweep below counts the drawn grapple-point meshes (fix-grapple-hook)
        let mut misc_drawn: BTreeMap<u8, usize> = BTreeMap::new(); // models-misc static dynamic entities
        // The drawn 0xe1 destroy leaves' (world centre, placement) pairs — the per-frame dead gate's
        // input (destroyed-glist swap's draw side: the dead leaf's materials stop drawing; see the
        // NOTE at the MISC_KINDS sweep). The centre is the exact float triple the group's cull sphere
        // carries, so the draw filter matches it by bits.
        let mut destroy_centres: Vec<([f32; 3], usize)> = Vec::new();
        // The level's room cels (see `Cel`): parsed before the draw so the static 0x800 world objects can snap
        // to their cel's ambient exactly as the game's link does at level start (`build_LinkToRoom` 0x1c9680 →
        // the snap in `control_link_object_to_cel` 0x1346b8; channels are all 0 here, so triple 1 reads).
        let (room_cels, mut cel_graph) = parse_cel_graph(level);
        let sim_cels = room_cels.clone();
        crate::phase_mark("b3 cel/portal graph", t); t = std::time::Instant::now();
        // Bug 3 diagnostic (`NF_COLL=1`): log objects that have collision but are not drawn — the "invisible walls".
        // An object is an invisible wall if it has collision triangles but: it's skipped by the cel pass (flag 0x10),
        // it's a door/searchlight/corona that the static pass skips (drawn elsewhere — but if those paths also skip
        // it, it's invisible), or its gfx leaves are all empty (no visible mesh). The named object can then be checked
        // against PCSX2 to see if the game hides, destroys or never creates it.
        let coll_debug = std::env::var_os("NF_COLL").is_some();
        let mut invisible = 0;
        // The list-0 key (`Material::chain`/`Material::node`): `View_AddCels` (0x1e6ba0, disassembled)
        // gives the −90000 + 10·k keys to the +0x20 chain `Vision_Portal_Recurse` (0x1e86e8) threads
        // per frame from the depth-sorted `VisCelList` — and only the room cels (placement flag words
        // 0xc023/0xc047, `parseentity_fixup_entity` 0x1d00c0) ride that chain [L ee_start_05.bin: 19
        // rooms + 113 inner cels; the port's 05 creates_cel set is exactly 19 + 113]. Every non-room
        // 0x8000 placement (the 0x8019/0x8018 marker classes: the baked decals, walkways, blockers)
        // hangs on a room's +0x18 inner chain and draws with a ±dist² key. So: rooms keep a chain
        // ordinal (their parse rank among rooms — the fallback when the per-frame walk cannot run)
        // plus their `cells` node for the per-frame lookup; every other creates_cel placement is
        // distance-keyed (chain = node = u32::MAX, the same path as the dynamics).
        // fix-portal2: every creates_cel placement now also carries its OWN cell in `node` (the
        // placement's pCelList identity) — `View_AddCels` (0x1e6ba0) draws a cel placement's
        // glist when ITS cel is visible, not when the room its centre lands in is. The per-
        // instance walk below is aligned with `instances()` (the main map's placements in order,
        // resolved ones kept) via the same resolution test over `own_cells_main` (by placement
        // ordinal).
        // fix-portal5: `own_cells` must line up 1:1 with the instances that feed it below — the
        // positional `own_i` advances only for non-room instances that survived the retain above,
        // so the build applies the SAME filters (unresolved dropped; the retained-out classes
        // 0x2a/0x22/0xd4 dropped; the room classes dropped — they take the room branch), plus,
        // when `cels_only`, the trailing 0x20/0xe2 extension `scene_instances` appends. The old
        // build listed every resolved placement, so after the first room every non-room placement
        // read an EARLIER placement's cell (07000006: 379 of 765 non-room instances misaligned;
        // the wall-region generics keyed under hidden ROOM cells 13/20/22/38 while their true
        // cells hung on room 21's chain — the courtyard walls did not draw).
        let own_cells: Vec<u32> = match level.main_map() {
            Some(m) => {
                let mut v: Vec<u32> = Vec::new();
                let resolved = |p: &nf_formats::placement::Placement| if p.by_hash() { level.by_hash.contains_key(&p.hashcode) } else { (p.index as usize) < level.maps[m].objects.len() };
                for (pi, p) in level.maps[m].placements.iter().enumerate() {
                    if !resolved(p) { continue; }
                    if cels_only && !p.creates_cel() { continue; }
                    if matches!(p.flags & 0xff, 0x2a | 0x22 | 0xd4) { continue; }
                    if matches!(p.flags & 0xffff, 0xc023 | 0xc047) { continue; }
                    v.push(cel_graph.own_cells_main.get(pi).copied().flatten().unwrap_or(u32::MAX));
                }
                if cels_only {
                    // `scene_instances(true)`'s extension: the static dynamic entities (0xe2,
                    // non-cel placements) appended after the cel placements, in placement order —
                    // minus the 0x20 breakables the retain above moved to the posed slots.
                    for (pi, p) in level.maps[m].placements.iter().enumerate() {
                        if !resolved(p) || p.creates_cel() || p.flags & 0xff != 0xe2 { continue; }
                        if p.flags & 0xff == 0xe2 && p.extra[7] & 0xffff != 0 { continue; }
                        v.push(u32::MAX);
                    }
                }
                v
            }
            None => Vec::new(),
        };
        let mut own_i = 0usize;
        let mut room_rank = 0u32;
        for inst in &instances {
            let obj = &level.maps[inst.map].objects[inst.object];
            // The room rank counts every resolved main-map room-class placement in parse order — the
            // same order `parse_cel_graph` pushes `rooms`/`room_to_cell` — before any skip so the
            // rank stays aligned when a room is cel-pass-skipped or NF_HIDE'd.
            let room_class = matches!(inst.placement.flags & 0xffff, 0xc023 | 0xc047);
            if room_class { room_rank += 1; }
            let (chain, node) = if room_class {
                let node = cel_graph.room_to_cell.get(room_rank as usize - 1).copied().flatten().unwrap_or(usize::MAX) as u32;
                (room_rank - 1, node)
            } else {
                // fix-portal2: the placement's own cell (u32::MAX when it creates no cel — the
                // dynamics stay distance-keyed).
                let own = own_cells.get(own_i).copied().unwrap_or(u32::MAX);
                own_i += 1;
                (u32::MAX, own)
            };
            let has_coll = obj.collision.is_some();
            let has_mesh = obj.gfx.leaves.iter().any(|l| !l.batches.is_empty());
            let skipped = inst.placement.creates_cel() && obj.skipped_by_cel_pass();
            if skipped { hidden += 1; }
            // An invisible wall: collision but no visible mesh, or skipped by the cel pass while not a door/searchlight/corona.
            if has_coll && coll_debug && (!has_mesh || (skipped && !matches!(inst.placement.flags & 0xff, 0xdb | 0xde | 0xfc))) {
                let n_tris = obj.collision.as_ref().map(|c| c.triangles().len()).unwrap_or(0);
                eprintln!("coll: invisible wall \"{}\" (map {} obj {}): {} tris, flags {:#x}, has_mesh={has_mesh}, skipped={skipped} at {:?}",
                    obj.name.trim(), inst.map, inst.object, n_tris, obj.flags, inst.placement.translation);
                invisible += 1;
            }
            if skipped { continue; }
            // floating-rifle-02-2: NO bake skip. 786 read `parseentity_fixup_entity` 0x1d00c0's
            // 0x8019/0x8018 flag bit 0x8000000 as "baked, not drawn" and skipped every 0x8019/0x8018
            // placement whose matrix rows 1+2 looked identity — that broke decal_order_live (07000007's
            // hall_shadows*/OBJECT_banistershadows* dropped, their textures never loaded) and the
            // decompile does not support the reading: the bit is set when the placement's TRANSLATION
            // and ROTATION vectors (cel+0x60/+0x70, the placement record's two float triples) are BOTH
            // exactly zero (`Vec_IsEqual` with epsilon 0.0f against the zero vector at 0x2d6200 —
            // disasm of 0x1d02e0-0x1d0324), and its only consumers are the collision identity fast
            // paths `Collide_Pick` 0x1eb320:166,249 / `Collide_Intersect` 0x1ebaa8:142 (`Ident`). No
            // draw path tests it: `View_AddCels` 0x1e6ba0 walks every inner-chain cel (gated on
            // glist+0x44 bit 0x10, the 0x8018-vs-0xc023 special case and Vision_InView) and
            // `View_DrawCelGlist` 0x1e6538 draws it at Rot(cel+0x70)·Trans(cel+0x60) behind
            // psiSetTweakARGB(cel+0xa3-derived alpha, 0xff, 0xff, 0xff). The game DRAWS the flagged
            // records — 07000002's sandals (t=r=0 [D]) at their absolute art coords, 07000007's shadow
            // decals with them. docs/systems/baked-statics.md holds the full chain + the open
            // visibility question at the doorway eye.
            // Searchlight parts are dynamic objects (`Searchlight_Update` moves them every tick): drawn from
            // the simulation below, not from their placements.
            if inst.placement.flags & 0xff == 0xde { continue; }
            // Doors likewise (`Door_Update`/`Door_Interp` move them): posed from the simulation below.
            if inst.placement.flags & 0xff == 0xdb { continue; }
            // Level coronas are dynamic objects too (`Corona_Create`, class 1 with no update function): added
            // below from their placements so the alpha-list pass draws them (docs/systems/world-objects.md).
            if inst.placement.flags & 0xff == 0xfc { continue; }
            // Debug: `NF_HIDE=name,name` leaves out world objects whose name contains one of them.
            if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && obj.name.contains(n)) { continue; } }
            // The static 0x800 world objects (pickups and props flagged `celglist+0x44 & 0x800`): the game dims
            // them by tint × obj+0x100..0x102 (`View_SetupRenderModes_Tweak` 0x1e5888, the 0x800 branch),
            // snapped to their cel's ambient at the level-start link. World-cel geometry goes through
            // `View_DrawCelGlist` (0x1e6538) which forces the white tweak — multiplier 1.0 [M gs-leaf.md].
            let mult = if !inst.placement.creates_cel() && obj.flags & 0x800 != 0 {
                let p = transform(&inst.matrix, obj.centre);
                mul_bytes(cel_ambient(&room_cels, p, 0).unwrap_or([0xff; 3]))
            } else { [1.0; 3] };
            add_instance(level, inst, Layer::World, None, &mut groups, Some(&mut bounds), None, None, None, mult, &room_cels, chain, node);
        }
        let mut characters = Vec::new();
        crate::phase_mark("b4 static world instances", t); t = std::time::Instant::now();
        let (mut attachments, mut armed, mut weapons_missing) = (Vec::new(), std::collections::HashSet::new(), 0);
        // Spawner roles of the main map's drone placements, keyed by the placement's own index
        // (the translation key collapsed 0c's bit-identical pairs — 992-0c).
        let roles: HashMap<usize, drone::SpawnRole> = match level.main_map() {
            Some(m) => {
                let ps = &level.maps[m].placements;
                let list: Vec<(u32, Vec<(u32, u32)>)> = ps.iter().map(|p| (p.flags & 0xff, p.params.clone())).collect();
                ps.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x0f)
                    .map(|(i, _)| i).zip(drone::spawn_roles(&list)).collect()
            }
            None => HashMap::new(),
        };
        // The placement index of each character, aligned with `characters` (None = not a drone
        // placement's mesh: MP bodies, script characters, view models, the dev pool copies).
        let mut char_placements: Vec<Option<usize>> = Vec::new();
        for (inst_pi, inst) in &char_instances {
            let obj = &level.maps[inst.map].objects[inst.object];
            let raw_hash = inst.placement.params.iter().find(|(i, _)| *i == 0).map(|&(_, v)| v).unwrap_or(0);
            // drone-skins-g021: the captain-swapped drone draws its swapped skin (`+0xe0`,
            // `NDrone2_DefaultInit` 0x14b300:563-565) — `drone_skins` only carries the
            // 0x0f placements whose final +0xe0 hash differs from the raw param (the
            // rewritten 0x050000b1 and the captain swaps, see the extension above);
            // everything else keeps the raw param.
            let skin_hash = drone_skins.get(&inst.placement.translation.map(f32::to_bits)).copied().unwrap_or(raw_hash);
            let Some(skin) = level.skins.get(&skin_hash) else { continue };
            let Some(skeleton) = skeletons.get(&skin.skeleton) else { continue };
            if !obj.gfx.is_skinned() || skin.inverse_bind.is_empty() {
                add_instance(level, inst, Layer::World, None, &mut groups, Some(&mut bounds), None, None, None, if obj.flags & 0x800 != 0 {
                    // A non-skinned character object drawn through the World layer: the same 0x800 dim.
                    mul_bytes(cel_ambient(&room_cels, transform(&inst.matrix, obj.centre), 0).unwrap_or([0xff; 3]))
                } else { [1.0; 3] }, &room_cels, u32::MAX, u32::MAX);
                continue;
            }
            add_instance(level, inst, Layer::Character, Some((characters.len() as u32, None)), &mut groups, Some(&mut bounds), None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            let start = drones.and_then(|d| {
                let param = |k: u32| inst.placement.params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v).unwrap_or(0);
                let hash = drone::start_script(&d.tables, param, d.level_id).script()?;
                let script = d.scripts.get(&hash)?.clone();
                let sequences = script.sequences()
                    .filter_map(|(a, b, q)| by_hash.get(&q).filter(|(sk, _)| *sk == skin.skeleton).map(|&(_, i)| (a, b, i))).collect();
                Some(StartAnim { script, sequences })
            });
            let pooled = pool_sentinel(inst.placement.translation);
            let role = if pooled { drone::SpawnRole::Reinforcement { channel: 0 } } else {
                inst_pi.and_then(|pi| roles.get(&pi).copied()).unwrap_or(drone::SpawnRole::Placed)
            };
            if pooled {
                let ci = characters.len();
                match npc_pool.iter_mut().find(|n| n.key == inst.placement.translation.map(f32::to_bits)) {
                    Some(n) => n.chars.push(ci),
                    None => npc_pool.push(NpcSlot { key: inst.placement.translation.map(f32::to_bits), skin: skin_hash, params: inst.placement.params.clone(), chars: vec![ci] }),
                }
            }
            let morph_of_main = skin.main.iter().flatten()
                .filter_map(|mh| level.by_hash.get(mh).and_then(|&(m2, o2)| level.maps[m2].objects[o2].morph_targets.clone()))
                .next();
            char_placements.push(*inst_pi);
            characters.push(Character { matrix: inst.matrix, skin: skin.clone(), skeleton: skeleton.clone(), slot_bones: obj.gfx.slot_bones.clone(), start, role, floor: None, script: None, morph_targets: obj.morph_targets.clone().or(morph_of_main) });
            // The held weapon (`AnimObjectDraw`): weapon_data[w] +0x80 mesh on the datum whose id is +0x50 (0xFF → 0),
            // once per drone (a skin may have several main meshes).
            let key = (inst.placement.translation.map(f32::to_bits), skin_hash);
            if !armed.insert(key) { continue; }
            let Some(wd) = drones.and_then(|d| d.weapons.as_ref()) else { continue };
            let param9 = inst.placement.params.iter().find(|(i, _)| *i == 9).map(|&(_, v)| v).unwrap_or(0);
            let weapon = drone::weapon_id(skin_hash, param9);
            let Some((mesh, datum_id)) = (weapon < 0x73).then(|| wd.third_person(weapon as u8)).flatten() else { continue };
            let datum_id = if datum_id == 0xff { 0 } else { datum_id as u32 };
            let Some(dt) = skin.datums.iter().find(|d| d.id == datum_id && d.bone >= 0) else { weapons_missing += 1; continue };
            let Some(&(map, object)) = level.by_hash.get(&mesh) else { weapons_missing += 1; continue };
            let winst = Instance { map, object, matrix: anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]), placement: inst.placement.clone() };
            let character = characters.len() as u32 - 1;
            add_instance(level, &winst, Layer::Character, Some((character, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            let local = anim::quat_trans_to_mat(dt.q, dt.t);
            attachments.push(Attachment { character, bone: dt.bone as usize, local, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
            // The same gun as a pickup once the drone drops it.
            add_instance(level, &winst, Layer::Character, Some((character, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            attachments.push(Attachment { character, bone: 0, local, flash: None, tracer: None, dropped: true, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
            // Its muzzle flash: `DroneWeap_FireWeapon` asks `AnimGetBoneWorldTrans(obj, datum id, 1, …)`, which looks up
            // the datum whose id is id + 1 — the muzzle (07000005: 8 → 9 at x −0.77, 16 → 17 at x −1.18).
            if let Some((fmesh, rgb)) = wd.muzzle_flash(weapon as u8) {
                let muzzle = skin.datums.iter().find(|d| d.id == datum_id + 1 && d.bone >= 0);
                if let (Some(&(fmap, fobject)), Some(md)) = (level.by_hash.get(&fmesh), muzzle) {
                    let finst = Instance { map: fmap, object: fobject, ..winst };
                    add_instance(level, &finst, Layer::Effect, Some((character, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    attachments.push(Attachment { character, bone: md.bone as usize, local: anim::quat_trans_to_mat(md.q, md.t), flash: Some(rgb), tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
            }
        }
        // mp-character-data: the MP bodies — one per human slot and per bot, parked hidden. The
        // game builds each slot's body at `Player_Init`/`BOT_init` from `MP_skins[settings skin]`
        // [M 0x1910f8:108-117, 0x1249e8:100]; every MP arena's map pack carries ALL 29 skins +
        // their data files (the load-time `MP_NeedSkin`/`MP_NeedFile` filter picks the selected
        // ones — 0x1c block discard 0x1d12e8 / skin gate 0x1bdf40), so the arena archive the port
        // parsed IS the source; nothing to merge (the G-021 session pattern has no MP gap: the
        // game loads no other archive for MP characters — `MP_PostLoad_Init` 0x183718 inits the
        // game-mode objects, no loads [M]).
        let mut mp_char_bodies: Vec<MpBody> = Vec::new();
        // The 16 descriptors for the draw's per-frame pick (`mp_body_pose`).
        let mp_anim_sets = drones.and_then(|d| d.mp_anims.as_ref()).map(|a| nf_formats::mpanim::MpAnimSets { sets: a.sets });
        if let (Some(mp_set), Some(d)) = (mp, drones) {
            if let (Some(skins), Some(anim_sets)) = (d.mp_skins.as_ref(), d.mp_anims.as_ref()) {
                // mp-16-slots: humans 0..h, bots right after them (h..h+b), up to 16 [S].
                let humans = (mp_set.human_count as usize).clamp(1, nf_game::mp::MP_MAX_SLOTS);
                let bots = (mp_set.bot_count as usize).min(nf_game::mp::MP_MAX_SLOTS - humans);
                let mut slot_ids: Vec<usize> = (0..humans).collect();
                slot_ids.extend(humans..humans + bots);
                for slot in slot_ids {
                    let char_idx = mp_set.players[slot].skin as usize;
                    let Some(entry) = skins.entries.get(char_idx) else {
                        eprintln!("mp body: slot {slot} character index {char_idx} is outside MP_skins (29 entries) — no body");
                        continue;
                    };
                    let Some(skin) = level.skins.get(&entry.skin) else {
                        eprintln!("mp body: slot {slot} skin {:08x} is not in this arena's archive", entry.skin);
                        continue;
                    };
                    let Some(skeleton) = skeletons.get(&skin.skeleton) else {
                        eprintln!("mp body: slot {slot} skin {:08x} skeleton {} not on the disc", entry.skin, skin.skeleton);
                        continue;
                    };
                    // The parked translation: the NPC-pool sentinel scale (y <= -999_000.0), one
                    // x-slot per body so the draw can tell them apart until it moves them.
                    let tr = [-1.0e6 - slot as f32, -1.0e6, -1.0e6];
                    let matrix = nf_formats::placement::object_matrix([0.0, 0.0, 0.0], tr);
                    let placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                    let mut chars = Vec::new();
                    for mh in skin.main.iter().flatten() {
                        let Some(&(map, object)) = level.by_hash.get(mh) else { continue };
                        let obj = &level.maps[map].objects[object];
                        if !obj.gfx.is_skinned() || skin.inverse_bind.is_empty() { continue; }
                        let mut p2 = placement.clone();
                        p2.translation = tr;
                        p2.params = vec![(0, entry.skin)];
                        let inst = Instance { map, object, matrix, placement: p2 };
                        add_instance(level, &inst, Layer::Character, Some((characters.len() as u32, None)), &mut groups, Some(&mut bounds), None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        let morph_of_main = skin.main.iter().flatten()
                            .filter_map(|mh2| level.by_hash.get(mh2).and_then(|&(m2, o2)| level.maps[m2].objects[o2].morph_targets.clone()))
                            .next();
                        char_placements.push(None);
                        characters.push(Character { matrix, skin: skin.clone(), skeleton: skeleton.clone(), slot_bones: obj.gfx.slot_bones.clone(), start: None, role: drone::SpawnRole::Placed, floor: None, script: None, morph_targets: obj.morph_targets.clone().or(morph_of_main) });
                        chars.push(characters.len() - 1);
                    }
                    if chars.is_empty() {
                        eprintln!("mp body: slot {slot} skin {:08x} has no drawable main mesh in this arena", entry.skin);
                        continue;
                    }
                    // The body's sequence set: every script id the pose can return, resolved
                    // through the arena's kind-5 scripts (the StartAnim resolution).
                    let mut body_scripts = HashMap::new();
                    for id in anim_sets.all_body_scripts() {
                        let Some(script) = d.scripts.get(&id) else { continue };
                        let seqs = script.sequences()
                            .filter_map(|(a, b, q)| by_hash.get(&q).filter(|(sk, _)| *sk == skin.skeleton).map(|&(_, i)| (a, b, i))).collect();
                        body_scripts.insert(id, seqs);
                    }
                    mp_char_bodies.push(MpBody { slot, char_index: char_idx as u32, skin: entry.skin, file: entry.file, sleeve: entry.sleeve, skeleton: skin.skeleton, chars, scripts: body_scripts });
                }
                eprintln!("mp bodies: {} parked ({} humans, {} bots)", mp_char_bodies.len(), humans, bots);
            }
        }
        // The first-person gun models: one character per owned weapon with a first-person skin (`weapon_data+0xDC`),
        // drawn with a sleeve mesh and rigid parts on their bones. The active one is selected each frame
        // in `animate` from `sim.gun.weapon`. The P2K (weapon 2) is always first; others follow the owned list [M].
        crate::phase_mark("b5a characters + held weapons", t); t = std::time::Instant::now();
        let mut view_models: HashMap<u8, usize> = HashMap::new();
        if let Some(wd) = drones.and_then(|d| d.weapons.as_ref()) {
            const SLEEVES: [u32; 8] = [0x0200_09d0, 0x0200_0900, 0x0200_01f7, 0x0200_0929, 0x0200_09e6, 0x0200_09ff, 0x0200_0a14, 0x0200_0a13];
            // Build a view model for each weapon with a first-person skin present in the level. The P2K (weapon 2)
            // is always first; others follow the weapon_data order.
            for wid in 1..=0x72u8 {
                let fp = wd.first_person(wid as usize);
                if fp == 0 { continue; }
                let Some(skin) = level.skins.get(&fp) else { continue };
                let Some(&sleeve) = SLEEVES.iter().find(|h| level.by_hash.contains_key(h)) else { continue };
                let Some(skeleton) = skeletons.get(&skin.skeleton) else { continue };
                let ci = characters.len() as u32;
                let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                let placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                let (map, object) = level.by_hash[&sleeve];
                let inst = Instance { map, object, matrix: ident, placement: placement.clone() };
                add_instance(level, &inst, Layer::Character, Some((ci, None)), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                // silencer-white-artifact: the record's +0x7C view-entity (the silencer mesh, records
                // whose +0x68 flags carry 0x800 — the pair's bare record shares the mesh [L the golden
                // weapon table 2fd48d6d]) is owned by the DATUM-0 attach below. The game hides the
                // weapon-slot objects EVERY tick before the attach (32 `View_SetDrawInNoViews` calls,
                // `Player_WeaponFiring` 0x1a5460 — the loop over BL+0x7ec..+0x86c and +0x86c), and the
                // fp skin's extra list carries the SAME mesh on its suppressor bone (0x05000088's
                // extra bone 43 = 0x02000842 [D]; likewise 05000047→0x020001f2, 05000050→0x020005b2,
                // 050000a8→0x0200099d, 050000a9→0x020009a3, all bone 43). Drawing it here as well as
                // the datum attach doubled the mesh's additive sheen batch (ALPHA_1 0x7f00000048 [D])
                // — a white artifact on attachments. Skip the extra whose hash is the
                // record's +0x7C; the twin arm (0x1a5460:111-118) and the datum attach keep the bare
                // twin bare and the silenced twin with exactly one suppressor.
                let r = &wd.records[wid as usize];
                let ventity = u32::from_le_bytes(r[0x7c..0x80].try_into().unwrap());
                for &(bone, part) in &skin.extra {
                    let swap_piece = ventity != 0 && part == Some(ventity);
                    let Some(&(pmap, pobject)) = part.and_then(|h| level.by_hash.get(&h)) else { continue };
                    let pinst = Instance { map: pmap, object: pobject, matrix: ident, placement: placement.clone() };
                    add_instance(level, &pinst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                    attachments.push(Attachment { character: ci, bone: bone as usize, local: ident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece, watch_hand: None, pda_digit: None });
                }
                // The muzzle flash (`Player_MuzzleFlash`): mesh `weapon+0x48` on datum 0, colour (+0x56, +0x55, +0x54).
                let flash = u32::from_le_bytes(r[0x48..0x4c].try_into().unwrap());
                if let (Some(&(fmap, fobject)), Some(dt)) = (level.by_hash.get(&flash), skin.datums.iter().find(|d| d.id == 0 && d.bone >= 0)) {
                    let finst = Instance { map: fmap, object: fobject, matrix: ident, placement: placement.clone() };
                    add_instance(level, &finst, Layer::Effect, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                    attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: Some([r[0x56], r[0x55], r[0x54]]), tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
                // fix-gadgets-live: the AIMS-20's four side lights [M Player_WeaponFiring 0x1a5460:256-360 —
                // for the records whose class halfword (+0x02) is 0x1a/0x1b, `AnimDatumSetEntity(animobj,
                // bone, 0x2000132, 0xff)` runs every tick on bones 8..0xB: the gun's row of red lights]. The port attaches the mesh to the same bones;
                // the blink/off arms (idle > 15 → bone 8 blinks on GameState+0x76 % 40's 20/40 duty; idle
                // > 45 → all four off) live in `frame_state` off `drones.aims_idle`. The bone-9/10/11
                // extra gates (+0x93e / +0x2fc / +0x2f0) are unmeasured — live check in the report.
                let wclass = u16::from_le_bytes([r[2], r[3]]);
                if matches!(wclass, 0x1a | 0x1b) {
                    if let Some(&(lmap, lobject)) = level.by_hash.get(&0x0200_0132) {
                        // 8..0xB are DATUM ids (`AnimDatumSetEntity(animobj, datum, mesh, 0xff)` — the silencer's
                        // datum 0 is the same call): each datum carries its own bone and local q/t, and
                        // `AnimObjectDraw` 0x1c4490 draws the entity at `QuatTransToMat(datum.q, datum.t) · bone
                        // world`. Treated as raw bone indices with no local, the four quads sat ~1.3 units from the
                        // receiver — off the gun, never seen.
                        for id in 8u8..=11 {
                            let Some(dt) = skin.datums.iter().find(|d| d.id == id as u32 && d.bone >= 0) else { continue };
                            let linst = Instance { map: lmap, object: lobject, matrix: ident, placement: placement.clone() };
                            add_instance(level, &linst, Layer::Effect, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                            attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: None, tracer: None, dropped: false, aims_light: Some(id), projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                        }
                    }
                }
                // fix-silencer-visual: the silenced twins' view-entity — the silencer MESH (the
                // suppressor stayed on the gun in the sim but did not draw). While the HELD record's flags (+0x68) carry 0x800, `Player_WeaponFiring`
                // 0x1a5460:96-110 attaches the mesh at weapon_data+0x7C to DATUM 0 of the weapon
                // AnimObj (BL+0x7e8) every tick — `AnimDatumSetEntity(vm, 0, weapon+0x7C, 0xff)`,
                // cleared in the swap/holster state 8 and by the twin arm (0x1a5460:111-118) on the
                // bare record; `AnimObjectDraw` 0x1c4490 draws a datum's entity at
                // `QuatTransToMat(datum.q, datum.t) · bone world`. The silenced twins 0x03/0x05/
                // 0x07/0x09 carry flags 0x1884/0x188c over the bare 0x02/0x04/0x06/0x08's
                // 0x1004/0x100c, sharing +0x7C and the fp skin [L the golden weapon table 2fd48d6d;
                // the P2K's mesh 0x02000842, on 05 natively [D]]. The mesh rides the record's own
                // view-model character, so it draws only while that record is held — the game's
                // per-tick gate. (Session 102b's open decode: fix_silencer_modes_live.rs.)
                let wflags = u32::from_le_bytes(r[0x68..0x6c].try_into().unwrap());
                let ventity = u32::from_le_bytes(r[0x7c..0x80].try_into().unwrap());
                if wflags & 0x800 != 0 && ventity != 0 {
                    if let (Some(&(emap, eobject)), Some(dt)) = (level.by_hash.get(&ventity), skin.datums.iter().find(|d| d.id == 0 && d.bone >= 0)) {
                        let einst = Instance { map: emap, object: eobject, matrix: ident, placement: placement.clone() };
                        add_instance(level, &einst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                        attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: true, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                    }
                }
                // delta-repeater: the Delta Repeater's loaded bolts (the model
                // drew without them). The game re-issues the attachments EVERY tick in
                // `Player_WeaponFiring` 0x1a5460:331-381 for the records whose base (+0x02) is 0x11:
                // clear the view model's datums 4/5/6, then `AnimDatumSetEntity(vm, d, 0x0200816, 0xff)`
                // — datum 4 when count > 0 (NOT while the view state is 9, the fire script's), datum 5
                // when > 1, datum 6 when >= 3 [M :361-381]. The mesh is the record's +0x5C — the SAME
                // hash the flying projectile draws (0x0200816 "Bolt", its own ENTITY_PARAMS object
                // [L ee_start_05.bin @ 0x2bf150 + 0x11·0x10C + 0x5C; in 07000014's map right after the
                // Crossbow_3rd_LOD0]). The datums live on the FP skin (0x05000083, +0xDC): 4/5/6 on
                // bone 40 at t y 0.17/0.06/0.00, z −0.19 — the three magazine slots top→bottom, plus
                // 0/3 on bone 38 [D 0700000c.bin/07000014.bin kind-3 skin, `nf-extract datums`]. The
                // per-frame count (clip[0x11], +1 in fire state 9, min(carried[0x0F], clip_size 3) in
                // reload state 6 past frame 80) lives in frame_state off `quiver_bolt` — the port's
                // build-time attachment + visibility arm is the AIMS-lights pattern.
                if u16::from_le_bytes([r[2], r[3]]) == 0x11 {
                    let bolt = u32::from_le_bytes(r[0x5c..0x60].try_into().unwrap());
                    if let Some(&(bmap, bobject)) = level.by_hash.get(&bolt) {
                        for id in 4u8..=6 {
                            let Some(dt) = skin.datums.iter().find(|d| d.id == id as u32 && d.bone >= 0) else { continue };
                            let binst = Instance { map: bmap, object: bobject, matrix: ident, placement: placement.clone() };
                            add_instance(level, &binst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                            attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: Some(id), swap_piece: false, watch_hand: None, pda_digit: None });
                        }
                    }
                }
                // The wristwatch hands (Player_SetWatch 0x1a69d0 [M :79-137]): the fp skin 0x0500004B
                // (the watch gadgets' shared arm) carries the hour/minute/second hand meshes on datum 2.
                if skin.hashcode == 0x0500_004b {
                    if let Some(dt) = skin.datums.iter().find(|d| d.id == 2 && d.bone >= 0) {
                        for (k, mesh) in [0x0200_0876u32, 0x0200_0877, 0x0200_0878].into_iter().enumerate() {
                            let Some(&(hmap, hobject)) = level.by_hash.get(&mesh) else { continue };
                            let hinst = Instance { map: hmap, object: hobject, matrix: ident, placement: placement.clone() };
                            add_instance(level, &hinst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                            attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: Some(k as u8), pda_digit: None });
                        }
                    }
                }
                // The decryptor's screen digits (`Player_WeaponFiring` 0x1a5460:155-194 [M]): for the
                // 0x56/0x57 records the game sets entity `PdaDigits[c − '0']` on the view model's datum
                // 3−i for each display char c (entity 0 for '*', and all four cleared once idle). The
                // port attaches every digit mesh to each of the four datums; `frame_state` shows the
                // one the sim's `pda_display` names.
                if nf_game::gadget::is_decryptor(wid) {
                    if let Some(pda) = drones.and_then(|d| d.hud.as_ref()).map(|h| h.pda_digits) {
                        for id in 0u8..4 {
                            let Some(dt) = skin.datums.iter().find(|d| d.id == id as u32 && d.bone >= 0) else { continue };
                            for (digit, mesh) in pda.into_iter().enumerate() {
                                let Some(&(dmap, dobject)) = level.by_hash.get(&mesh) else { continue };
                                let dinst = Instance { map: dmap, object: dobject, matrix: ident, placement: placement.clone() };
                                add_instance(level, &dinst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, VIEW_MODEL_NODE);
                                attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: Some((id, digit as u8)) });
                            }
                        }
                    }
                }
                char_placements.push(None);
                characters.push(Character { matrix: ident, skin: skin.clone(), skeleton: skeleton.clone(), slot_bones: level.maps[map].objects[object].gfx.slot_bones.clone(), start: None, role: drone::SpawnRole::Placed, floor: None, script: None , morph_targets: None });
                view_models.insert(wid, ci as usize);
            }
        }
        // MP bodies (without them the players could not see each other): one character per used
        // player slot with the slot's `MP_skins` body [M Player_Init 0x1910f8:111], parked at the origin
        // and posed each frame from the slot's movement (`Scene::mp_body_state`).
        let mut mp_bodies: Vec<(usize, usize)> = Vec::new();
        let mut mp_body_weapons: HashMap<u32, (usize, u8)> = HashMap::new();
        if let Some(d) = drones {
            let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
            let placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
            for (slot, &sh) in d.mp_body_skins.iter().enumerate() {
                if sh == 0 { continue; }
                let (Some(skin), Some(skeleton)) = (level.skins.get(&sh), level.skins.get(&sh).and_then(|k| skeletons.get(&k.skeleton))) else { continue };
                let ci = characters.len() as u32;
                let mut slot_bones = Vec::new();
                let mut morph = None;
                for &mh in skin.main.iter().flatten() {
                    let Some(&(map, object)) = level.by_hash.get(&mh) else { continue };
                    let inst = Instance { map, object, matrix: ident, placement: placement.clone() };
                    add_instance(level, &inst, Layer::Character, Some((ci, None)), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    slot_bones = level.maps[map].objects[object].gfx.slot_bones.clone();
                    morph = level.maps[map].objects[object].morph_targets.clone();
                }
                for &(bone, part) in &skin.extra {
                    let Some(&(map, object)) = part.and_then(|ph| level.by_hash.get(&ph)) else { continue };
                    let pinst = Instance { map, object, matrix: ident, placement: placement.clone() };
                    add_instance(level, &pinst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    attachments.push(Attachment { character: ci, bone: bone as usize, local: ident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
                char_placements.push(None);
                characters.push(Character { matrix: ident, skin: skin.clone(), skeleton: skeleton.clone(), slot_bones, start: None, role: drone::SpawnRole::Placed, floor: None, script: None, morph_targets: morph });
                mp_bodies.push((slot, ci as usize));
                // The held weapon (`AnimObjectDraw`): weapon_data[w]+0x80 on the datum +0x50 names, as a
                // guard holds his (the drone block above) — one attachment per weapon the level carries,
                // shown only while the slot holds it (`mp_body_weapons`).
                if let Some(wd) = d.weapons.as_ref() {
                    for w in 1u8..0x73 {
                        let Some((mesh, datum_id)) = wd.third_person(w) else { continue };
                        let datum_id = if datum_id == 0xff { 0 } else { datum_id as u32 };
                        let Some(dt) = skin.datums.iter().find(|x| x.id == datum_id && x.bone >= 0) else { continue };
                        let Some(&(map, object)) = level.by_hash.get(&mesh) else { continue };
                        let winst = Instance { map, object, matrix: ident, placement: placement.clone() };
                        add_instance(level, &winst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        mp_body_weapons.insert(attachments.len() as u32, (slot, w));
                        attachments.push(Attachment { character: ci, bone: dt.bone as usize, local: anim::quat_trans_to_mat(dt.q, dt.t), flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                        // mp-remote-fx: its muzzle flash — the guards' `DroneWeap_FireWeapon` flash (mesh
                        // weapon_data+0x4C on the datum id + 1, the muzzle [M]); shown 2 ticks per shot of
                        // the slot (`Sim::mp_fx.last_shot`). The PS2's human players draw theirs on the
                        // first-person view model only (`Draw_MuzzleFlash` 0x1a3fe0); the opponent's
                        // third-person flash is the port's own netcode extension: in the game
                        // the human MP body's fire path (Player_WeaponFiring ->
                        // Player_WeaponInitBullet -> Player_MuzzleFlash) spawns NO world flash
                        // sprite on the body — the sprite flash is
                        // View_DrawInThisViewOnly(owner slot) [M Draw_MuzzleFlash 0x1a3fe0:116],
                        // the owner's screen only; the other split-screen player sees the world
                        // dynamic light [M Player_MuzzleFlash 0x1a4298:86-90 Light_Create] and
                        // the body's firing anim-datum swap [0x1a4298:140-144], not a sprite.
                        // (Only the bots/guards' DroneWeap_FireWeapon 0x178f68:189-213 spawns a
                        // world-visible flash object.) The port's flash mesh here stands in for
                        // what the game's light + datum swap show the other player.
                        if let Some((fmesh, rgb)) = wd.muzzle_flash(w) {
                            let muzzle = skin.datums.iter().find(|x| x.id == datum_id + 1 && x.bone >= 0);
                            if let (Some(&(fmap, fobject)), Some(md)) = (level.by_hash.get(&fmesh), muzzle) {
                                let finst = Instance { map: fmap, object: fobject, matrix: ident, placement: placement.clone() };
                                add_instance(level, &finst, Layer::Effect, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                                mp_body_weapons.insert(attachments.len() as u32, (slot, w));
                                attachments.push(Attachment { character: ci, bone: md.bone as usize, local: anim::quat_trans_to_mat(md.q, md.t), flash: Some(rgb), tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                            }
                        }
                    }
                }
            }
            if d.mp_body_skins.iter().any(|&s| s != 0) {
                eprintln!("mp: bodies {:?} (slot, character) from skins {:x?}", mp_bodies, d.mp_body_skins);
            }
        }
        // cable-3p: Bond's own third-person body (single player). `Player_Init` 0x1910f8:94-108 builds
        // the player's anim object from the per-level skin [M]; first person never draws it, but the
        // third-person camera modes do (`Player_PositionCamera` 0x18f5d8 case 0xb:
        // `View_SetDrawInAllViews(obj)` + `View_SetDrawInNoViews(BLData+0x7e8)` [M :228-229]). Built
        // hidden; `frame_state_for` poses it while the wire camera runs.
        let mut bond_body: Option<usize> = None;
        if let Some(d) = drones.filter(|d| d.mp_body_skins.iter().all(|&s| s == 0)) {
            let sh = nf_game::player::player_skin_hash(d.level_id);
            if let (Some(skin), Some(skeleton)) = (level.skins.get(&sh), level.skins.get(&sh).and_then(|k| skeletons.get(&k.skeleton))) {
                let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                let placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                let ci = characters.len() as u32;
                let mut slot_bones = Vec::new();
                let mut morph = None;
                for &mh in skin.main.iter().flatten() {
                    let Some(&(map, object)) = level.by_hash.get(&mh) else { continue };
                    let inst = Instance { map, object, matrix: ident, placement: placement.clone() };
                    add_instance(level, &inst, Layer::Character, Some((ci, None)), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    slot_bones = level.maps[map].objects[object].gfx.slot_bones.clone();
                    morph = level.maps[map].objects[object].morph_targets.clone();
                }
                for &(bone, part) in &skin.extra {
                    let Some(&(map, object)) = part.and_then(|ph| level.by_hash.get(&ph)) else { continue };
                    let pinst = Instance { map, object, matrix: ident, placement: placement.clone() };
                    add_instance(level, &pinst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    attachments.push(Attachment { character: ci, bone: bone as usize, local: ident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
                char_placements.push(None);
                characters.push(Character { matrix: ident, skin: skin.clone(), skeleton: skeleton.clone(), slot_bones, start: None, role: drone::SpawnRole::Placed, floor: None, script: None, morph_targets: morph });
                bond_body = Some(ci as usize);
            }
        }
        // Script objects (docs/systems/script-player.md): every animation and drawn entity the main map's script players
        // can make; each shows only while the simulation has the object.
        let mut script_objects = 0;
        let mut script_billboards: std::collections::HashSet<u32> = std::collections::HashSet::new();
        if let Some(m) = level.main_map() {
            let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
            let placement = level.maps[m].placements.first().cloned().unwrap_or_default();
            let mut seen = BTreeSet::new();
            let players: Vec<nf_game::nis::Player> = level.maps[m].placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0xd9)
                .filter_map(|(i, p)| nf_game::nis::Player::from_placement(i, p)).collect();
            // laser-scripts: the switches' embedded players (`Switch_Create` 0x1adcc0:44-45,
            // `SP_Create(pos, rot, param 1, 0, 8, 0, obj, 0)`) — the laser-grid show on 07000009
            // (switch #71 "SCRIPTPLAYER_Lasers", script 0x06000667) and the keyboard lasers on
            // 0700000a (switch #643, script 0x06000664) draw through them; the sim mounts the
            // same players (Drones::attach_level), so their entity streams need attachment slots
            // exactly like a placed player's. Param 1 < 0x06000000 is a model id, not a script —
            // skipped the same way the sim skips it.
            let switch_scripts = level.maps[m].placements.iter()
                .filter(|p| p.flags & 0xff == 0x29)
                .filter_map(|p| p.params.iter().find(|(k, _)| *k == 1).map(|&(_, v)| v))
                .filter(|&v| v >= 0x0600_0000);
            // The explosion scripts (`Explode_Create` 0x17cbd0 loads a per-source effect script —
            // its `Script_Load` arg a3): 0x6000052 from the fuse/contact call sites [M 0x12cd98/
            // 0x12ce80 disasm], 0x60007c4 from the Samurai bolt's 0x100000/0x200000 arms [M
            // 0x12d0e8-0x12d188], 0x6000027/0x600004f/0x6000052 from `Destroy_Smash` by the tag's
            // type byte [M 0x12bc50-0x12bd20]. The sim spawns them per blast, so their entity
            // streams need attachments exactly like a placed player's. (fix-blast-scripts.)
            // fix-grenade-release: a blast's VISIBLE puffs can live in a SUB-script — 0x06000052's
            // op-0x1b (`Script_SubScriptStart` 0x1db050) spawns 0x06000047 [L the live frag blast's
            // Obj::Entity src.0 = 0x06000047, the lib key], whose streams carry the puff meshes
            // (0x020003cc/cd/ce/c7...). Walk the op-0x1b hashes recursively (the runtime `src` is
            // the sub's own lib key, so the attachment's script tuple must carry the sub's hash).
            // copter-script-draw (the helicopter was missing from The Exchange): `Copter_Create`
            // 0x17d340 attaches NO glist to the copter (obj+0xD8 = 0 [M :60-66]) — its whole look is
            // the script player `SP_Create(.., params 4/5, mode 8, ..)` makes [M 0x17d340 SP_Create
            // call: +0x3c/+0x40 = the assembly 0x06000051 / the flight 0x06000050 on 07000008 #45
            // [D]]. The sim spawns the flight on the gate (Drones::tick's copter_flight), but the
            // flight's entities (the body 0x02000427 type 0x2F and its parented parts) had no draw
            // slots — those scripts were never in this root list.
            let copter_scripts: Vec<u32> = level.main_map().map(|mm| level.maps[mm].placements.iter()
                .filter(|p| p.flags & 0xff == 0x2f)
                .flat_map(|p| [4u32, 5].map(|k| p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v)))
                .filter(|&h| h != 0).collect()).unwrap_or_default();
            // fusebox-panel-14: a fusebox placement (0x31) carries its script in param 0 and
            // `FuseBox_Create` runs it at load [M docs/levels/07000014.md 5; nf-game/fusebox.rs], so
            // its entity streams (the wires behind the front panel) need draw slots like a placed
            // player's. Without them the wires exist in `scripts.objects` but nothing draws them --
            // the opened panel's recess rendered empty.
            let fusebox_scripts: Vec<u32> = level.maps[m].placements.iter()
                .filter(|p| p.flags & 0xff == 0x31)
                .filter_map(|p| p.params.iter().find(|(k, _)| *k == 0).map(|&(_, v)| v))
                .filter(|&v| v >= 0x0600_0000)
                .collect();
            let mut hashes: Vec<u32> = players.iter().flat_map(|p| p.scripts).filter(|&h| h != 0)
                .chain(switch_scripts)
                .chain(copter_scripts)
                .chain(fusebox_scripts.iter().copied())
                .chain([0x0600_0052u32, 0x0600_0027, 0x0600_004f, 0x0600_07c4]).collect();
            let mut queued: std::collections::HashSet<u32> = hashes.iter().copied().collect();
            // lift-indicators-09: a SUB-script's entities need one draw slot per LIVE copy — every placed
            // SP running a parent script starts the sub once per `Script_SubScriptStart` op [M 0x1db050],
            // so 07000009's two cab SPs (0x060007D/0x060007E) start 0x0600064E/0x0600064F three times each
            // and the sim holds six indicator plates / six button plates. The old count (placed SPs whose
            // OWN scripts contain the hash) read 1 for every sub hash, and `Sim::script_entity_k` serves
            // only the newest match — one cab's plates drew, the other cab's never did. Accumulated in the
            // sub walk (parents may pop after their subs), read in the Entity arm below.
            let mut sub_copies: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
            // The blast scripts run on no placed SP: the sim spawns one player per explosion
            // (`Explode_Create` 0x17cbd0 → the effect script), so several can be live at once (two
            // grenades, a chain of boxes). With one slot set only the newest blast drew — a
            // same-tick second blast hid the first. BLAST_COPIES slot sets per blast script (and its
            // subs) let that many concurrent blasts draw; `Sim::script_entity_k` serves the newest.
            const BLAST_ROOTS: [u32; 4] = [0x0600_0052, 0x0600_0027, 0x0600_004f, 0x0600_07c4];
            const BLAST_COPIES: usize = 4;
            let mut order: Vec<u32> = Vec::new();
            while let Some(h) = hashes.pop() {
                order.push(h);
                let Some(nis) = level.nis.get(&h) else { continue };
                let running = if BLAST_ROOTS.contains(&h) { BLAST_COPIES } else { players.iter().filter(|p| p.scripts.contains(&h)).count().clamp(1, 16) };
                for st in nis.streams.iter() {
                    for cmd in st.decode() {
                        if let nf_formats::nis::Cmd::SubScript { hash: sub, .. } = cmd {
                            if sub != 0 {
                                *sub_copies.entry(sub).or_insert(0) += running;
                                if queued.insert(sub) { hashes.push(sub); }
                            }
                        }
                    }
                }
            }
            // The slots are created after the WHOLE walk so every sub hash's copy count is final
            // (a parent may pop after its sub); `order` preserves the original pop order, so the
            // attachment indices match the pre-fix build.
            for &h in &order {
                let Some(nis) = level.nis.get(&h) else { continue };
                for (si, st) in nis.streams.iter().enumerate() {
                    for cmd in st.decode() {
                        match cmd {
                            nf_formats::nis::Cmd::Anim { skin: sh, .. } => {
                                if !seen.insert((h, si, sh)) { continue; }
                                let (Some(skin), Some(skeleton)) = (level.skins.get(&sh), level.skins.get(&sh).and_then(|k| skeletons.get(&k.skeleton))) else { continue };
                                let ci = characters.len() as u32;
                                let mut slot_bones = Vec::new();
                                let mut morph_of_object = None;
                                for &mh in skin.main.iter().flatten() {
                                    let Some(&(map, object)) = level.by_hash.get(&mh) else { continue };
                                    let inst = Instance { map, object, matrix: ident, placement: placement.clone() };
                                    add_instance(level, &inst, Layer::Character, Some((ci, None)), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                                    slot_bones = level.maps[map].objects[object].gfx.slot_bones.clone();
                                    morph_of_object = level.maps[map].objects[object].morph_targets.clone();
                                }
                                for &(bone, part) in &skin.extra {
                                    let Some(&(map, object)) = part.and_then(|ph| level.by_hash.get(&ph)) else { continue };
                                    let pinst = Instance { map, object, matrix: ident, placement: placement.clone() };
                                    add_instance(level, &pinst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                                    attachments.push(Attachment { character: ci, bone: bone as usize, local: ident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                                }
                                char_placements.push(None);
                                characters.push(Character { matrix: ident, skin: skin.clone(), skeleton: skeleton.clone(), slot_bones, start: None, role: drone::SpawnRole::Placed, floor: None, script: Some((h, si as u8)), morph_targets: morph_of_object.clone() });
                                script_objects += 1;
                                // The held weapon (`AnimObjectDraw` 0x1c4490:256-265): Event 0xB on the next stream
                                // sets the weapon byte on this Anim (`sAnimObject+0x62`), then `AnimDatumGetWeaponInfo`
                                // reads `weapon_data[w]+0x80` (3rd-person mesh) on datum `weapon_data[w]+0x50`. The
                                // scene's attachments are fixed at build time, so we scan stream si+1's commands for
                                // the static `Cmd::Event { kind: 0xB, params: [w, ..] }` (the live `Obj::Anim.weapon`
                                // is 0 until the script ticks). Same path as in-play drones (scene.rs:1099-1127).
                                let weapon = nis.streams.get(si + 1).and_then(|next| {
                                    next.decode().into_iter().find_map(|c| match c {
                                        nf_formats::nis::Cmd::Event { kind: 0xB, params } => params.first().copied().map(|w| w as u8),
                                        _ => None,
                                    })
                                });
                                if let (Some(w), Some(wd)) = (weapon, drones.and_then(|d| d.weapons.as_ref())) {
                                    if let Some((mesh, datum_id)) = (w < 0x73).then(|| wd.third_person(w)).flatten() {
                                        let datum_id = if datum_id == 0xff { 0 } else { datum_id as u32 };
                                        if let Some(dt) = skin.datums.iter().find(|d| d.id == datum_id && d.bone >= 0) {
                                            if let Some(&(wmap, wobject)) = level.by_hash.get(&mesh) {
                                                let winst = Instance { map: wmap, object: wobject, matrix: ident, placement: placement.clone() };
                                                add_instance(level, &winst, Layer::Character, Some((ci, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                                                let local = anim::quat_trans_to_mat(dt.q, dt.t);
                                                attachments.push(Attachment { character: ci, bone: dt.bone as usize, local, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                                            } else { weapons_missing += 1; }
                                        } else { weapons_missing += 1; }
                                    }
                                }
                            }
                            // Triggers, AI volumes and emitters draw nothing (`Script_CreateEntity`).
                            nf_formats::nis::Cmd::Entity { hash, obj_type, .. } if !matches!(obj_type, 0xea | 0xeb | 0xf5 | 0xf2) => {
                                if !seen.insert((h, si, hash)) { continue; }
                                let Some(&(map, object)) = level.by_hash.get(&hash) else { continue };
                                // The billboard set: glist flags bit 0x8 (`View_DrawObjectGlist`'s test).
                                if level.maps[map].objects[object].flags & 0x8 != 0 { script_billboards.insert(hash); }
                                // One draw slot per ScriptPlayer running this script: every SP creates its own
                                // copy of the stream's entities (07000014's three vent lids are nine SPs on
                                // 0x06000A3B — one slot drew only the newest lid; the hatch under the spawn
                                // stayed invisible while it opened).
                                // fusebox-panel-14: each fusebox placement runs its own player of the
                                // same script, so its copies count toward the slot count the way a
                                // placed SP's does.
                                let boxes = fusebox_scripts.iter().filter(|&&v| v == h).count();
                                let copies = sub_copies.get(&h).copied()
                                    .unwrap_or_else(|| if BLAST_ROOTS.contains(&h) { BLAST_COPIES } else { players.iter().filter(|p| p.scripts.contains(&h)).count() + boxes })
                                    .clamp(1, 16);
                                let inst = Instance { map, object, matrix: ident, placement: placement.clone() };
                                for k in 0..copies {
                                    add_instance(level, &inst, Layer::Character, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                                    attachments.push(Attachment { character: 0, bone: 0, local: ident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: Some((h, si as u8, hash, k as u8)), gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                                    script_objects += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        // Bullet tracers: drone bullets draw `GUN_BULLET_TRAIL` (Bullet_update state 0), stretched along their
        // direction by the speed (Control_GetObjMatrix scale 1, 1, speed); a pool of slots, hidden when unused.
        // Slot TRACER_SLOTS is the gadget beam (fix-laser-beam: the Laser/Stunner's per-shot beam draw).
        if let (Some(_), Some(&(tmap, tobj))) = (drones, level.by_hash.get(&0x0200_0123)) {
            // Pool-slot replay (level-load-time): slot 0 decodes, slots 1..N replay it (identical
            // identity-matrix geometry; only the attachment index and `tracer` slot differ).
            let tplacement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
            let tinst = Instance { map: tmap, object: tobj, matrix: anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]), placement: tplacement.clone() };
            let before = pool_snapshot(&groups);
            add_instance(level, &tinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            let rec = pool_record(&groups, &before);
            attachments.push(Attachment { character: 0, bone: 0, local: tinst.matrix, flash: None, tracer: Some(0), dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
            for slot in 1..TRACER_SLOTS {
                pool_replay(&mut groups, &rec, Some(attachments.len() as u32));
                attachments.push(Attachment { character: 0, bone: 0, local: tinst.matrix, flash: None, tracer: Some(slot), dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
            }
            // mp-remote-fx: the other players' / bots' trails (`Sim::mp_tracer`), MP only.
            if mp.is_some() {
                for k in 0..crate::mpfx::MP_TRACER_SLOTS {
                    pool_replay(&mut groups, &rec, Some(attachments.len() as u32));
                    attachments.push(Attachment { character: 0, bone: 0, local: tinst.matrix, flash: None, tracer: Some(MP_TRACER_BASE + k), dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
            }
            // fix-gadgets-live: the gadget BEAM draws the game's own sprites (the old trail
            // stand-in is gone) —
            // `Draw_LaserBeam` 0x1a4560 sets the beam object's gfx to 0x2000131 "lazer" (the class-'N'
            // 0x4E record; the 0x4F upgrade takes 0x20006c1 "uv_lazer") and `Draw_TaserBeam` 0x1a46a8
            // to 0x200026d "taser" [M], all via hashtable_set_object_to_entity_gfx; the objects resolve
            // in every level archive [D probe]. Two beam slots after the bullet pool:
            // TRACER_SLOTS = "lazer" (0x4E), +1 = "taser" (0x4A/0x4C), +2 = "uv_lazer" (the 0x4F
            // upgrade [M 0x1a4560's class-'N' pick]); `frame_state` shows the one for the active
            // weapon (all stretch muzzle→hit off sim.laser_beam).
            for (slot, hash) in [(TRACER_SLOTS, 0x0200_0131u32), (TRACER_SLOTS + 1, 0x0200_026d), (TRACER_SLOTS + 2, 0x0200_06c1),
                // fix-satchel-tripbomb: the tripbomb's own beam mesh — the laser block creates the
                // beam object with `hashtable_set_object_to_entity_gfx(uVar9, 0x2000133)` [M
                // `Bullet_update` case 3, the +0x70 & 0x80000 arm 0x12ee30-0x12ee50]; two slots
                // (the 6-placed cap keeps placed tripbombs bounded; the port draws the first two).
                (TRACER_SLOTS + 3, 0x0200_0133), (TRACER_SLOTS + 4, 0x0200_0133),
                // sg5-laser-draw: the weapon laser sight (`Player_LaserPointer` 0x1a65d8) — the beam
                // sprite 0x2000133 and the red dot 0x2000132 [M ll. 189-205, the `Player_InitWeapon`
                // 0x1a7ef8 objects]; both follow `Sim::laser_sight`.
                // gunimp-beam: the mounted laser's beam quad `lazer_Big` (0x20005cd) and its impact
                // flare `taser_muzzle` (0x200026e) — the game's own draw for weapon 0x67
                // [M GunImp_Update 0x1320d8:394-407 `PositionBeam(muzzle, ray hit)`; D 0700000d
                // resolves both, the quad extending along -z like the watch laser's sprite].
                (TRACER_SLOTS + 5, 0x0200_0133), (TRACER_SLOTS + 6, 0x0200_0132),
                (TRACER_SLOTS + 7, 0x0200_05cd), (TRACER_SLOTS + 8, 0x0200_026e)] {
                if drones.is_some() {
                    if let Some(&(bmap, bobj)) = level.by_hash.get(&hash) {
                        let binst = Instance { map: bmap, object: bobj, matrix: anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]), placement: level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default() };
                        add_instance(level, &binst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        attachments.push(Attachment { character: 0, bone: 0, local: binst.matrix, flash: None, tracer: Some(slot), dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                    }
                }
            }
        }
        // Projectile meshes (`models-weapons-items`): a bullet whose weapon has a `+0x5C` mesh draws THAT mesh
        // instead of the trail — `Bullet_update` state 0 (disasm 0x12e8f0-0x12e900): `weapon+0x5C != 0` →
        // `hashtable_set_object_to_entity_gfx(obj, weapon+0x5C)` + `View_SetDrawInAllViews`; the mesh sits at the
        // bullet's matrix, aligned to the flight direction (`Bullet_init` 0x12c5a0:185
        // `Mat_Align2Dir(obj+0x90, dir, up (0,1,0) [L ee_start_05.bin @0x2d6210], fallback (1,0,0) @0x2d6220)`),
        // while the position lives in `obj+0x30` (the per-tick tail 0x12f1ac-0x12f1e4 moves `obj+0x30..0x38` only).
        // Grenades are gravity bullets: the stun (0x35 → 0x02000726), the smoke (0x36 → 0x0200071f), the mine
        // (0x37 → 0x02000a6e) and the rocket (0x3A → 0x0200074f) all show their body in flight [L ee_start_05.bin].
        // The port pools the slots like the tracers: GRENADE_SLOTS grenades, then TRACER_SLOTS drone bullets,
        // then PLAYER_BULLET_SLOTS of Bond's. One instance per (slot, mesh) for every distinct projectile mesh the
        // level archive resolves — the others hide (FAR) in `frame_state`.
        let proj_slots = GRENADE_SLOTS + TRACER_SLOTS + PLAYER_BULLET_SLOTS;
        crate::phase_mark("b5b view models (sleeves/FP)", t); t = std::time::Instant::now();
        let mut proj_meshes: Vec<u32> = Vec::new();
        if drones.is_some() {
            for w in 1..0x73usize {
                let m = placed_weapons.projectile_mesh(w);
                if m != 0 && !proj_meshes.contains(&m) && level.by_hash.contains_key(&m) { proj_meshes.push(m); }
            }
        }
        if let Some(wd) = drones.and_then(|d| d.weapons.as_ref()) {
            for w in 1..0x73usize {
                let m = wd.projectile_mesh(w);
                if m != 0 && !proj_meshes.contains(&m) && level.by_hash.contains_key(&m) { proj_meshes.push(m); }
            }
        }
        let pslot_placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
        for &mesh in &proj_meshes {
            let Some(&(pmap, pobject)) = level.by_hash.get(&mesh) else { continue };
            // Pool-slot replay (level-load-time): slot 0 decodes, slots 1..N replay it (identical geometry).
            let pident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
            let pinst = Instance { map: pmap, object: pobject, matrix: pident, placement: pslot_placement.clone() };
            let before = pool_snapshot(&groups);
            add_instance(level, &pinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            let rec = pool_record(&groups, &before);
            attachments.push(Attachment { character: 0, bone: 0, local: pident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: Some((0, mesh)), script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
            for slot in 1..proj_slots {
                pool_replay(&mut groups, &rec, Some(attachments.len() as u32));
                attachments.push(Attachment { character: 0, bone: 0, local: pident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: Some((slot as u16, mesh)), script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
            }
        }
        // rocket-swoosh: the ribbon segments (`Swoosh_Create` 0x18d8e8 sets the object's gfx from the hash —
        // 0x2000918 `SwooshStart` / 0x2000481 `Swoosh`, a 0.15-wide tube along local −z [D 07000008 bounds]).
        // One pool per mesh the level resolves; `frame_state` poses the k-th live segment of that mesh.
        if drones.is_some() {
            for mesh in [0x0200_0918u32, 0x0200_0481] {
                let Some(&(smap, sobject)) = level.by_hash.get(&mesh) else { continue };
                let pident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                let sinst = Instance { map: smap, object: sobject, matrix: pident, placement: pslot_placement.clone() };
                let before = pool_snapshot(&groups);
                add_instance(level, &sinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                let rec = pool_record(&groups, &before);
                attachments.push(Attachment { character: 0, bone: 0, local: pident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: Some((SWOOSH_SLOT_BASE, mesh)), script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                for k in 1..SWOOSH_SLOTS {
                    pool_replay(&mut groups, &rec, Some(attachments.len() as u32));
                    attachments.push(Attachment { character: 0, bone: 0, local: pident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: Some((SWOOSH_SLOT_BASE + k as u16, mesh)), script: None, gas: None, effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
            }
        }
        crate::phase_mark("b5c projectile pools", t); t = std::time::Instant::now();
        for (inst, follows_camera) in &sky {
            add_instance(level, inst, if *follows_camera { Layer::SkyCamera } else { Layer::SkyFixed }, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
        }
        // fix-impacts-render: the impact-effect draw pools — the game's own meshes for the bullet decals, the
        // shell casings and the debris. There is no dedicated draw function in ACTION.ELF: the decal nodes ride
        // the host obj/cel's sub-object list (`View_DrawObjectGlist` 0x1e6780 / `View_DrawCelGlist` 0x1e6538 draw
        // `node+0x10`'s glist at `node+0x20`), casings and debris are ordinary class-0x40 objects drawn through
        // `Control_GetObjMatrix` 0x134210's cached `obj+0x90`. All three render in the game's pass 0 — the port
        // instances them as Effect-layer pools (the leaf GS registers carry each mesh's own blend/z state; the
        // Effect layer only skips the static-World cull/sort). One pool per distinct hash the level archive
        // resolves: the decal mesh `EffectInfo+4` (`hashtable_set_sub_object_to_entity_gfx` 0x1ce9c8), the
        // casing mesh `ammo_data[type]+4` (`Bullet_init_casing` 0x12fe2c) and the debris mesh `EffectInfo+0x74`
        // (`Debris_Create` 0x1aab10) — the same hashes the sim stores per effect. Slot counts: the game's
        // 200-node decal FIFO (`DList_Init(0x3496e0, 0x60, 200)` 0x1cf268) and 32 casing objects
        // (`Bullet_InitCasingLists` 0x12fd48); debris are unbounded per-impact objects (≤ 6, ≤ ~54 ticks) —
        // the port caps the draw slots.
        if let Some(dc) = drones {
            if let Some(surf) = &dc.surfaces {
                let mut decal_meshes: Vec<u32> = surf.effects.iter().map(|r| r.decal).filter(|m| *m != 0).collect();
                let mut debris_meshes: Vec<u32> = surf.effects.iter().map(|r| r.debris_mesh).filter(|m| *m != 0).collect();
                // break-debris draw: a killed breakable's pieces are the same `Debris_Create` class,
                // models from `EffectInfo[mat & 0x3f]+0x74` (nf_game::shatter::EFFECT_INFO) — they share
                // the debris slots (slot k ≥ effects.debris.len() poses breaks.shatter.debris).
                debris_meshes.extend(nf_game::shatter::EFFECT_INFO.iter().map(|e| e.model).filter(|m| *m != 0));
                // The blast debris (`Explode_Update` 0x17c350 → `Debris_CreateEx` 0x1ac808, the same class-0x10
                // object): `MetalHash_132` / `StoneHash_133` meshes; their slots follow the breakable pieces.
                debris_meshes.extend(nf_game::explode::METAL_HASH_132.iter().chain(nf_game::explode::STONE_HASH_133.iter()).copied());
                let mut casing_meshes: Vec<u32> = surf.ammo.casing_mesh.iter().copied().filter(|m| *m != 0).collect();
                // The gas puffs (`Gas_Init` 0x1aacc8): one draw slot per pool index for each mesh a puff can carry —
                // SMOKE_QUAD (muzzle smoke, searchlight smoke, trails) and every `EffectInfo` gas mesh (the impact
                // puffs: snow material 12's only effect is its 0x02000039 floor puff). The draw side (`a.gas`) was
                // written with the gas system but no slot was ever created, so no puff ever drew — shooting
                // snow showed no impact (snow leaves no decal).
                let mut gas_meshes: Vec<u32> = surf.effects.iter().flat_map(|r| r.gas.iter().map(|g| g.mesh)).filter(|m| *m != 0).collect();
                // flesh-hit-effects: the pool's OWN spawn meshes too — the blood puff 0x0200_08bc
                // (`NDrone2_DealWithObjHit` 0x13fff0's flesh arm), the bolt puff 0x0200_026e and the
                // trail smoke 0x0200_001f are hardcoded `Gas_Init` sites, not `EffectInfo` rows, and
                // had no slot: the puff sat in the pool with nothing to draw it through (no red
                // puffs). The list lives with the spawners (`gas::POOL_MESHES`).
                gas_meshes.extend_from_slice(nf_game::gas::POOL_MESHES);
                gas_meshes.sort(); gas_meshes.dedup();
                {
                    let gslot_placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                    let gident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                    for &mesh in &gas_meshes {
                        let Some(&(m, o)) = level.by_hash.get(&mesh) else { continue };
                        // Pool-slot replay (level-load-time): slot 0 decodes, slots 1..N replay it.
                        let ginst = Instance { map: m, object: o, matrix: gident, placement: gslot_placement.clone() };
                        let before = pool_snapshot(&groups);
                        add_instance(level, &ginst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        let rec = pool_record(&groups, &before);
                        attachments.push(Attachment { character: 0, bone: 0, local: gident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: Some((0, mesh)), effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                        for slot in 1..GAS_DRAW_SLOTS {
                            pool_replay(&mut groups, &rec, Some(attachments.len() as u32));
                            attachments.push(Attachment { character: 0, bone: 0, local: gident, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: Some((slot as u16, mesh)), effect: None, silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                        }
                    }
                }
                for v in [&mut decal_meshes, &mut debris_meshes, &mut casing_meshes] { v.sort(); v.dedup(); }
                let slot_placement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                for (meshes, slots, kind) in [
                    (&decal_meshes, DECAL_SLOTS, 0u8),
                    // casings-persist: the pool sizes to KEEP_MAX, not POOL — the PC-port "Keep
                    // shell casings: N" option grows the sim's pool past the game's 32 [S
                    // Casings::set_keep] and every kept casing needs a draw slot. The per-frame
                    // pose draws `sim.drones.casings.list.get(slot)` only while that slot carries
                    // this mesh and is flying (state 0), so at the game's default (keep = 0,
                    // 32 slots) slots 32..127 pose `far` — no visual change.
                    (&casing_meshes, nf_game::casing::KEEP_MAX, 1u8),
                    (&debris_meshes, DEBRIS_SLOTS, 2u8),
                ] {
                    for &mesh in meshes {
                        let Some(&(m, o)) = level.by_hash.get(&mesh) else { continue };
                        // Pool-slot replay (level-load-time): slot 0 decodes, slots 1..N replay it.
                        let inst = Instance { map: m, object: o, matrix: ident, placement: slot_placement.clone() };
                        let before = pool_snapshot(&groups);
                        add_instance(level, &inst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        let rec = pool_record(&groups, &before);
                        attachments.push(Attachment { character: 0, bone: 0, local: inst.matrix, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: Some((kind, 0, mesh)), silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                        for slot in 1..slots {
                            pool_replay(&mut groups, &rec, Some(attachments.len() as u32));
                            attachments.push(Attachment { character: 0, bone: 0, local: inst.matrix, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: Some((kind, slot, mesh)), silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                        }
                    }
                }
            }
        }
        // fix-grenade-throw: the smoke grenade's occluder cloud (`Occlude_Create(pos, 1200, 1)`
        // 0x17ea80 — mesh 0x2000273, kind 1): a static class-0x2A object whose only update is the
        // 1200-tick countdown (0x17eba0), drawn at `obj+0x30` with no rotation. One Effect pool per
        // level archive that resolves the mesh, 4 slots (the game caps nothing; clouds are rare).
        crate::phase_mark("b5d impact pools (decal/casing/debris/gas)", t); t = std::time::Instant::now();
        if drones.is_some() {
            if let Some(&(m, o)) = level.by_hash.get(&0x0200_0273) {
                let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                let cplacement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                for slot in 0..4 {
                    let cinst = Instance { map: m, object: o, matrix: ident, placement: cplacement.clone() };
                    add_instance(level, &cinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    attachments.push(Attachment { character: 0, bone: 0, local: cinst.matrix, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: Some((3, slot, 0x0200_0273)), silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
            }
        }
        // fix-microcam-brackets: the Micro-Camera's target brackets — `HUD_UpdateCameraPane` 0x1a12d8 creates one
        // one-frame object per candidate drone at its root with gfx "DroneScan" 0x02000832 (Effect kind 6).
        if drones.is_some() {
            if let Some(&(m, o)) = level.by_hash.get(&DRONE_SCAN) {
                let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                let splacement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                for slot in 0..DRONE_SCAN_SLOTS {
                    let sinst = Instance { map: m, object: o, matrix: ident, placement: splacement.clone() };
                    add_instance(level, &sinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    attachments.push(Attachment { character: 0, bone: 0, local: sinst.matrix, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: Some((6, slot, DRONE_SCAN)), silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
            }
        }
        // rain-drops: `DrawDrops` 0x18cb90 for DropType 1 [M disasm] — every live drop draws the
        // rain-streak MESH 0x02000290 with an identity rotation translated to the drop (Mat_Identity,
        // row 3 = DropVector, psiDrawObjectMatrix), and on 07000009..0700000b a second pass draws
        // 0x02000039 per drop with the viewer's rotation (Mat_CopyRot viewer+0x1A0) under
        // psiSetTweakARGB(36, 32, 32, 32). Effect kinds 7 / 8, one slot per drop (the rain cap is
        // 0x80 on 09-0b, 0x40 elsewhere — 128 slots cover both [M InitDrops]).
        // Only a rain level gets the pool: a SNOW_gen placement (type 0xDF) with DropType (param 0) = 1,
        // the same placement `Drones` builds its weather from — elsewhere the 128 slots were dead weight.
        let rains = level.maps.iter().flat_map(|m| m.placements.iter())
            .any(|p| p.flags & 0xff == 0xdf && p.params.iter().find(|(i, _)| *i == 0).is_some_and(|&(_, v)| v as u8 == 1));
        if drones.is_some() && rains {
            for (kind, mesh) in [(7u8, 0x0200_0290u32)] {
                let Some(&(m, o)) = level.by_hash.get(&mesh) else { continue };
                let ident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
                let rplacement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
                for slot in 0..RAIN_SLOTS {
                    let rinst = Instance { map: m, object: o, matrix: ident, placement: rplacement.clone() };
                    add_instance(level, &rinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    attachments.push(Attachment { character: 0, bone: 0, local: rinst.matrix, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: Some((kind, slot, mesh)), silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                }
            }
        }
        // fix-satchel-tripbomb: the satchel's countdown digits — the 0x3b tail of `Bullet_DoTrails`
        // (0x12d890:485-553) spawns two kind-0x40 objects every tick, gfx `SatchelDigits[sec/10]` and
        // `SatchelDigits[sec%10]` [L SatchelDigits @0x2bed90 = 0x2000791..0x200079a], linked to the
        // satchel's cel. The port pools them as Effect kinds 4 (tens) / 5 (units): 2 slots per digit
        // mesh (the first two stuck satchels draw).
        if drones.is_some() {
            let dplacement = level.maps[level.main_map().unwrap_or(0)].placements.first().cloned().unwrap_or_default();
            let dident = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], [0.0; 3]);
            for digit in 0..10u32 {
                let mesh = 0x0200_0791 + digit;
                let Some(&(dm, dobj)) = level.by_hash.get(&mesh) else { continue };
                for kind in [4u8, 5u8] {
                    for slot in 0..2 {
                        let dinst = Instance { map: dm, object: dobj, matrix: dident, placement: dplacement.clone() };
                        add_instance(level, &dinst, Layer::Effect, Some((0, Some(attachments.len() as u32))), &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        attachments.push(Attachment { character: 0, bone: 0, local: dinst.matrix, flash: None, tracer: None, dropped: false, aims_light: None, projectile: None, script: None, gas: None, effect: Some((kind, slot, mesh)), silencer: false, quiver_bolt: None, swap_piece: false, watch_hand: None, pda_digit: None });
                    }
                }
            }
        }
        // Level coronas (`Corona_Create` 0x18da00, placement type 0xFC): class-1 objects with NO update
        // function — static glow quads at their placement transforms, in every view mode. The glist's +0x44
        // flags word (bit 0x1: `View_AddObjects` draw list 1, mirrored sort key) is the object's own flags
        // here, so `Layer::World` puts them in the alpha list with the leaves' own blend state [L 13/13].
        crate::phase_mark("b5e occluder/scan/digit pools", t); t = std::time::Instant::now();
        if let Some(m) = level.main_map() {
            for pl in nf_game::corona::placements(&level.maps[m].placements) {
                let p = &level.maps[m].placements[pl];
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                if let Some((map, object)) = target {
                    // `NF_HIDE` applies here too (the 0xFC skip above leaves these out of the loop's check).
                    // fix-striped-boxes: `Corona_Create` 0x18da40-48 copies `*(glist+0x44)` to obj+0xF0, so
                    // `View_AddObjects` 0x1e62d8:33's `(obj+0xF0 & 0x8010) == 0` gate applies — a 0x10-flagged
                    // corona glist never draws (none on the disc today; the census pins it).
                    if level.maps[map].objects[object].flags & 0x10 != 0 { continue; }
                    if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && level.maps[map].objects[object].name.contains(n)) { continue; } }
                    let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                    add_instance(level, &inst, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                }
            }
        }
        // Wire visuals (`Wire_Create` 0x1981d8, placement type 0x3E): the wire's visual mesh is the placement's
        // object (the celglist), like coronas. The wire placement is a dynamic entity (no 0x8000 CREATE_CEL),
        // so it's filtered from the main instance loop; re-add it here [M 0x1981d8].
        if let Some(m) = level.main_map() {
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x3e) {
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                if let Some((map, object)) = target {
                    // fix-striped-boxes: `Wire_Create` 0x198254-5c copies `*(glist+0x44)` to obj+0xF0, so
                    // `View_AddObjects` 0x1e62d8:33's `(obj+0xF0 & 0x8010) == 0` gate applies — a 0x10-flagged
                    // wire glist never draws (none on the disc today; the census pins it). Ladders stay
                    // ungated: `Ladder_Create` 0x1966c0 stores only obj+0xD8 (0x19671c), never obj+0xF0.
                    if level.maps[map].objects[object].flags & 0x10 != 0 { continue; }
                    let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                    add_instance(level, &inst, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                }
            }
        }
        // Ladder visuals (`Ladder_Create` 0x1966c0, placement type 0x22, class 0x2D): the drawn mesh is the
        // placement's resolved object — `Ladder_Create` stores it at `obj+0xD8` (direct store 0x19671c, no
        // `Control_SetGList`), and the generic dynamic-object draw `View_DrawObjList` 0x1e69a0 →
        // `View_DrawObjectGlist` 0x1e6780 renders `obj+0xD8` at `obj+0x90` — the placement transform
        // `control_create_object` 0x1344e0 built (`View_RotTransMatrix` 0x1e5168). Class 0x2D has no update
        // function (docs/atlas/object.md control_funcs), so `obj+0x90` never changes: a static World-layer
        // mesh at the placement transform is the game's draw exactly. No ambient dim: `Ladder_Create` never
        // calls `Control_SetGList`, so `obj+0xF0 & 0x800` is clear and `View_SetupRenderModes_Tweak`
        // 0x1e5888 takes the white-tweak branch [M]. A ladder whose sphere straddles cels is drawn by
        // `View_AddForcedObjects` 0x1e5fb8 instead of the per-cel path — still exactly once [M
        // Controls_StraddleTest 0x134778].
        if let Some(m) = level.main_map() {
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x22) {
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                if let Some((map, object)) = target {
                    let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                    add_instance(level, &inst, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    ladders_drawn += 1;
                }
            }
        }
        // Models-misc: the static dynamic entities (docs/models/misc.md has the full per-kind evidence).
        // `View_DrawObjList` 0x1e69a0 → `View_DrawObjectGlist` 0x1e6780 draws `obj+0xD8` at `obj+0x90`;
        // `control_create_object` 0x1344e0:20-29 builds `obj+0x90` from the placement pos/rot, and none of
        // these creators' updates move the matrix (the lock's unlock spline, the rotor's obj+0x50 spin, the
        // rigid bodies' RB_EulerEvolve and the gun imp's turret child all start from here), so a static
        // World-layer instance at the placement transform is the game's draw at rest exactly. Per kind:
        //   0x2E Lock_Create 0x1ae1f8:55 (direct obj+0xD8 store) · 0x30 Monitor_Create 0x1b08b0:26
        //   (Control_SetGList) · 0x34 GunImp_Create 0x131f20:68-69 (parent; the fixed-hashcode turret
        //   child needs the ELF TurretGuns table, still open) · 0x35 PCQWorm_Create 0x1b07a8:12-13 ·
        //   0x3B DynamicObject_Create 0x18dbf8:19-31 (hidden until its
        //   switch channel when param 0 ≠ 0 — those stay skipped)
        //   0x71 rotor_init 0x18f4b8:24 · 0xD5 OneSided_Create 0x1e7ce8:14 (its per-frame backface self-cull
        //   is the port's triangle culling: `OneSided_Update` 0x1e7d58 — control_funcs[0x4c], dispatched by
        //   `control_movement_object_handler` 0x1336f8's `control_funcs + kind*0xC` walk [M 0x1336f8; the
        //   kind byte OneSided_Create writes at obj+0xff = 0x4c is exactly (0x26fc20 − control_funcs
        //   0x26f890)/0xC [M symtab + decomp]] — dots (obj pos − cam pos) against the obj matrix's dir and
        //   sets obj+0xF0 bit 0x10 when facing away; reached whenever a level places type-0xd5 objects —
        //   07000041 does [D levels.md census]. The port's per-triangle backface cull of the placed glist
        //   is the same job at draw time) · 0xD8 RB_Create 0x1d6bf0:52 (rigid bodies evolve obj+0x90 once
        //   disturbed; at rest = placement) · 0xE1 Destroy_Create 0x12c318:23 (death hides the
        //   mesh: Destroy_Smash 0x12bb60 swaps the glist to tag+0x00 — the LOD pointer or the
        //   next 0x50 glist slot, undrawable for the C5 leaves [L]; see destroy_object_state
        //   below — the pre-2026-10-28 "death keeps the object drawn" reading missed the swap's
        //   always-nonzero argument [M 0x12bbb8 asm + 0x1d1b68])
        //   · 0xE3 Flicker_Create 0x1acb00:49 · 0xFA ThirdIcon_Create 0x197070:33 / 0xFD CamSubject_Create
        //   0x1bc5d8:17 / 0x4A Apocalypse_Create 0x18dcc0:20 — these attach whatever glist the placement
        //   carries; `psiDrawObjectMatrix` no-ops on an empty one, so only objects with gfx batches draw.
        //   0xDA Car_Create 0x1305b0:20-21 body, plus Car_InitBits 0x12fec8's two fixed-hashcode children:
        //   0x2000502 at the body's exact matrix (Mat_Copy 0x12fec8:33) and 0x2000505 at body translation
        //   + identity rotation [M 0012fec8:13-33]. The +1.5 lift and the 0x2000194 re-skin are the MPSettings
        //   variant (data+0x112), not the SP default.
        // Kinds that must NOT draw here (docs/models/misc.md): 0x29 switch and 0x31 fusebox (obj+0xD8 = 0 —
        // no mesh is ever attached [M 001adcc0:35, 001b0448:25]); 0x27 GT (obj+0xF0 |= 0x10 at create: hidden
        // until deployed [M 0017ffa0:78]); 0x32 rainbox (the mesh is never drawn — RainBox_Clip 0x18d0d0 only
        // reads the AABB [M 0018d0d0:20-35]); 0x38/0x39 space laser/missile (invisible controllers, visuals
        // via scripts); 0xF2 emitter (no glist ever [M 0017ae00:100-101]); 0xD4 cloud (sky pipeline, drawn by
        // sky_instances); 0xF3 tree (anim-object path, like characters).
        // fix-gadgets2 (red/yellow boxes drew at the wall shimmy and the grapple):
        // 0x3F creep walls are invisible: the creator builds a bare control object and attaches NO gfx
        // hash — `CreepWall_Create` 0x196fc8 sets class 0x2B with no gfx attach — so `View_DrawObjList`'s
        // empty-glist no-op never draws them; live, the 0x2B glists chain to boxes 0 [L ee_start_05.bin:
        // obj 0x16d69a0/0x16e8d90, psiDraw chain boxes 0x0]. The marker meshes the sweep used to draw are
        // the placements' referenced objects, which the game discards.
        // fix-grapple-hook CORRECTION for 0x3A: the grapple points ARE drawn. The dynamic-block parser
        // resolves the placement's object and hands it to the creator as the glist argument
        // [M parsemap_block_map_data_dynamic 0x1d1b68: hashtable_getitem(hashcode), or pCelList +
        // index*0x50 when the hashcode is −1; case 0x3a passes it through 0x1d04f0:237-239],
        // `Grapple_Create` 0x198298:24-30 stores it at obj+0xD8 with obj+0xF0 = *(u32*)(glist+0x44),
        // and the generic dynamic-object draw `View_DrawObjList` 0x1e69a0 → `View_DrawObjectGlist`
        // 0x1e6780 renders obj+0xD8 at obj+0x90 (the placement transform `control_create_object`
        // 0x1344e0 built) via `psiDrawObjectMatrix` 0x124708. Live proof [L ee_start_05.bin]: both
        // class-0x51 objects (0x16d80a0 at 13.75,32.29,57.30 = placement #282; 0x16d81c0 at
        // −3.40,42.94,−71.77 = #281) hold glist 0x170d6a0 (the `graple_pulley` entity), +0xF0 = 0x4000
        // (no 0x10 hide bit, no 0x8000 forced bit — the point did not straddle), +0xF8 = 0x1f (display
        // bit 0 passes `View_CaptureScene` 0x1e6df0's `object_display_mask = 1 << viewer_LOD`, 0x30d87c),
        // and the psiDraw chain glist+4 → +8 → +4 lands on box tree 0x9b3260 — DRAWABLE. The zero
        // `hashtable_set_object_to_entity_gfx` calls fix-gadgets2 cited are irrelevant: the glist comes
        // in as an argument, not from a hash attach. PCSX2 shows the pulley at the
        // spot; the port drew only the bracket. obj+0xF0 & 0x800 is clear, so no ambient dim (the
        // `View_SetupRenderModes_Tweak` 0x1e5888 white-tweak branch, like the ladders).
        if cels_only {
        // Grapple-point visuals (`Grapple_Create` 0x198298, placement type 0x3A, class 0x51): the
        // pulley-with-hook the Q-Claw latches onto (fix-grapple-hook). The mesh is the placement's
        // resolved object (the `graple_pulley` entity, handed over as the glist argument — the chain
        // documented at the fix-gadgets2 correction above), drawn once at the placement transform.
        // Gate: `psiDrawObjectMatrix` 0x124708 no-ops when the glist's box chain is empty, so objects
        // with no batches stay undrawn (the creep walls' case); a 0x400 mirror-only object has no
        // mirror viewer here. The 0x3A placements never carry 0x8000 CREATE_CEL [D placements], so
        // the static instance loop skips them and this sweep is the only draw — in --all the generic
        // instance loop covers them (debug view), hence the cels_only gate.
        // fix-striped-boxes: also the game's no-draw bit — `View_AddObjects`
        // 0x1e62d8:33 submits a dynamic object to the viewer draw lists only when
        // `(obj+0xF0 & 0x8010) == 0` (obj+0xF0 is `Control_SetGList` 0x134960's copy of
        // `celglist+0x44`, i.e. MapObject.flags; disasm 0x1e6344-4c), so a 0x10-flagged glist never
        // draws even when attached — bit 0x10 marks collision-only brushes (the striped editor
        // markers) which `Collide_Pick` keeps solid. The pulley's glist carries 0x4000 [L
        // ee_start_05.bin 0x16d80a0/0x16d81c0] and passes.
        if let Some(m) = level.main_map() {
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x3a) {
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let obj = &level.maps[map].objects[object];
                if !obj.gfx.leaves.iter().any(|l| !l.batches.is_empty()) || obj.flags & 0x400 != 0 || obj.flags & 0x10 != 0 { continue; }
                if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && obj.name.contains(n)) { continue; } }
                let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                grapple_hooks_drawn += 1;
            }
        }
        if let Some(m) = level.main_map() {
            const MISC_KINDS: [u8; 15] = [0x2e, 0x30, 0x34, 0x35, 0x3b, 0x71, 0xd5, 0xd8, 0xe1, 0xe3, 0xda, 0xfa, 0xfd, 0xfe, 0x4a];
            let nf_hide = std::env::var("NF_HIDE").ok();
            // mp-lobby-settings: the MP creates that return no object draw nothing — `GunImp_Create`
            // 0x131f20:52-55 (Gun Emplacements off), `Destroy_Create` 0x12c318:14 (Explosive
            // Scenery off), `Car_Create` 0x1305b0:12-17 (Vehicles off, and always on Ravine
            // 0x0700004b) [M]. Vehicles on: `Car_Create`'s variant — Helicopters (and Random's
            // 51 %) swap the body to entity 0x2000194 raised 1.5 [M :30-49] with the
            // 0x2000b4d/0x2000b4e children [M Car_InitBits 0x12fec8:13-23]. The Random roll:
            // the game draws it from the GLOBAL RNG pair — Rand_Rand(100) < 51 -> heli [M
            // Car_Create 0x1305b0:40, disasm 0x1306c0 jal Rand_Rand__FUi(100), 0x1306c8 sltiu
            // 0x33; Rand_Rand 0x1e3738:6-8 mutates the shared SEED_X/SEED_Y globals] — one
            // stream shared by every draw in the game, so its state at create time is
            // play-history the port cannot reproduce. The port rolls the SAME rule
            // (rand(100) < 51) from a stream seeded by the lockstep seed (`^ 0x4341_5253`), the
            // determinism substitution that keeps every machine's draw identical; the roll
            // probability and threshold are the game's, the stream is the documented port
            // choice. The vehicles draw only — the port's MP has no drivable car (docs/GAPS.md).
            let ctx_level = drones.map_or(0, |d| d.level_id);
            let mut car_rng = mp.map(|ms| nf_game::drone::Rng::new(ms.seed ^ 0x4341_5253));
            for (pi, p) in level.maps[m].placements.iter().enumerate().filter(|(_, p)| MISC_KINDS.contains(&((p.flags & 0xff) as u8))) {
                let kind = (p.flags & 0xff) as u8;
                let mut heli = false;
                if let Some(ms) = mp {
                    if kind == 0x34 && ms.gun_emplacements == 0 { continue; }
                    if kind == 0xe1 && ms.explosive_scenery == 0 { continue; }
                    if kind == 0xda {
                        if ms.vehicles == 0 || ctx_level == 0x0700_004b { continue; }
                        heli = match ms.vehicles { 1 => false, 3 => car_rng.as_mut().map_or(true, |r| r.rand(100) < 0x33), _ => true };
                    }
                }
                // 0x3B hidden-until-channel (DynamicObject_Create 0x18dbf8:31, param 0 = the s16 channel).
                if kind == 0x3b && p.params.iter().find(|(i, _)| *i == 0).is_some_and(|(_, v)| *v != 0) { continue; }
                // NOTE on the 0xe1 statics: they bake here (build time — nothing is dead yet), so the
                // destroyed-glist swap's draw side is a PER-FRAME gate, not a build skip. [M
                // Destroy_Smash 0x12bb60 → `Control_SetGList(obj, *(tag+0x00))` — the swap the sim
                // models as `Destroy::dead`]: a dead leaf's object rides the destroyed glist entry —
                // the block-0x27 LOD pointer when the disc word is set, else the parsemap fallback
                // slot `pCelList + idx·0x50 + 0x50` [M 0x12bbb8 asm + 0x1d1b68]. The C5 leaves carry
                // no LOD variant [D probe lod_word −1 for every 0d 0xe1], so the swap target has no
                // batches and the doorway ends open [L]. The drawn 0xe1 instance's world centre is
                // recorded below (the same `transform(matrix, obj.centre)` the group's cull sphere
                // uses), and the per-frame draw filter (gpu.rs, over `Renderer::destroy_dead`) skips
                // the dead leaf's materials. A leaf WITH a LOD variant would draw the LOD mesh here
                // instead — no disc row on 0d needs it [S unverified].
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let obj = &level.maps[map].objects[object];
                // Empty glists draw nothing (`View_DrawObjectGlist`'s psiDrawObjectMatrix no-ops), and a
                // mirror-only object (glist flags 0x400, View_AddObjects 0x1e62d8:38-47) has no mirror viewer here.
                // fix-striped-boxes: the game's no-draw bit 0x10 — `View_AddObjects` 0x1e62d8:33 submits a
                // dynamic object to the viewer draw lists only when `(obj+0xF0 & 0x8010) == 0` (obj+0xF0 is
                // the creators' copy of `celglist+0x44`, i.e. MapObject.flags) — but only where a creator
                // actually makes the copy. fix-striped-boxes2 verified the copy for EVERY kind in the list
                // (address + the store): 0x30 `Monitor_Create` 0x1b08b0:26 → `Control_SetGList` 0x134960:12-14
                // (`obj+0xF0 = *(glist+0x44) | obj+0xF0 & 0x30008000`); 0x34 `GunImp_Create` 0x131f20:68 →
                // `Control_CreateObjEx` 0x1343c8:29-36 (`obj+0xF0 = *(glist+0x44) | param_10`, then |= 0x1000
                // at 0x131f20:75/87); 0x35 `PCQWorm_Create` 0x1b07a8:12 → `Control_CreateObjEx` (same copy);
                // 0x3b `DynamicObject_Create` 0x18dbf8:21-23 (direct `obj+0xF0 = *(glist+0x44)`); 0x71
                // `rotor_init` 0x18f4b8:25 (direct); 0xe1 `Destroy_Create` 0x12c318:23 → `Control_SetGList`
                // — and the creator itself honours the bit, zeroing the smash target when set (0x12c318:57-59
                // `if (obj+0xF0 & 0x10) *tag = 0`); 0xe3 `Flicker_Create` 0x1acb00:52 (direct); 0xfa
                // `ThirdIcon_Create` 0x197070:34-37 (`obj+0xF0 = *(glist+0x44) | 0x1000`); 0xfd
                // `CamSubject_Create` 0x1bc5d8:20 (direct); 0x4a `Apocalypse_Create` 0x18dcc0:24 (direct);
                // 0xda `Car_Create` 0x1305b0:24 → `Control_CreateObjEx` with OR-mask 0x44 (then |= 0x40 at
                // :57; latent only — no 0x10-flagged mesh-ful car glist on the disc, the census pins it).
                // Nothing clears bit 0x10 later for these kinds: none of their Update/Activate functions
                // write obj+0xF0 (checked Monitor_Update/Activate/ApplyDamage 0x1afcc8/0x1b0058/0x1b0968,
                // PCQWorm_Update/Activate 0x1aee20/0x1b0840, Destroy_Update/Smash/CollisionHandler
                // 0x12c4f8/0x12bb60/0x12bd48, ThirdIcon_Update 0x196e80, Flicker_Update 0x1ab960,
                // Apocalypse_Update 0x18d4f0, rotor_update 0x18f568, CamSubject_Update 0x1bc668); the only
                // +0xF0 rewriters elsewhere are 0x3b's own channel restore (`DynamicObject_Update` 0x18d3b8:16/
                // 34/39 stores back the word `DynamicObject_Create` 0x18dbf8:27 captured from *(glist+0x44)
                // — a set bit survives), `Drone_PauseNoDraw` 0x13a710 (drone objects only), `SP_HideObj`
                // 0x1dcbf0 → `Script_HideObj` 0x1db808 (a script's own stream entities only), and the
                // bullet/player/pickup/car-child paths (other kinds). So the game never draws a
                // FLAGS_COPYING-kind object whose glist carries 0x10 — the probed instances (05's
                // "WallHug_Blocker*" 0x3b + "TouchPlate_GrappleLedge"/"Valley grapple_touchplate" 0xfa,
                // 08's SmallExplosionBox/0d's WallHug_Blocker1_Long, 11/12/14's 3RD_Person_Icon 0xfa, 12's
                // "Computer to hack" 0x35 (an invisible use volume — the laptop is baked geometry), 14's 12
                // SWITCH_Consoles_Evilbase 0x30 + 19 DESTROY_BoxSatellite 0xe1, 15's 3 DESTROY_BoxSatellite,
                // 1b's 8 Apocalypse 0x4a + the DESTROY_Box* 0xe1) are all invisible gameplay/collision
                // volumes in the real game too; the visible consoles/laptops are baked world geometry.
                // NON-copying kinds (the game draws their 0x10-flagged meshes — no gate): `Lock_Create`
                // 0x1ae1f8 (0x2e), `OneSided_Create` 0x1e7ce8 (0xd5), `RB_Create` 0x1d6bf0 (0xd8) — plain
                // `control_create_object` + a direct obj+0xD8 store, no obj+0xF0 write — and now also
                // `Mine_Create` 0x17ce28 (0xfe, verified the same way; `Mine_Update` 0x17c930:142's 0x30
                // store is the mine hiding ITSELF when armed, not a glist copy). Bit 0x10 marks
                // collision-only brushes which `Collide_Pick` (no 0x10 test) keeps solid while nothing
                // ever draws them. The sweep drew the striped ones because it lacked the gate — a
                // red/yellow striped box on 07000005's trench.
                const FLAGS_COPYING: [u8; 11] = [0x30, 0x34, 0x35, 0x3b, 0x71, 0xda, 0xe1, 0xe3, 0xfa, 0xfd, 0x4a];
                if obj.flags & 0x10 != 0 && FLAGS_COPYING.contains(&kind) { continue; }
                if !obj.gfx.leaves.iter().any(|l| !l.batches.is_empty()) || obj.flags & 0x400 != 0 { continue; }
                if let Some(h) = &nf_hide { if h.split(',').any(|n| !n.is_empty() && obj.name.contains(n)) { continue; } }
                // level-1b-audit: in the sim-posed build the gated dynamic object rides a posed slot
                // below (after the sensors) instead of this static instance — the static draw never
                // collapses when the sim deletes the object (`DynamicObject_Update` 0x18d3b8:26 mode-0
                // armed-B → obj+0xFE bit 0 → the delete pass), so 0700001b's control-room glass kept
                // rendering after ch 10 (Drake's gate) fired. In --all the static draw stays.
                if kind == 0x3b && cels_only { *misc_drawn.entry(0x3b).or_default() += 1; continue; }
                // The rotors spin on a posed slot below (after the dynamic objects): the static draw
                // held the placement pose forever [M rotor_update 0x18f568 writes obj+0x50 every tick].
                if kind == 0x71 && cels_only { *misc_drawn.entry(0x71).or_default() += 1; continue; }
                // A lock whose object has a LOD variant (the keypads: "Lock - KeypadLocked" →
                // "Lock - KeypadUnlocked" [D block 0x27]) rides the posed slot space below, so the
                // unlock's `Control_NextLOD` can swap it.
                if kind == 0x2e && level.maps[map].lod_target(object).is_some() { *misc_drawn.entry(kind).or_default() += 1; continue; }
                // A gun emplacement (0x34) turns with its mount: its base and gun child ride the posed
                // slot space below (`GunImp_Update` 0x1320d8:162-173 re-poses both every tick).
                if kind == 0x34 { *misc_drawn.entry(kind).or_default() += 1; continue; }
                // The 0x800 dim, the same rule the main static loop applies (View_SetupRenderModes_Tweak 0x1e5888).
                let mult = if obj.flags & 0x800 != 0 {
                    let mx = nf_formats::placement::object_matrix(p.rotation, p.translation);
                    mul_bytes(cel_ambient(&room_cels, transform(&mx, obj.centre), 0).unwrap_or([0xff; 3]))
                } else { [1.0; 3] };
                if heli {
                    let has_gfx = |(cm, co): (usize, usize)| level.maps[cm].objects[co].gfx.leaves.iter().any(|l| !l.batches.is_empty());
                    let mut hm = nf_formats::placement::object_matrix(p.rotation, p.translation);
                    hm[3][1] += 1.5;
                    for (k, h) in [0x0200_0194u32, 0x0200_0b4d, 0x0200_0b4e].into_iter().enumerate() {
                        let Some(c) = level.by_hash.get(&h).copied().filter(|&c| has_gfx(c)) else { continue };
                        let mut cm = hm;
                        if k == 2 { cm[0] = [1.0, 0.0, 0.0, 0.0]; cm[1] = [0.0, 1.0, 0.0, 0.0]; cm[2] = [0.0, 0.0, 1.0, 0.0]; }
                        add_instance(level, &Instance { map: c.0, object: c.1, matrix: cm, placement: p.clone() }, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        *misc_drawn.entry(0xda).or_default() += 1;
                    }
                    continue;
                }
                let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::World, None, &mut groups, Some(&mut bounds), None, None, None, mult, &room_cels, u32::MAX, u32::MAX);
                // The 0xe1 destroy leaf's world centre for the per-frame dead gate (see the NOTE above):
                // the same `transform(matrix, obj.centre)` add_instance bakes into the group's cull
                // sphere, so the bits match the material's `centre` the draw filter reads.
                if kind == 0xe1 { destroy_centres.push((transform(&inst.matrix, obj.centre), pi)); }
                *misc_drawn.entry(kind).or_default() += 1;
                // The car's two fixed-hashcode children (Car_InitBits 0x12fec8), drawn only with a car body.
                if kind == 0xda {
                    let has_gfx = |(cm, co): (usize, usize)| level.maps[cm].objects[co].gfx.leaves.iter().any(|l| !l.batches.is_empty());
                    // Child 1 (data+0xE0, 0x2000502): a full Mat_Copy of the body's matrix [M 0x12fec8:33].
                    if let Some(c) = level.by_hash.get(&0x0200_0502).copied().filter(|&c| has_gfx(c)) {
                        add_instance(level, &Instance { map: c.0, object: c.1, matrix: inst.matrix, placement: p.clone() }, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        *misc_drawn.entry(0xda).or_default() += 1;
                    }
                    // Child 2 (data+0xE4, 0x2000505): identity rotation + the body's translation [M 0x12fec8:13-33].
                    if let Some(c) = level.by_hash.get(&0x0200_0505).copied().filter(|&c| has_gfx(c)) {
                        let mut cm = inst.matrix;
                        cm[0] = [1.0, 0.0, 0.0, 0.0]; cm[1] = [0.0, 1.0, 0.0, 0.0]; cm[2] = [0.0, 0.0, 1.0, 0.0];
                        add_instance(level, &Instance { map: c.0, object: c.1, matrix: cm, placement: p.clone() }, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                        *misc_drawn.entry(0xda).or_default() += 1;
                    }
                }
            }
        }
        } // cels_only: in --all the generic instance loop already draws these kinds (debug view only).
        // Searchlight parts (`Searchlight_Create`'s creation order = `part_placements`): each part's object as
        // Character-layer vertices posed by the simulation every tick — the mount and head rotating, the beam
        // cone along the aim, the spot on its ray hit, the corona a camera billboard (docs/systems/world-objects.md).
        crate::phase_mark("b5f coronas/wires/ladders/misc", t); t = std::time::Instant::now();
        let mut lock_matrices: Vec<Mat4> = Vec::new();
        let mut dynobj_matrices: Vec<Mat4> = Vec::new();
        let mut rotor_placements: Vec<([f32; 3], [f32; 3])> = Vec::new();
        let mut gunimp_placements = 0usize;
        let mut door_scene_hidden = std::collections::HashSet::new();
        let mut sl_lod: Vec<u16> = Vec::new();
        if let Some(m) = level.main_map() {
            for (pi, pl) in nf_game::searchlight::part_placements(&level.maps[m].placements).into_iter().enumerate() {
                let p = &level.maps[m].placements[pl];
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                if let Some((map, object)) = target {
                    let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                    add_instance(level, &inst, Layer::Character, None, &mut groups, None, Some(pi as u16), None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                }
            }
            // The off-kill's / break's `Control_NextLOD(head)` [M Searchlight_Update 0x1cb900:138, :454;
            // Control_NextLOD 0x1349b0:8-12]: the head swaps to its block-0x27 LOD object (07000006:
            // OBJECT_SearchLight1 -> OBJECT_SearchLightBroken [D]). Each part whose object carries a LOD
            // target gets that mesh on its own slot after the parts (`sl_lod[k]` = the part it shadows);
            // the palette shows it, and hides the part's own mesh, once the sim sets `lod_next`.
            let sl_parts = nf_game::searchlight::part_placements(&level.maps[m].placements);
            for (pi, &pl) in sl_parts.iter().enumerate() {
                let p = &level.maps[m].placements[pl];
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let Some(lod) = level.maps[map].lod_target(object) else { continue };
                let slot = (sl_parts.len() + sl_lod.len()) as u16;
                let inst = Instance { map, object: lod, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, Some(slot), None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                sl_lod.push(pi as u16);
            }
            // Doors (`Door_Create`'s creation order = `door_placements`): each door's object as Character-layer
            // vertices posed by the simulation every tick (`Door_Interp`'s matrix at `obj+0x90`); the mesh is
            // opaque, so the pass ordering is depth-correct against the world (docs/systems/world-objects.md).
            // cutscene-door-dup: collect the door placements whose object's glist flags (celglist+0x44 =
            // MapObject.flags) carry 0x40000: Control_SetGList 0x134960 clears obj+0xF8's channel-4 bit for
            // them, and Script_CameraStart 0x1d9140 renders ONLY camera 4 while a scene runs (mask 0x10
            // [M View_CaptureScene 0x1e6df0 / build_alloc_viewer 0x1c9378]), so both draw walks
            // (View_AddObjects 0x1e62d8:33 / View_AddForcedObjects 0x1e5fb8:79) skip them: the placed doors
            // hide during a cut-scene. 07000014's control-room pair rides the packs Door_Slide_3A/3B
            // (entity-params block+8 = 0x00040000, idx 116/117 [D]); the scene's leaf hashes 0x02000891/92
            // resolve to Elevator_Door_Slide_3A/3B (flags 0, map 010001c6 [D]) - one door, moving.
            for (di, pl) in nf_game::door::door_placements(&level.maps[m].placements).into_iter().enumerate() {
                let p = &level.maps[m].placements[pl];
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                if level.maps[map].objects[object].flags & 0x40000 != 0 { door_scene_hidden.insert(pl); }
                // `NF_HIDE` leaves the door out of the draw (the A/B check; the simulation still runs it).
                if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && level.maps[map].objects[object].name.contains(n)) { continue; } }
                let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some(di as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
            // SimpleScripts (`SS_Create` 0x1a9c58 / `SS_Update` 0x1a9948, map-tower2): the animated 0xE2
            // placements (key count non-zero) ride the door slot space — their vertices pose from
            // `door_base + n_doors + k`, the SS list's matrices, every tick. At rest each sits at key 0
            // (= its placement transform [D]); the switch channel swaps it to the other key (the
            // 07000012 lift screens, shutters and lift doors).
            let n_doors = nf_game::door::door_placements(&level.maps[m].placements).len();
            // The same construction the sim's `Drones::attach_level` runs, so the slot order matches
            // the matrix upload's `sss.list` order (deterministic: placement order, same filter).
            let ss_list = nf_game::ss::SsList::create(&level.maps[m].placements, &level.path_blocks(), |_p| (0, 0.0));
            // (hash 0 here: the render pass does not need the collision hash — it reads the placement
            // object directly below; the sim's attach_level resolves the real one for set_dynamic.)
            for (si, ss) in ss_list.list.iter().enumerate() {
                let p = &level.maps[m].placements[ss.placement];
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some((n_doors + si) as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
            // The breakables (0x20, `Break_Create` 0x12c230): `nf_game::breakable::Breaks::create`'s
            // order (every 0x20 placement, placement order) after the SS slots — posed at their
            // placement matrix while alive and collapsed once `Break_Kill` 0x12b970 marks them dead
            // [M obj+0xFE bit 0 → the delete pass]. A cel-creating 0x20 keeps its static cel draw
            // and its slot stays empty.
            let n_ss = ss_list.list.len();
            let mut bi = 0usize;
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x20) {
                let slot = n_doors + n_ss + bi;
                bi += 1;
                if p.creates_cel() { continue; }
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && level.maps[map].objects[object].name.contains(n)) { continue; } }
                let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some(slot as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
            // The locks with a LOD variant (`Lock::create`'s order — every 0x2E placement), after the
            // breakables: two slots per lock, the placed object at `2·li` and its LOD variant at
            // `2·li + 1`, both at the placement matrix [M Lock_Create 0x1ae1f8: obj+0xD8 = the glist;
            // `Control_NextLOD` 0x1349b0 swaps obj+0xD8 to glist+0x40]. `frame_state` shows the variant
            // once the sim's `lod_next` is set. (In --all the generic loop draws the lock statically.)
            let lock_base = n_doors + n_ss + bi;
            for (li, p) in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x2e).enumerate() {
                let mx = nf_formats::placement::object_matrix(p.rotation, p.translation);
                lock_matrices.push(mx);
                if !cels_only { continue; }
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let Some(next) = level.maps[map].lod_target(object) else { continue };
                for (k, o) in [object, next].into_iter().enumerate() {
                    let inst = Instance { map, object: o, matrix: mx, placement: p.clone() };
                    add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some((lock_base + 2 * li + k) as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                }
            }
            // The gun emplacements (0x34, `Props::create`'s order — every 0x34 placement), after the
            // locks: GUNIMP_SLOTS per mount — the placement's base object, then one slot per
            // `TURRET_ARTS` gun child (`GunImp_Create` 0x131f20:85-89 builds the child from
            // `TurretGuns[ext+0x9d].art`; the MP weapon set picks the entry later in this build, so
            // every art gets a slot and `frame_state` shows the mount's own). Base at `base_matrix`,
            // gun at `barrel_matrix` [M GunImp_Update 0x1320d8:162-173].
            let n_locks = level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x2e).count();
            let gun_base = lock_base + 2 * n_locks;
            if cels_only {
                for (gi, p) in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x34).enumerate() {
                    let slot0 = gun_base + GUNIMP_SLOTS * gi;
                    let mx = nf_formats::placement::object_matrix(p.rotation, p.translation);
                    let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                        else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                    let has_gfx = |(cm, co): (usize, usize)| level.maps[cm].objects[co].gfx.leaves.iter().any(|l| !l.batches.is_empty());
                    if let Some((map, object)) = target.filter(|&t| has_gfx(t)) {
                        let inst = Instance { map, object, matrix: mx, placement: p.clone() };
                        add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some(slot0 as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    }
                    for (a, art) in nf_game::gunimp::TURRET_ARTS.iter().enumerate() {
                        let Some((map, object)) = level.by_hash.get(art).copied().filter(|&t| has_gfx(t)) else { continue };
                        let inst = Instance { map, object, matrix: mx, placement: p.clone() };
                        add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some((slot0 + 1 + a) as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    }
                }
            }
            gunimp_placements = level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x34).count();
            // The security cameras (0x28, `Sensors::create`'s order and spawn gate), after the gun
            // emplacements: three slots each — the camera body (the placement's own glist, `Sensor_Create`
            // 0x1af010 stores it at obj+0xD8) at the swept/tracked pose `obj+0x90`, the mount proxy
            // (param 7) re-aligned upright to the pose's direction by `Sensor_SetBeam` 0x1af398:50-54
            // every tick [M], and the sight beam: the lens proxy (param 0) is a gfx object whose glist
            // (`hashtable_set_object_to_entity_gfx`, 0x1af010:106-107) `Sensor_SetBeam` poses at the
            // camera's own matrix (`Mat_Copy(lens+0x90, obj+0x90)` 0x1af398:58) and stretches to the
            // beam's hit length via obj+0xE8 (range 0x1af398:72, or the mask-8 ray's first-hit
            // distance :78) [M]. The draw is the object's plain glist path (`View_DrawObjectGlist`
            // 0x1e6780:43-45 — the Camera_Beam glist's flags 0x241 [D probe] carry no bit-8 billboard
            // bit) with the object scale riding the mesh's length axis: the create ORs the
            // OnlyScaleZ bit (0x1af010:102) and the Camera_Beam mesh is unit-length along local +z
            // (z −0.0085..1.0283, x/y ±0.16 [D probe 0x02000360]), so the drawn beam runs lens →
            // hit at constant cross-section. State 3 clears the lens's display mask (0x1af618:248-250)
            // — no beam while disabled.
            let cam_base = gun_base + GUNIMP_SLOTS * gunimp_placements;
            if cels_only {
                for (si, p) in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x28 && p.flags & 0xa000 == 0).enumerate() {
                    let mx = nf_formats::placement::object_matrix(p.rotation, p.translation);
                    let has_gfx = |(cm, co): (usize, usize)| level.maps[cm].objects[co].gfx.leaves.iter().any(|l| !l.batches.is_empty());
                    let body = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                        else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                    let mount = p.params.iter().find(|(k, _)| *k == 7).map(|&(_, v)| v).filter(|&h| h != 0).and_then(|h| level.by_hash.get(&h).copied());
                    let beam = p.params.iter().find(|(k, _)| *k == 0).map(|&(_, v)| v).filter(|&h| h != 0).and_then(|h| level.by_hash.get(&h).copied());
                    for (k, t) in [body, mount, beam].into_iter().enumerate() {
                        let Some((map, object)) = t.filter(|&t| has_gfx(t)) else { continue };
                        let inst = Instance { map, object, matrix: mx, placement: p.clone() };
                        add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some((cam_base + 3 * si + k) as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
                    }
                }
            }
            // The gated dynamic objects (0x3B, `WorldObjects::create`'s order — every 0x3b placement,
            // placement order), after the sensors: one slot each at the placement matrix, collapsed
            // once the sim deletes the object (`DynamicObject_Update` 0x18d3b8:26 — mode 0 with armed
            // B sets obj+0xFE bit 0, the movement pass's delete; 0700001b's GLASS_ControlRoom (p1 =
            // ch 10, Drake's gate) is the level's one). The create-time hidden gate (p0 != 0 →
            // obj+0xF0 = glist | 0x30... the draw excluded) matches the static loop's.
            let n_sensors_build = level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x28 && p.flags & 0xa000 == 0).count();
            let dynobj_base = cam_base + 3 * n_sensors_build;
            for (di, p) in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x3b).enumerate() {
                let mx = nf_formats::placement::object_matrix(p.rotation, p.translation);
                dynobj_matrices.push(mx);
                if !cels_only { continue; }
                // A p0-gated object is created hidden (|0x30) and shown when p0's channel rises — the
                // slot carries it from load; the per-tick pose collapses it while hidden [M 0x18dbf8:27-29].
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let obj = &level.maps[map].objects[object];
                // The static loop's gates: the glist's own no-draw bit 0x10 (a FLAGS_COPYING kind), the
                // mirror-only 0x400, and an empty glist [M View_AddObjects 0x1e62d8:33,38-47].
                if obj.flags & 0x410 != 0 || !obj.gfx.leaves.iter().any(|l| !l.batches.is_empty()) { continue; }
                if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && level.maps[map].objects[object].name.contains(n)) { continue; } }
                let inst = Instance { map, object, matrix: mx, placement: p.clone() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some((dynobj_base + di) as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
            // The rotors (0x71, `WorldObjects::create`'s order), after the dynamic objects: one slot each,
            // posed every tick at the placement ROT + the sim's accumulated spin [M 0x18f4b8, 0x18f568].
            let rotor_base = dynobj_base + level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x3b).count();
            for (ri, p) in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0x71).enumerate() {
                rotor_placements.push((p.rotation, p.translation));
                if !cels_only { continue; }
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                let obj = &level.maps[map].objects[object];
                if obj.flags & 0x410 != 0 || !obj.gfx.leaves.iter().any(|l| !l.batches.is_empty()) { continue; }
                if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && level.maps[map].objects[object].name.contains(n)) { continue; } }
                let mx = nf_formats::placement::object_matrix(p.rotation, p.translation);
                let inst = Instance { map, object, matrix: mx, placement: p.clone() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, Some((rotor_base + ri) as u16), None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
            // Level-placed pickups (`Pickup_Create` 0x18dda0): `View_DrawObjectGlist` 0x1e6780 draws
            // `obj+0xD8` at `obj+0x90` — the same object carries the glist the renderer draws [M], so the
            // port draws them the way it draws doors: Character-layer vertices posed by the simulation every
            // tick at the pickup's matrix, collapsed once taken or hidden. Before fix-player2 only the
            // drone-dropped guns were drawn — a placed armour (kind 3, placed-only per the atlas) had no
            // instance at all and was invisible while fully takeable.
            for (pi, p) in placed_pickups.iter().enumerate() {
                if p.gone || p.hidden { continue; }
                // fix-objects: draw the RESOLVED object (`Pickup.obj` — the game draws obj+0xD8, the
                // celglist handed to `Pickup_Create`, not a hashcode re-lookup). The by-hash fallback
                // stays for pickups built without an object (tests) but cannot show an object whose
                // own hashcode is 0xFFFFFFFF (07000005's `oOBJECT_AmmoPickUp_9mm` [D]).
                let target = p.obj.or_else(|| level.by_hash.get(&p.model).copied());
                let Some((om, oi)) = target else { continue };
                let inst = Instance { map: om, object: oi, matrix: p.m, placement: level.maps[om].placements.first().cloned().unwrap_or_default() };
                add_instance(level, &inst, Layer::Character, None, &mut groups, None, None, None, Some(pi as u16), [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
            // LOAD_LEVEL triggers (0xE8): the placement carries a visible mesh (e.g. 07000005's `Door_Gate8` —
            // the castle gate door — and the route-c zip-wire exit `Locked_Door_Top_I`). In the real game
            // `Trigger_LoadLevelCreate` (0x1b2950) → `Trigger_Create` (0x1b1720) calls `control_create_object`
            // which attaches the glist to `obj+0xF0`, so the trigger entity IS drawn. The port's `instances()`
            // skips these placements (no 0x8000 CREATE_CEL bit), so without this loop the castle gate door is
            // invisible in play — nothing to interact with at the gate. Draw them here as
            // static World-layer meshes at their placement transform (they don't animate) [M 0x1b2950, 0x1b1720].
            for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0xe8) {
                let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
                let Some((map, object)) = target else { continue };
                // fix-striped-boxes AUDIT CORRECTION: LOAD_LEVEL triggers DO draw whatever their glist
                // flags. `Trigger_LoadLevelCreate` 0x1b2950 forces the type to 10 (0x1b2998/a0 store 10
                // to placement+0x2c) and `Trigger_Create` 0x1b1720 clears the no-draw bit for exactly
                // that type — 0x1b18c8-d4 `obj+0xF0 &= 0xffffffef` [M] — so the game draws these meshes
                // even when `glist+0x44` carries 0x10 (the four "Load Point"-family meshes on
                // 01/03/12/13, and 07000005's Door_Gate8). An earlier revision of this fix gated them
                // and hid four game-drawn meshes; reverted.
                if let Ok(h) = std::env::var("NF_HIDE") { if h.split(',').any(|n| !n.is_empty() && level.maps[map].objects[object].name.contains(n)) { continue; } }
                let inst = Instance { map, object, matrix: nf_formats::placement::object_matrix(p.rotation, p.translation), placement: p.clone() };
                add_instance(level, &inst, Layer::World, None, &mut groups, None, None, None, None, [1.0; 3], &room_cels, u32::MAX, u32::MAX);
            }
        }
        let (mut vertices, mut indices, mut materials, mut char_verts) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        // level-load-time: pre-reserve the final arrays (the groups' totals are known) — the extends
        // used to realloc through several doublings of ~500 MB of vertex/CharVertex traffic.
        {
            let (tv, ti, tc) = groups.values().fold((0usize, 0usize, 0usize), |(a, b, c), g| (a + g.0.len(), b + g.1.len(), c + g.2.len()));
            vertices.reserve(tv); indices.reserve(ti); char_verts.reserve(tc);
        }
        crate::phase_mark("b5 characters + attachments + pools", t); t = std::time::Instant::now();
        let mut char_vertex_start = None;
        for ((layer, ikey, _, chain, key, draw, lflags), (vs, is, cv, centre, mut radius, cel, node)) in groups {
            let skin = (ikey[2] == SKIN_TAG).then(|| (ikey[0], ikey[1].checked_sub(1)));
            if is.is_empty() { continue; }
            // fix-render-artifacts: the cull sphere must contain the group's DRAWN geometry — the glist
            // radius can undershoot it (0700000c's floor decal: glist r 0.6 vs a 0.87 half-diagonal box),
            // which dropped in-view statics at the frustum edge (pale fog-coloured patches).
            // The game's cull consults the bbox next to the sphere (`View_AddObjects` 0x1e62d8 passes
            // obj+0x8c AND obj+0x80 to `Vision_InView` 0x1e89e0); this is that bbox term.
            // level-load-time: this pass is NOT redundant with add_instance's per-batch fold — that
            // measures each instance's verts against the INSTANCE centre, this measures every vert of
            // the MERGED group against the group's (last-instance) centre; non-World groups share one
            // key across instances (ikey = 0), so this pass is the only sound bound. Kept verbatim.
            {
                let mut d2max = radius * radius;
                for v in &vs {
                    let d2 = (v.pos[0] - centre[0]).powi(2) + (v.pos[1] - centre[1]).powi(2) + (v.pos[2] - centre[2]).powi(2);
                    if d2.is_finite() && d2 > d2max { d2max = d2; }
                }
                radius = d2max.sqrt();
            }
            if matches!(layer, Layer::Character | Layer::Effect) && char_vertex_start.is_none() { char_vertex_start = Some(vertices.len()); }
            let base = vertices.len() as u32;
            materials.push(Material { layer, draw, centre, radius, cel, chain, node, key, lflags, skin, first_index: indices.len() as u32, index_count: is.len() as u32 });
            indices.extend(is.iter().map(|i| i + base));
            vertices.extend(vs);
            char_verts.extend(cv);
        }
        let keys: BTreeSet<(usize, u32)> = materials.iter().filter_map(|m| m.key).collect();
        crate::phase_mark("b6 materials assembly", t); t = std::time::Instant::now();
        // Bug 1 diagnostic: how many world materials have no texture (render as the white fallback). A high count on a
        // non-07000005 level would explain "ghostly white": texture × white vertex × GAIN → white.
        let untextured = materials.iter().filter(|m| m.key.is_none()).count();
        if untextured > 0 { eprintln!("render: {untextured} of {} materials have no texture (white fallback)", materials.len()); }
        let textures: Vec<TextureImage> = keys.into_iter().map(|key| {
            TextureImage::from_texture(key.0, &level.maps[key.0].textures[key.1 as usize])
        }).collect();
        let mut hud_textures: Vec<(u32, TextureImage)> = crate::hud::TEXTURES.iter().chain(crate::menu::MENU_TEXTURES.iter()).chain(crate::menu::mp_textures().iter()).filter_map(|&h| level.maps.iter().enumerate().find_map(|(mi, m)| m.textures.iter().find(|t| t.hashcode == h).map(|t| {
            (h, TextureImage::from_texture(mi, t))
        }))).collect();
        // fix-pause-menu2: the pause page's figure (0x030000db) keeps the standard 8bpp decode —
        // the disc palette's entry 0x43 is pure WHITE and the figure's texels are ALL index 0x43
        // [L the retail palette (pal[067]=ffffff80) + the index histogram: a flat run of 987
        // 0x4343 words in the figure's bbox]. The blobs live in the texture's LEFT half and the
        // control's draw never samples them (the menu's uv rect covers the right half only); the
        // flags-0x100 blend (ALPHA_1 0x48 = Cs·As + Cd, additive in the port [M 0x120e68:56-68])
        // drops the black background (it adds 0). The menu draws the right half tinted orange.
        let hud = drones.and_then(|d| d.hud.clone());
        let n_seq: usize = sequences.values().map(Vec::len).sum();
        let stats = format!("{} world instances ({hidden} skipped by the cel pass: flag 0x10), {} animated character meshes, {chars_missing} drones with a skin from another archive, {} sky objects, {missing} unresolved, {} triangles, {} materials, {n_seq} sequences",
            instances.len(), characters.len(), sky.len(), indices.len() / 3, materials.len());
        let char_vertex_start = char_vertex_start.unwrap_or(vertices.len());
        crate::phase_mark("b7 texture decode", t); t = std::time::Instant::now();
        for c in characters.iter_mut() {
            c.floor = floor_below(&vertices, &indices, &materials, [c.matrix[3][0], c.matrix[3][1], c.matrix[3][2]]);
        }
        if std::env::var_os("NF_FLOOR_PROBE").is_some() {
            for (n, c) in characters.iter().enumerate() {
                eprintln!("floor probe: character {n} skin {:08x} at {:?}: placement − floor = {:?}", c.skin.hashcode, &c.matrix[3][..3], c.floor.map(|f| c.matrix[3][1] - f));
            }
        }
        let started = characters.iter().filter(|c| c.start.as_ref().is_some_and(|s| !s.sequences.is_empty())).count();
        crate::phase_mark("b8 floor_below", t); t = std::time::Instant::now();
        let flashes = attachments.iter().filter(|a| a.flash.is_some()).count();
        let held_back = characters.iter().filter(|c| matches!(c.role, drone::SpawnRole::Reinforcement { .. })).count();
        let stats = format!("{stats}; {started}/{} characters with their start animation; {} weapons held ({weapons_missing} without mesh or datum), {flashes} with a muzzle flash; {held_back} character meshes held back by spawners (R shows them); {script_objects} script objects", characters.len(), attachments.iter().filter(|a| a.script.is_none() && a.flash.is_none() && a.tracer.is_none()).count());
        // The guard AI: one drone per main-map drone placement present at level start; characters map to it by
        // placement translation.
        crate::phase_mark("b9 sim create", t); t = std::time::Instant::now();
        // cross-process-determinism: same resolution rule as `by_hash` above — ascending skeleton
        // id, so a duplicate hashcode resolves identically in every process.
        let seqs: HashMap<u32, Sequence> = sk_keys.iter().flat_map(|&sk| sequences[&sk].iter().map(|s| (s.hashcode, s.clone()))).collect();
        let mut sim = drones.and_then(|d| Some((d, d.props.clone()?, d.weapons.clone()?))).and_then(|(d, props, weapons)| {
            let m = level.main_map()?;
            // Breakables (type 0x20) are drawn but not solid yet: their collision rules (glass that sight and bullets
            // pass or break) come with the Break_Create port.
            // fix-safe08b: the cel scope is built (`CollisionWorld::build_scoped`, the straddle walk
            // of `Collide_StraddleCels` 0x1eac98 via `Intersect_Portal` 0x1ea7b8). The TAKE query uses
            // it (`segment_take_scoped`: flags 0x721's cel scoping + the 0xd/0xe surface skip +
            // `Collide_Pick`'s 0x1000 skip). The ACTOR CAPSULE and the floor ray stay FLAT for now:
            // with the scope, Bond falls through the GuardRoom floor at the fusebox — the shared
            // `Room_Stairwell_T_2` mesh maps to cell 59 by the port's smallest-volume rule, which the
            // capsule's straddle walk from cell 40 never keeps; the game's `build_LinkToRoom` 0x1c9680
            // links by `build_FindCel` 0x1c8cb0's RAYCAST tie-break among all containing cels, which
            // the port has not ported (the next session's item, with the live check).
            let instances = level.instances(true).0;
            let mut world = nf_game::world::CollisionWorld::build_scoped(level, &instances, Some(scope_graph(&cel_graph)));
            // The parse-wide cell counter replay of `parse_cel_graph` (sub-maps first, main last),
            // paired with the `instances(true)` resolution so each pushed object (`stat.owners`'
            // source, in push order) maps to its placement's OWN graph cell — replicating
            // `build_scoped`'s push filter (collision present, not the 0x1000 missile flag).
            let inst_cells: Vec<usize> = {
                let main = level.main_map();
                let mut v = Vec::new();
                let mut cell_counter = 0usize;
                for (mi, map) in level.maps.iter().enumerate() {
                    for p in &map.placements {
                        if !p.creates_cel() { continue; }
                        if Some(mi) == main {
                            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < map.objects.len() { Some((mi, p.index as usize)) } else { None };
                            if let Some((om, oi)) = target {
                                let obj = &level.maps[om].objects[oi];
                                if obj.collision.is_some() && obj.flags & 0x1000 == 0 { v.push(cell_counter); }
                            }
                        }
                        cell_counter += 1;
                    }
                }
                v
            };
            // The owner pick below measures each candidate room's OWN collision
            // (`Collide_SphereIntersect` 0x1ec940 scoped to the room [M 0x1c9150]). `build_scoped`
            // keys its buckets by push ordinal, which is not the graph cell index once sub-maps
            // add cels first — the pick then measured other rooms' geometry: 07000002's bathhouse
            // floor (generic placement 270) hung on the big room 13 instead of the bathhouse room
            // [L 2026-10-17 fresh-boot snapshot: generic cel 0x17086f0 rides the +0x18 chain of room
            // 0x17087b0, sphere (−38.15, 7.24, 7.97) r 6.11 = the port's cell 185], so the floor
            // vanished through the doorway. Key the buckets by each object's own cell first.
            world.attribute_scope(&inst_cells.iter().map(|&c| Some(c as u32)).collect::<Vec<_>>());
            // fix-portal4: the +0x18 chains hang by the game's build_FindCel (0x1c8cb0) owner pick
            // over each generic's sphere centre — needs the scoped world for the tie-break rays.
            build_chains(&mut cel_graph, Some(&world));
            // fix-portal-eyecel: the collision side of the game's per-cel attribution. The game's
            // `build_FindCel` (0x1c8cb0) tie-break rays are owner-filtered to the CANDIDATE's own
            // chain geometry (`Collide_RayIntersect` 0x1ec690, flags arg 1: only hits whose
            // `HITDATA+0x54` static-geometry cel equals the passed cel are exact-tested — the
            // disassembly compares hit+0x54 against HITTEST+0x80), and the loader hangs every
            // generic's collision on the room `build_link_objects_to_rooms` (0x1c9150) picks — the
            // same owners `build_chains` just computed. So the scope buckets re-key by the chain
            // owners: a room cell keeps its own runs, a generic's runs join its chain owner's
            // bucket, unchained/unresolvable cells fall to the `always` bucket. The smallest-volume
            // centre rule `build_scoped` stands in with left the trench floor outside BOTH
            // eye-room candidates' buckets, so both tie-break rays missed and the first candidate
            // (the outdoor room) stood [L ee_start_05.bin: the trench room's chain holds all 58
            // trench generics; the outdoor room's 41 hold none near it].
            {
                let mut node_owner: Vec<Option<u32>> = vec![None; cel_graph.cells.len()];
                for (o, ch) in cel_graph.chains.iter().enumerate() {
                    for &n in ch { if (n as usize) < node_owner.len() { node_owner[n as usize] = Some(o as u32); } }
                }
                let instance_cell: Vec<Option<u32>> = inst_cells.iter().map(|&c| {
                    let f16 = cel_graph.cells.get(c).map(|n| n.flags & 0xffff).unwrap_or(0);
                    if matches!(f16, 0xc023 | 0xc047) { Some(c as u32) } else { node_owner.get(c).copied().flatten() }
                }).collect();
                world.attribute_scope(&instance_cell);
            }
            crate::phase_mark("b9a collision world + scope", t); let mut t9 = std::time::Instant::now();
            let nav = nf_game::nav::Network::build(level.maps[m].ai_network.as_ref().unwrap_or(&Default::default()), &world, d.level_id);
            let covers = nf_game::cover::build(level, &nav, &world);
            let ai_points = nf_game::aipoint::build(&level.maps[level.main_map().unwrap()].placements, &nav, &world);
            let hitboxes = d.hitboxes.clone()?;
            let volumes = nf_game::aivolume::build(level, d.level_id);
            let models = nf_game::pickup::models(level);
            // fix-script-objects: every glist's bounds (min, max) by hashcode — the script-created
            // AI volumes' half-extents resolve here (`Drone_AIVolume_Create` 0x1376c0:84-92).
            let glist_bounds: HashMap<u32, ([f32; 3], [f32; 3])> = level.maps.iter()
                .flat_map(|m| m.objects.iter())
                .map(|o| (o.hashcode, (o.min, o.max))).collect();
            // difficulty-effects2: the create-time Data carries the load's difficulty — the game's
            // GameState+0x28 (0x2a3790) already holds the pick when the load's creates run
            // [M C_SBNFDFCTY 0x216ca8:15 writes it at confirm; Drone_Create 0x136be0:49 reads it],
            // so `load_with_carry_difficulty(3)` must create the full roster here (the hard-coded 1
            // starved the 00-Agent load to the Operative roster). The direct loads keep 1 — the
            // documented replay parity (the recorded sessions ran at difficulty 1).
            let data = nf_game::drone::Data { tables: &d.tables, props: &props, scripts: &d.scripts, weapons: &weapons, world: &world, level: d.level_id, sequences: &seqs, skins: &level.skins, skeletons, nav: &nav, covers: &covers, tuning: &d.tuning, hitboxes: &hitboxes, volumes: &volumes, sfx: d.sfx.as_ref(), models: &models, surfaces: d.surfaces.as_ref(), voices: d.voices.as_deref(), difficulty,
            ai_points: &ai_points };
            // net-match-host: an online match builds from the host's seed (equal on every machine — the
            // settings blob carries it); local play keeps the pre-net constant.
            let mut rng = nf_game::drone::Rng::new(if d.mp_seed == 0 { 0x1234_5678 } else { d.mp_seed });
            // The drones' creates (and the interleaved type-0x4E light creates) draw from the shared
            // generator here; the map parse's ambient radiators went in first, inside `level_drones`.
            let mut lights = nf_game::light::LightList::default();
            let list = nf_game::drone::level_drones(&data, level, &mut rng, &mut lights);
            // Every drone placement; spawner templates stay absent until a spawner brings them in.
            // The character link keys on the placement's OWN index (`level_drones` records it, the
            // instances carry it): 0c ships three bit-identical-XZ drone placement pairs [D
            // #263/#461 t [6.5508847,12.309799,-224.56429], #261/#462, #310/#463 — the group-1
            // immediate guard and the group-4 ch-4 swap wave at one spot], and the old
            // translation key gave both of a pair's characters to ONE drone, leaving the other —
            // active, patrolling, shooting — with nothing drawn at all (992-0c-invisible-soldiers;
            // the game's objects carry their own placement, `parsemap_create_dynamic_objects`
            // 0x1d04f0, and never re-derive it by position).
            let by_placement: HashMap<usize, usize> =
                list.iter().enumerate().map(|(i, dr)| (dr.placement, i)).collect();
            let char_drone = char_placements.iter()
                .map(|cp| cp.and_then(|pi| by_placement.get(&pi).copied())).collect();
            // Per-character ambient state (`Lights_CalcAmbientLight` 0x1cb178): the 0xff,ff,ff sentinel of
            // `control_init_object` (0x134630) until the first cel link snaps it.
            let char_ambient = vec![[0xffu8; 3]; characters.len()];
            let char_script = characters.iter().map(|c| c.script).collect();
            let char_script_skin = characters.iter().map(|c| Some(c.skin.hashcode)).collect();
            let seq_index = by_hash.clone();
            // Bond at the "Player1" placement, on the floor below (`Player_StandAtNewPosition`: floor + 1.1).
            let (p0, yaw0) = start.unwrap_or(([0.0; 3], 0.0));
            // The game's `Player_Init` 0x1910f8:117 next `JAL`s `AnimDebugCreate` 0x1bcfa8 after the player's
            // `AnimObjectNew` — a body that only acts on the wrapper codes 0x07000043/0x07000049 (the
            // all-models mannequin circle) or 0x07000041 (the body-glow figure), which nothing on the retail
            // disc ever loads [D: every 0xE8 param-0 scanned — no loader of 0x07000041/43/46/49, pinned in
            // `doublecross_transitions_live.rs:44`; M 0x1bcfa8:72,140]. On every shipped on-foot level, front
            // end and MP, the call returns at once — the port's spawn does the same job by doing nothing
            // (docs/systems/anim-debug-and-queries.md, docs/port-map/unreached.md §863).
            let bond = nf_game::player::Bond::stand_at(&world, p0, yaw0);
            // The spawn arrival settle is NOT armed on this structural fallback: the level start
            // re-places the bond in lib.rs (the `Player_Init` 0x1910f8 path) and arms it there
            // (`arm_spawn_settle` doc — the [L] evidence is the boot's level-start placement).
            let mut drones = nf_game::drone::Drones::new(list, rng);
            drones.lights = lights;
            // The selected difficulty BEFORE the creates: every create-time gate reads it
            // (captain_setup 0x14b300:504-535, Drone_Create's min-difficulty tag 0x136be0:49,
            // Lock_Create's Operative auto-unlock 0x1ae1f8:67, Copter_Create's HP 0x17d340:76,
            // Hint_Create's stage gate 0x1b3248) — the game loads with GameState+0x28 already
            // holding the menu's pick [M C_SBNFDFCTY 0x216ca8:15].
            drones.set_difficulty(difficulty);
            drones.attach_level_id(level, d.level_id);
            // mp-lobby-settings: the MP creates' enviro-mod gates. `GunImp_Create` 0x131f20 returns
            // 0 in MP while Gun Emplacements (+0x1B8) is off and otherwise mounts
            // `TurretGuns[weapon set]` (Random: `Rand_Rand(10)`) [M :52-63]; `Destroy_Create`
            // 0x12c318 creates nothing in MP while Explosive Scenery (+0x1D4) is off [M :14]. The
            // Random turret draws from the lockstep seed's own stream, not the shared one [S, the
            // weapon_row note].
            if let Some(ms) = mp {
                if ms.gun_emplacements == 0 {
                    drones.props.gunimps.clear();
                } else {
                    let mut trng = nf_game::drone::Rng::new(ms.seed ^ 0x4755_4e49);
                    for g in drones.props.gunimps.iter_mut() {
                        g.turret = if ms.weapon_set >= 10 { trng.rand(10) as u8 } else { ms.weapon_set as u8 };
                    }
                }
                if ms.explosive_scenery == 0 { drones.props.destroy.clear(); }
            }
            crate::phase_mark("b9b nav/cover/volumes/drone creates", t9);
            // fix-door-portal-links (G-035): `build_LinkDoors2Portals` (0x1c8ac8) runs at load — each
            // portal-linked door (placement param 4) takes the portal quad it sits in; the vision
            // walk then skips it while `Door_IsClosed` reads closed. The Scene's graph and the Sim's
            // clone below both carry the links.
            let linked_doors = link_doors_to_portals(&mut cel_graph, &drones.doors);
            let _ = linked_doors;
            // Ladders (`Ladder_Create` 0x1966c0, placement type 0x22, class 0x2D): fix-ladder. The world
            // sphere comes from the placement's object glist bounds, like the game's
            // `Control_BuildWorldSph` 0x133070.
            let ladders = nf_game::ladder::all(level);
            // The ladders' runtime cel link [M build_LinkToRoom 0x1c9680:
            // build_FindCel(world-sphere centre) → control_link_object_to_cel] — the pick admits a
            // ladder only from a straddled cel's chain [M Collide_Pick 0x1eb320:125-134], so the
            // per-tick straddle set tests against THIS cel. Unresolved (None) = the game's
            // unlinked object: never admitted while scoping is active.
            let mut ladders = ladders;
            for lad in ladders.iter_mut() {
                lad.cell = world.find_cel(lad.centre).map(|c| c as u32);
            }
            // Wires (`Wire_Create` 0x1981d8, placement type 0x3E): the level's zip wires, resolved
            // against their map objects like the ladders (fix-wire: the world sphere replaces the
            // hardcoded 4.0 length; the endpoints derive at grab time per the atlas).
            let wires = nf_game::wire::all(level);
            // The creepwalls (`CreepWall_Create` 0x196fc8, placement type 0x3F): the crawlable vents.
            let creepwalls = nf_game::creepwall::all(level);
            // The hurt volumes (`Hurt_Create` 0x17e870, placement type 0xE4): the contact damage
            // volumes (level-02-interactables).
            let hurts = nf_game::hurt::Hurt::create(level);
            // The wires' owning cels (see `wire_cels`): the game links every dynamic object to its
            // runtime cel at load — `build_LinkToRoom` 0x1c9680:40-49 → `build_FindCel` 0x1c8cb0
            // over the world-sphere centre, `control_link_object_to_cel` 0x1346b8 — and
            // `Collide_Pick` 0x1eb320:125,312-330 admits a pick only from a straddled cel's own
            // list. The placement's glist (0x1d1b68) is the wire's geometry record, NOT its cel.
            // `world.find_cel` is the faithful `build_FindCel` port (the ladders use the same).
            let wire_cels = wires.iter().map(|w| world.find_cel(w.centre)).collect::<Vec<_>>();
            // The third icons (placement kind 0xFA, `ThirdIcon_Create` 0x197070): the interaction
            // contexts written to `BLData+0x95F` each tick (fix-wire2 — the cable's type-3 icons gate
            // the jump-grab). fix-08-walls: each icon carries its object-resolved world sphere
            // (`Icon::centre`/`radius`, `Control_BuildWorldSph` 0x133070 via `build_LinkToRoom`
            // 0x1c9680:40) and its straddle-cel set — the arm region the game's obj+0xd0 chain
            // implements [M Collide_StraddleCels 0x1eac98 + Intersect_Portal 0x1ea7b8 v0x101 +
            // Collide_Update 0x1ea398:160-233], resolved once here over the final cel graph (the
            // icon's own cel via the faithful `build_FindCel` 0x1c8cb0 port, `find_cell_raycast`).
            let icons = nf_game::wire::icons(level);
            let icon_cels = icons.iter().map(|ic| match find_cell_raycast(&cel_graph, ic.centre, Some(&world)) {
                Some(c) => icon_straddle_cels(&cel_graph, c, ic.centre, ic.radius),
                None => Vec::new(),
            }).collect::<Vec<_>>();
            // The hint markers (placement kind 0x33, `Hint_Create` 0x1b3248 — fix-context-hints):
            // the same world-sphere + straddle-cel resolution as the third icons (the marker's
            // obj+0xD0 chain is filled by the player's pick through it [M Collide_Update
            // 0x1ea398:160-233 + the reciprocal node append]). `Hint_Update` fires once when the
            // player's cel is in the set with the marker's channels satisfied.
            let hints = nf_game::player::hints(level, drones.difficulty);
            let hint_cels = hints.iter().map(|h| match find_cell_raycast(&cel_graph, h.centre, Some(&world)) {
                Some(c) => icon_straddle_cels(&cel_graph, c, h.centre, h.radius),
                None => Vec::new(),
            }).collect::<Vec<_>>();
            let hint_fired = vec![false; hints.len()];
            let icon_msg_timers = vec![0u8; icons.len()];
            // The wire anims' root tracks: the level archive's kind-5 scripts (the player body anims)
            // resolved to their sequences [D 07000005: 0x060005e5 101 frames → 0x04000536, the move pair
            // 0x060000b4/b5 → 0x04000034/35 travelling 0.7637/0.7106 per cycle].
            let mut wire_anims = nf_game::wire::WireAnims::default();
            for h in [0x0600_05e5u32, 0x0600_05ec, 0x0600_05ed, 0x0600_05ee, 0x0600_05ea, 0x0600_00b4, 0x0600_00b5, 0x0600_0095, 0x0600_0096, 0x0600_0094] {
                if let Some(sc) = d.scripts.get(&h) {
                    for (_a, _b, q) in sc.sequences() {
                        if let Some(s) = seqs.get(&q) { wire_anims.tracks.insert(h, nf_game::wire::RootTrack::of(s)); break; }
                    }
                }
            }
            // Level-placed pickups (`Pickup_Create` 0x18dda0 via `parsemap_create_dynamic_objects` type 0xF0):
            // the placement's PARAMS array gives the type, id, rounds, switch, sound, respawn and label [M 0x1d04f0
            // line 459-462: reads from the param buffer at +0x2c..+0x38, NOT the placement's extra words].
            // The model comes from the placement's object (the celglist), not from weapon_data.
            let pickup_models = nf_game::pickup::models(level);
            // mp-lobby-settings: an MP match's set pickups (kinds 7..11) come from the lobby's
            // Weapon Set row [M Pickup_CreateFromSet 0x18f1b0:56 — row = MPSettings+0x1B4].
            let mp_row = mp.map(nf_game::mp::equip::weapon_row);
            // mp-pickups-visible: the ONE builder both sides call (docs/systems/mp-pickups-visible.md §4) —
            // kinds 7..11 take the match's row and the weapon's own model, `obj` stays None, so the sim's
            // pickups[k] pair the draw slots and model the match's weapons.
            let placed_pickups: Vec<nf_game::pickup::Pickup> = level.maps[m].placements.iter()
                .filter(|p| p.flags & 0xff == 0xf0)
                .filter_map(|p| Self::placed_pickup_from_placement(level, p, m, mp_row.as_ref(), &weapons, d.level_id, &pickup_models, d.has_gold_medal(d.level_id)))
                .collect();
            let (reserved, sim_slots) = script_pickup_reservations(level, &weapons, &pickup_models, d.level_id, d.has_gold_medal(d.level_id));
            let placed_len = placed_pickups.len();
            drones.pickups.extend(placed_pickups);
            drones.pickups.extend(reserved);
            // dc-items: the slot indices are RESERVATION-relative (`out.len()` inside the function) but
            // `drones.pickups` holds the placed pickups FIRST — the drain (`Script_CreateEntity`'s 0xF0 arm,
            // drone.rs) would look up `pickups[0]` (the first PLACED pickup) and, once that one is taken
            // (gone), convert it into the script's pickup instead of activating the reservation — and the
            // reservation itself stayed a gone zombie (07000003's keycard spawned as a duplicate push while
            // slot (0x06000563,1) pointed at the armour). Offset by the placed count so the slot points at
            // the reservation actually appended. [Port bug — the game creates a fresh object every spawn
            // (Pickup_Create 0x18dda0 via Script_CreateEntity 0x1d8968's 0xF0 arm); the port's
            // slot reuse is its documented safe08 design and points at the right entry — the
            // offset bug it fixes is the port's own, the game never had it.]
            drones.script_pickup_slots = sim_slots.into_iter().map(|(k, v)| (k, v + placed_len)).collect();
            // The difficulty was set before the creates above (the game's GameState+0x28 pick);
            // the old direct-load hardcode `= 1` moved into the build parameter so replay loads
            // pass 1 and the front-end path passes the menu's pick.
            Some(Sim { drones, tables: d.tables.clone(), props, scripts: d.scripts.clone(), weapons, world, level: d.level_id, char_drone, char_script, char_script_skin, seq_index, seqs, skins: level.skins.clone(), skeletons: skeletons.clone(), nav, covers, ai_points, tuning: d.tuning.clone(), hitboxes, sfx: d.sfx.clone(), volumes, glist_bounds, models, gun: nf_game::gun::Gun::new(2, 7, 41), bond, cursor: [0.0; 2], cross: [0.0; 2], view_script: 0, first_person: false, last_shot: None, shot_tick: false, view_time: 0.0, view_speed: 1.0, bob: [0.0; 2], aiming: false, free_aim: false, auto_aim: false, rapid_fire: false, zoom: 1.0, zoom_target: 1.0, disabled: false, surfaces: d.surfaces.clone(), voices: d.voices.clone(), step_dist: 0.0, wires, creepwalls, creeping: false, wire_anims, wire_channel: 0, ladders, icons, view_model_ids: std::collections::HashSet::new(), cross_state: 2, cels: sim_cels, char_ambient, view_model_chars: Vec::new(), mp_body_chars: Vec::new(), mp_last_held: [2; nf_net::MAX_AGENTS], mp_weapon_loaded: Vec::new(), ambient_lookups: std::cell::Cell::new(0), mp_draw_cache: Default::default(), mp_anim: (0..nf_net::MAX_AGENTS).map(|_| nf_game::mp::body_anim::MpBodyAnim::new()).collect(), mp_anim_sets: d.mp_anims.as_ref().map(|a| nf_formats::mpanim::MpAnimSets { sets: a.sets }), sun_seen: false, lang: None, hud: Default::default(), smoke: Vec::new(), room_cell: cel_graph.room_to_cell.clone(), prev_trigger: false,
            mp_bonds: [const { None }; nf_net::MAX_AGENTS],
            mp_local_slot: 0,
            mp_body_speed: [0.0; nf_net::MAX_AGENTS],
            net_rng: None,
            net_rng_shared: None,
            mp_hp: [100.0; nf_net::MAX_AGENTS],
            mp_bot_pickups: Vec::new(),
            mp_bot_weapons: None,
            mp_armour: [0.0; nf_net::MAX_AGENTS],
            mp_dead_ticks: [0; nf_net::MAX_AGENTS],
            net_row: None,
            net_remotes_done: false,
            net_last_takes: Vec::new(),
            use_key_prev: false,
            mp_combat: Default::default(),
            net_shot: None,
            mp_held: [2; nf_net::MAX_AGENTS],
            mp_fx: Default::default(),
                wire_cels, cel_graph: cel_graph.clone(), icon_cels, laser_beam: None, stunner_target: None, decryptor_target: None, decryptor_scanned: None, laser_charge: 100.0, missile_cam: None,
                // The decryptor contact pipe's second register (fields on their own lines, lesson 4).
                decryptor_scanned_lag: None,
                pda_display: [0; 4],
                pda_hold: 0,
                pda_lock: None,
                hints, hint_cels, hint_fired, icon_msg_timers,
                view_alert: false,
                prev_aiming: false,
                view_aim_anim: None,
                view_aim_raising: false,
                mp: None,
                // screen-adjust-model: viewer+0x114's 4:3 default; the host feeds 16/9 when the AV
                // options' widescreen is on [M Camera_CalcViewAngles 0x1baf70:28-32].
                view_aspect: 4.0 / 3.0,
                zoom_wheel: false,
                laser_sight: None,
                fmv_script: None,
                hurts,
                // unported-front-end-menus-3: the viewer's shear/swing phases and the underwater
                // blur byte.
                shear_phase: 0.0,
                global_blur: 0,
                swing_phase: 0.0,
            })
        });
        let fog = drones.and_then(|d| fog_for(d.level_id));
        crate::phase_mark("b9 sim create (total)", t); t = std::time::Instant::now();
        let snow_texture = level.maps.iter().enumerate().find_map(|(mi, m)| m.textures.iter().find(|t| t.hashcode == 0x0300_0045).map(|t| {
            TextureImage::from_texture(mi, t)
        }));
        // Bug 3: build the collider name index for the "name what I'm touching" debug overlay (`C` key).
        let colliders: Vec<(String, [f32; 3], [f32; 3])> = instances.iter()
            .filter(|i| level.maps[i.map].objects[i.object].collision.is_some())
            .map(|i| {
                let obj = &level.maps[i.map].objects[i.object];
                let lo = transform(&i.matrix, obj.min);
                let hi = transform(&i.matrix, obj.max);
                (obj.name.trim().to_string(), [lo[0].min(hi[0]), lo[1].min(hi[1]), lo[2].min(hi[2])], [lo[0].max(hi[0]), lo[1].max(hi[1]), lo[2].max(hi[2])])
            }).collect();
        // Populate the Sim's view_model_ids from the view_models map (D-003).
        if let Some(sim) = sim.as_mut() {
            sim.view_model_ids = view_models.keys().copied().collect();
            // cross-process-determinism: collected in the build loop's ascending-wid order, not
            // `view_models.values()`'s per-process random order — today the only consumer is
            // `contains` (order-insensitive), but a hash-ordered Vec sitting in the Sim is one
            // index-pairing away from a real divergence.
            sim.view_model_chars = (1..=0x72u8).filter_map(|wid| view_models.get(&wid).copied()).collect();
        }
        if let Some(sim) = sim.as_mut() { sim.mp_body_chars = mp_bodies.clone(); }
        let models_0x800: HashMap<u32, bool> = level.maps.iter()
            .flat_map(|m| m.objects.iter().map(|o| (o.hashcode, o.flags & 0x800 != 0))).collect();
        // fix-portal4: sim-less loads never built a world — the chains fall back to the
        // bbox-face FindCel approximation (documented deviation; every live load path builds sim).
        if sim.is_none() { build_chains(&mut cel_graph, None); }
        crate::phase_mark("b10 collider index + final assembly", t);
        if mp.is_some() {
            // mp-lobby-settings: the enviro-mod creates as built (the settings check reads it).
            let (g, t, dz) = sim.as_ref().map_or((0, Vec::new(), 0), |s| (s.drones.props.gunimps.len(), s.drones.props.gunimps.iter().map(|g| g.turret).collect::<Vec<_>>(), s.drones.props.destroy.len()));
            eprintln!("[mp-props] gunimps {g} turrets {t:?} destroy {dz} drawn: guns {} destroy {} vehicles {}",
                misc_drawn.get(&0x34).copied().unwrap_or(0), misc_drawn.get(&0xe1).copied().unwrap_or(0), misc_drawn.get(&0xda).copied().unwrap_or(0));
        }
        // break-shards draw: each glass pane's face words → the pane map's loaded texture.
        let mut shard_tex: HashMap<u32, usize> = HashMap::new();
        if let (Some(sm), Some(main)) = (sim.as_ref(), level.main_map()) {
            for b in sm.drones.breaks.list.iter().filter(|b| b.kind == 2) {
                let Some(p) = level.maps[main].placements.get(b.placement) else { continue };
                let map = if p.by_hash() { level.by_hash.get(&p.hashcode).map_or(main, |&(m, _)| m) } else { main };
                for g in &b.src.glass {
                    if let Some(i) = textures.iter().position(|t: &TextureImage| t.key == (map, g.material)) { shard_tex.entry(g.material).or_insert(i); }
                }
            }
        }
        Scene { shard_tex, cels: room_cels, cel_graph, streams: None, music_files: None, music_tables: None, sound_bank: None, sound_spares: Vec::new(), snow_texture, vertices, indices, materials, textures, hud_textures, woman: None, hud, script_billboards, view_models, mp_bodies, bond_body, mp_body_weapons, mp_char_bodies, mp_anim_sets, fog, bounds, start, stats, characters, char_vertex_start, char_verts, sequences, attachments, sim, colliders, models_0x800, npc_pool, ladders_drawn, grapple_hooks_drawn, misc_drawn, lock_matrices, dynobj_matrices, rotor_placements, gunimp_placements,
            door_scene_hidden,
            sl_lod,
            destroy_centres }
    }

    /// Sequence index and frame a character shows after `ticks` 60 Hz updates: its start script (`choice = None`) or the
    /// `choice`-th sequence of its skeleton (modulo the list), looped over frames 1 … frames − 1.
    fn frame_of(&self, c: &Character, choice: Option<usize>, ticks: f32) -> Option<(usize, u32)> {
        let seqs = self.sequences.get(&c.skin.skeleton).filter(|v| !v.is_empty())?;
        match choice {
            None => {
                let start = c.start.as_ref()?;
                // A finished non-looping start script is restarted, as the drone AI calls the same idle again
                // (`DroneAnim_SetStandIdleAnim`); AnimScriptEnd's wrap-around otherwise.
                let looped = Script { looping: true, ..start.script.clone() };
                let t = looped.time_at(ticks);
                let &(a, b, i) = start.sequences.iter().find(|(a, b, _)| (*a as f32) <= t && t <= *b as f32).or(start.sequences.first())?;
                let frame = ((t - a as f32) as u32 + 1).min((b - a) as u32 + 1).clamp(1, seqs[i].frames.max(1) as u32);
                Some((i, frame))
            }
            Some(k) => {
                let i = k % seqs.len();
                let span = (seqs[i].frames as f32 - 1.0).max(1.0);
                Some((i, 1 + (ticks % span) as u32))
            }
        }
    }

    /// Skins every character for `choice` at `ticks` and returns the new Character-layer vertex positions (world).
    ///
    /// Root handling as the game: `AnimSeqTick` (0x1c28d0) zeroes the root bone's translation after taking its
    /// per-frame change as root motion, so the root sits at the object position; the drone's height comes from
    /// `Drone+0xA0` (hip height − 0.02), whose collision sphere rests its lowest point `0xA0` below the object on the
    /// floor (`NDrone2_Collision` 0x140e90). nf-view has no collision or AI yet, so drones stay at their placement x/z
    /// (horizontal root motion is dropped) and stand on the World floor below them; without one found they keep the
    /// pre-collision height (placement + 0xA0 − 1.0).
    /// The per-frame matrices shared by the CPU pose (`animate`) and the GPU palette (`skin_palette`): each
    /// character's skin matrices, bone world matrices and object matrix (None = unposed), every attachment's
    /// matrix, and which characters are hidden (collapsed to their object origin).
    pub fn frame_state(&self, choice: Option<usize>, ticks: f32, opts: ViewOpts) -> FrameState {
        self.frame_state_for(choice, ticks, opts, false)
    }

    /// `frame_state` for the GPU palette (frame-240): with `skip_hidden`, a hidden character is not
    /// posed at all (`posed[ci] = None`). The palette collapses every hidden character's bones to its
    /// placement point without reading the pose, and its attachments collapse to a zero-size point
    /// (`collapse`) — drawn nothing either way — so the pose was pure cost: on the pool-heavy levels
    /// most of the ~150 characters are parked reinforcements.
    pub fn frame_state_for(&self, choice: Option<usize>, ticks: f32, opts: ViewOpts, skip_hidden: bool) -> FrameState {
        let hidden = |c: &Character, ci: usize| {
            if choice.is_some() {
                match c.script {
                    Some(src) => self.sim.as_ref().and_then(|s| s.script_anim(src, c.skin.hashcode)).is_none(),
                    None => false,
                }
            } else {
                self.char_hidden(ci, opts)
            }
        };
        // With the AI running (and no sequence browsing), each guard shows its drone's animation, yaw and position.
        let drone_of = |ci: usize| choice.is_none().then(|| self.sim.as_ref().and_then(|s| s.char_drone[ci].map(|di| (s, di)))).flatten();
        let pose_one = |ci: usize, c: &Character, cache: &mut HashMap<(u32, usize, u32), (Vec<Mat4>, Vec<Mat4>, f32)>| -> Option<(Vec<Mat4>, Vec<Mat4>, Mat4)> {
            if skip_hidden && hidden(c, ci) { return None; }
            // Drones: the simulation's pose — blended scripts, fractional frames, upper-body aim — the same one the hit
            // boxes use (`Drone::object_pose`).
            if let Some((s, di)) = drone_of(ci) {
                let _pd = nf_game::prof::scope("fs: drone pose");
                let dr = &s.drones.list[di];
                let (world, hip) = dr.object_pose(&s.data())?;
                let m = nf_formats::placement::object_matrix([0.0, dr.yaw, 0.0], [dr.pos[0], dr.draw_y(hip), dr.pos[2]]);
                return Some((anim::skin_matrices(&world, &c.skin), world, m));
            }
            // cable-3p: Bond's body on the wire - the playing wire anim (`Player_Wire`/`Player_Zipline`'s
            // AnimScriptAdd scripts, the sim's `wire_anim` at `wire_anim_time`) on the player object's
            // matrix (obj+0x30 at the hang point, the yaw the grab set [M Player_CollWire 0x197130]).
            if self.bond_body == Some(ci) {
                let s = self.sim.as_ref()?;
                s.wire_camera()?;
                let local = nf_game::anim::script_pose_drone(&s.scripts, &s.seqs, &c.skeleton, &c.skin, s.bond.wire_anim, s.bond.wire_anim_time.max(0.0))?;
                // The player object (obj+0x30) IS the hip: the anim's root height comes off so the root
                // bone sits at the object [the grab's 1.0207 hang offset then puts the hands on the cable -
                // L PCSX2 07000005 cable, audits/render-1029-cable p-stand].
                let hip = local.first().map_or(0.0, |r| r.0[1]);
                let world = anim::world_pose(&local, &c.skin);
                let p = s.bond.pos;
                let m = nf_formats::placement::object_matrix([0.0, s.bond.yaw, 0.0], [p[0], p[1] - hip, p[2]]);
                return Some((anim::skin_matrices(&world, &c.skin), world, m));
            }
            // Script animations: the object's frame, the root kept (the object sits at the origin).
            if let Some(src) = c.script {
                let s = self.sim.as_ref()?;
                let (script, frame) = s.script_anim(src, c.skin.hashcode)?;
                // Every started opcode-0 window plays together (`AnimProcessScriptCmds` 0x1c1b38): the
                // 22-channel face track keeps the bones its channels reach, the 222-channel body track
                // fills the rest — the cut-scene lip-sync (fix-cutscenes2; the old single-window read
                // played the body only, so the mouths never moved).
                let local = nf_game::anim::script_pose_together(&s.scripts, &s.seqs, &c.skeleton, &c.skin, script, frame)?;
                let world = anim::world_pose(&local, &c.skin);
                // fix-cutscene-brightness2: the object matrix stays the character's own (`c.matrix`,
                // identity for script actors — the object sits at the origin and the pose's root
                // carries it in world space [M Obj::Anim doc; 0x2d6200 is the origin vector
                // [L ACTION.ELF], so `Script_AnimStart` 0x1d8f40 creates the object at the origin]).
                // The held fix substituted `Player::live_matrix` — stream 0's object ∘ the script
                // matrix, for the 07000002 intro the CAMERA (obj[0] trans == player[1] live trans
                // [-27.25, 1.41, 34.39]) — which drew the actors inside the camera: a
                // render showed the intro room with NO characters at all.
                // NOT the SP base: most scenes' casts carry WORLD roots (07000014's control-room scene:
                // roots near (26..34, 1.2, 48) beside the room, SP base (52.16, 2.64, 46.62) — composing
                // it threw the cast ~50 units away, an empty cut-scene). 0700004a's ride pack is the
                // exception (car-local roots; the cast stands in the car in PCSX2) — its mechanism is open.
                return Some((anim::skin_matrices(&world, &c.skin), world, c.matrix));
            }
            // MP bodies (mp-body-anim): the PLAYER anim set the game's MP body runs, from the slot's
            // script list (`Sim::mp_anim`, ticked by `advance_mp_anim` — idle, the distance-driven
            // walk/run, the strafe overlay, the stance transitions, the death [M AnimSetUpdate
            // 0x1c0938, Player_ChangeSubState 0x190630]) composed as `AnimFrameResolve` blends, then
            // the upper-body aim on bone 2 (`AnimObjectAimAt` 0x1bfe58). The speed-driven
            // drone-set fallback below is the port's own defensive path: the game REQUIRES the
            // anim set — Player_Init adds script 0x60001d4 unconditionally [M 0x1910f8:155-158],
            // every substate re-inits the set with no success check [M 0x190630:16-17,
            // PlayerAnimSetInitNormal 0x18fcd8:50-51], AnimSetUpdate just bails on a null set
            // (the body freezes unanimated, no substitute) [M 0x1c0938:83-85], and the MP load
            // filter loads the 0x6/0x4 anim blocks unconditionally [M 0x1bdf40:55-86; FILES.BIN
            // carries 060001d4 in every MP arena, 8x] — no fallback exists in the shipped game
            // [D census].
            if let Some(&(slot, _)) = self.mp_bodies.iter().find(|(_, c)| *c == ci) {
                let _pm = nf_game::prof::scope("fs: mp body (total)");
                let s = self.sim.as_ref()?;
                // mp-remote-fx: a dead slot's body plays the death script (substate 0xd) from its
                // death tick, held on the last frame until the respawn [M Player_CheckForDeath
                // 0x190810:89-93, Player_ChangeSubState 0x190630 case 0xd — `mp_body_pose`'s arm 1].
                if let Some((pos, yaw, ticks)) = s.mp_body_death(slot) {
                    let st = nf_formats::mpanim::MpBodyState {
                        mp: true, weapon: 0, substate: 0xd, fwd: 0.0, strafe: 0.0,
                        phase: MP_WALK_INIT, hold: 0, one_shot_time: 0.0, gravity_y: -9.8,
                    };
                    let script = self.mp_anim_sets.as_ref().map(|sets| nf_formats::mpanim::mp_body_pose(sets, 0, &st).script)
                        .unwrap_or(nf_formats::mpanim::DEATH_SCRIPT);
                    let sc = s.scripts.get(&script)?;
                    let time = (ticks as f32).min(sc.length.max(1) as f32);
                    let mut local = nf_game::anim::script_pose_drone(&s.scripts, &s.seqs, &c.skeleton, &c.skin, script, time)?;
                    let hip = local.first().map_or(0.0, |r| r.0[1]);
                    if let Some(r) = local.first_mut() { r.0 = [0.0; 3]; }
                    let world = anim::world_pose(&local, &c.skin);
                    let m = nf_formats::placement::object_matrix([0.0, yaw, 0.0], [pos[0], pos[1] - 1.0 + drone_height(hip), pos[2]]);
                    return Some((anim::skin_matrices(&world, &c.skin), world, m));
                }
                let (pos, yaw, speed) = s.mp_body_state(slot)?;
                let local = match s.mp_body_local_pose(slot, &c.skeleton, &c.skin) {
                    Some(p) => p,
                    None => {
                        let st = if speed < 0.5 { nf_game::drone::anim_state::STAND_IDLE } else if speed < 4.5 { nf_game::drone::anim_state::WALK } else { nf_game::drone::anim_state::RUN };
                        let danim = s.tables.danim_for_state(st, 0);
                        let script = danim.filter(|&d| d >= 0).and_then(|d| (0..27u16).filter_map(|set| s.tables.script_for(d as u16, set)).find(|h| s.scripts.contains_key(h)))?;
                        let time = 1.0 + s.scripts.get(&script)?.time_at(s.drones.frame as f32);
                        nf_game::anim::script_pose_drone(&s.scripts, &s.seqs, &c.skeleton, &c.skin, script, time)?
                    }
                };
                if std::env::var_os("NF_MPBODY_DBG").is_some() && s.drones.frame % 60 == 0 {
                    let a = s.mp_anim.get(slot);
                    eprintln!("[mpbody] slot {slot} pos {pos:?} yaw {yaw:.2} speed {speed:.2} layers {:?} phase {:?} aim {:?}",
                        a.map(|a| a.layers.iter().map(|l| (l.kind, l.time, l.weight())).collect::<Vec<_>>()), a.map(|a| a.phase), a.map(|a| a.aim));
                }
                let mut local = local;
                let hip = local.first().map_or(0.0, |r| r.0[1]);
                if let Some(r) = local.first_mut() { r.0 = [0.0; 3]; }
                let mut world = anim::world_pose(&local, &c.skin);
                if let Some(a) = s.mp_anim.get(slot).filter(|a| !a.dead) {
                    let held = s.mp_body_held(slot);
                    // `AnimObjectAimAt` returns for an MP weapon of class 0 (fists, grenades, gadgets)
                    // and without a weapon datum [M 0x1bfe58: AnimDatumGetWeaponInfo < 0].
                    if nf_formats::mpanim::weapon_anim_class(&s.weapons, held) != 0 {
                        // The gun line: `AnimGetBoneWorldTrans2(obj, weapon datum, 3 or 1)` — the body
                        // skin's datum id + 3 when it exists (`AnimDatumGetIndex`), else id + 1 (the
                        // muzzle, as `DroneWeap_FireWeapon`'s lookup) — its −X row [M 0x1bfe58].
                        let gun = s.weapons.third_person(held).and_then(|(_, id)| {
                            if id == 0xff { return None; }
                            let id = id as u32;
                            let find = |k: u32| c.skin.datums.iter().find(|d| d.id == k && d.bone >= 0);
                            find(id + 3).or_else(|| find(id + 1)).map(|d| (d.bone as usize, anim::quat_trans_to_mat(d.q, d.t)))
                        });
                        // Humans are class-3 player objects (pitch only, [−π/2, 1.2708]); the bots are
                        // the game's class-0x11 drones (twist and pitch, ±0.9) [M 0x1bfe58 uStack_16c].
                        let is_bot = s.mp.as_ref().is_some_and(|m| slot >= m.settings.human_count as usize);
                        mp_body_aim(&mut world, &c.skin, gun, a.aim, is_bot);
                    }
                }
                // The feet: the body root (`obj+0x30`) stands `cc` above them (the crouched capsule's
                // 0.55, standing 1.0347); the skin's hip rides on the feet as before.
                let feet = pos[1] - s.mp_body_cc(slot);
                let m = nf_formats::placement::object_matrix([0.0, yaw, 0.0], [pos[0], feet + nf_game::player::ROOT_HEIGHT - 1.0 + drone_height(hip), pos[2]]);
                return Some((anim::skin_matrices(&world, &c.skin), world, m));
            }
            let view = self.sim.as_ref().and_then(|s| self.view_models.get(&s.gun.weapon).filter(|&&v| v == ci)).and(self.sim.as_ref());
            let (i, frame) = match (view, drone_of(ci)) {
                (Some(s), _) => s.view_frame(c.skin.skeleton)?,
                (None, Some((s, di))) => s.frame_of(di, c.skin.skeleton)?,
                (None, None) => self.frame_of(c, choice, ticks)?,
            };
            let seq = &self.sequences[&c.skin.skeleton][i];
            let (mats, world, hip) = cache.entry((c.skin.hashcode, i, frame)).or_insert_with(|| {
                let mut local = anim::local_pose(seq, &c.skeleton, &c.skin, frame);
                let hip = local[0].0[1];
                local[0].0 = [0.0; 3];
                let world = anim::world_pose(&local, &c.skin);
                (anim::skin_matrices(&world, &c.skin), world, hip)
            }).clone();
            if let Some(s) = view {
                let g = s.gun_matrix();
                let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                // `Player_PositionCamera` (0x18f5d8:100-106): hide the view model when the aim flag is set
                // AND the weapon is scoped (`+0x68 & 0x40`). `zoom_target > 1.0` ⇔ scoped && zoom held.
                let scoped_zoom = s.zoom_target > 1.0;
                return Some((mats, world, if s.first_person && !scoped_zoom { g } else { far }));
            }
            let mut m = c.matrix;
            let a0 = drone_height(hip);
            m[3][1] = match c.floor { Some(f) => f + a0, None => c.matrix[3][1] + a0 - 1.0 };
            Some((mats, world, m))
        };
        // One memo across the characters: the crowds share (mesh, sequence, frame) poses.
        let mut cache = HashMap::new();
        let prof_pose = nf_game::prof::scope("fs: pose characters");
        let posed: Vec<Option<(Vec<Mat4>, Vec<Mat4>, Mat4)>> = self.characters.iter().enumerate().map(|(ci, c)| pose_one(ci, c, &mut cache)).collect();
        drop(prof_pose);
        // Held back: spawner reinforcements and drones disabled while they wait for a switch channel (object flags
        // 0x30, `NDrone2_Enable(0)`); R shows them. The inspect path (`choice`) pins a drone-backed
        // character visible; script actors keep their liveness gate either way.
        // A matrix that collapses a mesh to one point: its triangles become degenerate and draw nothing.
        let collapse = |at: &Mat4| -> Mat4 { [[0.0; 4], [0.0; 4], [0.0; 4], [at[3][0], at[3][1], at[3][2], 1.0]] };
        // Attached meshes: datum local · bone world · object (bind world when the character has no animation).
        let att_one = |k: usize, a: &Attachment| -> Mat4 {
            // The gas puffs: a camera billboard with the puff's scale and spin (the game's draw path is
            // `View_DrawObjectGlist`'s obj+0xF0 & 8 billboard: viewer rotation · the object's angles, scaled
            // by obj+0xE8 [M 0x1e6780:43-65]); free slots collapse like the tracers.
            if let Some((gi, gmesh)) = a.gas {
                let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                let Some(s) = self.sim.as_ref() else { return far };
                let Some(g) = s.drones.gas.puffs.get(gi as usize).filter(|g| g.mesh == gmesh) else { return far };
                let (c, sn) = g.spin.sin_cos();
                let (rx, ry) = (opts.right, opts.up);
                let (r2, u2) = ([rx[0] * c - ry[0] * sn, rx[1] * c - ry[1] * sn, rx[2] * c - ry[2] * sn],
                                [rx[0] * sn + ry[0] * c, rx[1] * sn + ry[1] * c, rx[2] * sn + ry[2] * c]);
                let sc = g.scale;
                return [
                    [r2[0] * sc, r2[1] * sc, r2[2] * sc, 0.0],
                    [u2[0] * sc, u2[1] * sc, u2[2] * sc, 0.0],
                    [0.0, 0.0, sc, 0.0],
                    [g.pos[0], g.pos[1], g.pos[2], 1.0],
                ];
            }
            if let Some(slot) = a.tracer {
                if slot >= MP_TRACER_BASE {
                    // mp-remote-fx: a remote shot's trail — `tracer_matrix`'s convention (+Z along the
                    // flight, scaled by one tick of travel, cut at the contact).
                    let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                    let Some(t) = self.sim.as_ref().and_then(|s| s.mp_tracer(slot - MP_TRACER_BASE)) else { return far };
                    let d = t.dir;
                    // R6 class sweep: the tracer's Mat_Align2Dir, as `tracer_matrix` [M 0x12c5a0:185].
                    let mut m = nf_game::searchlight::align2dir(d, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
                    m[3] = [t.pos[0], t.pos[1], t.pos[2], 1.0];
                    let len = t.speed.min(t.left.max(0.0));
                    for c in 0..3 { m[2][c] *= len; }
                    return m;
                }
                // fix-laser-beam: the gadget BEAM (Draw_LaserBeam / Draw_TaserBeam via
                // Player_MuzzleFlash 0x1a4298:107-137 — one stretched sprite from the muzzle to the
                // crosshair-ray hit). fix-gadgets-live: slots TRACER_SLOTS / +1 carry the game's own
                // sprites (0x2000131 "lazer" for the 0x4E record — the class-'N' pick [M 0x1a4560] —
                // and 0x200026d "taser" [M 0x1a46a8]), and `Draw_LaserBeam`'s stretch is the beam
                // object's own scale (obj+0xE8 = the muzzle→hit distance) with the yaw/pitch of
                // muzzle→hit in obj+0x50 (the negated-yaw euler) [M 0x1a4560:44-62].
                // gunimp-beam: the mounted laser's quad along `GunImp.beam` (muzzle, ray hit) — the
                // game's `PositionBeam` [M 0x1320d8:394-407]. The sprite extends along -z and the
                // object's +z points hit -> muzzle (the same tail as PositionBeam 0x1a9400), built
                // with the gadget beam's matrix helper; the flare sits at the hit, unscaled.
                if (slot as usize == TRACER_SLOTS + 7 || slot as usize == TRACER_SLOTS + 8) {
                    // gunimp-beam-lifetime: only the MOUNTED imp's beam draws - a dismounted gun's last shot must not
                // leave its quad hanging in the room (the sim clears eam while the imp is active).
                let pair = self.sim.as_ref().and_then(|s| s.drones.props.gunimps.iter().find(|g| g.active).and_then(|g| g.beam));
                    let Some((a, b)) = pair else { return [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]] };
                    if slot as usize == TRACER_SLOTS + 8 {
                        return nf_game::aivolume::matrix([0.0, 0.0, 0.0], b);
                    }
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if len < 1e-4 { return [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]]; }
                    let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                    let yaw = v[0].atan2(v[2]);
                    let pitch = -v[1].atan2((v[0] * v[0] + v[2] * v[2]).sqrt());
                    let mut m = nf_game::aivolume::matrix([pitch, yaw, 0.0], a);
                    for k in 0..3 { m[2][k] *= len; }
                    return m;
                }
                if slot as usize >= TRACER_SLOTS + 5 {
                    // sg5-laser-draw: the laser sight [M Player_LaserPointer 0x1a65d8]. Slot +5 the
                    // beam, datum → end, stretched like the tripbomb beam below (the same −z mesh);
                    // view-only in first person [M: `BLData+0x950 == 0`]. Slot +6 the dot at
                    // hit + normal·0.01, its z on the normal, scale 1 + d²/MAX_VIEW_CONE (2000) capped
                    // at 4 with d = datum → end [M ll. 189-205; L 0x30d0fc]. The flicker (Rand_FRand
                    // on the size byte) and the night-sight sprite 0x2000131 are not drawn yet.
                    let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                    let Some(s) = self.sim.as_ref() else { return far };
                    let Some((a, b, dot)) = s.laser_sight else { return far };
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if slot as usize == TRACER_SLOTS + 5 {
                        if !s.first_person || len < 1e-4 { return far; }
                        // The mesh's −z runs start→end (the +z-backward convention below), so z = a − b.
                        let mut m = z_basis([a[0] - b[0], a[1] - b[1], a[2] - b[2]], a);
                        for k in 0..3 { m[2][k] *= len; }
                        return m;
                    }
                    let Some((p, n)) = dot else { return far };
                    let sc = (1.0 + len * len / 2000.0).min(4.0);
                    let mut m = z_basis(n, [p[0] + n[0] * 0.01, p[1] + n[1] * 0.01, p[2] + n[2] * 0.01]);
                    for r in 0..3 { for k in 0..3 { m[r][k] *= sc; } }
                    return m;
                }
                if slot as usize >= TRACER_SLOTS + 3 {
                    // fix-satchel-tripbomb: the tripbomb's laser beam (mesh 0x2000133 [M the beam
                    // create]) — the sim rebuilds `tripbomb_beams` (start, end) every tick [M the
                    // laser block 0x12efb8-0x12ef78 + `PositionBeam` 0x1a9400]. Same stretch as the
                    // gadget beams (the beam object's z scaled by the length).
                    let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                    let Some(s) = self.sim.as_ref() else { return far };
                    let k = slot as usize - TRACER_SLOTS - 3;
                    let Some(&(a, b)) = s.drones.tripbomb_beams.get(k) else { return far };
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if len < 1e-4 { return far; }
                    // fix-laser-watch-look: PositionBeam 0x1a9400 stores the spherical of (start − end)
                    // with the elevation negated — the same +z-backward convention as Draw_LaserBeam
                    // [M 0x1a9400 tail], whose −z geometry [D the "watch_lazer" mesh 0x2000133:
                    // z ∈ [−1.0, 0]] flows start→end. The old +z pointed start→end.
                    // sg5-laser-draw: built from the direction (`z_basis`) — the atan2 euler missed a
                    // tilted beam's end; level beams are unchanged.
                    let mut m = z_basis([a[0] - b[0], a[1] - b[1], a[2] - b[2]], a);
                    for k in 0..3 { m[2][k] *= len; }
                    return m;
                }
                if slot as usize >= TRACER_SLOTS {
                    let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                    let Some(s) = self.sim.as_ref() else { return far };
                    // The sprite follows the active weapon [M 0x1a4560: lazer for class 'N' (0x4E),
                    // uv_lazer otherwise; 0x1a46a8: taser]; the other slots collapse.
                    let want = match s.gun.weapon {
                        0x4a | 0x4c => TRACER_SLOTS + 1,
                        0x4f => TRACER_SLOTS + 2,
                        _ => TRACER_SLOTS,
                    };
                    if slot as usize != want { return far; }
                    let Some((mut a, b)) = s.laser_beam else { return far };
                    // fix-laser-watch-look: the beam leaves the view model's POSED BONE 0 —
                    // AnimGetBoneWorldTrans(BLData+0x7e8, 0, 0) [M 0x1a4560:20] — composed with the
                    // object matrix (the first-person gun transform), the same composition the
                    // attachment draw below uses. The old read took the Character's STATIC creation
                    // matrix (identity) — the beam started at the WORLD ORIGIN and was never
                    // visible.
                    if s.first_person {
                        if let Some(vm) = self.view_models.get(&s.gun.weapon) {
                            if let Some((_, world, m)) = &posed[*vm] {
                                // A collapsed object matrix (the scoped-zoom hide) must not drag the
                                // beam to the collapse point — the watch lasers never scope
                                // (`+0x68 & 0x40` = 0 [L weapon_data 0x4e]), so this is a guard only.
                                if m[3][1] > -9999.0 {
                                    // `AnimGetBoneWorldTrans(obj, 0, 0)`'s second argument is a DATUM id when
                                    // non-negative [M 0x1c3120: param_2 >= 0 looks up the skin's datum list,
                                    // `QuatTransToMat(datum.q, datum.t) · bone world`; only negative ids index
                                    // bones]: datum 0 of the watch view model — the watch face. Bone 0 put the
                                    // beam's wide end at the screen edge (an 08 render).
                                    let c = &self.characters[*vm];
                                    let bm = match c.skin.datums.iter().find(|d| d.id == 0 && d.bone >= 0) {
                                        Some(dt) => anim::mul(&anim::mul(&anim::quat_trans_to_mat(dt.q, dt.t), &world[dt.bone as usize]), m),
                                        None => anim::mul(&world[0], m),
                                    };
                                    a = [bm[3][0], bm[3][1], bm[3][2]];
                                }
                            }
                        }
                    }
                    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if len < 1e-4 { return far; }
                    // fix-laser-watch-look: the game's angles are the spherical of (start − end) with
                    // the elevation negated [M 0x1a4560:44-62; the same tail in PositionBeam
                    // 0x1a9400] — the object's +z points from the hit BACK to the watch, and the
                    // sprite's geometry extends along −z [D the "lazer" mesh 0x2000131 on 07000005:
                    // z ∈ [−1.0198, 0.00016]], so the wide origin end sits at the watch and the beam
                    // tapers to the hit. The old +z pointed start→end: the −z sprite stretched
                    // BACKWARDS through the camera ("The laser watch sucks").
                    let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                    let yaw = v[0].atan2(v[2]);
                    let pitch = -v[1].atan2((v[0] * v[0] + v[2] * v[2]).sqrt());
                    // R6 (2026-10-22): the beam object's matrix is `Control_GetObjMatrix` 0x134210 →
                    // `View_RotTransMatrix` 0x1e5168 on these angles (z row (cos x·sin y, −sin x,
                    // cos x·cos y)) — `aivolume::matrix` is that builder. The placement-euler
                    // `object_matrix` put the far end off the crosshair on steep, angled shots
                    // (the 05 snow capture vs PCSX2). Scale z only [M 0x134210: +0xFA bit 1].
                    let mut m = nf_game::aivolume::matrix([pitch, yaw, 0.0], a);
                    for k in 0..3 { m[2][k] *= len; }
                    return m;
                }
                let b = self.sim.as_ref().and_then(|s| s.drones.bullets.get(slot)).filter(|b| b.state == 1);
                return match b { Some(b) => tracer_matrix(b), None => [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]] };
            }
            // The silencer tube hides during the variant swap (state 8) — the game clears datum 0 there
            // [M 0x1a5460:96-110]; the swap anim's own piece is what gets screwed on.
            if a.silencer && self.sim.as_ref().is_some_and(|s| s.gun.switch_state == 8) {
                return [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
            }
            // …and the swap piece is the one visible exactly then (the
            // silencer was invisible through the screw-on after silencer-white-artifact dropped it).
            if a.swap_piece && !self.sim.as_ref().is_some_and(|s| s.gun.switch_state == 8) {
                return [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
            }
            if let Some((slot, mesh)) = a.projectile {
                // A projectile-mesh slot: visible only while the slot's live projectile flies AND carries THIS
                // attachment's mesh (the game's `hashtable_set_object_to_entity_gfx(obj, weapon+0x5C)`). The drawn
                // matrix = `Mat_Align2Dir(dir, (0,1,0), (1,0,0))` at the projectile's position — `Bullet_init`
                // 0x12c5a0:185 aligns the bullet's matrix to its flight direction (the up/fallback vectors [L]
                // ee_start_05.bin @0x2d6210/0x2d6220); the per-tick tail moves `obj+0x30..0x38` only.
                let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                let Some(s) = self.sim.as_ref() else { return far };
                if slot >= SWOOSH_SLOT_BASE {
                    // rocket-swoosh: `Swoosh_Update` 0x18d200:64-80 — at the tail, `Mat_Align2Dir(tail − head,
                    // up = viewer − tail)` (the ribbon turns its face to the camera), then `Mat_Scale3f
                    // (life/max, 1, |tail − head|)`: the width narrows with the life, the length spans the
                    // segment. The viewer is the eye (the script camera while one runs) [S: glb_viewer+0xc0].
                    let k = (slot - SWOOSH_SLOT_BASE) as usize;
                    let Some(sw) = s.drones.swooshes.iter().flatten().filter(|w| w.mesh == mesh).nth(k) else { return far };
                    let d = [sw.tail[0] - sw.head[0], sw.tail[1] - sw.head[1], sw.tail[2] - sw.head[2]];
                    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                    if len < 1e-5 { return far; }
                    let eye = s.script_camera().map_or(s.bond.head(), |c| c.0);
                    let up = [eye[0] - sw.tail[0], eye[1] - sw.tail[1], eye[2] - sw.tail[2]];
                    let mut m = nf_game::searchlight::align2dir([d[0] / len, d[1] / len, d[2] / len], up, up);
                    let wsc = sw.life as f32 / sw.max.max(1) as f32;
                    for c in 0..3 { m[0][c] *= wsc; m[2][c] *= len; }
                    m[3] = [sw.tail[0], sw.tail[1], sw.tail[2], 1.0];
                    return m;
                }
                let norm = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt(); if l > 1e-9 { [v[0] / l, v[1] / l, v[2] / l] } else { [0.0, 1.0, 0.0] } };
                let (pos, dir, live_mesh): ([f32; 3], [f32; 3], u32) = if (slot as usize) < GRENADE_SLOTS {
                    // fix-hat-details: newest-first slot mapping — the game's draw ring keeps the NEWEST
                    // objects when a frame's adds overflow (the cel lists are newest-first [M
                    // `control_link_object_to_cel` 0x1346b8], so the reused entries are the oldest); with
                    // ≤ GRENADE_SLOTS live projectiles every one draws (the ring never bites below 512).
                    let gi = if s.drones.grenades.len() > GRENADE_SLOTS { s.drones.grenades.len() - GRENADE_SLOTS + slot as usize } else { slot as usize };
                    match s.drones.grenades.get(gi) {
                        // State 0 (spawned, not yet updated) has no mesh in the game: `Bullet_update` state 0
                        // assigns `weapon+0x5C` (`hashtable_set_object_to_entity_gfx`, disasm 0x12e8f0-0x12e900
                        // [M]) on the projectile's first update. The port drew it on the spawn tick at the
                        // camera — a screen-filling frag on the release frame, which PCSX2 frozen at that frame
                        // doesn't show (released while the pin was still being pulled).
                        Some(g) if g.state == 1 => {
                            // fix-satchel-tripbomb: a stuck satchel/tripbomb draws aligned to the
                            // NEGATED surface normal (the stick arm's `Mat_Align2Dir(obj+0x90,
                            // −normal, …)` [M 0x12ce80:256-266]) instead of the flight direction.
                            match &g.phase {
                                // Only a STUCK gadget's normal draws (a flying 0x3a/0x3b carries the
                                // placeholder phase box for the beep latch — the stick overwrites it).
                                Some(p) if g.stuck => (g.pos, [-p.normal[0], -p.normal[1], -p.normal[2]], g.mesh),
                                _ => (g.pos, norm(g.velocity), g.mesh),
                            }
                        }
                        _ => return far,
                    }
                } else if (slot as usize) < GRENADE_SLOTS + TRACER_SLOTS {
                    match s.drones.bullets.get(slot as usize - GRENADE_SLOTS) {
                        // camera-beam-satchel-draw: a LANDED satchel charge draws aligned to the
                        // NEGATED landing normal (the game's stick pose [M Bullet_CollisionHandler
                        // 0x12ce80:256-266]) — `Bullet_update` case 3 zeroes the flight dir
                        // [M 0x12edec] and the old read collapsed the mesh to a point (align2dir
                        // of the zero vector zeroes the rotation rows).
                        Some(b) if b.state == 1 => {
                            let dir = match (b.landed, b.stick_normal) {
                                (true, Some(n)) => [-n[0], -n[1], -n[2]],
                                _ => b.dir,
                            };
                            (b.pos, dir, s.weapons.projectile_mesh(b.weapon as usize))
                        }
                        _ => return far,
                    }
                } else {
                    match s.drones.player_bullets.get(slot as usize - GRENADE_SLOTS - TRACER_SLOTS) {
                        // The ridden guided rocket is never drawn in its own camera: `Player_PositionCamera`
                        // 0x18f5d8 case 0xc `View_SetDrawInOtherViewsOnly(BL+0x878, view)` [M]. Drawing it put
                        // the eye inside the rocket's mesh — a big dark octagon instead of the view.
                        Some(b) if b.guided_cam && s.missile_cam.is_some() => return far,
                        Some(b) if !b.dead => (b.pos, b.dir, s.weapons.projectile_mesh(b.weapon as usize)),
                        _ => return far,
                    }
                };
                if live_mesh != mesh { return far; }
                let mut m = nf_game::searchlight::align2dir(dir, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
                m[3] = [pos[0], pos[1], pos[2], 1.0];
                return m;
            }
            // fix-impacts-render: the impact-effect slots — decals, casings, debris at the game's own draw
            // transforms (docs/atlas/effect.md). A slot draws only while its live effect carries the slot's
            // mesh (the game sets the node/object's glist from the hash at create); free slots collapse far
            // below the world like the tracers.
            if let Some((kind, slot, mesh)) = a.effect {
                let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                let Some(s) = self.sim.as_ref() else { return far };
                return match kind {
                    // The decal node: `Mat_Align2Dir(node+0x20, normal, (0,1,0) @0x2d6210, (1,0,0) @0x2d6220)`
                    // 0x1df2a8 with the translation `hit + normal·0.01` stored at node+0x50 [M 0x1ce9f0]; drawn
                    // by `View_DrawObjectGlist`/`View_DrawCelGlist`'s `psiDrawObjectMatrix(node+0x10, node+0x20)`
                    // (0x1e6780/0x1e6538). No scale — the quad size lives in the mesh [M 0x1e6780:58-65].
                    0 => match s.drones.effects.decals.get(slot) {
                        Some(d) if d.mesh == mesh => {
                            let mut m = nf_game::searchlight::align2dir(d.normal, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
                            m[3] = [d.pos[0], d.pos[1], d.pos[2], 1.0];
                            m
                        }
                        _ => far,
                    },
                    // The casing object: `Control_GetObjMatrix` 0x134210's euler(+0x50)·pos(+0x30)
                    // (`View_RotTransMatrix`), scaled by obj+0xe8 (= 1 here — no scale write at eject);
                    // only the flying state draws (`Bullet_init_casing` 0x12fde0 parks the slot
                    // `View_SetDrawInNoViews` until `Bullet_init_casing_ex` 0x12f678 re-shows it).
                    1 => match s.drones.casings.list.get(slot) {
                        Some(c) if c.state == 0 && c.mesh == mesh => casing_matrix(c),
                        _ => far,
                    },
                    // The debris object: euler spin·age at pos, scaled by obj+0xe8 (0.4 + FRand(0.1), applied
                    // once by `Control_GetObjMatrix` 0x134210:25-33 — `View_DrawObjectGlist` copies the cached
                    // matrix unscaled for non-billboards [M 0x1e6780:47-49]). The tick count is 255 − alpha:
                    // `Debris_Create` stores obj+0x106 = 0xff and `Debris_Update` 0x1aac00 steps it down 1/tick.
                    2 => match s.drones.effects.debris.get(slot) {
                        Some(d) if d.mesh == mesh => {
                            let age = 255.0 - d.alpha as f32;
                            let eul = [d.spin[0] * age, d.spin[1] * age, d.spin[2] * age];
                            let mut m = nf_formats::placement::object_matrix(eul, d.pos);
                            for r in 0..3 { for k in 0..3 { m[r][k] *= d.scale; } }
                            m
                        }
                        Some(_) => far,
                        // A killed breakable's pieces (`Break_Kill` 0x12b970 → `Debris_Create`): the
                        // same class and draw — euler `rot` (the accumulated spin [M 0x1aac00:49-52]) at
                        // obj+0x30, scaled by obj+0xe8 (`Control_GetObjMatrix` 0x134210:25-33).
                        None => match s.drones.breaks.shatter.debris.get(slot - s.drones.effects.debris.len()) {
                            Some(d) if d.model == mesh => {
                                let mut m = nf_formats::placement::object_matrix(d.rot, d.pos);
                                for r in 0..3 { for k in 0..3 { m[r][k] *= d.e8; } }
                                m
                            }
                            Some(_) => far,
                            // Past the breakable pieces: the blast debris — the same `Debris_Update` 0x1aac00
                            // object and draw (euler `rot` at obj+0x30, scaled by obj+0xe8).
                            None => match s.drones.blast_debris.get(slot - s.drones.effects.debris.len() - s.drones.breaks.shatter.debris.len()) {
                                Some(d) if d.model == mesh => {
                                    let mut m = nf_formats::placement::object_matrix(d.rot, d.pos);
                                    for r in 0..3 { for k in 0..3 { m[r][k] *= d.e8; } }
                                    m
                                }
                                _ => far,
                            },
                        },
                    },
                    // fix-grenade-throw: the smoke grenade's cloud (`Occlude_Create` 0x17ea80) — a
                    // static object at `obj+0x30`, no rotation (`Occlude_Update` 0x17eba0 only counts
                    // down; `Control_GetObjMatrix` 0x134210 has no euler write on this class).
                    3 => match s.drones.smoke_clouds.get(slot) {
                        Some(c) => {
                            let mut m = [[0.0f32; 4]; 4];
                            m[0][0] = 1.0; m[1][1] = 1.0; m[2][2] = 1.0;
                            m[3] = [c.pos[0], c.pos[1], c.pos[2], 1.0];
                            m
                        }
                        _ => far,
                    },
                    // The DroneScan bracket: at the candidate's root (`obj+0x30` [M 0x1a12d8:53]), no rotation
                    // (control_create_object's null angles) — its glist flag 8 makes `View_DrawObjectGlist` 0x1e6780
                    // billboard it (the viewer's rotation), scale 1.
                    7 | 8 => match s.drones.env.as_ref().filter(|e| e.kind == 1) {
                        // rain-drops: the live drop at this slot [M DrawDrops 0x18cb90: DropState == 1].
                        Some(e) if e.alive.get(slot).copied().unwrap_or(false) => {
                            // The second pass is level-gated: GameState+12 − 0x07000009 < 3 [M].
                            if kind == 8 && !matches!(s.level.wrapping_sub(0x0700_0009), 0..=2) { return far; }
                            let p = e.drops[slot];
                            let mut m = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [p[0], p[1], p[2], 1.0]];
                            if kind == 8 {
                                // Mat_CopyRot(viewer+0x1A0): the camera's rotation — right / up / forward rows.
                                let (r, u) = (opts.right, opts.up);
                                let f = [r[1] * u[2] - r[2] * u[1], r[2] * u[0] - r[0] * u[2], r[0] * u[1] - r[1] * u[0]];
                                m[0] = [r[0], r[1], r[2], 0.0]; m[1] = [u[0], u[1], u[2], 0.0]; m[2] = [f[0], f[1], f[2], 0.0];
                            }
                            m
                        }
                        _ => far,
                    },
                    6 => match crate::hud::camera_scan_roots(s).get(slot) {
                        Some(p) => {
                            let (r, u) = (opts.right, opts.up);
                            let f = [r[1] * u[2] - r[2] * u[1], r[2] * u[0] - r[0] * u[2], r[0] * u[1] - r[1] * u[0]];
                            [[r[0], r[1], r[2], 0.0], [u[0], u[1], u[2], 0.0], [f[0], f[1], f[2], 0.0], [p[0], p[1], p[2], 1.0]]
                        }
                        None => far,
                    },
                    // fix-satchel-tripbomb: the satchel's countdown digits (kinds 4 = tens, 5 = units)
                    // — the meshes SatchelDigits[sec/10]/[sec%10] [L 0x2bed90], drawn at the stuck
                    // satchel's matrix (aligned to −normal [M 0x12ce80:266]) offset along its own
                    // axes [*suggested*: the game offsets by its own small rotation/translation pair
                    // 0x12d890:486-511 — the port's (±0.03, 0.032, 0.06) is cosmetic].
                    4 | 5 => {
                        // satchel-stick2: NO `stuck` gate — `Bullet_DoTrails`' 0x3b tail spawns the
                        // two digit objects EVERY tick in every state < 2 [M 0x12d890:418-553; the
                        // sim writes `digits` in flight too — grenade.rs's tail runs before the
                        // stuck early-out], so the numbers ride the satchel from the throw. In flight the thrown one's
                        // `phase.normal` is the spawn's [0, 1, 0] placeholder (the stick overwrites it with the
                        // surface normal), so the flight digits stand upright on the satchel [S —
                        // the game aligns them to the bullet's own matrix].
                        // camera-beam-satchel-draw: the source covers BOTH satchel carriers — the
                        // thrown grenades AND the drone-fired 0x3b bullets (the 4a lift shooters
                        // [D cameras-turrets.md §3 #286/#287 weapon 0x3d → family 0x3b]) whose
                        // `Bullet::digits` the sim's DoTrails tail writes every tick [M 0x12d890:418-553].
                        let mut satchels: Vec<([f32; 3], [f32; 3], Option<(u8, u8)>)> = s.drones.grenades.iter()
                            .filter(|g| g.weapon_type == 0x3b && g.state < 2 && g.phase.is_some())
                            .map(|g| {
                                let p = g.phase.as_ref().unwrap();
                                (g.pos, [-p.normal[0], -p.normal[1], -p.normal[2]], p.digits)
                            })
                            .collect();
                        for b in s.drones.bullets.iter()
                            .filter(|b| b.state < 2 && b.digits.is_some() && b.family(&s.weapons) == 0x3b)
                        {
                            // A landed charge aligns to its stick normal; a flying one keeps the
                            // upright placeholder the thrown arm uses [S as above].
                            let dir = match (b.landed, b.stick_normal) {
                                (true, Some(n)) => [-n[0], -n[1], -n[2]],
                                _ => [0.0, 1.0, 0.0],
                            };
                            satchels.push((b.pos, dir, b.digits));
                        }
                        let Some((pos, align, digits)) = satchels.get(slot) else { return far };
                        let Some((tens, units)) = digits else { return far };
                        let digit = if kind == 4 { tens } else { units };
                        if mesh != 0x0200_0791 + *digit as u32 { return far; }
                        let mut m = nf_game::searchlight::align2dir(*align, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
                        let (x, y, z) = (m[0], m[1], m[2]);
                        let (lx, ly, lz) = if kind == 4 { (-0.03f32, 0.032f32, 0.06f32) } else { (0.03, 0.032, 0.06) };
                        m[3] = [
                            pos[0] + x[0] * lx + y[0] * ly + z[0] * lz,
                            pos[1] + x[1] * lx + y[1] * ly + z[1] * lz,
                            pos[2] + x[2] * lx + y[2] * ly + z[2] * lz,
                            1.0,
                        ];
                        m
                    }
                    _ => far,
                };
            }
            let far = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
            if let Some((h, si, hash, k)) = a.script {
                // dragon-safe-follow: a script's 0xF0 entity IS the pickup it created [M 0x1d8968 →
                // `Pickup_Create`]; the game stops drawing it once taken (deleted, respawn 0) or while
                // it waits to respawn (hidden) [M 0x18e650's tail]. This slot draws it at the stream's
                // pose (the game's obj+0x90 [M 0x1d9a08:94-110]) — collapse it with the pickup.
                if let Some(s) = self.sim.as_ref() {
                    if let Some(p) = s.drones.script_pickup_of.get(&(h, si)).and_then(|&pk| s.drones.pickups.get(pk)) {
                        if p.model == hash && (p.gone || p.hidden) { return far; }
                    }
                }
                return self.sim.as_ref().and_then(|s| s.script_entity_k((h, si), hash, k as usize)).map(|m| {
                    // The billboard entities (`View_DrawObjectGlist` 0x1e6780:43-65): the glist flags
                    // bit 0x8 (reached via `Control_SetGList` 0x134960's `obj+0xF0 = glist+0x44`)
                    // replaces the drawn ROTATION with the viewer's — `Mat_CopyRot(viewer+0x1a0)` —
                    // keeping the interp translation and the interp scale `obj+0xE8` (the key scale
                    // × the script scale via `Script_GetInterp` 0x1da158:117, `Mat_Scale3f`
                    // 0x1e35f0). The port's entity matrix is rotation·s with s uniform (the interp
                    // builds it from pos/quat/scale keys), so s = |column 0|. The obj+0x58 spin arm, the GAME'S
                    // EXACT RULE for the render pass [M View_DrawObjectGlist 0x1e6780:48-57]: when the
                    // entity's obj+0x58 != 0 the drawn rotation is `viewer_rot · RotZ(spin)` — RotZ about the
                    // SCREEN-FACING axis (spin in the screen plane), built from euler (0, 0, +0x58) via
                    // RotTransMatrix and Concat'd INSIDE the viewer alignment — not the plain viewer rotation this
                    // port draws; the spin accumulates per tick, `obj+0x58 += def[3][0]·FRAME_RATE_MUL` in the
                    // effect updaters (gases_smoke 0x1ab2d0:50/85/185; the rate sign flips per gas type at
                    // def[3][0xc]). Not ported here (the blast's first-frame flash quad dominates and the puffs'
                    // spin is sub-visible); a live check can calibrate the per-def rates from def[3] if this is
                    // ever ported.
                    if !self.script_billboards.contains(&hash) { return m; }
                    let s = (m[0][0] * m[0][0] + m[1][0] * m[1][0] + m[2][0] * m[2][0]).sqrt();
                    let (r, u) = (opts.right, opts.up);
                    let n = |v: [f32; 3]| { let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-9); [v[0] / l, v[1] / l, v[2] / l] };
                    let (r, u) = (n(r), n(u));
                    let f = n([r[1] * u[2] - r[2] * u[1], r[2] * u[0] - r[0] * u[2], r[0] * u[1] - r[1] * u[0]]);
                    [
                        [r[0] * s, r[1] * s, r[2] * s, 0.0],
                        [u[0] * s, u[1] * s, u[2] * s, 0.0],
                        [f[0] * s, f[1] * s, f[2] * s, 0.0],
                        m[3],
                    ]
                }).unwrap_or(far);
            }
            // An MP body's weapon draws only while its slot holds that weapon.
            if let Some(&(slot, w)) = self.mp_body_weapons.get(&(k as u32)) {
                if self.sim.as_ref().is_none_or(|s| s.mp_body_held(slot) != w) { return far; }
                // mp-remote-fx: the dead body holds nothing — `Player_CheckForDeath` 0x190810 drops
                // the gun as a pickup and runs `Player_WeaponNone` [M :43-63].
                if self.sim.as_ref().is_some_and(|s| s.mp_body_death(slot).is_some()) { return far; }
            }
            if a.dropped {
                // fix-objects: `!p.gone` — a TAKEN drop stays in the list as a tombstone (drone.rs
                // `update_pickups` keeps the Vec indices stable for the draw slots) and must not be
                // drawn; the game deleted the object (`Pickup_Handler` 0x18e650:338-348).
                let p = drone_of(a.character as usize).and_then(|(s, di)| s.drones.pickups.iter().find(|p| p.from == Some(di) && !p.hidden && !p.gone));
                return p.map_or(far, |p| p.m);
            }
            let c = &self.characters[a.character as usize];
            // A hidden owner's attachments collapse at its placement point (frame-240: the palette path
            // no longer poses hidden characters; the collapsed mesh draws nothing wherever it sits, and
            // the CPU path collapses at the same point so the two stay vertex-equal).
            if hidden(c, a.character as usize) { return collapse(&c.matrix); }
            let datum = match &posed[a.character as usize] {
                Some((_, world, m)) => {
                    let d = anim::mul(&anim::mul(&a.local, &world[a.bone]), m);
                    match a.watch_hand {
                        // RotMatrixX(−ang − π/2) [M 0x1e46a0: rows (1,0,0), (0,c,s), (0,−s,c), PS2Sinf2]
                        // concatenated in the datum's frame (Concat 0x1e4a90: B ← B·A), translation the datum's.
                        Some(k) => {
                            let t = self.sim.as_ref().map_or(0.0, |s| s.drones.progress.stats[0].elapsed as f32 + nf_game::watch::level_clock_base(s.level));
                            let rate = [nf_game::watch::HOURS_RATE, nf_game::watch::MINUTES_RATE, nf_game::watch::SECONDS_RATE][k as usize];
                            let x = -nf_game::watch::hand_angle(t, rate) - std::f32::consts::FRAC_PI_2;
                            let (sn, cs) = (nf_game::trig::sinf(x), nf_game::trig::cosf(x));
                            let r = [[1.0, 0.0, 0.0, 0.0], [0.0, cs, sn, 0.0], [0.0, -sn, cs, 0.0], [0.0, 0.0, 0.0, 1.0]];
                            anim::mul(&r, &d)
                        }
                        None => d,
                    }
                }
                None => {
                    let ib = c.skin.inverse_bind.get(a.bone).map(|b| b.0).unwrap_or([0.0; 3]);
                    let bind = anim::quat_trans_to_mat([0.0, 0.0, 0.0, 1.0], ib.map(|v| -v));
                    anim::mul(&anim::mul(&a.local, &bind), &c.matrix)
                }
            };
            // Dropped: the drone holds nothing (`anim+0x62` = 0).
            // Unarmed (dropped, or surrendered: `ChangeWeapon(1)`): no gun. *Deviation*: a drawn sidearm (weapon 6) still
            // shows the original weapon's mesh.
            if drone_of(a.character as usize).is_some_and(|(s, di)| s.drones.list[di].weapon <= 1) { return collapse(&datum); }
            if let Some((id, digit)) = a.pda_digit {
                // The decryptor screen: datum `id` shows `PdaDigits[digit]` while the display char
                // 3−id is that digit, on the held decryptor's own view model [M 0x1a5460:155-194].
                let Some(s) = self.sim.as_ref() else { return collapse(&datum) };
                if self.view_models.get(&s.gun.weapon) != Some(&(a.character as usize)) { return collapse(&datum); }
                if s.pda_display[3 - id as usize] != b'0' + digit { return collapse(&datum); }
                return datum;
            }
            if let Some(slot) = a.quiver_bolt {
                // delta-repeater: the Delta Repeater's bolt count, re-evaluated per frame exactly where
                // `Player_WeaponFiring` 0x1a5460:331-381 re-issues the datum entities each tick [M]:
                // count = clip[0x11] (the s16 at BLData+0x1B8 + 0x11·0xC — the port's `clip[0x11]`);
                // +1 while the view state is 9 (`sVar13 == 9` → `lVar11 = clip + 1`, the fire script's
                // state, the port's `gun.fire_state`); once the reload script (view state 6) passes
                // frame 80.0 the count is min(carried[weapon+0x90 = 0x0F], clip_size +0x92 = 3)
                // [M :345-360]. Attachment rule [:368-381]: datum 4 when count > 0 AND the view state
                // is NOT 9; datum 5 when count > 1; datum 6 when count >= 3. Only the held weapon's
                // quiver draws (the game writes the held vm's datums; a holstered crossbow keeps its
                // last state — the port collapses the un-held model's bolts, the AIMS arm's pattern).
                let Some(s) = self.sim.as_ref() else { return collapse(&datum) };
                if self.view_models.get(&s.gun.weapon) != Some(&(a.character as usize)) { return collapse(&datum); }
                if u16::from_le_bytes(s.weapons.records[s.gun.weapon as usize][2..4].try_into().unwrap()) != 0x11 { return collapse(&datum); }
                let mut count = s.drones.inventory.clip.get(0x11).copied().unwrap_or(0) as i32;
                if s.gun.fire_state {
                    count += 1;
                } else if s.gun.reload_state != 0 && s.view_time > 80.0 {
                    let at = s.weapons.ammo_type(0x11);
                    let carried = s.drones.inventory.carried.get(at).copied().unwrap_or(0) as i32;
                    count = carried.min(s.weapons.clip_size(0x11) as i32);
                }
                let show = match slot {
                    4 => count > 0 && !s.gun.fire_state,
                    5 => count > 1,
                    _ => count >= 3,
                };
                return if show { datum } else { collapse(&datum) };
            }
            if let Some(bone) = a.aims_light {
                // fix-gadgets-live: the AIMS-20's side lights (mesh 0x2000132 on bones 8..0xB, attached
                // in build for the 0x1a/0x1b records [M Player_WeaponFiring 0x1a5460:256-360]). Only the
                // in-hand weapon's lights draw; the blink/off arms run here:
                // - idle > 15 → bone 8 blinks on the game clock's 20/40 duty: dark while
                //   `(GameState+0x76 % 40) > 0x14` [M 0x1a5460:264-266], the clock the integer
                //   100ths `psiGetTimeIn100ths` [M 0x1094a0:8] written every flow pass [M
                //   0x1c98c8:84] — the port runs the same integer formula over its ticks
                //   (`clock_100ths`), a 0.4 s cycle like the game's;
                // - the all-off gate is record-dependent (fix-aims20-lights-render below): +0x93a in
                //   burst, clip[0x1A] in grenade [M 0x1a5460 disasm 0x1a5bf4-68];
                // fix-aims20-modes-lights: the bone-9/10/11 gates, previously unmeasured/always lit —
                // the ground truth measured the display: grenade rounds 0 → red, red, dark, red; the
                // long-no-anim state → red, dark, red, red; all four lit with rounds present.
                // - bone 9 dark when the raise cooldown BL+0x93e has drained [M 0x1a5460:275-281];
                // - bone 10 (the grenade light) dark when the 0x1b clip is dry [M 0x1a5460:282-288],
                //   blinking in grenade mode below 3 rounds on FlashMod > 0.75 [M 0x1a5460:289-294];
                // - bone 11 (the burst light) dark when the 0x1a clip is dry [M 0x1a5460:298-304],
                //   blinking in burst mode below 7 rounds [M 0x1a5460:305-308]. FlashMod is the
                //   pulsing flash global = `(int)GameState+0x76 · 0.01` shimmered [M 0x1a7810:95-97],
                //   the port's `flash_mod(frame)` runs the same integer-100ths formula — the
                //   0.75 crossing lands on the same clock ticks the game's does.
                let active = self.sim.as_ref().and_then(|s| self.view_models.get(&s.gun.weapon)) == Some(&(a.character as usize));
                let Some(s) = self.sim.as_ref() else { return collapse(&datum) };
                if !active { return collapse(&datum); }
                let idle = s.drones.aims_idle;
                // fix-aims20-lights-render: the all-off gate is RECORD-DEPENDENT [M 0x1a5460 disasm
                // 0x1a5bf4-68]. The C's line-312 `sVar5 = *(short *)(iVar1 + 0x93a)` in the else arm is a
                // decompiler artifact: the BNEL at 0x1a5bf8 (record id != 0x1a → jump 0x1a5c40) EXECUTES
                // its delay slot `lh $v0, 0x2f0($s2)`, so the `FRAME_RATE * 45.0 <` compare at 0x1a5c60
                // reads **clip[0x1A]** (the burst slot, BL+0x2f0) when the held record is 0x1b, and the
                // idle counter +0x93a (the reload at 0x1a5c3c) only when it is 0x1a. The disc's burst mag
                // is 30 [L weapon_data 0x1a +0x92], so in grenade mode the gate can never fire — the row
                // stays lit while idle, as the ground truth measured ("staying lit while idle",
                // docs/live/2026-10-14, "The side lights"). Burst mode keeps the 0.75 s dead-still all-off.
                let all_off = if s.gun.weapon == 0x1a {
                    idle > 45
                } else {
                    s.drones.inventory.clip.get(0x1a).copied().unwrap_or(0) > 45
                };
                if all_off { return collapse(&datum); }
                if bone == 8 && idle > 15 && clock_100ths(s.drones.frame) % 40 > 20 { return collapse(&datum); }
                if bone == 9 && s.drones.aims_raise == 0 { return collapse(&datum); }
                if bone == 10 || bone == 11 {
                    let (wid, slot_of) = if bone == 10 { (0x1b_u8, 0x1b_usize) } else { (0x1a_u8, 0x1a_usize) };
                    let rounds = s.drones.inventory.clip.get(slot_of).copied().unwrap_or(0);
                    if rounds == 0 { return collapse(&datum); }
                    if s.gun.weapon == wid {
                        let low = if bone == 10 { rounds < 3 } else { rounds < 7 };
                        if low && flash_mod(s.drones.frame) > 0.75 { return collapse(&datum); }
                    }
                }
                return datum;
            }
            if a.flash.is_none() { return datum; }
            // Bond's flash: `BL+0x938` = 3 at the shot, and `Player_CollisionHandler` decrements it in the
            // same tick before the draw test, so `Draw_MuzzleFlash` runs on 2 ticks [M 0x1949a8:192-202]
            // (the port drew 3); along the datum's +Z with a random roll and scale.
            if self.sim.as_ref().and_then(|s| self.view_models.get(&s.gun.weapon)) == Some(&(a.character as usize)) {
                return match self.sim.as_ref().and_then(|s| s.last_shot.filter(|&f| s.drones.frame.saturating_sub(f) < 2).map(|f| (s, f))) {
                    Some((s, f)) => player_flash_matrix(&datum, s.drones.frame ^ f.wrapping_mul(0x9e37_79b9)),
                    None => collapse(&datum),
                };
            }
            // mp-remote-fx: an MP body's flash (the held weapon's — the gate above) lives 2 ticks
            // after each of the slot's shots, as the guards' (`DroneWeap_FireWeapon`).
            if let Some(&(slot, _)) = self.mp_body_weapons.get(&(k as u32)) {
                let Some(s) = self.sim.as_ref() else { return collapse(&datum) };
                let now = s.drones.frame;
                return match s.mp_fx.last_shot.get(slot).copied().flatten().filter(|&f| now.saturating_sub(f) < 2) {
                    Some(f) => muzzle_flash_matrix(&datum, f ^ (k as u32).wrapping_mul(0x9e37_79b9)),
                    None => collapse(&datum),
                };
            }
            // AI-driven: the flash object lives 2 ticks after each shot (`DroneWeap_FireWeapon`).
            if let Some((s, di)) = drone_of(a.character as usize) {
                let now = s.drones.frame;
                return match s.drones.list[di].shots.iter().rev().find(|&&f| now.saturating_sub(f) < 2) {
                    Some(&f) => muzzle_flash_matrix(&datum, f ^ (k as u32).wrapping_mul(0x9e37_79b9)),
                    None => collapse(&datum),
                };
            }
            // Firing demo: a shot every 8 ticks, the flash object living 2 ticks (`Drone+0x30` life = 2), staggered
            // per drone. The rate is nf-view's, not the weapon's (G-025).
            let tick = ticks as u32 + 3 * a.character;
            if !opts.firing || tick % 8 >= 2 { return collapse(&datum); }
            muzzle_flash_matrix(&datum, (tick / 8) ^ (k as u32).wrapping_mul(0x9e37_79b9))
        };
        let attach: Vec<Mat4> = self.attachments.iter().enumerate().map(|(k, a)| att_one(k, a)).collect();
        let mut hidden: Vec<bool> = self.characters.iter().enumerate().map(|(ci, c)| hidden(c, ci)).collect();
        // fix-sentinel-camera-play: the guided ride draws no view model — the game's player
        // substate 10 cleared the anim set [M `Player_ChangeSubState(10)`, `Player_Update`
        // 0x193600:242-256; L PCSX2 "no view model, no crosshair"] — the palette collapse (the
        // port's established not-drawn state) does it, and it restores when the ride ends.
        if self.sim.as_ref().is_some_and(|s| s.missile_cam.is_some()) {
            if let Some(s) = self.sim.as_ref() {
                if let Some(&vm) = self.view_models.get(&s.gun.weapon) {
                    if let Some(h) = hidden.get_mut(vm) { *h = true; }
                }
            }
        }
        FrameState { posed, attach, hidden }
    }

    /// fix-facial2: the morph weights of a morph-bearing script character right now — the face-track
    /// window (the started opcode-0 window whose sequence carries the skin's facial channel count,
    /// the same window `script_pose_together` poses bones from) sampled at the object's frame via
    /// `AnimFacialBlend`'s source arithmetic [M `facial_weights`, `AnimFacialBlend` 0x1be648].

    /// The per-frame draw gate of character `ci` (`frame_state`'s `hidden`, hoisted here so the live
    /// tests read the same gate the render walks). A script actor is hidden while its
    /// `Script_AnimStart` object is not live (created at `Script_AnimStart` 0x1d8f40, deleted by
    /// `Script_KillStream` — 0x06000072/73 set no keep-objects flag [D]); a drone-backed character is
    /// hidden when its drone is absent or drawn-frozen: `enabled` models the game's `obj+0xF0`
    /// bit 0x10 (standins-scene-929: the mapping is the game's own, not a stand-in) —
    /// `NDrone2_Enable(0)` 0x149dd0:8-15 sets `obj+0xF0 |= 0x10`, the bit `View_AddObjects`
    /// 0x1e62d8:33 excludes from the draw list, and `Drone_EnableAll(0)` [M 0x1383e8] applies it to
    /// every present drone when a cut-scene fires event 5; a spawner-held-back reinforcement stays
    /// hidden unless R asks for the reinforcements.
    /// A lock's posed slot (door slot `k`, past the doors, SSs and breakables): `2·li` is the placed
    /// object, `2·li + 1` its LOD variant; the one `obj+0xD8` points at draws at the placement
    /// matrix, the other is `None` (collapsed). `Control_NextLOD` 0x1349b0 is the swap (`Lock::lod_next`).
    fn lock_slot(&self, s: &Sim, k: usize) -> Option<Mat4> {
        let j = k.checked_sub(s.drones.doors.list.len() + s.drones.sss.list.len() + s.drones.breaks.list.len())?;
        let Some(l) = s.drones.locks.get(j / 2) else {
            // Past the locks: the gun emplacements, GUNIMP_SLOTS each — slot 0 the base at
            // `base_matrix`, slot 1 + a the `TURRET_ARTS[a]` gun child at `barrel_matrix`, shown only
            // when it is this mount's art [M GunImp_Update 0x1320d8:162-173].
            let g = j.checked_sub(2 * s.drones.locks.len())?;
            if g < GUNIMP_SLOTS * self.gunimp_placements {
                let gi = s.drones.props.gunimps.get(g / GUNIMP_SLOTS)?;
                return match g % GUNIMP_SLOTS {
                    0 => Some(gi.base_matrix()),
                    p => (gi.art_slot() == Some(p - 1)).then(|| gi.barrel_matrix()),
                };
            }
            // Past the emplacements: the security cameras, three slots each — the body at the sweep
            // pose, the mount upright along the pose's direction [M Sensor_SetBeam 0x1af398:50-54],
            // and the sight beam at the same pose with the lens obj+0xE8 stretch on the mesh's
            // length axis [M 0x1af398:58,72,78 + 0x1af010:102; the Camera_Beam mesh's +z unit
            // length, D probe]. State 3 collapses it (the lens display mask cleared [M 0x1af618:248-250]).
            let c = g - GUNIMP_SLOTS * self.gunimp_placements;
            // Past the sensors: the gated dynamic objects (0x3B, `WorldObjects::create`'s order), one
            // slot each — the placement matrix until the sim deletes the object (`DynamicObject_Update`
            // 0x18d3b8:26 mode-0 armed-B → obj+0xFE bit 0 → the delete pass), then collapsed.
            if c >= 3 * s.drones.sensors.list.len() {
                let di = c - 3 * s.drones.sensors.list.len();
                if di >= self.dynobj_matrices.len() {
                    let ri = di - self.dynobj_matrices.len();
                    let (rot, trans) = *self.rotor_placements.get(ri)?;
                    let a = s.drones.worldobjs.rotors.get(ri)?.angle;
                    return Some(nf_formats::placement::object_matrix([rot[0] + a[0], rot[1] + a[1], rot[2] + a[2]], trans));
                }
                // The live object by its create slot (the delete pass drops dead ones from the list);
                // gone, or hidden by the no-draw bits `View_AddObjects` 0x1e62d8:33 tests
                // (`obj+0xF0 & 0x8010` — the create's |0x30 while p0's channel is off, mode != 0's
                // |0x30 while B is armed [M 0x18dbf8:27-29, 0x18d3b8:34-36]) → collapsed.
                let d = s.drones.worldobjs.dynobjs.iter().find(|d| d.slot as usize == di)?;
                if d.dead || d.flags & 0x8010 != 0 { return None; }
                return self.dynobj_matrices.get(di).copied();
            }
            let sn = s.drones.sensors.list.get(c / 3)?;
            return match c % 3 {
                0 => Some(sn.matrix),
                1 => Some(sn.mount_matrix()),
                _ => {
                    if sn.lens == 0 || sn.state == 3 { return None; }
                    // `Mat_Copy(lens+0x90, obj+0x90)` [M 0x1af398:58]: the lens rides the camera's
                    // own pose matrix; the stretch scales the mesh's local z (the game's OnlyScaleZ
                    // [M 0x1af010:102], the same row-2 scale the port's other beams use).
                    let mut m = sn.matrix;
                    for k in 0..3 { m[2][k] *= sn.beam_len; }
                    Some(m)
                }
            };
        };
        (l.lod_next == (j % 2 == 1)).then(|| self.lock_matrices.get(j / 2).copied()).flatten()
    }

    pub fn char_hidden(&self, ci: usize, opts: ViewOpts) -> bool {
        let Some(s) = self.sim.as_ref() else { return false };
        // A first-person view model that is not the held weapon's is never drawn: only the held
        // one is placed at the eye (`frame_state`'s view arm); the rest sat at their load matrix
        // near the world origin — every weapon in one spot, seen through the map (0700000a).
        if self.view_models.values().any(|&v| v == ci) {
            return self.view_models.get(&s.gun.weapon) != Some(&ci);
        }
        if let Some(&(slot, _)) = self.mp_bodies.iter().find(|(_, c)| *c == ci) {
            return s.mp_body_state(slot).is_none() && s.mp_body_death(slot).is_none();
        }
        if self.bond_body == Some(ci) { return s.wire_camera().is_none(); }
        let c = &self.characters[ci];
        // mp-character-data: a parked MP body is hidden — the draw moves the Character matrix to
        // the slot's live position to reveal it (the sentinel is the NPC-pool scale).
        if self.mp_char_bodies.iter().any(|b| b.chars.contains(&ci)) && c.matrix[3][1] <= -999_000.0 {
            return true;
        }
        match c.script {
            Some(src) => s.script_anim(src, c.skin.hashcode).is_none(),
            None => !opts.reinforcements && match s.char_drone.get(ci).copied().flatten() {
                Some(di) => { let dr = &s.drones.list[di]; !dr.present || !dr.enabled }
                None => matches!(c.role, drone::SpawnRole::Reinforcement { .. }),
            },
        }
    }

    pub fn facial_weights_now(&self, c: &Character) -> Option<Vec<f32>> {
        self.facial_weights_at(self.characters.iter().position(|cc| std::ptr::eq(cc, c))?, c)
    }

    /// `facial_weights_now` with the character's index known (the drone link is `Sim::char_drone[ci]`).
    pub fn facial_weights_at(&self, ci: usize, c: &Character) -> Option<Vec<f32>> {
        let s = self.sim.as_ref()?;
        let count = c.skin.facial as usize;
        if count == 0 { return None; }
        // fix-facial3: drone-backed characters sample their own playing script's face window — the
        // same source the root-motion path reads (`Drone::script_frame`, the drone's live
        // `anim.script`/`ftime`); the script-object path keeps its live `Obj::Anim` frame.
        // level-07-guest-faces: a drone whose body anim carries no facial window contributes NO
        // base window — the blend still runs over a zero base (the game's gameplay base is the
        // idle marker script's +0x54 buffer, `AnimFacialBlend` 0x1be648 reads only the three
        // +0x80 marker scripts in gameplay — the body script's marker is 0 [M AnimScriptInit
        // 0x1c1600:58-61]); the guest's idle anim (07000007 census) carries no face window, so the
        // zero base is what lets the facial slots animate the face at all.
        let (script_h, frame) = match c.script {
            Some(src) => { let (h, f) = s.script_anim(src, c.skin.hashcode)?; (Some(h), f) }
            None => {
                let di = (*s.char_drone.get(ci)?)?;
                let dr = &s.drones.list[di];
                (dr.anim.script, 1.0 + dr.anim.ftime)
            }
        };
        let mut out = match script_h.and_then(|h| s.scripts.get(&h)) {
            Some(script) => {
                // cutscene-faces: the face window is the game's own classifier — the SEQUENCE's
                // flags bit 0x8 [M AnimProcessScriptCmds 0x1c1b38: `*(int *)(lVar6 + 0xc) >> 3 & 1`
                // on the raw sequence record; a facial window plays in the script's +0x54 slot via
                // AnimSeqSet 0x1c7460's param_7 slot pick], the same rule `script_pose_together`
                // poses bones by. The old `channels.len() == count` probe was fix-facial2's
                // stand-in; the two agree on EVERY sequence on the disc — the census: 18,244
                // kind-4 sequences, flags {0x00 × 17363, 0x01 × 10, 0x08 × 871} and every
                // flags-0x08 one has exactly 22 channels (a body window is 3·bones +
                // 3·popcount(mask) — always ≡ 0 mod 3, so 22 can never be one). The game reads the
                // slot buffer as `skin.facial` floats whatever the track carries
                // [M AnimFacialBlend 0x1be648's uVar9 loop], and `facial_weights` keeps that count
                // guard.
                let win = script.sequences().filter(|(a, _, _)| (*a as f32) <= frame)
                    .filter_map(|(a, _, q)| s.seqs.get(&q).filter(|sq| sq.flags & 0x8 != 0).map(|sq| (a, q)))
                    .last();
                match win {
                    Some((wstart, q)) => {
                        let seq = s.seqs.get(&q)?;
                        nf_game::anim::facial_weights(seq, (frame - wstart as f32 + 1.0).max(1.0), count)?
                    }
                    // no face window on this script: a script actor keeps the cutscene-faces
                    // model (None — no morph term; the game reads its never-started facial
                    // slot's zeroed buffer [M AnimSeqNew 0x1c7248], visually the same neutral).
                    // A DRONE falls through to the zero base — the facial slots (the talk
                    // marker) still drive the face there (see below; the 07000007 guests'
                    // idle anims carry no face window — the frozen-face bug).
                    None if c.script.is_some() => return None,
                    None => vec![0.0; count],
                }
            }
            None if c.script.is_some() => return None,
            None => vec![0.0; count],
        };
        // fix-facial3: `AnimFacialBlend` (0x1be648) — the drone's facial script slots over the track
        // weights: slot 0 (the idle face) copies straight, slot 1 (the blink) overrides |w| > 0.01.
        // The drone's script-object sibling has no slots (the game's script objects get theirs from
        // the same driver — the difference is [S] unverified (kept out of pass/fail; the tracks
        // dominate there — AnimFacialBlend 0x1be648's per-object slot source not traced).
        // level-07-guest-faces: slot 2 — the TALK marker 0xe0000002 script `NDrone2_SpeechSFX`
        // 0x141b00 attached through `AnimFacialAppend` 0x1c6490 (`Drone.facial_talk`/`_start`, the
        // port's +0x53c) — now feeds the blend too: it overrides |w| > 0.01 over the base, the
        // game's gameplay talk layer [M 0x1be648's lVar6 arm; the marker set at 0x1c6490 (+0x80 =
        // 0xe0000002)]. Slot time is the talk clock the op-1 sounds already run on (`now_t = frame
        // − start + 1`, 1 on the attach tick — drone.rs's op-1 gate [M speech.md §5]); the
        // window's frame is that minus the window's first frame.
        if c.script.is_none() {
            if let Some(di) = s.char_drone.get(ci).copied().flatten() {
                let dr = &s.drones.list[di];
                let now = s.drones.frame; // the clock `facial_talk_start` is stamped with (`let frame = self.frame`, Drones::tick_inner)
                let slot_w = |slot: Option<(u32, u32)>| -> Option<Vec<f32>> {
                    let (h, start) = slot?;
                    if (now as i64) < start as i64 { return None; }
                    let fscript = s.scripts.get(&h)?;
                    let (first, _, q) = fscript.sequences().next()?;
                    let fseq = s.seqs.get(&q)?;
                    nf_game::anim::facial_weights(fseq, (now - start) as f32 - first as f32 + 1.0, count)
                };
                let idle = slot_w(dr.facial_idle);
                let blink = slot_w(dr.facial_blink);
                // the talk slot runs the op-1 talk clock: script time now_t = frame − start + 1
                // (1 on the attach tick), and the window's frame = now_t − first + 1 — the same
                // rule the body-window path applies (`(frame − wstart + 1)` above; `facial_weights`
                // takes the frame directly).
                let talk = dr.facial_talk.and_then(|h| {
                    let start = dr.facial_talk_start;
                    if (now as i64) < start as i64 { return None; }
                    let fscript = s.scripts.get(&h)?;
                    let (first, _, q) = fscript.sequences().next()?;
                    let fseq = s.seqs.get(&q)?;
                    nf_game::anim::facial_weights(fseq, (now - start) as f32 - first as f32 + 2.0, count)
                });
                if idle.is_some() || blink.is_some() || talk.is_some() {
                    nf_game::anim::facial_blend(&mut out, idle.as_deref(), blink.as_deref(), talk.as_deref(), [0, 0], false);
                }
            }
        }
        Some(out)
    }

    pub fn animate(&self, choice: Option<usize>, ticks: f32, opts: ViewOpts) -> Vec<[f32; 3]> {
        let FrameState { posed, attach, hidden } = self.frame_state(choice, ticks, opts);
        let hidden = |_: &Character, ci: usize| hidden[ci];
        // fix-facial2: the morph weights per morph-bearing character right now — the face-track
        // window sampled at the object's frame (`SkinIt` 0x10bf30 consumes them per vertex).
        // fix-facial3's CPU-only gate is LIFTED here (its item 3): the GPU half landed with
        // fix-gpu-facial-morphs — the shader adds the same term from the same weight source, so
        // including the drone-backed characters keeps the two paths identical (the gpu_palette
        // oracle 0.0033 disagreement the gate was avoiding is exactly the term both now apply).
        let facial: Vec<Option<Vec<f32>>> = self.characters.iter().enumerate().map(|(ci, c)| {
            if c.morph_targets.is_some() { self.facial_weights_at(ci, c) } else { None }
        }).collect();
        self.char_verts.iter().map(|v| {
            // Searchlight parts: the simulation's matrices (`Searchlight_Update`), coronas as camera billboards
            // with their scale (the alpha byte is not wired to the shader — G-033); hidden parts collapse below
            // the world like the game's `View_SetDrawInNoViews`.
            if let Some(sl) = v.searchlight {
                const FAR: Mat4 = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
                let Some(s) = self.sim.as_ref() else { return [0.0, -10000.0, 0.0] };
                let n_parts = s.drones.searchlights.parts.len();
                if sl as usize >= n_parts {
                    let sp = self.sl_lod.get(sl as usize - n_parts).and_then(|&h| s.drones.searchlights.parts.get(h as usize));
                    return match sp { Some(sp) if sp.drawn && sp.lod_next => transform(&sp.matrix, v.pos), _ => transform(&FAR, v.pos) };
                }
                let Some(p) = s.drones.searchlights.parts.get(sl as usize) else { return [0.0, -10000.0, 0.0] };
                if !p.drawn || (p.lod_next && self.sl_lod.contains(&sl)) { return transform(&FAR, v.pos); }
                if p.kind == 4 {
                    let m: Mat4 = [
                        [opts.right[0] * p.scale, opts.right[1] * p.scale, opts.right[2] * p.scale, 0.0],
                        [opts.up[0] * p.scale, opts.up[1] * p.scale, opts.up[2] * p.scale, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [p.pos[0], p.pos[1], p.pos[2], 1.0],
                    ];
                    return transform(&m, v.pos);
                }
                return transform(&p.matrix, v.pos);
            }
            // Level-placed pickups: the pickup's own matrix (`obj+0x90`), collapsed once taken or hidden
            // (`View_DrawObjectGlist` 0x1e6780 draws `obj+0xD8` at `obj+0x90`; fix-player2).
            if let Some(pk) = v.pickup {
                let Some(s) = self.sim.as_ref() else { return [0.0, -10000.0, 0.0]; };
                let Some(p) = s.drones.pickups.get(pk as usize).filter(|p| !p.gone && !p.hidden) else { return [0.0, -10000.0, 0.0]; };
                return transform(&p.m, v.pos);
            }
            // Doors: the simulation's pose (`Door_Interp`'s matrix at `obj+0x90`, docs/systems/world-objects.md).
            // Without the drone context (the sequence browser) there is no door state: hidden below the world,
            // like the searchlight parts.
            if let Some(dr) = v.door {
                let Some(s) = self.sim.as_ref() else { return [0.0, -10000.0, 0.0]; };
                // The door slot space also holds the SimpleScripts (map-tower2): indices past the doors
                // list index `drones.sss.list`.
                if let Some(d) = s.drones.doors.list.get(dr as usize) { return transform(&d.matrix, v.pos); }
                if let Some(q) = s.drones.sss.list.get(dr as usize - s.drones.doors.list.len()) { return transform(&q.matrix, v.pos); }
                // Past the SSs: the breakables (`Breaks::create` order) — gone once `Break_Kill` 0x12b970 ran.
                let bi = dr as usize - s.drones.doors.list.len() - s.drones.sss.list.len();
                if let Some(b) = s.drones.breaks.list.get(bi) { if b.dead { return [0.0, -10000.0, 0.0]; } return transform(&b.matrix, v.pos); }
                // Past the breakables: the locks' placed object / LOD variant pairs.
                let Some(m) = self.lock_slot(s, dr as usize) else { return [0.0, -10000.0, 0.0]; };
                return transform(&m, v.pos);
            }
            if let Some(a) = v.attachment { return transform(&attach[a as usize], v.pos); }
            let c = &self.characters[v.character as usize];
            if hidden(c, v.character as usize) { return [c.matrix[3][0], c.matrix[3][1], c.matrix[3][2]]; }
            // fix-facial2: the morph term (object space, before the bone skinning — the game's
            // `SkinIt` 0x10bf30 adds Σ weight[m] × target(base + (morph + m) × 12) to the source
            // position while it streams through the scratchpad; the vertex's morph index is the
            // skinning word's upper half-word, −1 = none).
            let pos = match (facial.get(v.character as usize).and_then(|w| w.as_deref()), &c.morph_targets, v.morph) {
                (Some(w), Some(t), mo) if mo >= 0 => morph_vertex(v.pos, mo, w, t),
                _ => v.pos,
            };
            match &posed[v.character as usize] {
                Some((mats, _, m)) => transform(m, anim::skin_vertex(pos, v.weight, v.slots, &c.slot_bones, mats)),
                None => transform(&c.matrix, pos),
            }
        }).collect()
    }

    /// The Character-layer vertex tail posed and colour-multiplied: `animate` gives the positions, and the rgba
    /// gets the per-object `TAmbient` multiplier of `View_SetupRenderModes_Tweak` (0x1e5888) — the dim every
    /// `obj+0xF0 & 0x800` object carries in the game. fix-render2: characters and view models keep the
    /// per-character ramp; placed pickups, dropped guns and 0x800-glist script entities (the truck) dim by the
    /// cel ambient at their live matrix; searchlight/door parts, the additive attachments (muzzle flash,
    /// tracers) and non-0x800 entities stay at 1.0 (doors carry no 0x800 glist flag on 07000005 [D — flags 0x0],
    /// so the game draws them untinted).
    /// The cel's frustum-test sphere (the bbox centre + half-diagonal): the shape `Vision_InView`
    /// guards in `View_AddCels` (0x1e6ba0) — the game tests each cel's bbox per viewer.
    pub fn cel_sphere(&self, i: usize) -> Option<([f32; 3], f32)> {
        let c = self.cels.get(i)?;
        let centre = [0, 1, 2].map(|k| (c.min[k] + c.max[k]) * 0.5);
        let r = [0, 1, 2].map(|k| (c.max[k] - c.min[k]) * 0.5);
        Some((centre, (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt()))
    }
    /// `build_FindCel` (0x1c8cb0) over the Scene's room cels — the same lookup the ambient code and
    /// the World-draw attribution use.
    pub fn cel_index_at(&self, p: [f32; 3]) -> Option<usize> { find_cel(&self.cels, p) }

    /// fix-impacts-render: the attachment index that draws impact-effect slot `(kind, slot, mesh)` —
    /// kind 0 decal, 1 casing, 2 debris (`Scene::build`'s per-mesh pools; `frame_state` poses it from
    /// the sim each frame). Tests and the dev panel read the pools through this.
    pub fn effect_attachment(&self, kind: u8, slot: usize, mesh: u32) -> Option<u32> {
        self.attachments.iter().position(|a| a.effect == Some((kind, slot, mesh))).map(|i| i as u32)
    }

    /// The GPU skinning layout (G-043): each Character-layer vertex's two palette matrices, blend weight and
    /// colour slot, plus the palette and colour-table sizes. The palette holds, in order: per character its
    /// object matrix then one matrix per skeleton bone (skin · object), then the attachments, searchlight
    /// parts, doors and placed pickups. The colour table: characters, attachments, pickups, then 1.0. The
    /// shader computes `w·(P[a]·p) + (1−w)·(P[b]·p)`, which equals the CPU path's `transform(m, skin_vertex(..))`
    /// because the blend weights sum to 1.
    pub fn skin_layout(&self) -> (SkinLayout, Vec<SkinAttr>) {
        let mut char_base = Vec::with_capacity(self.characters.len());
        let mut n = 0u32;
        for c in &self.characters { char_base.push(n); n += 1 + c.skeleton.bones as u32; }
        // fix-gpu-facial-morphs: the morph layout — per-character weight blocks (a fixed stride =
        // the widest facial count in vec4s) and the concatenated target element ranges in
        // character order (`morph_target_blob`'s layout). A scene with no morph bearer gets
        // stride 0 and every attr's mtb −1 (the shader's morph term is skipped).
        let wt_stride = self.characters.iter().map(|c| c.skin.facial as usize).max()
            .map_or(0, |m| m.div_ceil(4)) as u32;
        let mut n_tgt = 0u32;
        let mut tgt_base = Vec::with_capacity(self.characters.len());
        for c in &self.characters {
            tgt_base.push(n_tgt);
            n_tgt += c.morph_targets.as_ref().map_or(0, |t| (t.len() / 4) as u32);
        }
        let max_of = |f: &dyn Fn(&CharVertex) -> Option<u32>| self.char_verts.iter().filter_map(|v| f(v)).max().map_or(0, |m| m + 1);
        let n_sl = max_of(&|v| v.searchlight.map(u32::from));
        let n_door = max_of(&|v| v.door.map(u32::from));
        let n_pick = max_of(&|v| v.pickup.map(u32::from));
        let n_att = self.attachments.len() as u32;
        let attach_base = n;
        let sl_base = attach_base + n_att;
        let door_base = sl_base + n_sl;
        let pick_base = door_base + n_door;
        let n_mats = pick_base + n_pick;
        let nc = self.characters.len() as u32;
        let (col_att, col_pick) = (nc, nc + n_att);
        // fix-searchlight-remainder: one colour slot per searchlight part, so the corona's alpha byte
        // (obj+0x106, `(dot−0.2)·4·255` low byte [M 0x1cb900:334-336]) reaches the shader — the game's
        // draw reads obj+0x106 as the quad's alpha. All other parts keep alpha 1.0; the slots sit
        // before the shared 1.0 entry (`col_one`).
        let col_sl = col_pick + n_pick;
        let col_one = col_sl + n_sl;
        let attrs = self.char_verts.iter().map(|v| {
            let one = |m: u32, col: u32| SkinAttr { a: m, b: m, w: 1.0, col, mtb: -1, mwb: 0, cnt: 0, tend: 0 };
            if let Some(sl) = v.searchlight { return one(sl_base + sl as u32, col_sl + sl as u32); }
            if let Some(pk) = v.pickup { return one(pick_base + pk as u32, col_pick + pk as u32); }
            if let Some(dr) = v.door { return one(door_base + dr as u32, col_one); }
            if let Some(a) = v.attachment { return one(attach_base + a, col_att + a); }
            let c = &self.characters[v.character as usize];
            let base = char_base[v.character as usize];
            let bone = |s: u16| c.slot_bones.get(s as usize).filter(|&&b| (b as u32) < c.skeleton.bones as u32).map(|&b| base + 1 + b as u32);
            // fix-gpu-facial-morphs: the character branch also carries the morph plumbing —
            // `SkinIt` 0x10bf30:90 gates on the vertex's morph index (−1 = none) and the object's
            // Morph_Count; the target block is this character's [tgt_base, tgt_base + len). The
            // single-bone/no-bone fallbacks keep the same plumbing (the game's SkinIt morphs every
            // vertex carrying a morph index, before any skinning); `one()`'s mtb −1 stays only on
            // the non-character verts (searchlights, doors, pickups, attachments).
            let (mtb, mwb, cnt, tend) = {
                let tb = tgt_base[v.character as usize];
                let tlen = c.morph_targets.as_ref().map_or(0, |t| (t.len() / 4) as u32);
                let ok = tlen > 0 && c.skin.facial > 0 && v.morph >= 0;
                (if ok { (tb + (v.morph as u32) * 3) as i32 } else { -1 },
                 (v.character as u32) * wt_stride,
                 if ok { c.skin.facial as u32 } else { 0 },
                 tb + tlen)
            };
            let one_char = |m: u32| SkinAttr { a: m, b: m, w: 1.0, col: v.character, mtb, mwb, cnt, tend };
            match (bone(v.slots[0]), bone(v.slots[1])) {
                (Some(a), Some(b)) => SkinAttr { a, b, w: v.weight, col: v.character, mtb, mwb, cnt, tend },
                (Some(a), None) => one_char(a),
                _ => one_char(base),
            }
        }).collect();
        (SkinLayout { char_base, attach_base, sl_base, door_base, pick_base, n_mats, col_att, col_pick, col_sl, col_one, wt_stride, n_tgt }, attrs)
    }

    /// fix-gpu-facial-morphs: the morph targets of every character, concatenated in character order
    /// (the GPU target buffer's contents; `skin_layout`'s `n_tgt` element ranges index it). Static
    /// per scene — the MORPH_DATA blocks never change at runtime.
    pub fn morph_target_blob(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for c in &self.characters { out.extend_from_slice(c.morph_targets.as_deref().unwrap_or(&[])); }
        out
    }

    /// fix-gpu-facial-morphs: one frame's facial morph weights, laid out for the GPU weight buffer
    /// — per character a `SkinLayout::wt_stride`-vec4 block (`mwb`), `skin.facial` live floats
    /// then zeros (`AnimFacialBlend` 0x1be648's array at animobj+0xb0, which `AnimObjectDraw`
    /// 0x1c4490:147-152 marshals into `Morph_Weight` for `SkinIt`). Characters with no live
    /// window stay zero (the game's Morph_Count gate) — the shader adds nothing. The drone-backed
    /// characters are INCLUDED (the un-gate this fix lands with the GPU half — fix-facial3's
    /// item 3): `facial_weights_at` reads their own playing script's face window.
    pub fn skin_morph_weights(&self, l: &SkinLayout) -> Vec<f32> {
        let stride = l.wt_stride as usize * 4;
        if stride == 0 { return Vec::new(); }
        let mut out = vec![0.0f32; self.characters.len() * stride];
        for (ci, c) in self.characters.iter().enumerate() {
            if c.skin.facial == 0 { continue; }
            if let Some(w) = self.facial_weights_at(ci, c) {
                let n = w.len().min(stride);
                out[ci * stride..ci * stride + n].copy_from_slice(&w[..n]);
            }
        }
        out
    }

    /// The frame's resolved lights for the renderer: the sim's `LightList` with the kind-2
    /// positions resolved against their searchlight parts. Per draw the renderer picks the ≤2
    /// closest within the draw's bounding sphere — the game's
    /// `Lights_CalcClosestLights` 0x1cb2e0 / `psiLight_SetLights` 0x10e588 per object/cel.
    /// light-actors: per character, the light-pick centre when it is NOT the palette root slot —
    /// the script actors' drawn root (`Sim::characters_script_pos`, the same point the ambient ramp
    /// samples); None = the root slot's translation is right (drones, view models, props).
    pub fn char_light_centres(&self) -> Vec<Option<[f32; 3]>> {
        let Some(s) = self.sim.as_ref() else { return Vec::new() };
        self.characters.iter().enumerate()
            .map(|(ci, c)| c.script.and_then(|_| s.characters_script_pos(ci)))
            .collect()
    }

    pub fn frame_lights(&self) -> nf_game::light::ResolvedLights {
        let Some(s) = self.sim.as_ref() else { return Default::default() };
        let mut r = s.drones.lights.resolve(&|i: u32| s.drones.searchlights.parts.get(i as usize).map(|p| p.pos).unwrap_or([0.0; 3]));
        // A kind-2 record's psiLight node dies with its tracked spot part: the kill and break arms free
        // the node (`psiLight_Delete` [M Searchlight_Update 0x1cb900:107,432]) and the port marks that as
        // `part.light = None` — a tracked part without a node is an inactive light (the game's list no
        // longer holds it, so it can never be picked).
        for l in r.lights.iter_mut() {
            if let Some(t) = l.track {
                if let Some(p) = s.drones.searchlights.parts.get(t as usize) {
                    if p.light.is_none() { l.active = false; }
                }
            }
        }
        r
    }

    /// One frame's palette and colour table for `skin_layout` (the GPU replacement for `posed_vertices`).
    /// `vis`: the frame's room-level visibility mask (the portal walk's set, `Renderer::cel_visibility`)
    /// — the object half of the game's cull [M `View_AddObjects` 0x1e62d8 / `View_DrawObjList`
    /// 0x1e69a0: only the VISIBLE cels' object lists draw; each object's cel via `build_LinkToRoom`
    /// 0x1c9680 → `build_FindCel`]. A posed object whose room cel is not visible collapses its
    /// palette to FAR (off-picture — the port's established "not drawn" state, the same mechanism as
    /// `fs.hidden`); objects the cel pass cannot place draw (the game's unlinked objects draw from
    /// the forced list, `View_AddForcedObjects` 0x1e5fb8). `None` = the walk is off (`NF_NOPORTAL`).
    /// Render interpolation: the palette entries of the
    /// gadget beams (the laser/stunner/uv beam slots TRACER_SLOTS..+3). They stretch from the
    /// first-person watch to the hit, so the draw loop re-attaches them to the interpolated camera
    /// like the view model. Port-side presentation only; the sim is untouched.
    pub fn beam_entries(&self, l: &SkinLayout) -> Vec<usize> {
        let att = l.attach_base as usize;
        self.attachments.iter().enumerate()
            .filter(|(_, a)| a.tracer.is_some_and(|t| (TRACER_SLOTS..TRACER_SLOTS + 3).contains(&t)))
            .map(|(k, _)| att + k).collect()
    }


    pub fn skin_palette(&self, l: &SkinLayout, choice: Option<usize>, time: f32, opts: ViewOpts, vis: Option<&[bool]>) -> (Vec<Mat4>, Vec<[f32; 4]>) {
        const FAR: Mat4 = [[0.0; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
        let prof_fs = nf_game::prof::scope("pal: frame_state_for");
        let fs = self.frame_state_for(choice, time, opts, true);
        drop(prof_fs);
        let _prof_rest = nf_game::prof::scope("pal: palette fill");
        let collapse = |at: &Mat4| -> Mat4 { [[0.0; 4], [0.0; 4], [0.0; 4], [at[3][0], at[3][1], at[3][2], 1.0]] };
        // fix-dev-spawn-visible: a LIVE drone's character (present + enabled — the dev NPC spawns, and
        // the placed guards while they fight) must not vanish when the strict portal walk drops the
        // room he stands in. The measured case: the skin-row guard settles 4.4 m in front of the
        // 07000005 spawn into a room whose cell (31) is in NEITHER mask from the eye's cell (14) —
        // game 75/137, strict 5/137 — while the player looks straight at him; the game draws an
        // object there (View_AddCels' depth-0 frustum band covers 200 units, View_DrawObjList 0x1e69a0
        // draws the VisCelList cels' object lists, and the object is linked to the room build_FindCel
        // 0x1c8680 picks at its own position). The port's walk is the approximation that already
        // needed `near_los_rescue` for the World draws (fix-portal-sweep: "the near unoccluded
        // generics the walk drops where the port's per-cel attribution approximation disagrees");
        // for a live NPC the palette trusts the position + the GPU clip instead of the walk. The
        // hidden/present collapse below still removes dead and disabled ones.
        // Only the dev panel's pool copies (`npc_pool` chars): exempting EVERY live drone let the strict
        // walk's hidden rooms draw their partygoers (fix_portal_live's ballroom).
        let live_drone = |ci: usize| {
            self.npc_pool.iter().any(|s| s.chars.contains(&ci)) && self.sim.as_ref()
                .and_then(|s| s.char_drone.get(ci).copied().flatten())
                .and_then(|di| self.sim.as_ref().map(|s| (s, di)))
                .map_or(false, |(s, di)| s.drones.list[di].present && s.drones.list[di].enabled)
        };
        let vis_room = |pos: [f32; 3]| -> bool {
            vis.map_or(true, |v| self.cel_index_at(pos).map_or(true, |r| v.get(r).copied().unwrap_or(true)))
        };
        let mut p = vec![FAR; l.n_mats as usize];
        let pch = nf_game::prof::scope("pal: chars+attach+objects");
        for (ci, c) in self.characters.iter().enumerate() {
            // A character added after the layout was built (the dev spawn) has no palette slots
            // until the renderer is rebuilt.
            let Some(&base) = l.char_base.get(ci) else { break };
            let base = base as usize;
            let nb = c.skeleton.bones as usize;
            if fs.hidden[ci] { for k in 0..=nb { p[base + k] = collapse(&c.matrix); } continue; }
            let cpos = fs.posed[ci].as_ref().map_or(c.matrix[3], |(_, _, m)| m[3]);
            // The first-person view model is never room-culled: the game draws it in its own list-2
            // pass after the world, not through any cel's object list [M Game_Draw: the view model's
            // list-2 draw after psiLight_SetLights(0); View_SetDrawInNoViews is its only hide]. The
            // room test here sent the hands/gun to FAR whenever the point in front of the camera fell
            // in a cel the walk had not marked (doorways, room edges), so the hands and gun
            // vanished.
            let view_model = self.view_models.values().any(|&v| v == ci);
            if !live_drone(ci) && !view_model && !vis_room([cpos[0], cpos[1], cpos[2]]) { for k in 0..=nb { p[base + k] = FAR; } continue; }
            match &fs.posed[ci] {
                Some((mats, _, m)) => {
                    p[base] = *m;
                    for b in 0..nb { p[base + 1 + b] = mats.get(b).map_or(*m, |s| anim::mul(s, m)); }
                }
                None => for k in 0..=nb { p[base + k] = c.matrix; },
            }
        }
        for (a, m) in fs.attach.iter().enumerate() { p[l.attach_base as usize + a] = *m; }
        if let Some(s) = self.sim.as_ref() {
            for k in 0..(l.door_base - l.sl_base) as usize {
                let keep = match s.drones.searchlights.parts.get(k) {
                    Some(sp) => vis_room(sp.pos),
                    None => true,
                };
                if !keep { p[l.sl_base as usize + k] = FAR; continue; }
                let n_parts = s.drones.searchlights.parts.len();
                if k >= n_parts {
                    // A LOD slot: the shadowed part's matrix once its head has swapped.
                    let sp = self.sl_lod.get(k - n_parts).and_then(|&h| s.drones.searchlights.parts.get(h as usize));
                    p[l.sl_base as usize + k] = match sp { Some(sp) if sp.drawn && sp.lod_next => sp.matrix, _ => FAR };
                    continue;
                }
                p[l.sl_base as usize + k] = match s.drones.searchlights.parts.get(k) {
                    Some(sp) if !sp.drawn => FAR,
                    Some(sp) if sp.lod_next && self.sl_lod.contains(&(k as u16)) => FAR,
                    Some(sp) if sp.kind == 4 => [
                        [opts.right[0] * sp.scale, opts.right[1] * sp.scale, opts.right[2] * sp.scale, 0.0],
                        [opts.up[0] * sp.scale, opts.up[1] * sp.scale, opts.up[2] * sp.scale, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [sp.pos[0], sp.pos[1], sp.pos[2], 1.0],
                    ],
                    Some(sp) => sp.matrix,
                    None => FAR,
                };
            }
            let scene_cam = s.script_camera().is_some();
            for k in 0..(l.pick_base - l.door_base) as usize {
                // cutscene-door-dup: while the script camera renders (the scene's camera 4, object_display_mask
                // 0x10), an object whose glist flags carry 0x40000 is invisible [M Control_SetGList 0x134960
                // clears obj+0xF8's bit 0x10 for it; View_AddObjects 0x1e62d8:33 / View_AddForcedObjects
                // 0x1e5fb8:79 gate first on obj+0xF8 & object_display_mask]. Collapse the placed door's slot
                // (the port's FAR not-drawn state) so the static closed leaf stops drawing behind the scene's
                // own leaf movers (07000014's control-room duplicate).
                if scene_cam && s.drones.doors.list.get(k).is_some_and(|d| self.door_scene_hidden.contains(&d.placement)) { p[l.door_base as usize + k] = FAR; continue; }
                // The door region also holds the SimpleScripts' matrices (map-tower2): the ss vertices
                // index `n_doors + k`, and `pick_base − door_base` = max door-or-ss index + 1.
                // After the SS slots: the breakables (`Breaks::create` order), alive → placement matrix.
                let brk = |k: usize| k.checked_sub(s.drones.doors.list.len() + s.drones.sss.list.len())
                    .and_then(|bi| s.drones.breaks.list.get(bi));
                let dpos = s.drones.doors.list.get(k).map(|d| d.matrix[3])
                    .or_else(|| s.drones.sss.list.get(k - s.drones.doors.list.len()).map(|q| q.matrix[3]))
                    .or_else(|| brk(k).map(|b| b.matrix[3]));
                // A door linked to a portal (`build_LinkDoors2Portals` 0x1c8ac8, `obj+0xF0 |= 0x8000`) straddles the
                // two cels its portal joins (`Collide_StraddleCels` 0x1eac98) and draws while EITHER is visible — the
                // shut door is what you see from the near side once its portal stops the walk (render, 07000001: the
                // shoji vanished and the sky showed through the doorway when only its centre's cel counted).
                let linked: Vec<usize> = self.cel_graph.portal_doors.iter().enumerate()
                    .filter(|(_, d)| **d == Some(k as u32))
                    .filter_map(|(e, _)| self.cel_graph.portals.get(e))
                    .flat_map(|pe| [pe.cel_a, pe.cel_b]).collect();
                let straddle_seen = !linked.is_empty() && vis.is_some_and(|v| self.cel_graph.room_to_cell.iter().enumerate()
                    .any(|(r, c)| c.is_some_and(|c| linked.contains(&c)) && v.get(r).copied().unwrap_or(false)));
                if !straddle_seen && dpos.is_some_and(|p3| !vis_room([p3[0], p3[1], p3[2]])) { p[l.door_base as usize + k] = FAR; continue; }
                p[l.door_base as usize + k] = match s.drones.doors.list.get(k) {
                    Some(d) => d.matrix,
                    None => match s.drones.sss.list.get(k - s.drones.doors.list.len()) {
                        Some(q) => q.matrix,
                        // `Break_Kill` 0x12b970: obj+0xFE bit 0 → the delete pass removes the object [M].
                        None => match brk(k) {
                            Some(b) => if b.dead { FAR } else { b.matrix },
                            None => self.lock_slot(s, k).unwrap_or(FAR),
                        },
                    },
                };
            }
            for k in 0..(l.n_mats - l.pick_base) as usize {
                let pk = s.drones.pickups.get(k).filter(|q| !q.gone && !q.hidden);
                if pk.is_some_and(|q| !vis_room([q.m[3][0], q.m[3][1], q.m[3][2]])) { continue; } // stays FAR
                p[l.pick_base as usize + k] = pk.map_or(FAR, |q| q.m);
            }
        }
        drop(pch);
        let pcm = nf_game::prof::scope("pal: colour_mults");
        let (mults, pickup_mult, attach_mult) = self.colour_mults_dense();
        drop(pcm);
        let mut cols = vec![[1.0f32; 4]; l.col_one as usize + 1];
        let has = !mults.is_empty();
        let rgb = |m: [f32; 3]| [m[0], m[1], m[2], 1.0];
        // fix-searchlight-remainder: the per-part alpha slot — kind-4 coronas carry obj+0x106/255 in
        // .a (the shader's ×255/128 rescale makes byte 0x2e → 0.36 exactly like the GS); the other
        // parts stay 1.0. The byte is the sim's (`Searchlight_Update` 0x1cb900:334-336).
        if let Some(s) = self.sim.as_ref() {
            for (k, sp) in s.drones.searchlights.parts.iter().enumerate() {
                if let Some(c) = cols.get_mut(l.col_sl as usize + k) {
                    c[3] = if sp.kind == 4 { sp.alpha as f32 / 255.0 } else { 1.0 };
                }
            }
        }
        if has {
            // Each run writes only its own section of the layout. Pickups created after the build (a
            // dead guard's dropped gun) have no draw slot here (the drop draws as its attachment);
            // writing them past `col_pick + n_pick` overwrote the searchlight slots and the shared
            // `col_one` entry every door and interactable draws with, dimming them all to the
            // drop's cel ambient.
            for (ci, m) in mults.iter().enumerate().take(l.col_att as usize) { cols[ci] = rgb(*m); }
            for (a, m) in attach_mult.iter().enumerate().take((l.col_pick - l.col_att) as usize) { cols[(l.col_att + a as u32) as usize] = rgb(*m); }
            for (pk, m) in pickup_mult.iter().enumerate().take((l.col_sl - l.col_pick) as usize) { cols[(l.col_pick + pk as u32) as usize] = rgb(*m); }
        }
        // blood-puff-render: the gas puffs' draw colour IS the pool's live rgba — the game's
        // `View_SetupRenderModes_Tweak` 0x1e5888:46-49 feeds the gas object's obj+0x103..0x106 bytes
        // into `psiSetTweakARGB` (0x124770), whose TAmbient/DAT_002650d4.. floats (= byte × 1/255)
        // multiply the drawn colour AND the alpha; `gases_smoke` 0x1ab2d0's tail rewrites those bytes
        // from the gas datum's colour every tick (the 255→0 fade, `Gas_Init` 0x1aacc8:56-59 seeds
        // them from the colour arg). The GPU palette carried only the ambient dim here, so the GPU
        // render — the one that draws — never faded the puff (the CPU `posed_vertices`' gas_rgba
        // did). The shader multiplies c0.rgb/α by cols.rgb/α, which is exactly the game's tweak.
        // Free slots keep the ambient value: their matrix is FAR, nothing draws.
        if let Some(s) = self.sim.as_ref() {
            for (a, att) in self.attachments.iter().enumerate() {
                let Some((gi, gm)) = att.gas else { continue };
                let Some(c) = cols.get_mut((l.col_att + a as u32) as usize) else { continue };
                if let Some(g) = s.drones.gas.puffs.get(gi as usize).filter(|g| g.mesh == gm) {
                    *c = [g.rgba[0] / 255.0, g.rgba[1] / 255.0, g.rgba[2] / 255.0, g.rgba[3] / 255.0];
                }
            }
        }
        // rocket-swoosh: the segment's alpha byte `obj+0x106 = life·128/max` [M Swoosh_Update 0x18d200:49-51],
        // the Tweak's alpha factor (byte/255) [M View_SetupRenderModes_Tweak 0x1e5888:47-49].
        if let Some(s) = self.sim.as_ref() {
            for (a, att) in self.attachments.iter().enumerate() {
                let Some((slot, mesh)) = att.projectile else { continue };
                if slot < SWOOSH_SLOT_BASE { continue; }
                let k = (slot - SWOOSH_SLOT_BASE) as usize;
                let Some(sw) = s.drones.swooshes.iter().flatten().filter(|w| w.mesh == mesh).nth(k) else { continue };
                if let Some(c) = cols.get_mut((l.col_att + a as u32) as usize) { c[3] = (sw.life as f32 * 128.0 / sw.max.max(1) as f32) / 255.0; }
            }
        }
        // Bond's muzzle flash alpha: `Draw_MuzzleFlash` writes the object's alpha byte (`obj+0x106`) on
        // every drawn tick as `wd[+0x51] + Rand_FRand(wd[+0x52] − wd[+0x51])` [M 0x1a3fe0:141-144], which
        // `View_SetupRenderModes_Tweak` 0x1e5888:47-49 hands to `psiSetTweakARGB` as the draw's alpha
        // factor (byte/255) — the launcher's 128..255, the P2K's 64..192 [L weapon_data]. The port drew
        // every flash at alpha 1: a solid shape where the game's flash is a flickering, see-through one.
        // (A render-side random like the roll/scale in `player_flash_matrix`, not the sim's RNG.)
        if let Some(s) = self.sim.as_ref() {
            if let (Some(&vm), Some(f)) = (self.view_models.get(&s.gun.weapon), s.last_shot) {
                if let Some(r) = s.weapons.records.get(s.gun.weapon as usize) {
                    let (lo, hi) = (r[0x51] as f32, r[0x52] as f32);
                    let mut z = (s.drones.frame ^ f.wrapping_mul(0x9e37_79b9)).wrapping_mul(2_654_435_761).wrapping_add(0x6d2b_79f5);
                    z ^= z >> 15; z = z.wrapping_mul(0x2c1b_3c6d); z ^= z >> 12;
                    let alpha = (lo + (z >> 8) as f32 / (1u32 << 24) as f32 * (hi - lo).max(0.0)) / 255.0;
                    for (a, att) in self.attachments.iter().enumerate() {
                        if att.flash.is_none() || att.character as usize != vm { continue; }
                        if let Some(c) = cols.get_mut((l.col_att + a as u32) as usize) { c[3] = alpha; }
                    }
                }
            }
        }
        // frame-cost-guard: the per-attachment fallback this comment used to describe ("attachments
        // without an entry fall back to their owner's multiplier") was dead code. `colour_mults`
        // inserts EVERY index `0..attachments.len()` into `attach_mult` whenever the sim is Some
        // (scene.rs:6863's `for ai in 0..self.attachments.len()`), and this loop only ran under
        // `has` — which is `!mults.is_empty()`, implying the sim is Some. So `contains_key` was
        // always true and the body never executed; on 07000005 it still cost 8561 hash lookups
        // per ticked frame. `colour_attach_map_covers_every_slot_on_the_lag_levels` pins the
        // invariant (every slot present), so a change that would make the fallback reachable
        // fails a test instead of silently changing the palette's colours.
        (p, cols)
    }

    /// The per-object colour multipliers of one frame (`View_SetupRenderModes_Tweak`'s TAmbient): per character,
    /// per placed pickup and per attachment. Shared by the CPU path (`posed_vertices`) and the GPU palette.
    pub fn colour_mults(&self) -> (Vec<[f32; 3]>, std::collections::HashMap<u16, [f32; 3]>, std::collections::HashMap<u32, [f32; 3]>) {
        let (mults, pickups, attach) = self.colour_mults_dense();
        let pickup_mult = pickups.into_iter().enumerate().map(|(k, m)| (k as u16, m)).collect();
        let attach_mult = attach.into_iter().enumerate().map(|(k, m)| (k as u32, m)).collect();
        (mults, pickup_mult, attach_mult)
    }

    /// `colour_mults` with the pickup / attachment multipliers as dense vectors (index = the
    /// pickup / attachment slot; both empty without a sim) — the same values. mp-perf-16: the GPU
    /// palette reads them by index; hashing every one of the 16-body arena's attachment slots into
    /// a map each ticked frame was a millisecond of the 240 Hz budget.
    pub fn colour_mults_dense(&self) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 3]>) {
        let pm = nf_game::prof::scope("cm: char mults");
        let mults: Vec<[f32; 3]> = self.sim.as_ref()
            .map(|s| (0..self.characters.len()).map(|ci| s.char_multiplier(ci)).collect())
            .unwrap_or_default();
        // fix-render3 (the 07000006 frame jitter): fix-render2 computed the pickup / dropped-gun /
        // script-entity ambient multipliers INSIDE the per-vertex closure — every vertex of a dimmed
        // character re-ran `find_cel` (a linear cel scan) and, for drops, a linear `pickups` scan.
        // The multiplier depends only on the pickup / attachment key, so it is computed once per
        // distinct key per frame and the vertex loop does a hash hit.
        let one = || [1.0; 3];
        let amb_at = |s: &Sim, m4: &Mat4| {
            s.ambient_at([m4[3][0], m4[3][1], m4[3][2]]).map(mul_bytes).unwrap_or_else(one)
        };
        let tintable = |p: &nf_game::pickup::Pickup| p.kind != nf_game::pickup::kind::ARMOUR && p.kind != nf_game::pickup::kind::BOND_BONUS;
        // Preallocated to the loop bounds (frame-cost-guard): one entry per pickup and per
        // attachment slot is inserted unconditionally below, so the maps can be sized up front
        // instead of rehash-growing to 12k+ entries every ticked frame.
        let mut pickup_mult: Vec<[f32; 3]> = Vec::with_capacity(self.sim.as_ref().map_or(0, |s| s.drones.pickups.len()));
        let mut attach_mult: Vec<[f32; 3]> = Vec::with_capacity(self.attachments.len());
        // The lag fix: the keys were collected by walking EVERY character
        // vertex — the effect pools (decal/casing/debris/gas/projectile slots, 8561 attachments on 07000005)
        // made that 34 ms of a 52 ms frame. The multiplier depends only on the key, so it is computed once per
        // pickup and per attachment; keys no vertex carries are never read (the palette's colour slot and
        // `posed_vertices`' lookup are both per-vertex), so the results the vertices see are unchanged.
        drop(pm);
        let pp = nf_game::prof::scope("cm: pickups");
        if let Some(s) = self.sim.as_ref() {
            for pk in 0..s.drones.pickups.len().min(u16::MAX as usize + 1) as u32 {
                let pk = pk as u16;
                {
                    pickup_mult.push((|| {
                        match s.drones.pickups.get(pk as usize) {
                            Some(p) if tintable(p) => amb_at(s, &p.m),
                            // Armour glow (fix-player3): kind 3 carries the 0x80000 "close-up flash" flag
                            // (`Pickup_Create` 0x18dda0:145-151) — `View_SetupRenderModes_Tweak`
                            // (0x1e5888:22-44) multiplies its tint by `FlashMod` while the viewer's camera
                            // (viewer+0xC0) is within 4.0 units of the pickup's world-sphere centre
                            // (obj+0x80, squared distance < 16.0) and the options byte DAT_002a38d4 is set
                            // (1 at level start [L]). The port's camera is Bond's eye (first person, as in
                            // play); no corona, no additive pass — the pulse rides the same vertex-colour
                            // multiply as the ambient dim.
                            Some(p) if p.kind == nf_game::pickup::kind::ARMOUR => {
                                let c = p.centre();
                                let e = s.bond.head();
                                let d2 = (c[0] - e[0]).powi(2) + (c[1] - e[1]).powi(2) + (c[2] - e[2]).powi(2);
                                if s.drones.near_flash && d2 < 16.0 { [flash_mod(s.drones.frame); 3] } else { one() }
                            }
                            _ => one(),
                        }
                    })());
                }
            }
            drop(pp);
            let _pa = nf_game::prof::scope("cm: attachments");
            for ai in 0..self.attachments.len() as u32 {
                {
                    attach_mult.push((|| {
                        let Some(a) = self.attachments.get(ai as usize) else { return one() };
                        if a.flash.is_some() || a.tracer.is_some() { return one(); }
                        // rocket-swoosh: a swoosh's tint bytes stay at control_init_object's 0xFF
                        // [M 0x134630:18-21; Swoosh_Create/Update never write obj+0x103..0x105], so its
                        // tweak RGB is 1.0 — no cel-ambient dim (the dim made the grey ribbon vanish).
                        if a.projectile.is_some_and(|(slot, _)| slot >= SWOOSH_SLOT_BASE) { return one(); }
                        // rocket-flare: the flying bullet objects (drone and player slots) keep the 0xFF tint
                        // too — `Bullet_init`/`Bullet_update` never write obj+0x103..0x105 [M 0x12c5a0,
                        // 0x12e7b0] — so the rocket's own orange light saturates it to the PS2's white flare
                        // (the cel-ambient dim left the blue low: a yellow star). Grenade slots unchanged.
                        if a.projectile.is_some_and(|(slot, _)| (slot as usize) >= GRENADE_SLOTS) { return one(); }
                        if a.dropped {
                            // The dropped gun is a pickup object (`DroneWeap_DropWeapon` → `Pickup_Create`
                            // 0x18dda0, the same forced 0x840): the ambient dim at its drop matrix.
                            // fix-objects: `!p.gone` — taken drops stay as tombstones (see the pose step).
                            return match s.char_drone.get(a.character as usize).copied().flatten()
                                .and_then(|di| s.drones.pickups.iter().find(|p| p.from == Some(di) && !p.hidden && !p.gone)) {
                                Some(p) if tintable(p) => amb_at(s, &p.m),
                                _ => one(),
                            };
                        }
                        if let Some((h, si, hash, k)) = a.script {
                            // A script entity is its own object (the truck: nine 0x8880 objects [L]):
                            // 0x800-glist entities dim by the ambient at their live matrix; the others
                            // (coronas, triggers) draw untinted — not character 0's ambient.
                            return match s.script_entity_k((h, si), hash, k as usize).map(|m4| amb_at(s, &m4)) {
                                Some(m) if self.models_0x800.get(&hash).copied().unwrap_or(false) => m,
                                _ => one(),
                            };
                        }
                        mults.get(a.character as usize).copied().unwrap_or_else(one)
                    })());
                }
            }
        }
        (mults, pickup_mult, attach_mult)
    }

    pub fn posed_vertices(&self, choice: Option<usize>, time: f32, opts: ViewOpts) -> Vec<Vertex> {
        let (mults, pickup_mult, attach_mult) = self.colour_mults();
        // The gas puffs' live rgba (the object's obj+0x103..0x106 bytes the draw reads [M 0x1ab2d0 tail]):
        // read straight from the pool — the alpha byte fades the quad out.
        let gas_rgba: Vec<Option<[u8; 4]>> = self.attachments.iter().map(|a| {
            a.gas.and_then(|(gi, gm)| self.sim.as_ref().and_then(|s| s.drones.gas.puffs.get(gi as usize).filter(|g| g.mesh == gm))
                .map(|g| g.rgba.map(|c| c.round().clamp(0.0, 255.0) as u8)))
        }).collect();
        // The corpse fade (fix-ai-remainder; G-026): the game's `NDrone2_FadeOut` 0x172dd8 decays the
        // drone object's alpha byte obj+0x106 (0.6 per tick in Fade 0x48, 8.0 in FadeFast 0x49) and the
        // character draw multiplies the vertices' alpha by it (Dead 0x47's timer-0xC hands the corpse to
        // Fade, which disables it at 0 — the removal). Per scene character, from its drone.
        let char_alpha: Vec<f32> = (0..self.characters.len()).map(|ci| {
            match self.sim.as_ref().and_then(|s| s.char_drone.get(ci).copied().flatten()) {
                Some(di) => self.sim.as_ref().map_or(1.0, |s| (s.drones.list[di].alpha / 255.0).clamp(0.0, 1.0)),
                None => 1.0,
            }
        }).collect();
        let one = || [1.0f32; 3];
        self.animate(choice, time, opts).into_iter()
            .zip(self.vertices[self.char_vertex_start..].iter().zip(self.char_verts.iter()))
            .map(|(pos, (v, cv))| {
                let m = if mults.is_empty() { one() }
                    else if cv.searchlight.is_some() || cv.door.is_some() { one() }
                    else if let Some(pk) = cv.pickup {
                        // The pickup is its own object (`View_DrawObjectGlist` 0x1e6780 draws obj+0xD8);
                        // `Pickup_Create` (0x18dda0) forces obj+0xF0 = glist_flags & ~0x2000 | 0x840 — every
                        // pickup is character-lit — except kind 3 (armour) and kind 6 (007 bonus). fix-render3:
                        // the multiplier is precomputed per distinct pickup above (was per vertex).
                        pickup_mult.get(&pk).copied().unwrap_or_else(one)
                    }
                    else if let Some(ai) = cv.attachment {
                        // fix-render3: precomputed per distinct attachment above (flash/tracer → 1.0,
                        // dropped → its pickup's dim, script entity → the 0x800 dim, else the owner's).
                        attach_mult.get(&ai).copied().unwrap_or_else(one)
                    }
                    else { mults.get(cv.character as usize).copied().unwrap_or_else(one) };
                let rgba = if let Some(ai) = cv.attachment.and_then(|ai| gas_rgba.get(ai as usize)).copied().flatten() {
                    ai
                } else if m == [1.0; 3] { v.rgba } else {
                    let rgb = [0, 1, 2].map(|k| ((v.rgba[k] as f32 * m[k]).round() as i32).clamp(0, 255) as u8);
                    [rgb[0], rgb[1], rgb[2], v.rgba[3]]
                };
                // The corpse fade rides the plain character vertices only (the pickups, the attachments and
                // the searchlight/door parts are their own objects with their own alpha).
                let ca = if cv.searchlight.is_none() && cv.door.is_none() && cv.pickup.is_none() && cv.attachment.is_none() {
                    char_alpha.get(cv.character as usize).copied().unwrap_or(1.0)
                } else { 1.0 };
                let a = ((rgba[3] as f32 * ca).round() as i32).clamp(0, 255) as u8;
                Vertex { pos, uv: v.uv, rgba: [rgba[0], rgba[1], rgba[2], a], nrm: v.nrm }
            }).collect()
    }

    /// Window-title text: the start scripts in use, or the chosen sequence per skeleton.
    pub fn sequence_label(&self, choice: Option<usize>) -> String {
        match choice {
            None => {
                let mut names: Vec<String> = self.characters.iter().filter_map(|c| c.start.as_ref())
                    .map(|s| format!("{:08x}", s.script.hashcode)).collect();
                names.sort();
                names.dedup();
                format!("game start animations (scripts {}); [ ] = browse sequences", names.join(", "))
            }
            Some(choice) => {
                let mut used: Vec<u8> = self.characters.iter().map(|c| c.skin.skeleton).collect();
                used.sort();
                used.dedup();
                used.iter().filter_map(|s| self.sequences.get(s).filter(|v| !v.is_empty()).map(|v| {
                    let q = &v[choice % v.len()];
                    format!("skel {s}: {:08x} ({} frames, {}/{})", q.hashcode, q.frames, choice % v.len() + 1, v.len())
                })).collect::<Vec<_>>().join("; ")
            }
        }
    }
}


// ── net-match-host: the online match's shared sim surface ─────────────────────────────────────
// port-only, with the disc census as proof [D]: ACTION.ELF's symbol table has zero
// socket/UDP/TCP/online symbols (the only "net" names are AINetwork_* nav-pathfinding) —
// the PS2 game has no online mode. The lockstep layer is the port's own determinism contract:
// every machine runs the same MP level from the host's settings + seed and exchanges only each
// agent's per-tick input (deterministic lockstep,
// `nf-net`, the one session layer). The shared state every machine must agree on is the bonds' poses, the MP match
// state and the shared RNG stream — `net_state_hash` folds exactly that in a fixed slot order.
/// mp-bots-in-match: the closest-approach parameter of the segment `a→b` to `center`, when the
/// ray passes within `radius` (the bot shots' body hit — the port's sim-level sweep; the
/// game's bot shots are shared-system bullets [M DroneWeap_FireWeapon 0x178f68], this helper
/// is its documented sim stand-in — see step_mp_bots' hit-model note).
/// The bots' head above a player position (`Bond::pos` = floor + 1.1, the head `EYE` above it): a
/// bot's `brain.pos` and every human row in its view sit at head height, so distances, sight lines
/// and aimed rounds compare like with like — the game's sight/aim is head-based too
/// (`DroneVision_CanSeePosition` 0x178438:54-75 from `NDrone2_GetHeadPos` [M]).
pub const BOT_EYE: f32 = nf_game::player::EYE;
/// The walk-cycle distance a player's anim set starts at (`ANIMOBJ+0xdc` = 0.37 at `AnimSetInit`,
/// passed by `PlayerAnimSetInitNormal` 0x18fcd8) [M mp-players.md §3, 0x1c6a48 disasm 0x1c6b58].
pub const MP_WALK_INIT: f32 = 0.37;
/// The set's stride factor (`set+0x28` = 0.5): `ANIMOBJ+0xd4 = fwd × 0.5` per tick [M 0x1c0938:96].
pub const MP_WALK_STRIDE: f32 = 0.5;

/// mp-world-sync: the segment `a→b`'s entry parameter into the box `lo..hi` grown by `g` (the
/// slab test; 0 when `a` starts inside), None = a miss — a port-only determinism helper
/// (there is no shared-shot box contact to mirror: the game has no online mode [D symtab
/// census, the net-match-host note above]).
fn seg_aabb(a: [f32; 3], b: [f32; 3], lo: [f32; 3], hi: [f32; 3], g: f32) -> Option<f32> {
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for k in 0..3 {
        let d = b[k] - a[k];
        let (l, h) = (lo[k] - g, hi[k] + g);
        if d.abs() < 1e-9 {
            if a[k] < l || a[k] > h { return None; }
            continue;
        }
        let (mut ta, mut tb) = ((l - a[k]) / d, (h - a[k]) / d);
        if ta > tb { std::mem::swap(&mut ta, &mut tb); }
        t0 = t0.max(ta);
        t1 = t1.min(tb);
        if t0 > t1 { return None; }
    }
    Some(t0)
}

fn ray_body_hit(a: [f32; 3], b: [f32; 3], center: [f32; 3], radius: f32) -> Option<f32> {
    let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let dd = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    if dd < 1e-9 { return None; }
    let t = (((center[0] - a[0]) * d[0] + (center[1] - a[1]) * d[1] + (center[2] - a[2]) * d[2]) / dd).clamp(0.0, 1.0);
    let p = [a[0] + d[0] * t - center[0], a[1] + d[1] * t - center[1], a[2] + d[2] * t - center[2]];
    (p[0] * p[0] + p[1] * p[1] + p[2] * p[2] <= radius * radius).then_some(t)
}

impl Sim {
    /// The load builds every human slot's bond in slot order (deterministic on every machine);
    /// slot 0's lands in `bond`. After the load, each machine moves ITS slot's bond into `bond`
    /// (the camera and the full local path tick it) and parks the old local one back at slot 0.
    /// mp-lobby-settings: `MP_EquipPlayer` 0x1855f8 for this machine's human — the lobby's Weapon
    /// Set spawn weapon (+ the grapple enviro-mod, the scenario bonus weapon) into the inventory and
    /// the gun drawn [M; see nf_game::mp::equip]. Every MP spawn runs it (the load, each respawn,
    /// the late join); the port used to hand out the P2K (and the load the SP default arm's
    /// give-everything kit) whatever the set.
    ///
    /// mp-respawn-fists-draw: the equip does NOT hold the spawn weapon. Its tail writes the anim
    /// trio `+0x64 = 0x47`, `+0x62 = 0x47` (the held id — the fists, the same bytes
    /// `Player_WeaponNone` 0x1a9858:12-14 writes) and `+0x63 = the spawn weapon` (the pending
    /// switch request), then zeroes the view state `+0xf4` [M 0x1855f8:50-53]. `Player_InitWeapon`
    /// 0x1a7ef8:137-139 writes the same `+0x64/+0x62 = 0x47` pair before the MP arm calls it, so
    /// EVERY spawn — the load, the respawn, the late join — holds the fists for a tick and the
    /// spawn weapon comes in through the normal weapon switch: `Player_SetWeaponAnimObj` 0x1a2e38
    /// (default state) notices `+0x63 != +0x62`, and `Player_WeaponSelect` 0x1a2cc0 direct-draws
    /// (the fists row has no holster script, `weapon_data[0x47]+0xc4 == 0` [M the same field read]
    /// → `+0x62 = +0x63`, view state 3, the draw script). The port returns the equipped id so the
    /// load path can seed the shot gates with the real weapon (gun.weapon is 0x47 right after).
    pub fn mp_equip_local(&mut self) -> u8 {
        let Some(m) = self.mp.as_ref() else { return 0 };
        let row = nf_game::mp::equip::weapon_row(&m.settings);
        let loaded = &self.mp_weapon_loaded;
        let lf = |w: u8| loaded.get(w as usize).copied().unwrap_or(true);
        let req = nf_game::mp::equip::mp_equip_player(&m.settings, &row, self.mp_local_slot, &mut self.drones.inventory, &self.weapons, lf);
        let wi = req as usize;
        let clip = self.drones.inventory.clip.get(wi).copied().unwrap_or(0).max(0) as u32;
        let spare = self.drones.inventory.carried.get(self.weapons.ammo_type(wi)).copied().unwrap_or(0).max(0) as u32;
        // The trio: held = fists, saved = fists, request = the spawn weapon, view state idle —
        // the draw runs on the next step's switch machinery [M 0x1855f8:50-53].
        self.gun = nf_game::gun::Gun::new(0x47, clip, spare);
        self.gun.switch_request = req;
        self.gun.saved_weapon = 0x47;
        self.gun.switch_state = 0;
        // The AI's class byte (obj+0xdc+0x62) reads the fists on the equip tick [M 0x1855f8:51];
        // the switch machinery's own rule takes over from the draw tick on (run()'s bond_weapon
        // write — the target weapon mid-switch).
        self.drones.bond_weapon = 0x47;
        if std::env::var_os("NF_MP_EQUIP_LOG").is_some() || std::env::var_os("NF_LOBBY_DBG").is_some() {
            let owned: Vec<usize> = (0..self.drones.inventory.owned.len()).filter(|&i| self.drones.inventory.owned[i]).collect();
            eprintln!("[mp-equip] slot {} set {} row {:?} -> weapon {:#x} clip {} spare {} owned {:x?} (held fists, draw pending)", self.mp_local_slot, m.settings.weapon_set, row, req, clip, spare, owned);
        }
        req
    }

    pub fn swap_local_slot(&mut self, slot: usize) {
        if slot == 0 || slot >= nf_net::MAX_AGENTS { return; }
        self.mp_local_slot = slot;
        if let Some(mut mine) = self.mp_bonds[slot].take() {
            mine = std::mem::replace(&mut self.bond, mine);
            self.mp_bonds[0] = Some(mine);
        }
    }

    // ======================= mp-bots-in-match (704) =======================
    // The bots' per-frame driver — the port's `BOT_update` 0x126448 site. The game's order per
    // frame: `Drone_Control` runs the bot drone's `ControlSTANDARD` → the msg-3 state dispatch
    // (the ported `BotBrain::step`) → `DroneWeap_HandleFiring`; death/respawn ride the drone
    // damage path (`BOT_respawn` 0x124e38 re-inits at a spawn point), the frags credit
    // `MPGame+slot·0x30+0x18`/`+0x04`/`+0x08` [M 0x188f70].
    // mp-never-mentioned: `BOT_update`'s own head calls `MP_pickupCountCheck` 0x18bed8
    // (jal 0x126450) — an **empty `jr ra` stub** in the shipped build [M size 8 @0x18bed8];
    // the port mirrors the stub (no count check around the bot walk).

    /// The 8-slot `others` view one brain reads (`BOT_setOtherPlayerInfo` 0x125390: the slot's
    /// obj position, the alive gate, the team word, the score/health for the trait picks).
    fn bot_others(&self) -> [nf_game::mp::bots::OtherPlayer; nf_net::MAX_AGENTS] {
        let mut others: [nf_game::mp::bots::OtherPlayer; nf_net::MAX_AGENTS] = Default::default();
        if let Some(m) = self.mp.as_ref() {
            // The humans, by SLOT: the local slot is the live `bond`, the others the parked
            // `mp_bonds` (`swap_local_slot` parks the old local body at slot 0). Online every
            // machine must build the same view (the bots draw the shared stream from it), so the
            // health is the per-slot online combat's; offline slot 0 is the local health.
            let online = self.net_rng.is_some() || self.net_rng_shared.is_some();
            for slot in 0..nf_net::MAX_AGENTS {
                let body = if slot == self.mp_local_slot { Some(&self.bond) } else { self.mp_bonds[slot].as_ref() };
                let Some(b) = body else { continue };
                let health = if online { self.mp_hp[slot] } else if slot == 0 { self.drones.health.health } else { 100.0 };
                let alive = if online { self.mp_dead_ticks[slot] == 0 && health > 0.0 } else { health > 0.0 };
                others[slot] = nf_game::mp::bots::OtherPlayer {
                    pos: [b.pos[0], b.pos[1] + BOT_EYE, b.pos[2]], alive, index: slot,
                    team: m.settings.players[slot].team, flags: 0, dist2: 0.0,
                    health, score: m.game.slots[slot].score,
                };
            }
            // The bots (positions snapshotted this tick — the brain's own row is skipped by
            // index). Their slots follow the humans [M MP_Start 0x183a50:136-162 — bot slot =
            // i + 4 after the humans; BOT_init 0x1249e8:155-156 stores slot and slot − 4]. The
            // game clamps the bot count at 4 [M 0x183a50:133-135]; 16 slots is the port's
            // mp-16-slots extension (more than 4 bots cycle the disc's 4 records).
            for (i, b) in m.bots.iter().enumerate() {
                let slot = m.settings.human_count as usize + i;
                if slot >= nf_net::MAX_AGENTS { continue; }
                others[slot] = nf_game::mp::bots::OtherPlayer {
                    pos: b.brain.pos,
                    alive: !b.dead,
                    index: slot,
                    team: m.settings.players[slot].team,
                    flags: 0,
                    dist2: 0.0,
                    health: if b.dead { 0.0 } else { b.health },
                    score: m.game.slots[slot].score,
                };
            }
        }
        others
    }

    /// One bot frame: the brain's step, then the shot resolution, the death/respawn and the
    /// score bookkeeping. Shot damage: the game's raw weapon damage is `weapon_data+0x0C`
    /// (the same word the drone fire path reads [M drone.rs FireWeapon's port]); the hit model
    /// here is a body-sphere sweep along the shot ray with the world's clear-line check — the
    /// documented sim-level stand-in for the game's hit model: the game's bots ARE Drone_Create
    /// class-2 objects hit through the shared collide pass with the drone's vertical capsule
    /// (radius 0.4 — 0.55 in the 0xC state, 0.3 crouched; ends pos+up·0.4 and
    /// pos+up·(0.4 − the +0xA0 hip scalar) [M NDrone2_Collision 0x140e90:73-121]). The sim's
    /// bots are not drone objects so the shared bullet system cannot hit them (see the report's
    /// left items — the capsule port).
    /// The MP respawn gate [M `Player_HandleDeath` 0x190bb0:41-54 — `FRAME_RATE_INT * 5` ticks
    /// from the death stamp; the port's dead-tick counters start at 0 on the stamp tick].
    pub const MP_RESPAWN_DELAY_TICKS: u32 = 300;

    /// mp-host-match-logic: the live match's kill-credit + respawn half for the local human
    /// (slot 0) — the pieces `MP_Update`/the death path drive in the game, wired into
    /// [`Sim::step_bond`] (the local and the online host tick the same path).
    ///
    /// - The death's start tick → `MP_PlayerKilled` 0x188f70 (`Player_CheckForDeath` 0x190810:91
    ///   calls it when the health runs out). The killer resolves from the slot's last-shooter word
    ///   (+0x20 — written per allowed bullet hit [M `MP_RegisterBulletHit` 0x1857c0:34]; the game
    ///   resolves by walking the death's HitByList — `Collide_GetDamageNObjects` on obj+0xD0,
    ///   per-hitter class switch (bot 2/0x11 → its DCVars slot, human 3/0x12 →
    ///   `Control_Plr2Ind`), then credits `MPGame[killer*0x30+4/0x10]` and writes the
    ///   last-killer word `MPGame[victim*0x30+0x28]` [M MP_PlayerKilled 0x188f70:121-165].
    ///   The port's registration word is the documented equivalent: one registration per
    ///   allowed bullet hit [M 0x1857c0:34] stands in for the per-death list walk). Values 0xfffe (the init / the respawn reset [M 0x183438:59,
    ///   0x185368:56]) and 0xffff (the restart reset [M 0x183fc0:81]) mean no credit.
    /// - After the 5 s death delay (`Player_HandleDeath` 0x190bb0:41-54 — `FRAME_RATE_INT * 5`
    ///   frames from the death stamp) the respawn runs `MP_ReSpawn` 0x185368 via
    ///   [`MpMatch::mp_respawn`]: the policy's spawn pick, the slot reset, the health
    ///   (`armor_bonus + 100.0` [M 0x185368:65-73]), and the body moves (`Player_StandAtNewPosition`
    ///   [M :97]). The bots respawn in [`Sim::step_mp_bots`] (`BOT_respawn` 0x124e38); the remote
    ///   humans have no death pipeline in the port (their slots never die here).
    pub fn step_mp_death_respawn(&mut self, death_was_active: bool) {
        if self.mp.is_none() { return; }
        if !death_was_active && self.drones.death.active {
            // The death start: score it. The killer word 0..8 is a slot id (self-damage credits
            // the victim — `player_killed`'s self-kill path [M 0x188f70:151-155,167-169]).
            let Some(m) = self.mp.as_mut() else { return };
            let st = m.game.slots[0].status;
            let killer = if st < 8 { Some(st as usize) } else { None };
            // mp-scenarios-online (agent6): a death the match itself decided (the GoldenEye
            // strike's `player_killed` [M 0x187568:98-108]) is already counted — the body's death
            // sequence only follows it (`step_mp_scenario_sync`).
            if !m.game.slots[0].dead {
                m.player_killed(0, killer, false, false, &mut self.drones.rng);
            }
        }
        // The 5 s respawn gate [M 0x190bb0:41-54: `FRAME_RATE_INT * 5 <= GameState+0x34 -
        // obj+0xec`] — the port's dead-tick counter starts at 0 on the stamp tick.
        if !self.drones.death.active || self.drones.death.ticks < Self::MP_RESPAWN_DELAY_TICKS {
            return;
        }
        let Some(m) = self.mp.as_mut() else { return };
        let Some((sp_idx, hp)) = m.mp_respawn(0, &mut self.drones.rng) else { return };
        let sp = m.spawns.points[sp_idx as usize];
        // The body: `Player_StandAtNewPosition`'s port is the same `Bond::stand_at` the load's
        // spawn uses; the health is `MP_ReSpawn`'s `Player_SetHealth(armor_bonus + 100.0)` [M
        // 0x185368:65-73] and `obj+0xff = 3` ends the death sequence [M :93].
        self.bond = nf_game::player::Bond::stand_at(&self.world, [sp.pos[0], sp.pos[1], sp.pos[2]], sp.facing[1]);
        self.drones.health.health = hp;
        self.drones.death = nf_game::death::Death::default();
        m.set_slot(0, self.bond.pos);
        // mp-respawn-ammo: `MP_ReSpawn`'s human arm re-runs `MP_EquipPlayer` 0x185368:74 — the
        // inventory is wiped and the set's spawn kit re-granted (`clip_size·2` rounds [M
        // 0x1855f8:26-29]); the death sequence had left the fists 0x47 [M 0x190810:61] and the
        // spent rounds. The online path re-equips the same way in `step_net_combat`.
        self.mp_equip_local();
        // mp-shot-gate: the shared gate re-seeds with the same equip (the settings clone keeps
        // the borrow split) — the hashed rounds must agree on every machine [M 0x185368:74].
        let settings = self.mp.as_ref().map(|m| m.settings.clone());
        if let Some(s) = settings {
            let lf = |w: u8| self.mp_weapon_loaded.get(w as usize).copied().unwrap_or(true);
            self.mp_combat.gate_equip(0, &s, &self.weapons, lf);
        }
    }

    /// mp-death-effects: the local player's death presentation — the getters the draw code reads
    /// (docs/systems/mp-death-effects.md). `None` while alive.
    ///
    /// The game's MP death runs the SP sequence's sim core with the SP-only pieces skipped: the
    /// blood pane (HUD pane 12) enables on the death tick [M `Player_HandleDeath` 0x190bb0:38-40]
    /// and grows 1 px/tick [M `HUD_UpdateBloodPane` 0x19a6a8:27-30] — in MP without the
    /// `HUD_DisableAll`, the HUD stays up [M 0x190bb0:24 vs :39]; the death sound 0x89 and cam
    /// mode 0 are common [M `Player_CheckForDeath` 0x190810:64,88]; the sting (ch 0x5e +
    /// `Music_Event(6,1)`) and the ✕/ch 0x62/fail-menu path are SP-only [M 0x190810:39-42,
    /// 0x190bb0:29-33]; MP respawns 300 ticks after the stamp [M 0x190bb0:41-54] and
    /// `MP_ReSpawn` turns the pane back off [M 0x185368:67]. No countdown/prompt is drawn — the
    /// 5 s wait is silent [M 0x190bb0:37-55]; the only MP-death text is the kill message
    /// [M 0x190810:67-87, the port's `kill_msgs`].
    pub fn local_death_fx(&self) -> Option<LocalDeathFx> {
        let d = &self.drones;
        let local = self.mp_local_slot;
        // Online the shared hurt pass marks the slot dead (`mp_dead_ticks`) a tick before
        // `drones.death` starts (the health mirror + `Drones::tick` follow it) — the dead slot
        // surfaces from the kill tick.
        let dead_mp = self.mp.is_some() && self.mp_dead_ticks[local] > 0;
        if !d.death.active && !dead_mp { return None; }
        let online = self.mp.is_some() && (self.net_rng.is_some() || self.net_rng_shared.is_some());
        let is_mp = self.mp.is_some();
        // The pane clock: elapsed since the death stamp [M 0x190810:59-60 obj+0xec]. Offline
        // (and SP) `death.ticks` is exactly that; online the death starts one tick after the
        // kill tick (`mp_dead_ticks` is set in `step_net_combat`'s shared hurt pass, the health
        // mirror + `Drones::tick` follow), so the elapsed rides `mp_dead_ticks - 1`.
        let (elapsed, gate_left) = if online {
            let dt = self.mp_dead_ticks[local];
            (dt.saturating_sub(1), Self::MP_RESPAWN_DELAY_TICKS.saturating_sub(dt))
        } else if is_mp {
            (d.death.ticks, Self::MP_RESPAWN_DELAY_TICKS.saturating_sub(d.death.ticks))
        } else {
            (d.death.ticks, 0)
        };
        let pane_px = elapsed.min(nf_game::death::RED_FADE_TICKS);
        Some(LocalDeathFx {
            phase: d.death.phase(),
            pane_px,
            // [S] unverified, visual-only (no draw code changed): the draw's alpha source,
            // derived from the px rule (448 px = full cover) — the look needs a render
            // comparison against PCSX2; the game's death-pane alpha ramp is
            // untraced here.
            fade_alpha: (pane_px as f32 / nf_game::death::RED_FADE_TICKS as f32).clamp(0.0, 1.0),
            // SP hides the whole HUD for the pane [M 0x190bb0:24]; MP keeps it [M :39].
            hud_hidden: !is_mp,
            // MP only: the ticks left of the 5 s respawn gate [M 0x190bb0:41-54]; SP waits for
            // the cover / the ✕ instead (None).
            respawn_ticks_left: if is_mp { Some(gate_left) } else { None },
            // The frozen first-person eye: cam mode 0 [M 0x190810:64], the pose source
            // `bond.head()` (the SP test `dead_bond_holds_the_death_camera_spot`). Online the
            // freeze starts on the kill tick itself (`net_dead`), one tick before
            // `drones.death` does.
            camera_frozen: d.death.active || (online && self.mp_dead_ticks[local] > 0),
        })
    }

    /// mp-scenarios-online (agent6): the match -> body half of the scenario rules, run every tick
    /// right after `MP_Update` on every machine (shared state + the shared stream only):
    ///
    /// - **the round restart** (`MP_RestartScenario` 0x183fc0:55-75, run by `MP_Update`'s state 6
    ///   [M 0x184bd0:79-92]): every present human respawns through `MP_ReSpawn` (the policy's
    ///   pick, the health, the body moves) and every bot through `BOT_respawn`; the slots'
    ///   last-killer/last-shooter words then reset to 0xffff [M :76-82]. The port only re-picked
    ///   the site before — Demolition/Protection's next round started with everyone where the
    ///   last one ended (and the dead still dead).
    /// - **a death the match decided** (the GoldenEye strike's target [M 0x187568:98-108]): the
    ///   slot is `dead` in the match but its body is alive — online the per-slot combat starts the
    ///   death (health 0, the 5 s respawn timer), offline slot 0's health drops to 0 (its death
    ///   sequence + respawn follow), a bot dies like a shot bot.
    pub fn step_mp_scenario_sync(&mut self) {
        let online = self.net_rng.is_some() || self.net_rng_shared.is_some();
        let frame = self.drones.frame;
        let local = self.mp_local_slot;
        let mut restart_equip = false;
        let mut restart_slots: Vec<usize> = Vec::new();
        let Some(m) = self.mp.as_mut() else { return };
        if std::mem::take(&mut m.game.restart_pending) {
            let humans = (m.settings.human_count as usize).min(nf_net::MAX_AGENTS);
            for slot in 0..humans {
                let has_body = slot == local || self.mp_bonds[slot].is_some();
                if !has_body || m.game.slots[slot].pos.is_none() { continue; }
                let Some((sp_idx, hp)) = m.mp_respawn(slot, &mut self.drones.rng) else { continue };
                restart_slots.push(slot);
                let sp = m.spawns.points[sp_idx as usize];
                let nb = nf_game::player::Bond::stand_at(&self.world, [sp.pos[0], sp.pos[1], sp.pos[2]], sp.facing[1]);
                m.set_slot(slot, nb.pos);
                let was_dead = self.mp_dead_ticks[slot] > 0 || (slot == local && self.drones.death.active);
                if slot == local {
                    self.bond = nb;
                    self.drones.health.health = hp;
                    if was_dead {
                        self.drones.death = nf_game::death::Death::default();
                    }
                    // mp-respawn-ammo: the restart's `MP_ReSpawn` re-arms EVERY present human —
                    // the full arm keys on obj+0xff ∈ {3, 0x12} and 3 is the live state too
                    // [M 0x185368:74, 0x183fc0:55-75], so a surviving local re-equips as well.
                    restart_equip = true;
                } else {
                    self.mp_bonds[slot] = Some(nb);
                }
                self.mp_hp[slot] = hp;
                self.mp_armour[slot] = 0.0; // the game's bots carry NO armour: BOTWEAP_setDroneArmour
                // 0x12af70:3-7 is an empty function (bare return) and BOT_init 0x1249e8:95-100 sets
                // no armour word [M] — 0.0 is the game's value (standins-scene-929).
                self.mp_dead_ticks[slot] = 0;
            }
            for b in m.bots.iter_mut() {
                b.dead = true; // BOT_respawn runs this tick in step_mp_bots (respawn_at = now)
                b.respawn_at = frame;
            }
            for s in m.game.slots.iter_mut() {
                s.last_killer = 0xffff; // [M 0x183fc0:76-82] — after the respawns
                s.status = 0xffff;
            }
        }
        for b in m.bots.iter_mut() {
            if b.slot < m.game.slots.len() && m.game.slots[b.slot].dead && !b.dead {
                b.dead = true;
                b.health = 0.0;
                b.deaths += 1;
                b.respawn_at = frame + 180; // the bots' death delay — the game's own constant:
                // NDrone2_DSTATE_BotDead stamps the SM timer at GameState + FRAME_RATE_INT * 3 =
                // 180 ticks, then BotRespawn runs BOT_respawn [M 0x171518:22-41; the human gate
                // is FRAME_RATE_INT * 5 = 300, 0x190bb0:41] (standins-scene-929).
            }
        }
        if online {
            let humans = (m.settings.human_count as usize).min(nf_net::MAX_AGENTS);
            for slot in 0..humans {
                let has_body = slot == local || self.mp_bonds[slot].is_some();
                if has_body && m.game.slots[slot].dead && self.mp_dead_ticks[slot] == 0 {
                    self.mp_hp[slot] = 0.0;
                    self.mp_dead_ticks[slot] = 1;
                }
            }
        } else if m.game.slots[0].dead && !self.drones.death.active && self.drones.health.health > 0.0 {
            self.drones.health.health = 0.0;
        }
        // The restart's `MP_ReSpawn` re-arms every present human (`MP_EquipPlayer` [M
        // 0x183fc0:55-75 → 0x185368:74]) and the shared gates re-seed the same way — the hashed
        // rounds must agree on every machine.
        if restart_equip { self.mp_equip_local(); }
        if !restart_slots.is_empty() {
            let settings = self.mp.as_ref().map(|m| m.settings.clone());
            if let Some(s) = settings {
                let lf = |w: u8| self.mp_weapon_loaded.get(w as usize).copied().unwrap_or(true);
                for slot in restart_slots {
                    self.mp_combat.gate_equip(slot, &s, &self.weapons, lf);
                }
            }
        }
    }

    pub fn step_mp_bots(&mut self) {
        if self.mp.as_ref().map_or(true, |m| m.bots.is_empty()) { return; }
        let others = self.bot_others(); // the immutable pass ends before the match is taken &mut
        {
            use nf_game::pickup::kind;
            // mp-bot-weapons: `available` = the pickup stands takeable (not in the state-2
            // respawn wait, not deleted) — `pickGoal` scores only the registered, present ones.
            let live: Vec<([f32; 3], u16, u8, bool)> = self.drones.pickups.iter()
                .filter(|p| matches!(p.kind, kind::WEAPON | kind::AMMO | kind::ARMOUR))
                .map(|p| ([p.m[3][0], p.m[3][1], p.m[3][2]], p.weapon, p.kind as u8, !p.gone && p.state != 2))
                .collect();
            if live.len() != self.mp_bot_pickups.len() {
                self.mp_bot_pickups = live.iter().map(|&(pos, w, c, a)| nf_game::mp::bots::BotPickup {
                    pos, weapon_id: w, mp_class: c, is_obj_flag2: false, visit: [0.0; 16], available: a,
                }).collect();
            } else {
                for (b, &(pos, w, c, a)) in self.mp_bot_pickups.iter_mut().zip(live.iter()) {
                    b.pos = pos; b.weapon_id = w; b.mp_class = c; b.available = a;
                }
            }
        }
        // mp-bot-weapons: the weapon table, once per arena.
        if self.mp_bot_weapons.is_none() {
            let (wd, skins) = (&self.weapons, &self.skins);
            self.mp_bot_weapons = Some(nf_game::mp::botweap::BotWeaponTable::from_weapon_data(wd, |w| skins.contains_key(&wd.first_person(w))));
        }
        let pickups = std::mem::take(&mut self.mp_bot_pickups);
        let Some(m) = self.mp.as_mut() else { self.mp_bot_pickups = pickups; return };
        let mut stamps: Vec<(usize, (usize, f32))> = Vec::new();
        let frame = self.drones.frame;
        let scenario = m.settings.scenario;
        let world = &self.world;
        let nav = &self.nav;
        let wdata = &self.weapons;
        let wtable = self.mp_bot_weapons.as_ref();
        let start_weapon = wtable.map_or(0, |t| t.start_weapon(m.settings.weapon_set));
        let multi = self.tuning.Plr_DMod_Multi;
        // The humans the bots fight (online: every human slot on every machine; offline slot 0).
        let online = self.net_rng.is_some() || self.net_rng_shared.is_some();
        let local = self.mp_local_slot;
        let pain = nf_game::mp::equip::MpPainMods::from_tuning(&self.tuning);
        // mp-bot-objectives: the scenario view the bots' objective goals read (the mode objects,
        // the status words, the roles, the other bots' objective goals — refreshed per bot below,
        // the game's bots read each other live in slot order).
        let mut objectives = m.bot_objectives();
        // The team view of `BOT_setOtherPlayerInfo` 0x125390:62-73: team games and Assassination
        // compare the team words; FFA compares against team 3 (no one is a team-mate) [M].
        let team_view = m.settings.team_game != 0 || scenario == nf_game::mp::scenario::ASSASSIN;
        if std::env::var_os("NF_BOTLOG").is_some() && frame % 60 == 0 {
            // mp-bot-objectives: the scenario objects + scores once a second (log only)
            eprintln!("[botlog] f{} objectives: team scores {:?} slot scores {:?} flags {:?} uplinks {:?} hill-status {:?} site {:?} ge {:?} bp {:?}",
                frame, m.game.team_scores, m.game.slots.iter().map(|s| s.score).collect::<Vec<_>>(),
                m.objects_rt.flags.iter().flatten().map(|f| (f.state, f.carrier)).collect::<Vec<_>>(),
                m.objects_rt.uplinks.iter().map(|u| (u.state, u.controller)).collect::<Vec<_>>(),
                m.game.slots.iter().map(|s| s.status_bits & 0x10 != 0).collect::<Vec<_>>(),
                m.objects_rt.site.as_ref().map(|x| (x.state, x.sub)),
                m.objects_rt.ge.iter().flatten().map(|g| (g.state, g.carrier)).collect::<Vec<_>>(),
                m.objects_rt.blueprint.as_ref().map(|b| (b.state, b.carrier)));
            for b in &m.bots {
                let g = &b.brain.vars.goals[1];
                eprintln!("[botlog]   slot {} team {} goal1 type {} target {:#x} state {:#x} opp {:?}", b.slot, b.team, g.goal_type, g.target, b.brain.state, b.brain.vars.opponent);
            }
        }

        for i in 0..m.bots.len() {
            let alive = !m.bots[i].dead;
            if !alive {
                // `BOT_respawn` 0x124e38: the death handler re-inits the bot at a spawn point
                // after the delay — the game's delay IS 180 ticks (BotDead's FRAME_RATE_INT * 3
                // timer [M 0x171518:22-24]); the port matches (standins-scene-929).
                if frame >= m.bots[i].respawn_at {
                    let slot = m.bots[i].slot;
                    // mp-scenarios-online (agent6): `BOT_respawn` 0x124e38:85 goes through
                    // `MP_ReSpawn` — its gates hold for bots too (no respawn while the round is
                    // over; Top Agent's out-of-lives) [M 0x185368:16-20].
                    if !m.respawn_allowed(slot) { continue; }
                    let team = m.bots[i].team;
                    // mp-16-slots: more than 4 bots cycle the disc's 4 records (as create_bots does).
                    // mp-lobby-settings: the bot's own prepared record (the lobby's character /
                    // skill / team), as create_bots used.
                    let rec = nf_game::mp::MpMatch::bot_record(&m.settings, i);
                    let sp = m.get_spawn_point(team, slot, &mut self.drones.rng) as usize;
                    let p4 = m.spawns.points[sp].pos;
                    let yaw = m.spawns.points[sp].facing[1];
                    let feet = nf_game::player::Bond::stand_at(world, [p4[0], p4[1], p4[2]], yaw).pos;
                    let pos = [feet[0], feet[1] + BOT_EYE, feet[2]];
                    let mut brain = nf_game::mp::bots::BotBrain::bot_init(&rec, slot as u8, pos, yaw);
                    m.arm_bot(&mut brain);
                    m.bots[i].brain = brain;
                    // mp-16-slots: the bot's index is its list position (the game's slot − 4 wraps
                    // once bots follow the humans).
                    m.bots[i].brain.vars.bot_index = i as i16;
                    m.bots[i].health = m.bots[i].max_health;
                    m.bots[i].dead = false;
                    m.bots[i].respawns += 1;
                    // mp-scenarios-online (agent6): `MP_ReSpawn`'s slot resets (streak, last
                    // shooter, status bits, alive) [M 0x185368:55-57] — the bot's slot stayed
                    // `dead` forever after its first death (KOTH's alive gate, the contacts and
                    // Top Agent's out-count all read it).
                    m.do_respawn(slot, sp as u32);
                    m.game.slots[slot].pos = Some(pos);
                }
                continue;
            }
            let health = m.bots[i].health;
            let max_health = m.bots[i].max_health;
            let my_team = m.settings.players[m.bots[i].slot].team;
            // This bot's view of the others: the squared distance and the candidate + seen bits
            // `find_opponent` reads (bit1|bit2 [M the CanSeeObject bit]) — the eye-to-chest line
            // clear of the world. The driver passed every row with flags 0 / dist2 0, so no bot
            // ever acquired anyone and they idled in 0xF9 all match. PROVEN the game's rule: bot vision is LOS-only —
            // DroneVision_CanSeePosition 0x178438:54-75 gates on the same nav cel /
            // AINetwork_TestRayCels then Collide_LineOfSight, with NO FOV cone or range constant
            // anywhere [M; 0x176f18:56-104 the same for objects] — the port's LOS-only read IS
            // the game's rule (standins-scene-929).
            let me = m.bots[i].brain.pos;
            let mut seen: Vec<nf_game::mp::bots::OtherPlayer> = others.to_vec();
            // mp-perf-16: the eye-to-chest sight line (`segment_surface_bullet`, eye = `me`, chest =
            // the other's eye − 0.4) is not run for every pair here (14 bots × 15 others = 210
            // segments a tick, 75% of the 16-participant tick): `find_opponent` — its only reader
            // whose result depends on it — runs it lazily for the candidates that can change its
            // pick (`BotCtx::lazy_sight`). The cheap bits (candidate, team-mate) stay precomputed.
            for o in seen.iter_mut() {
                if !o.alive || o.index == m.bots[i].slot { continue; }
                let d = [o.pos[0] - me[0], o.pos[1] - me[1], o.pos[2] - me[2]];
                o.dist2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
                // bit1 candidate for every present player [M 0x125390:99-101], bit3 the team-mate
                // in the team view [M :100-102] (mp-bot-objectives), bit2 seen (lazily, in `find_opponent`)
                o.flags |= 0x2;
                if team_view && o.team == my_team { o.flags |= 0x8; }
            }
            // mp-bot-objectives: the standing Demolition/Protection site rides the objective row
            // (`OBJ_ROW`) — the attackers take it as their opponent (`NDrone2_FindOpponent`
            // 0x1424e0:519-560 sets the site obj as `Drone+0x170` [M]); the row's position is the
            // site sphere centre — the game's aim point is the opponent obj's position
            // (+0x30 [M 0x125b30]), and the old +0.3 chest-aim compensation is gone with it
            // (bot-weapons-finish-3).
            if let Some(site) = objectives.site.filter(|s| s.radius > 0.0) {
                let c = site.centre;
                let d = [c[0] - me[0], c[1] - me[1], c[2] - me[2]];
                let clear = world.segment_surface_bullet(me, c).is_none();
                seen.push(nf_game::mp::bots::OtherPlayer {
                    pos: c, alive: true, index: nf_game::mp::bots::OBJ_ROW,
                    team: objectives.site_defenders, flags: 0x2 | if clear { 0x4 } else { 0 },
                    dist2: d[0] * d[0] + d[1] * d[1] + d[2] * d[2], health: site.state as f32, score: 0.0,
                });
            }
            let ctx = nf_game::mp::bots::BotCtx {
                now: 0.0, // the brain's own tick counter is the clock (net-lockstep F4)
                scenario,
                my_index: m.bots[i].slot,
                my_team,
                pickups: &pickups,
                others: &seen,
                world,
                nav: Some(nav),
                health,
                max_health,
                stationary: false,
                lazy_sight: true,
                objectives: Some(&objectives),
                weapons: wtable,
                start_weapon,
            };
            let before = m.bots[i].brain.shots.len();
            let held_before = m.bots[i].brain.vars.weapon_id;
            { let _p = nf_game::prof::scope("bots: brain.step"); m.bots[i].brain.step(&ctx, &mut self.drones.rng, true); }
            if std::env::var_os("NF_BOTLOG").is_some() && held_before != m.bots[i].brain.vars.weapon_id && m.bots[i].brain.tick > 2 {
                eprintln!("[botlog] f{} bot{} switches {:#x} -> {:#x} (state {:#x}, opp dist {:.1})", frame, i, held_before,
                    m.bots[i].brain.vars.weapon_id, m.bots[i].brain.state, m.bots[i].brain.vars.opp_dist);
            }
            if let Some(g) = objectives.bot_goals.get_mut(i) {
                let g1 = &m.bots[i].brain.vars.goals[1];
                *g = (m.bots[i].slot, g1.goal_type, g1.target);
            }
            stamps.push((i, m.bots[i].brain.last_visit_stamp));
            // (The brain's body walks the world itself — the player's movement core: collision,
            // floors, gravity — so the old per-tick re-stand, which could snap a bot onto a
            // lower floor, is gone.)
            if frame % 60 == 0 && env_flag(&ENV_BOTLOG, "NF_BOTLOG") {
                eprintln!("[botlog] f{} bot{} state {:#x} pending {:?} opp {:?} goals {:?} pos {:.1?} trace {} route {}/{} valid {:?} dest {:.1?} stuck {}",
                    frame, i, m.bots[i].brain.state, m.bots[i].brain.trace.last(),
                    m.bots[i].brain.vars.opponent,
                    m.bots[i].brain.vars.goals.iter().map(|g| (g.active, g.goal_type, g.target)).collect::<Vec<_>>(),
                    m.bots[i].brain.pos, m.bots[i].brain.trace.len(),
                    m.bots[i].brain.route.idx, m.bots[i].brain.route.nodes.len(), m.bots[i].brain.route.valid, m.bots[i].brain.route.dest, m.bots[i].brain.stuck_hits);
                let v = &m.bots[i].brain.vars;
                eprintln!("[botlog]   fire latch {} empty {} next {:.2} unseen {} clip {} yaw {:.2} tick {} weapon {} interval {} burst {} hp {:.0}/{:.0} deaths {} | state {:#x} nlst {} bl {} shots {}",
                    v.fire_latch, v.burst_empty, v.next_bullet_at, v.unseen_ticks, v.clip_rounds, m.bots[i].brain.yaw, m.bots[i].brain.tick,
                    v.weapon_id, m.bots[i].brain.weapon.interval, m.bots[i].brain.weapon.burst, m.bots[i].health, m.bots[i].max_health, m.bots[i].deaths,
                    m.bots[i].brain.state, v.next_shot_tick, v.burst_left, m.bots[i].shots);
            }
            let new_shots: Vec<nf_game::mp::bots::BotShot> =
                m.bots[i].brain.shots[before..].to_vec();
            m.bots[i].shots += new_shots.len() as u32;
            for s in &new_shots {
                let w = s.weapon as usize;
                if w < m.bots[i].shots_by_weapon.len() { m.bots[i].shots_by_weapon[w] += 1; }
            }
            m.bots[i].brain.shots.clear();
            let bot_pos = m.bots[i].brain.pos;
            let bslot = m.bots[i].slot;
            // The bot body's held weapon: its latest shot's (the brain has no weapon id of its own).
            if let Some(s0) = new_shots.last() { if bslot < nf_net::MAX_AGENTS { self.mp_held[bslot] = s0.weapon; } }
            if let Some(prev) = m.game.slots[bslot].pos {
                if bslot < nf_net::MAX_AGENTS { self.mp_body_speed[bslot] = ((bot_pos[0] - prev[0]).powi(2) + (bot_pos[2] - prev[2]).powi(2)).sqrt() * 60.0; }
            }
            m.game.slots[bslot].pos = Some(bot_pos);

            // The shots: real bullets at the human (the shared pipeline damages him), and the
            // body-sphere sweep at the other bots (the shared system cannot hit non-drones).
            for shot in new_shots {
                let w = shot.weapon as usize;
                let Some(rec) = wdata.records.get(w) else { continue };
                let dmg = f32::from_le_bytes(rec[0x0c..0x10].try_into().unwrap());
                let range = f32::from_le_bytes(rec[0x1c..0x20].try_into().unwrap());
                let end = [shot.pos[0] + shot.dir[0] * range,
                           shot.pos[1] + shot.dir[1] * range,
                           shot.pos[2] + shot.dir[2] * range];
                // The other bots: nearest body (the port's sim-level sphere, r 0.6 — the game's
                // capsule spec is the step's header note [M 0x140e90:73-121]) the ray passes
                // within, world-clear.
                let mut best: Option<(usize, f32)> = None;
                for (j, b) in m.bots.iter().enumerate() {
                    if j == i || b.dead { continue; }
                    if let Some(t) = ray_body_hit(shot.pos, end, b.brain.pos, 0.6) {
                        if best.map_or(true, |(_, bt)| t < bt) { best = Some((j, t)); }
                    }
                }
                // The human: the game's damage is the bullet event chain (armour, flash, the
                // death handler); the port applies the same raw `weapon_data+0x0C` word straight
                // to the health block here — the bullet-spawn route (bots are not drone objects,
                // the shared Bullet owners index the drones list) is the report's left item.
                // mp-host-match-logic: the hit first registers through the match
                // (`MP_RegisterBulletHit` 0x1857c0 — the friendly-fire gate + the last-shooter
                // word the kill credit later resolves; the port has no HitByList), and a blocked
                // hit deals no damage — the game's own order.
                let mut hit_human: Option<(usize, f32)> = None;
                for h in 0..m.settings.human_count as usize {
                    let o = &others[h];
                    if !o.alive || o.index != h { continue; }
                    if !online && h != 0 { continue; }
                    // The human's position is the feet; the round leaves the bot at eye height
                    // (its spawn point's floor + 1.6): the body sphere sits at the chest — the
                    // documented stand-in for the game's vertical capsule (r 0.4, ends
                    // pos+up·0.4 / pos+up·(0.4 − +0xA0) [M 0x140e90:104-121]).
                    if let Some(t) = ray_body_hit(shot.pos, end, [o.pos[0], o.pos[1] - 0.3, o.pos[2]], 0.7) {
                        if hit_human.is_none_or(|(_, bt)| t < bt) { hit_human = Some((h, t)); }
                    }
                }
                // One round hits one body — the nearest, bot or human — and only when the world is
                // clear up to it. The old gate tested the whole ray out to the weapon's range
                // (which nearly always ends in a wall), so no bot round ever hit another bot, while
                // the human test had no wall check at all. The game fires bot shots through the
                // SAME bullet system as humans (DroneWeap_FireWeapon 0x178f68 → Bullet_init; the
                // world collide applies identically [M]) — the port's two routes (this sim ray
                // for bots, the shared pipeline for the human) are the documented stand-in, with
                // the wall check matching the game's world collide (standins-scene-929).
                let tb = best.map_or(f32::MAX, |b| b.1);
                let th = hit_human.map_or(f32::MAX, |h| h.1);
                if th <= tb { best = None; } else { hit_human = None; }
                let tt = th.min(tb);
                // mp-bot-objectives: the Demolition/Protection site takes the round when it is the
                // nearest thing on the ray and the world is clear up to it (the human combat's
                // site test, `step_net_combat`). The single sphere is the port's documented
                // coarser stand-in for the game's per-stream-object walk (SP_GetHitDamage
                // 0x1dc5f8:80-93 [M]; the site SP's hit damage [M 0x1870a0:81-91]) — the same
                // shape the MP site damage carries.
                if let Some((c, r)) = m.site_sphere() {
                    if let Some(ts) = ray_body_hit(shot.pos, end, c, r) {
                        let p = [shot.pos[0] + (end[0] - shot.pos[0]) * ts, shot.pos[1] + (end[1] - shot.pos[1]) * ts, shot.pos[2] + (end[2] - shot.pos[2]) * ts];
                        if ts < tt && world.segment(shot.pos, p).is_none() {
                            let shooter = m.bots[i].slot;
                            m.damage_site(dmg, shooter);
                            continue;
                        }
                    }
                }
                if tt < f32::MAX {
                    let p = [shot.pos[0] + (end[0] - shot.pos[0]) * tt, shot.pos[1] + (end[1] - shot.pos[1]) * tt, shot.pos[2] + (end[2] - shot.pos[2]) * tt];
                    if !world.segment_clear(shot.pos, p) { best = None; hit_human = None; }
                }
                // mp-remote-fx: the resolved round for the effects pass (draw/audio only).
                {
                    use crate::mpfx::{MpShot, MpShotHit};
                    let at = |t: f32| [shot.pos[0] + (end[0] - shot.pos[0]) * t, shot.pos[1] + (end[1] - shot.pos[1]) * t, shot.pos[2] + (end[2] - shot.pos[2]) * t];
                    let (hit, fend) = if let Some((h, t)) = hit_human { (MpShotHit::Human(h), at(t)) }
                        else if let Some((j, t)) = best { (MpShotHit::Bot(m.bots[j].slot), at(t)) }
                        else if let Some((t, normal, material)) = world.segment_surface_bullet(shot.pos, end) { (MpShotHit::World { normal, material }, at(t)) }
                        else { (MpShotHit::Miss, end) };
                    self.mp_fx.events.push(MpShot { shooter: m.bots[i].slot, bot: true, weapon: shot.weapon, from: shot.pos, dir: shot.dir, end: fend, hit, damage: dmg });
                }
                if let Some((h, _)) = hit_human {
                    let shooter = m.bots[i].slot;
                    if m.register_bullet_hit(h, shooter) {
                        // mp-lobby-settings: `Player_HandlePain`'s MP arm [M 0x1902c8:51-82]; the
                        // bot round's chest sphere stands in for the whole body — the game's
                        // capsule has no bot-specific part split (the bullet event chain takes
                        // its part from the hit on the model [M the shared bullet system]);
                        // documented (standins-scene-929).
                        let dmg = nf_game::mp::equip::mp_pain(&m.settings, &pain, dmg, nf_game::mp::equip::PART_TORSO);
                        if online {
                            // The per-slot online combat owns the health and the death (every machine).
                            self.mp_hp[h] = (self.mp_hp[h] - dmg).max(0.0);
                            if self.mp_hp[h] <= 0.0 && self.mp_dead_ticks[h] == 0 {
                                self.mp_dead_ticks[h] = 1;
                                m.player_killed(h, Some(shooter), false, false, &mut self.drones.rng);
                                if std::env::var_os("NF_BOTLOG").is_some() { eprintln!("[botlog] f{frame} kill: slot {shooter} (weapon {:#x}) killed slot {h}", shot.weapon); }
                            }
                            if h == local { self.drones.health.health = self.mp_hp[h]; }
                        } else {
                            self.drones.health.health = (self.drones.health.health - dmg).max(0.0);
                        }
                    }
                }
                if let Some((j, _)) = best {
                    // mp-lobby-settings: the friendly-fire gate (`MP_RegisterBulletHit` 0x1857c0
                    // [M :43-45]) and `BOT_handlePain`'s MP arm [M 0x125ed8:48-79] — bot rounds hit
                    // team-mate bots at full damage before.
                    let (vs, ks) = (m.bots[j].slot, m.bots[i].slot);
                    if !m.register_bullet_hit(vs, ks) { continue; }
                    let dmg = nf_game::mp::equip::mp_pain(&m.settings, &pain, dmg, nf_game::mp::equip::PART_TORSO);
                    m.bots[j].health -= dmg;
                    if m.bots[j].health <= 0.0 {
                        // mp-host-match-logic: the frag goes through the match's real scoring
                        // (`MP_PlayerKilled` 0x188f70 — the killer/streak/team-score rules and
                        // the carried-object drop, previously an inline stand-in); the entity's
                        // own bookkeeping (the bot table's deaths/respawn timer) stays here.
                        let victim_slot = m.bots[j].slot;
                        let killer_slot = m.bots[i].slot;
                        m.bots[j].health = 0.0;
                        m.bots[j].dead = true;
                        m.bots[j].deaths += 1;
                        m.bots[j].respawn_at = frame + 180;
                        m.player_killed(victim_slot, Some(killer_slot), false, false, &mut self.drones.rng);
                        if std::env::var_os("NF_BOTLOG").is_some() { eprintln!("[botlog] f{frame} kill: slot {killer_slot} (weapon {:#x}) killed slot {victim_slot}", shot.weapon); }
                    }
                }
            }
        }
        // The visit stamps the brains left (`set_pickup_visit_time`'s MP_PICKUP+0x80 write).
        let mut pickups = pickups;
        for (i, (idx, t)) in stamps {
            if let Some(p) = pickups.get_mut(idx) { if i < p.visit.len() { p.visit[i] = t; } }
        }
        self.mp_bot_pickups = pickups;
    }

    /// mp-bot-weapons: `Pickup_Update` 0x18e148's taker loop for an MP match, run on every machine
    /// at one point of the tick (right after the bots, before any body moves — every taker's
    /// position is last tick's on every machine). Per pickup, in list order: the humans by slot,
    /// then the bots by slot; the first whose body is within the pickup radius + 1.1 with a clear
    /// line takes it, one taker a tick [M :147-208]. The humans: the local slot runs the full
    /// `Pickup_Handler` (`Drones::take_pickup` — inventory, auto-switch, sound); online a refused
    /// weapon/ammo take still consumes — the game reads the LOCAL inventory per machine
    /// (`Pickup_Handler` 0x18e148:203-246 [M]); one machine holds one inventory, so no shared
    /// rule exists in the game — the shared consumption is the port's determinism contract
    /// (documented MP port design),
    /// a remote human consumes weapon/ammo and takes armour by the shared `mp_armour` (refused at
    /// 50, the game's armour rule [M 0x18e650:220-225]). The bots: `Pickup_Handler`'s class-2 arm
    /// (`BotBrain::take_pickup`) and its visit stamp [M 0x18e650:34-130]. The game's bot arm
    /// handles kinds 0/1/3 and consumes every other kind unhandled [M 0x18e650:117-118 → 334-348];
    /// the port's bot pickup layer (the visit rows `step_mp_bots` builds, the arsenal's
    /// `BotArsenal::take_pickup`) models the weapon/ammo/armour kinds, so the taker loop offers
    /// the bots exactly those kinds. Pickups taken here skip `update_pickups`'
    /// own touch (`Drones::mp_shared_pickups`).
    pub fn step_mp_pickups(&mut self) {
        use nf_game::pickup::kind;
        let Some(m) = self.mp.as_ref() else { return };
        self.drones.mp_shared_pickups = true;
        let online = self.net_rng.is_some() || self.net_rng_shared.is_some();
        let local = self.mp_local_slot;
        let humans = (m.settings.human_count as usize).min(nf_net::MAX_AGENTS);
        let frame = self.drones.frame;
        enum Taker { Local, Remote(usize), Bot(usize) }
        for k in 0..self.drones.pickups.len() {
            let taker = {
                let p = &self.drones.pickups[k];
                if p.gone || p.state == 2 { continue; }
                let Some(m) = self.mp.as_ref() else { return };
                let mut taker = None;
                for slot in 0..humans {
                    let body = if slot == local { Some(&self.bond) } else { self.mp_bonds[slot].as_ref() };
                    let Some(b) = body else { continue };
                    let alive = if online {
                        self.mp_dead_ticks[slot] == 0 && self.mp_hp[slot] > 0.0
                    } else {
                        slot != local || self.drones.health.health > 0.0
                    };
                    if !alive { continue; }
                    if p.touches(&self.world, b.cel, b.pos, b.sphere_centre()) {
                        taker = Some(if slot == local { Taker::Local } else { Taker::Remote(slot) });
                        break;
                    }
                }
                if taker.is_none() && matches!(p.kind, kind::WEAPON | kind::AMMO | kind::ARMOUR) {
                    for (i, bot) in m.bots.iter().enumerate() {
                        if bot.dead { continue; }
                        // The game's bot loop measures the distance from the bot's `obj+0x30`
                        // and runs the line of sight from the same `obj+0x30` [M Pickup_Update
                        // 0x18e148:240-244] — unlike the players' loop, whose line runs from the
                        // world sphere at obj+0x80 [M 0x18e148:210-214]. `brain.body.pos` is the
                        // port's obj+0x30 (the body root), so the port's distance+line pair is
                        // the bot arm's own pair.
                        let root = bot.brain.body.pos;
                        if p.touches(&self.world, bot.brain.body.cel, root, root) {
                            taker = Some(Taker::Bot(i));
                            break;
                        }
                    }
                }
                taker
            };
            let Some(taker) = taker else { continue };
            let (pkind, pweapon, prounds) = {
                let p = &self.drones.pickups[k];
                (p.kind, p.weapon, p.rounds)
            };
            match taker {
                Taker::Local => {
                    if online { self.drones.health.armour = self.mp_armour[local]; }
                    let force = online && pkind != kind::ARMOUR;
                    let d = nf_game::drone::Data { tables: &self.tables, props: &self.props, scripts: &self.scripts, weapons: &self.weapons, world: &self.world, level: self.level, sequences: &self.seqs, skins: &self.skins, skeletons: &self.skeletons, nav: &self.nav, covers: &self.covers, tuning: &self.tuning, hitboxes: &self.hitboxes, volumes: &self.volumes, sfx: self.sfx.as_ref(), models: &self.models, surfaces: self.surfaces.as_ref(), voices: self.voices.as_deref(), difficulty: self.drones.difficulty,
                        ai_points: &self.ai_points };
                    self.drones.take_pickup(&d, k, frame, force);
                    if online { self.mp_armour[local] = self.drones.health.armour; }
                }
                Taker::Remote(slot) => {
                    if pkind == kind::ARMOUR {
                        if self.mp_armour[slot] < 50.0 {
                            self.mp_armour[slot] = 50.0;
                            self.drones.consume_pickup(k, frame);
                        }
                    } else {
                        self.drones.consume_pickup(k, frame);
                    }
                }
                Taker::Bot(i) => {
                    let Some(t) = self.mp_bot_weapons.as_ref() else { continue };
                    let Some(m) = self.mp.as_mut() else { return };
                    let team = m.settings.players.get(m.bots[i].slot).map_or(0, |p| p.team);
                    let mut health = m.bots[i].health;
                    let r = m.bots[i].brain.take_pickup(t, pkind, pweapon, prounds, team, &mut health);
                    m.bots[i].health = health;
                    if r == nf_game::mp::botweap::BotTake::Refused { continue; }
                    // the visit stamp (`setPickupVisitTime` [M 0x18e650:126-127]) into the
                    // bots' pickup row (the kind-filtered list `step_mp_bots` builds)
                    let row = self.drones.pickups[..k].iter()
                        .filter(|p| matches!(p.kind, kind::WEAPON | kind::AMMO | kind::ARMOUR)).count();
                    let stamp = m.bots[i].brain.pickup_visit_stamp(row);
                    if let Some(bp) = self.mp_bot_pickups.get_mut(row) {
                        if i < bp.visit.len() { bp.visit[i] = stamp; }
                    }
                    if std::env::var_os("NF_BOTLOG").is_some() {
                        eprintln!("[botlog] f{} bot{} took pickup {} (kind {} weapon {:#x} rounds {}) → weapon {:#x} armour {} hp {:.0}",
                            frame, i, k, pkind, pweapon, prounds, m.bots[i].brain.vars.weapon_id, m.bots[i].brain.vars.arsenal.armour, health);
                    }
                    self.drones.consume_pickup(k, frame);
                }
            }
        }
    }

    // The bots' observation API (the live tests read the per-bot table through these).
    pub fn mp_bot_count(&self) -> usize {
        self.mp.as_ref().map_or(0, |m| m.bots.len())
    }
    pub fn mp_bot_positions(&self) -> Vec<[f32; 3]> {
        self.mp.as_ref().map_or(Vec::new(), |m| m.bots.iter().map(|b| b.brain.pos).collect())
    }
    pub fn mp_bot_shots(&self, i: usize) -> u32 {
        self.mp.as_ref().map_or(0, |m| m.bots[i].shots)
    }
    pub fn mp_bot_deaths(&self, i: usize) -> u32 {
        self.mp.as_ref().map_or(0, |m| m.bots[i].deaths)
    }
    pub fn mp_bot_respawns(&self, i: usize) -> u32 {
        self.mp.as_ref().map_or(0, |m| m.bots[i].respawns)
    }
    pub fn mp_bot_score(&self, i: usize) -> i32 {
        self.mp.as_ref().map_or(0, |m| {
            let slot = m.settings.human_count as usize + i;
            if slot < m.game.slots.len() { m.game.slots[slot].score as i32 } else { 0 }
        })
    }


    /// Tick the remote agents' movement with their session inputs (one fixed 60 Hz tick). The
    /// remote path is `Bond::tick_ctx` — the same movement core the local path runs (the local
    /// machine's full weapon/view pipeline stays local-only in this first cut; see
    /// docs/systems/net-match-host.md's *Deviations*). `water` is `None` for the remotes (no swim
    /// state machine on a remote body yet) — the same `None` on every machine, so the lockstep
    /// holds.
    /// Online: this machine's own slot is dead in the per-slot combat state (`mp_dead_ticks`),
    /// which every machine holds identically. Offline (no private generator) always false.
    pub fn net_dead(&self) -> bool {
        (self.net_rng.is_some() || self.net_rng_shared.is_some())
            && self.mp_dead_ticks.get(self.mp_local_slot).is_some_and(|&d| d > 0)
    }

    pub fn step_net_remotes(&mut self, inputs: &[nf_game::player::Input; nf_net::MAX_AGENTS], local: usize) {
        let _p = nf_game::prof::scope("net: step_net_remotes");
        // mp-world-sync: `step_bond` already moved them this tick (it had the session's row).
        if std::mem::take(&mut self.net_remotes_done) { return; }
        self.step_net_remotes_inner(inputs, None, local);
    }

    /// The remotes' movement tick. `row` (the session's raw inputs, when the caller has them)
    /// adds the parts the local pipeline drives from the fire lane: the Q-Claw's latch on the
    /// held claw + the trigger (`Player_SetGrapplePoint` 0x195da0 — the local path's arm in
    /// `step_bond`, with the same session `held` weapon) and the reel's release on the trigger
    /// (the local body's `input.fire = trigger`) — a grapple was the local machine's alone.
    fn step_net_remotes_inner(&mut self, inputs: &[nf_game::player::Input; nf_net::MAX_AGENTS], row: Option<&[nf_net::Input; nf_net::MAX_AGENTS]>, local: usize) {
        for slot in 0..nf_net::MAX_AGENTS {
            if slot == local { continue; }
            let Some(b) = self.mp_bonds[slot].as_mut() else { continue };
            // A dead slot's body stands where it died until the respawn, on every machine — the
            // local machine's own body freezes on the same `mp_dead_ticks` (`Sim::net_dead`; the
            // game's dead branch moves no body [M `Player_Update` 0x193600:316-319]). The remote
            // replica used to keep walking on the dead player's live input while the owner's body
            // froze: the two-window "desync when I move" with bots (a bot kill, then any input).
            if self.mp_dead_ticks[slot] > 0 { self.mp_body_speed[slot] = 0.0; continue; }
            let before = b.pos;
            // mp-combat-weapons: a slot riding its guided rocket runs no arms — the local body's
            // `step_bond` zeroes its input on the same ticks (the shared `MpCombat::ride`).
            let input = if self.mp_combat.riding(slot) { nf_game::player::Input::default() } else { inputs[slot] };
            // The local body's movement core, step for step (`step_bond`): the ground ride, the
            // stateful cel relink on the last tick's move segment (the collision queries are
            // scoped to `cel` — a replica that kept its spawn cel collided differently from the
            // owner's body: the second two-window divergence, a crouch-walk along a wall, 0.004
            // units in one tick), the water level from that cel, `tick_ctx`, the `prev_pos` copy.
            let prev_tick_pos = b.pos;
            {
                let p = b.pos;
                if let Some(g) = self.world.ground_motion(p, [p[0], p[1] - 1.6347, p[2]]) {
                    let (sn, cs) = g.turn.sin_cos();
                    let (x, z) = (p[0] + g.delta[0] - g.pivot[0], p[2] + g.delta[2] - g.pivot[2]);
                    b.pos = [g.pivot[0] + x * cs + z * sn, p[1] + g.delta[1], g.pivot[2] - x * sn + z * cs];
                    b.yaw += g.turn;
                }
            }
            let cel = if (0..3).any(|k| (b.pos[k] - b.prev_pos[k]).abs() > 3.0) {
                b.prev_pos = b.pos;
                self.world.find_cel(b.pos)
            } else {
                self.world.player_cel_relink(b.cel, b.prev_pos, b.pos, b.pos)
            };
            let water = cel.and_then(|cell| self.room_cell.iter().position(|&c| c == Some(cell)))
                .map(|r| self.cels[r].water).filter(|w| *w > -400.0);
            b.cel = cel;
            let mut input = input;
            if let Some(r) = row {
                let mut trigger = r[slot].trigger;
                if trigger && b.substate == nf_game::player::SS_WALK && b.grapple.is_none()
                    && nf_game::gadget::is_grapple(r[slot].held)
                {
                    let (head, dir) = (b.head(), b.look());
                    if let Some(point) = nf_game::gadget::pick_grapple_point(&self.drones.grapple_points, head, dir, nf_game::gadget::grapple_range(&self.weapons, r[slot].held)) {
                        b.grapple = Some(nf_game::gadget::GrappleState { point, phase: 1, pull: 0.0 });
                        b.substate = nf_game::player::SS_GRAPPLE;
                        b.vel = [0.0; 3];
                        trigger = false;
                    }
                }
                input.fire = trigger;
            }
            let ctx = nf_game::player::LadderCtx {
                ladders: &self.ladders, wires: &self.wires, wire_anims: Some(&self.wire_anims),
                water, sink: input.crouch, creepwalls: &self.creepwalls, level_bump_5fb: false,
                straddle: &[], // MP remotes carry no per-body cel link yet — no scoping.
                bot_capsules: &[], // the remotes are bodies, not bots — no actor-actor push.
            };
            b.tick_ctx(&self.world, &ctx, &input);
            // mp-remote-fx: the body's jump launch / head bump sounds play at the flush.
            self.mp_fx.body_sounds.extend(b.sfx_out.drain(..));
            b.prev_pos = prev_tick_pos;
            self.mp_body_speed[slot] = ((b.pos[0] - before[0]).powi(2) + (b.pos[2] - before[2]).powi(2)).sqrt() * 60.0;
        }
    }

    // ── mp-world-sync (agent 9): the arenas' world interactions online ─────────────────────────
    // [M-grounded port design — the PS2's four players were one machine's objects and the game's
    // own walks already treat every player alike: `Pickup_Update` 0x18e148:202-253 loops the
    // players' glb_players[0..4] and the bots' MPGame slots in one walk, `Player_Activate`
    // 0x1a5170 runs per player, `Player_HandlePain` 0x1902c8 per player]. Online every machine
    // runs the world from
    // the same bodies: `Drones::net_bodies` (the pickup take, the trigger touchers, the doors'
    // actors), `net_use_pass` (every slot's use key) and `step_net_world` (every slot's world
    // damage and its shots at the breakables/destroyables); `net_state_hash` folds the world's
    // state so a divergence is detected. Docs: docs/systems/net-world-sync.md.

    /// The body of a human slot (the local slot's `bond`, the others' parked replicas).
    fn net_body(&self, slot: usize) -> Option<&nf_game::player::Bond> {
        if slot == self.mp_local_slot { Some(&self.bond) } else { self.mp_bonds.get(slot)?.as_ref() }
    }

    /// Every body the shared world sees now: the human slots in slot order, then the bots in
    /// list order (`Pickup_Update` 0x18e148's own order — the players' loop, then the bots').
    pub fn net_bodies_now(&self) -> Vec<nf_game::drone::NetBody> {
        let cel_of = |p: [f32; 3]| self.cel_at(p).and_then(|room| self.room_cell.get(room).copied().flatten());
        let mut v = Vec::new();
        for slot in 0..nf_net::MAX_AGENTS {
            let Some(b) = self.net_body(slot) else { continue };
            v.push(nf_game::drone::NetBody {
                slot, bot: false, local: slot == self.mp_local_slot,
                alive: self.mp_dead_ticks[slot] == 0 && self.mp_hp[slot] > 0.0,
                pos: b.pos, sph: b.sphere_centre(), head: b.head(), cel: cel_of(b.pos),
                hanging: matches!(b.substate, nf_game::player::SS_WIRE | nf_game::player::SS_ZIPLINE),
                armour: self.mp_armour[slot],
            });
        }
        if let Some(m) = self.mp.as_ref() {
            for bt in &m.bots {
                let e = bt.brain.pos;
                let pos = [e[0], e[1] - BOT_EYE, e[2]];
                // The bot's world sphere centre [M Control_BuildWorldSph 0x133070:33-44 — obj+0x80
                // is the local centre through the object matrix]: the [L]-backed standing local
                // offset rotated by the bot's own yaw. (The old `sph: pos` was the pre-fix
                // stand-in; the pickup-take test [M Pickup_Update 0x18e148:212-214] reads obj+0x80
                // for every body, bots included.)
                let sph = nf_game::player::Bond::sphere_centre_at(pos, bt.brain.yaw);
                v.push(nf_game::drone::NetBody {
                    slot: bt.slot, bot: true, local: false, alive: !bt.dead,
                    pos, sph, head: e, cel: cel_of(pos), hanging: false, armour: 0.0,
                });
            }
        }
        v
    }

    /// Every slot's use key this tick, in slot order (`Player_Activate` 0x1a5170 per player):
    /// the doors its sphere meets (`Door_Activate`), the deferred trigger half, the class-0x25
    /// SS pick (`SS_Activate` 0x1a9ea8 — the state + mask-4 gate, the kind:=1 latch, the channel
    /// [M 0x1a9ea8:6-12]). The reach is the local path's documented shape (`run`'s SS arm): the
    /// game's pick is a 1.0-radius `Collide_SphereIntersect` at head+dir walking every hit of the
    /// use classes [M Player_Activate 0x1a5170:84-88, 120-135]; the port's per-SS probe (proj ≥ 0,
    /// nearest within 2.2, one activation) is that arm's documented approximation, kept identical
    /// across the SP and net paths.
    /// The mounted guns are not created in an MP match (`GunImp_Create` 0x131f20:56-58 returns
    /// unless `MPSettings+0x1B8`), so no slot can mount one.
    fn net_use_pass(&mut self, row: &[nf_net::Input; nf_net::MAX_AGENTS]) {
        let frame = self.drones.frame;
        for slot in 0..nf_net::MAX_AGENTS {
            if !row[slot].use_key || self.mp_dead_ticks[slot] > 0 { continue; }
            let Some(b) = self.net_body(slot) else { continue };
            let (head, look) = (b.head(), b.look());
            let (evs, _) = self.drones.doors.player_use(head, look, &mut self.drones.channels, frame, &self.world);
            for (id, pos) in evs { self.drones.audio.play3d(id as u32, pos, None); }
            self.drones.net_use_triggers.push((head, look));
            let mut best: Option<(f32, usize)> = None;
            for (i, s) in self.drones.sss.list.iter().enumerate() {
                if s.state == 0 || s.mask & 4 == 0 { continue; }
                let p = s.pos();
                let to = [p[0] - head[0], p[1] - head[1], p[2] - head[2]];
                let proj = to[0] * look[0] + to[1] * look[1] + to[2] * look[2];
                if proj < 0.0 { continue; }
                let cl = [head[0] + look[0] * proj, head[1] + look[1] * proj, head[2] + look[2] * proj];
                let dd = ((p[0] - cl[0]).powi(2) + (p[1] - cl[1]).powi(2) + (p[2] - cl[2]).powi(2)).sqrt();
                if dd > 2.2 { continue; }
                if best.map_or(true, |(bd, _)| dd < bd) { best = Some((dd, i)); }
            }
            if let Some((_, i)) = best {
                let mut ch = std::mem::take(&mut self.drones.channels);
                self.drones.sss.list[i].activate(&mut ch, frame);
                self.drones.channels = ch;
            }
        }
    }

    /// One world-damage application to a human slot's shared health (`Player_HandlePain`
    /// 0x1902c8's body: × the level's damage factor, armour absorbing only for pain type 0
    /// [M 0x1902c8:84-104]); a death goes through `MP_PlayerKilled` 0x188f70 like the combat's
    /// (the credit is the attacker when another slot caused it; `explosive` = pain types 4..7).
    fn net_world_pain(&mut self, slot: usize, damage: f32, pain_type: u32, killer: Option<usize>) {
        if self.mp_dead_ticks[slot] > 0 || self.mp_hp[slot] <= 0.0 || damage <= 0.0 { return; }
        let d = damage * nf_game::bullet::damage_factor(self.level, self.drones.difficulty);
        let absorbed = if pain_type == 0 { d.min(self.mp_armour[slot]) } else { 0.0 };
        self.mp_armour[slot] -= absorbed;
        self.mp_hp[slot] = (self.mp_hp[slot] - (d - absorbed)).max(0.0);
        if slot == self.mp_local_slot {
            // The pain flash (presentation; `Player_HandlePain`'s +0x968 write).
            self.drones.health.flash = (self.drones.health.flash + (d * 21.333).min(128.0)).min(255.0);
        }
        if self.mp_hp[slot] <= 0.0 {
            self.mp_dead_ticks[slot] = 1;
            let k = killer.filter(|&k| k != slot);
            let explosive = (4..=7).contains(&pain_type);
            if let Some(m) = self.mp.as_mut() { m.player_killed(slot, k, explosive, false, &mut self.drones.rng); }
            eprintln!("[mp] slot {slot} died to the world (pain type {pain_type}, frame {})", self.drones.frame);
        }
    }

    /// The shared world's per-body consequences of the tick, on the shared stream (the end of
    /// `step_bond`, online):
    /// 1. the pickup takes' shared effects — a human's armour pickup sets the slot's armour to 50
    ///    [M Pickup_Handler 0x18e650:220-225]; a bot's raises its health by 20 to its max and its
    ///    armour byte by the pickup's amount, capped 50 (the bot arm's case 3 [M 0x18e650:119-133;
    ///    `BOTWEAP_setDroneArmour` 0x12af70 is an empty body]); the bots' weapon/ammo inventory is
    ///    the brain's (`BotArsenal::take_pickup` = the class-2 arm's cases 0/1 [M 0x18e650:61-116]);
    /// 2. every live body's hurt-volume contact (`Player_DealWithObjHit` 0x194458's class-0x27
    ///    arm), landing damage (type 6, `Player_CollisionHandler` 0x1949a8:117-126) and drowning
    ///    (type 7, `Player_MonitorAir` 0x191a70:136-143) into its slot's health;
    /// 3. every slot's shot (`Input::shot`, the same ray `step_net_combat` casts) against the
    ///    breakables (`Break_Update` 0x12bfd8 — the damage subtracts; < 0 dies) and destroyables
    ///    (`Destroy_CollisionHandler` 0x12bd48; the death writes its channel and, for a non-zero
    ///    type byte, `Destroy_Smash` 0x12bb60's blast — `Explode_Propagate` onto every body).
    ///    The object takes the shot only when it is nearer than every body the combat's sphere
    ///    test would hit — the combat itself is untouched. The contact is the shot's segment
    ///    against the object's bounds grown 0.1, the shape `props.rs`'s `bullet_hit` uses for the
    ///    game's bullet-vs-glist-bounds contact [M Destroy_CollisionHandler 0x12bd48 subtracts
    ///    the damage from the record's +0x04 and kills below 0; the 0.1 growth stands in for the
    ///    bullet's sphere — live check offered in the report];
    /// 4. a dead slot loses its armour (`MP_ReSpawn` 0x185368 re-inits the player).
    fn step_net_world(&mut self, row: &[nf_net::Input; nf_net::MAX_AGENTS]) {
        if self.mp.is_none() { return; }
        // 1. The takes.
        self.net_last_takes = std::mem::take(&mut self.drones.net_takes);
        for &(k, slot, bot) in &self.net_last_takes.clone() {
            let Some(pk) = self.drones.pickups.get(k) else { continue };
            let (kind, pweapon, prounds) = (pk.kind, pk.weapon, pk.rounds);
            eprintln!("[p-world] frame {}: {} {slot} took pickup {k} (kind {kind})", self.drones.frame, if bot { "bot" } else { "slot" });
            if bot {
                // mp-bot-weapons: the bot's weapon layer takes every pickup the shared walk
                // recorded — `Pickup_Handler`'s class-2 arm (`BotBrain::take_pickup`): kinds 0/1
                // the arsenal update + the preferred-weapon switch request [M 0x18e650:61-116],
                // kind 3 the armour take — the drone's health +20 clamped to BOT_vars+0xA4 AND
                // the armour byte +0x769 += the pickup's amount capped 50 [M 0x18e650:119-133;
                // `BOTWEAP_setDroneArmour` 0x12af70 is an empty body], any other kind consumed
                // unhandled [M 0x18e650:117-118]. The visit stamp follows for every handled kind
                // [M 0x18e650:135-136]. The pickup itself is already consumed by the shared walk
                // (the online walk consumes at the take so every machine's pickup state moves in
                // the same tick; the game's local-MP order is consume-after-success in the
                // handler's epilogue [M 0x18e650:334-348] — a refusal there leaves the pickup);
                // a refused take here is the brain ignoring it, the pickup stays consumed.
                let bi = self.mp.as_ref().and_then(|m| m.bots.iter().position(|b| b.slot == slot));
                let Some(i) = bi else { continue };
                let Some(t) = self.mp_bot_weapons.as_ref() else { continue };
                let Some(m) = self.mp.as_mut() else { continue };
                let team = m.settings.players.get(m.bots[i].slot).map_or(0, |p| p.team);
                let mut health = m.bots[i].health;
                let r = m.bots[i].brain.take_pickup(t, kind, pweapon, prounds, team, &mut health);
                m.bots[i].health = health;
                if r == nf_game::mp::botweap::BotTake::Refused { continue; }
                // the visit stamp (`setPickupVisitTime` [M 0x18e650:126-127]) into the
                // bots' pickup row (the kind-filtered list `step_mp_bots` builds)
                let row = self.drones.pickups[..k].iter()
                    .filter(|p| matches!(p.kind, nf_game::pickup::kind::WEAPON | nf_game::pickup::kind::AMMO | nf_game::pickup::kind::ARMOUR)).count();
                let stamp = m.bots[i].brain.pickup_visit_stamp(row);
                if let Some(bp) = self.mp_bot_pickups.get_mut(row) {
                    if i < bp.visit.len() { bp.visit[i] = stamp; }
                }
                if std::env::var_os("NF_BOTLOG").is_some() {
                    eprintln!("[botlog] f{} bot{} took pickup {} (kind {} weapon {:#x} rounds {}) → weapon {:#x} armour {} hp {:.0}",
                        self.drones.frame, i, k, kind, pweapon, prounds, m.bots[i].brain.vars.weapon_id, m.bots[i].brain.vars.arsenal.armour, health);
                }
            } else if kind == nf_game::pickup::kind::ARMOUR && slot < nf_net::MAX_AGENTS {
                self.mp_armour[slot] = 50.0;
            }
            // mp-shot-gate: the taker's shared gate inventory gets the same grant the local
            // inventory ran (docs/systems/mp-shot-gate.md) — a remote slot's pickups land in
            // the shared state identically on every machine.
            if !bot && slot < nf_net::MAX_AGENTS {
                if let Some(pk) = self.drones.pickups.get(k).cloned() {
                    let mut armour_scratch = self.mp_armour[slot];
                    self.mp_combat.gate_take(slot, &pk, &self.weapons, &mut armour_scratch, self.drones.frame);
                }
            }
        }
        // 2. The world damage, slot order.
        let mut hits: Vec<(usize, f32, u32, Option<usize>)> = Vec::new();
        for slot in 0..nf_net::MAX_AGENTS {
            if self.mp_dead_ticks[slot] > 0 { continue; }
            let Some(b) = self.net_body(slot) else { continue };
            let (pos, fall, drown) = (b.pos, b.fall_damage, b.drown_damage);
            for h in self.hurts.iter_mut() {
                if let Some((dmg, ty)) = h.update(self.drones.channels.on(h.gate as u32), pos) {
                    hits.push((slot, dmg, ty as u32, None));
                }
            }
            if let Some(d) = fall { hits.push((slot, d, 6, None)); }
            if let Some(d) = drown { hits.push((slot, d, 7, None)); }
        }
        for (slot, d, ty, k) in hits.drain(..) { self.net_world_pain(slot, d, ty, k); }
        // 3. The shots at the breakables / destroyables.
        let frame = self.drones.frame;
        for shooter in 0..nf_net::MAX_AGENTS {
            if !row[shooter].shot || self.mp_dead_ticks[shooter] > 0 { continue; }
            let Some(b) = self.net_body(shooter) else { continue };
            let head = b.head();
            let (sy, cy) = b.yaw.sin_cos();
            let (sp, cp) = b.pitch.sin_cos();
            let dir = [cp * sy, sp, cp * cy];
            let range = 200.0;
            let mut end = [head[0] + dir[0] * range, head[1] + dir[1] * range, head[2] + dir[2] * range];
            if let Some((t, _, _)) = self.world.segment_surface_bullet(head, end) {
                end = [head[0] + (end[0] - head[0]) * t, head[1] + (end[1] - head[1]) * t, head[2] + (end[2] - head[2]) * t];
            }
            // The nearest body on the ray (the combat's spheres) — a body in front takes the shot.
            let mut body_t = f32::MAX;
            for victim in 0..nf_net::MAX_AGENTS {
                if victim == shooter || self.mp_dead_ticks[victim] > 0 { continue; }
                let Some(v) = self.net_body(victim) else { continue };
                for hgt in [0.5f32, 1.0, 1.5] {
                    if let Some(t) = ray_body_hit(head, end, [v.pos[0], v.pos[1] + hgt, v.pos[2]], 0.45) { body_t = body_t.min(t); }
                }
            }
            if let Some(m) = self.mp.as_ref() {
                for bt in m.bots.iter().filter(|b| !b.dead) {
                    if let Some(t) = ray_body_hit(head, end, bt.brain.pos, 0.6) { body_t = body_t.min(t); }
                }
            }
            // The nearest live breakable / destroyable box (grown 0.1, the props' contact rule).
            let mut best: Option<(f32, bool, usize)> = None; // (t, destroyable, index)
            // The evidenced reject [M 0x12bfd8:15 + 0x1ece30:24-33]: the weapon-definition word
            // against (filter | 0x106), dropped when the AND is non-zero (the laser 0x0002 and
            // the watch 0x0100 classes are rejected even on a no-filter pane; guns 0x0008 pass).
            let weapon_word = self.weapons.records.get(row[shooter].held as usize)
                .map_or(0u16, |r| u16::from_le_bytes([r[0x10], r[0x11]]));
            for (i, br) in self.drones.breaks.list.iter().enumerate().filter(|(_, b)| !b.dead) {
                if weapon_word & (br.filter | 0x106) != 0 { continue; }
                if let Some(t) = seg_aabb(head, end, br.lo, br.hi, 0.1) { if best.is_none_or(|(bt, _, _)| t < bt) { best = Some((t, false, i)); } }
            }
            for (i, de) in self.drones.props.destroy.iter().enumerate().filter(|(_, d)| !d.dead) {
                if let Some(t) = seg_aabb(head, end, de.lo, de.hi, 0.1) { if best.is_none_or(|(bt, _, _)| t < bt) { best = Some((t, true, i)); } }
            }
            let Some((t, destroyable, i)) = best else { continue };
            if t > body_t { continue; }
            let damage = self.weapons.records.get(row[shooter].held as usize)
                .map_or(10.0, |r| f32::from_le_bytes(r[0x0c..0x10].try_into().unwrap()));
            if !destroyable {
                let dead = {
                    let br = &mut self.drones.breaks.list[i];
                    br.health -= damage;
                    br.health < 0.0
                };
                if dead {
                    // Break_Kill 0x12b970 through the shared kill path — the kill spawns the
                    // pieces (crate::shatter, docs/systems/break-debris.md) and returns the
                    // effect (the silent-kill case with no collision records returns None
                    // [M 0x12b970:59-63]).
                    if let Some(k) = self.drones.breaks.kill(i, head, end, &mut self.drones.rng) {
                        self.drones.break_kills.push((k.placement, k.kind, k.sound));
                        self.drones.audio.play3d(u32::from(k.sound), k.pos, None);
                    }
                }
                continue;
            }
            let de = &mut self.drones.props.destroy[i];
            de.health -= damage;
            if de.health >= 0.0 { continue; }
            de.dead = true;
            let (channel, kind, blast_damage, lo, hi) = (de.channel, de.kind, de.blast_damage, de.lo, de.hi);
            self.drones.channels.set(channel as u32, 1, frame);
            if kind == 0 { continue; }
            // `Destroy_Smash` 0x12bb60's blast: radius = the obj's +0x8C sphere × the type
            // multiplier {1→4, 2→2, else 3} [M 0x12bcdc `lwc1 $f12, 0x8c($s0)`;
            // 0x12bc74/0x12bc84/0x12bc94], the script {1→0x06000027, 2→0x0600004f, else
            // 0x06000052} [M 0x12bc70-0x12bc98], the damage = the record's info+0x08 float (the
            // level's param 5) [M 0x12bd04 `lwc1 $f13, 8($s1)`; a second float arg carries
            // kind×10.0, 0x12bd0c]. Position: the bounds centre. The +0x8C sphere itself is
            // props.rs's half-diagonal approximation (its comment carries the live check).
            let c = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
            let ext = [(hi[0] - lo[0]) / 2.0, (hi[1] - lo[1]) / 2.0, (hi[2] - lo[2]) / 2.0];
            let radius = (ext[0] * ext[0] + ext[1] * ext[1] + ext[2] * ext[2]).sqrt() * match kind { 1 => 4.0f32, 2 => 2.0, _ => 3.0 };
            let script = match kind { 1 => 0x0600_0027, 2 => 0x0600_004f, _ => 0x0600_0052 };
            // The look only (damage 0): the drones' blast pass plays the script next tick.
            // explode-debris-spawn: Destroy_Smash's 12th arg is 0 [M 0x12bcf8].
            self.drones.pending_blasts.push((c, radius, 0.0, usize::MAX, 0, script, radius, 0));
            let bots: Vec<(usize, [f32; 3], f32)> = self.mp.as_ref().map_or(Vec::new(), |m| m.bots.iter().enumerate()
                .filter(|(_, b)| !b.dead).map(|(j, b)| (j, [b.brain.pos[0], b.brain.pos[1] - BOT_EYE, b.brain.pos[2]], 0.4)).collect());
            for slot in 0..nf_net::MAX_AGENTS {
                if self.mp_dead_ticks[slot] > 0 { continue; }
                let Some(v) = self.net_body(slot) else { continue };
                let hits = nf_game::explode::propagate(c, radius, blast_damage, shooter,
                    &nf_game::explode::Targets { drones: &[], player: v.pos, breaks: &[], destroys: &[], copters: &[] });
                if let Some(ph) = hits.player { self.net_world_pain(slot, ph, 0, Some(shooter)); }
            }
            let hits = nf_game::explode::propagate(c, radius, blast_damage, shooter,
                &nf_game::explode::Targets { drones: &bots, player: [0.0, -1.0e9, 0.0], breaks: &[], destroys: &[], copters: &[] });
            for (j, dmg) in hits.drones {
                let Some(m) = self.mp.as_mut() else { break };
                let bt = &mut m.bots[j];
                if bt.dead { continue; }
                bt.health -= dmg;
                if bt.health <= 0.0 {
                    let victim_slot = bt.slot;
                    bt.health = 0.0;
                    bt.dead = true;
                    bt.deaths += 1;
                    bt.respawn_at = frame + 180;
                    m.player_killed(victim_slot, Some(shooter), false, false, &mut self.drones.rng);
                }
            }
        }
        // 4. The dead lose their armour; this machine's HUD armour is its slot's.
        for slot in 0..nf_net::MAX_AGENTS { if self.mp_dead_ticks[slot] > 0 { self.mp_armour[slot] = 0.0; } }
        self.drones.health.armour = self.mp_armour[self.mp_local_slot];
    }

    /// An MP body's (position, yaw, horizontal speed per second) now, None = not drawn (the local
    /// slot, an empty slot, no match).
    pub fn mp_body_state(&self, slot: usize) -> Option<([f32; 3], f32, f32)> {
        let s = self;
        let m = s.mp.as_ref()?;
        let humans = m.settings.human_count as usize;
        if slot < humans {
            if slot >= nf_net::MAX_AGENTS { return None; }
            // A dead slot's body stays drawn where it fell (the death script, mp-body-anim) until the
            // respawn moves it (`step_net_combat`).
            if slot == s.mp_local_slot { return None; }
            let b = s.mp_bonds[slot].as_ref()?;
            Some((b.pos, b.yaw, s.mp_body_speed[slot]))
        } else {
            let bot = m.bots.iter().find(|b| b.slot == slot)?;
            // mp-remote-fx: a dead bot lies in its death pose (`Sim::mp_body_death`), not standing.
            if bot.dead { return None; }
            // The bot's position is its eye (BOT_EYE above the feet); the body draws from the feet.
            let p = bot.brain.pos;
            // The draw's yaw is Bond's convention (forward = (sin, cos), `object_matrix`'s frame);
            // the brain's is the drone atan2(dz, dx) one (forward = (cos, sin)) — its walking body
            // carries HALF_PI − yaw (`BotBrain::walk_body`). Drawing the brain yaw mirrored every
            // bot's facing about the diagonal (a bot walking +x faced +z).
            Some(([p[0], p[1] - BOT_EYE, p[2]], nf_game::trig::HALF_PI - bot.brain.yaw, s.mp_body_speed[slot]))
        }
    }

    /// The body root's height above its feet (`obj+0xCC`: standing `ROOT_HEIGHT`, crouched 0.55) —
    /// a human's remote `Bond::cc`, a bot's walking body's.
    pub fn mp_body_cc(&self, slot: usize) -> f32 {
        let standing = nf_game::player::ROOT_HEIGHT;
        let Some(m) = self.mp.as_ref() else { return standing };
        if slot < m.settings.human_count as usize {
            self.mp_bonds.get(slot).and_then(|b| b.as_ref()).map_or(standing, |b| b.cc)
        } else {
            m.bots.iter().find(|b| b.slot == slot).map_or(standing, |b| if b.brain.body.cc > 0.0 { b.brain.body.cc } else { standing })
        }
    }

    /// mp-body-anim: one MP body's local pose from its script list (`mp_anim[slot]`), composed as
    /// `AnimFrameResolve` 0x1c35f8 blends — the layers in stamp order, `result = first`, then
    /// `result = blend(result, next, 1 − weight(previous))`. A walk layer's frame comes from the walk
    /// sequence's z-only distance table at the body's walk distance (`AnimDistanceTableDistanceToFrame2`
    /// 0x1c4270; a negative step plays it backwards) and its run is synced to it (mode 2) at the run
    /// weight. None when the list is empty or a script is missing (the caller's fallback).
    pub fn mp_body_local_pose(&self, slot: usize, skel: &nf_formats::character::Skeleton, skin: &nf_formats::character::Skin) -> Option<nf_game::anim::Pose> {
        use nf_game::mp::body_anim::LayerKind;
        let a = self.mp_anim.get(slot)?;
        if a.layers.is_empty() { return None; }
        let factors = a.blend_factors();
        let mut out: Option<nf_game::anim::Pose> = None;
        for (l, &f) in a.layers.iter().zip(&factors) {
            let pose = match l.kind {
                LayerKind::Idle(id) | LayerKind::Strafe(id) | LayerKind::OneShot(id) =>
                    nf_game::anim::script_pose_drone(&self.scripts, &self.seqs, skel, skin, id, l.time)?,
                LayerKind::Walk { walk, run, run_weight } => {
                    let walk_q = self.scripts.get(&walk)?.sequences().next().and_then(|(_, _, q)| self.seqs.get(&q))?;
                    let table = nf_game::anim::DistanceTable::build(walk_q, 4, true)?;
                    let mut w = a.phase.rem_euclid(table.total.max(1e-3));
                    let frame_a = table.distance_to_frame(&mut w, 0.0, true).max(1.0);
                    let mut pose = nf_game::anim::script_pose_drone(&self.scripts, &self.seqs, skel, skin, walk, frame_a)?;
                    let len_a = self.scripts.get(&walk)?.length.max(2) as f32;
                    if let Some(len_b) = self.scripts.get(&run).map(|r| r.length.max(2) as f32).filter(|_| run != walk) {
                        let frame_b = 1.0 + (frame_a - 1.0) / (len_a - 1.0) * (len_b - 1.0);
                        if let Some(rp) = nf_game::anim::script_pose_drone(&self.scripts, &self.seqs, skel, skin, run, frame_b) {
                            nf_game::anim::blend(&mut pose, &rp, run_weight.clamp(0.0, 1.0));
                        }
                    }
                    pose
                }
            };
            match out.as_mut() {
                None => out = Some(pose),
                Some(r) => nf_game::anim::blend(r, &pose, f.clamp(0.0, 1.0)),
            }
        }
        out
    }

    /// The weapon an MP body holds in its hand (the third-person mesh `weapon_data[w]+0x80` on datum
    /// `+0x50`, `AnimObjectDraw` 0x1c4490:256-266): a bot's is its brain's current weapon
    /// (`BOT_vars+0x768`, the id its shots fire — `bot_init` gives the P2K at spawn); a human's rides
    /// the session input (`mp_held`). The bots have no session input — `step_net_combat` copies the
    /// empty row's 0 into their `mp_held` every tick, which hid every bot's gun.
    pub fn mp_body_held(&self, slot: usize) -> u8 {
        if let Some(b) = self.mp.as_ref().and_then(|m| m.bots.iter().find(|b| b.slot == slot)) {
            return b.brain.vars.weapon_id;
        }
        match self.mp_held.get(slot).copied() {
            Some(0) | None => self.mp_last_held.get(slot).copied().unwrap_or(2),
            Some(w) => w,
        }
    }

    /// mp-combat-weapons: every MP body as the shared combat hits it — the human slots (the local
    /// slot's `bond`, the parked `mp_bonds`; a dead slot is not a target) and the bots (their
    /// eye `brain.pos`, the body BOT_EYE below it), in slot order.
    pub fn mp_combat_bodies(&self, local: usize) -> Vec<nf_game::mp::combat::Body> {
        let mut v = Vec::new();
        for slot in 0..nf_net::MAX_AGENTS {
            let b = if slot == local { Some(&self.bond) } else { self.mp_bonds[slot].as_ref() };
            let Some(b) = b else { continue };
            v.push(nf_game::mp::combat::Body { slot, pos: b.pos, head_y: b.head()[1], yaw: b.yaw, alive: self.mp_dead_ticks[slot] == 0 && self.mp_hp[slot] > 0.0 });
        }
        if let Some(m) = self.mp.as_ref() {
            for bt in &m.bots {
                let p = bt.brain.pos;
                v.push(nf_game::mp::combat::Body { slot: bt.slot, pos: [p[0], p[1] - BOT_EYE, p[2]], head_y: p[1], yaw: bt.brain.yaw, alive: !bt.dead });
            }
        }
        v
    }

    /// mp-combat-weapons: one damage event of the shared combat through the game's MP pain path.
    ///
    /// A human victim: a round's hit first registers (`Player_DealWithObjHit` 0x194458 case 5 →
    /// `MP_RegisterBulletHit` 0x1857c0 — the team game's friendly-fire gate, the last-shooter
    /// word); a blast reaches `Player_Hurt` 0x196060 directly (no gate). The assassination rule
    /// [M 0x194458:44-74]: the assassin's round on the target kills outright on the head (part 5)
    /// and on parts 1..4 from within 70° of the target's back. Then `Player_HandlePain` 0x1902c8:
    /// the MP multipliers (`mp_damage`), armour first, health below 1 → 0, the pain grunt's
    /// `Rand_Rand(4)` on the shared stream [M :111]. A bot victim: `BOT_handlePain` 0x125ed8's
    /// identical multipliers (the bots carry no armour here). A kill scores through
    /// `player_killed` [M MP_PlayerKilled 0x188f70] with the round/blast owner as the killer.
    pub fn mp_hurt(&mut self, h: nf_game::mp::combat::Hurt, local: usize) {
        use nf_game::mp::combat::{self as cb, Hurt};
        let frame = self.drones.frame;
        let Some(m) = self.mp.as_mut() else { return };
        let rules = cb::PainRules::new(&self.tuning, m.settings.location_damage != 0, m.settings.capture_reward != 0);
        let (victim, attacker, mut amount, part, dir, located) = match h {
            Hurt::Site { attacker, amount } => { m.damage_site(amount, attacker); return; }
            Hurt::Bullet { victim, attacker, amount, part, dir, .. } => (victim, attacker, amount, part, dir, true),
            Hurt::Blast { victim, attacker, amount, .. } => (victim, attacker, amount, cb::part::NONE, [0.0; 3], false),
        };
        // A bot.
        if let Some(j) = m.bots.iter().position(|b| b.slot == victim) {
            if m.bots[j].dead { return; }
            if located && !m.register_bullet_hit(victim, attacker) { return; }
            let d = cb::mp_damage(&rules, amount, part);
            m.bots[j].health -= d;
            if m.bots[j].health <= 0.0 {
                m.bots[j].health = 0.0;
                m.bots[j].dead = true;
                m.bots[j].deaths += 1;
                m.bots[j].respawn_at = frame + 180;
                m.player_killed(victim, Some(attacker), false, false, &mut self.drones.rng);
            }
            return;
        }
        if victim >= nf_net::MAX_AGENTS || self.mp_dead_ticks[victim] > 0 || self.mp_hp[victim] <= 0.0 { return; }
        // mp-remote-fx: a human hit is an effects event too (the victim's hit flash / sound /
        // direction glow on its own machine, the blood on the others) — draw/audio only.
        if let Some(v) = (if victim == local { Some(&self.bond) } else { self.mp_bonds[victim].as_ref() }).map(|b| b.pos) {
            let chest = [v[0], v[1] + 1.0, v[2]];
            let from = [chest[0] - dir[0], chest[1] - dir[1], chest[2] - dir[2]];
            self.mp_fx.events.push(crate::mpfx::MpShot { shooter: attacker, bot: false, weapon: 0, from, dir, end: chest,
                hit: crate::mpfx::MpShotHit::Human(victim), damage: amount });
        }
        if located {
            if !m.register_bullet_hit(victim, attacker) { return; }
            // The assassination rule [M 0x194458:44-74].
            if m.settings.scenario == nf_game::mp::scenario::ASSASSIN && m.assassin == Some(attacker) && m.assassin_target == Some(victim) {
                let vyaw = if victim == local { self.bond.yaw } else { self.mp_bonds[victim].as_ref().map_or(0.0, |b| b.yaw) };
                if part == cb::part::HEAD {
                    amount = self.mp_hp[victim];
                } else if (1..5).contains(&part) {
                    let a = nf_game::trig::atan2_approx(dir[0], dir[2]);
                    if (nf_game::trig::angle_difference(a, vyaw) * 57.295776).abs() < 70.0 { amount = self.mp_hp[victim]; }
                }
            }
        }
        let d = cb::mp_damage(&rules, amount, part);
        if !(d > 0.0) { return; }
        let (hp, armour) = cb::apply_pain(self.mp_hp[victim], self.mp_armour[victim], d);
        if std::env::var_os("NF_MPCOMBAT_LOG").is_some() {
            eprintln!("[mpcombat] frame {frame}: slot {attacker} -> slot {victim} {} part {part} raw {amount} dealt {d} hp {} -> {hp}", if located { "round" } else { "blast" }, self.mp_hp[victim]);
        }
        self.mp_hp[victim] = hp;
        self.mp_armour[victim] = armour;
        let _grunt = self.drones.rng.rand(4); // `Rand_Rand(4)` [M 0x1902c8:111] — the shared stream
        if victim == local {
            // The local HUD's flash and hit-direction glow (`BL+0x968` / `+0x967` [M 0x1902c8:120-134;
            // 0x194458:199-232 — the hit direction in the viewer's frame]) — this machine's view only.
            let hl = &mut self.drones.health;
            hl.flash = (hl.flash + (d * 21.333).min(128.0)).min(255.0);
            hl.dir_bits = 0xC;
            if located {
                // The source side: back along the round's travel direction.
                let sl = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(1e-6);
                let sv = [-dir[0] / sl, -dir[1] / sl, -dir[2] / sl];
                let (rx, rz) = (-self.bond.yaw.cos(), self.bond.yaw.sin());
                let vx = sv[0] * rx + sv[2] * rz;
                let vy = sv[1];
                if vx.abs() >= 0.5 || vy.abs() >= 0.5 {
                    hl.dir_bits = if vy.abs() < vx.abs() { if vx < 0.0 { 0x4 } else { 0x8 } } else if vy < 0.0 { 0x1 } else { 0x2 };
                }
            }
        }
        if self.mp_hp[victim] <= 0.0 {
            self.mp_hp[victim] = 0.0;
            self.mp_dead_ticks[victim] = 1;
            if let Some(m) = self.mp.as_mut() { m.player_killed(victim, Some(attacker), false, false, &mut self.drones.rng); }
            eprintln!("[mp] slot {attacker} killed slot {victim} (frame {})", self.drones.frame);
        }
    }

    /// Online combat (700v6, every weapon since mp-combat-weapons) [M-grounded port design — the
    /// PS2 shipped four players on one machine (the game's own MP walks, Pickup_Update
    /// 0x18e148:202-253) and no online mode; the port drives the game's own local-MP combat
    /// machinery (every address below is the game's MP code) from the session's inputs]: every
    /// machine runs every human slot's shots from the session's inputs, in
    /// slot order, on the shared stream. A shot (`Input::shot` — the sender's own fire pipeline
    /// decided the release: rate, burst, clip, the release frame) carries its description (the
    /// fired record `shot_weapon`, the aim `shot_aim`); the shared combat (`nf_game::mp::combat`)
    /// spawns the record's rounds / grenade from the shooter's head (`Bullet_init` 0x12c5a0),
    /// flies them (`Bullet_update` 0x12e7b0), resolves contacts and blasts (`Explode_Propagate`
    /// 0x17c518) and the damage goes through the game's MP pain path ([`Sim::mp_hurt`]). Death
    /// scores through `player_killed` [M MP_PlayerKilled 0x188f70], the respawn waits 5 s [M
    /// Player_HandleDeath 0x190bb0:41-54] and takes `mp_respawn`'s spawn pick + health [M
    /// MP_ReSpawn 0x185368]. The local slot's health drives this machine's HUD and death sequence.
    pub fn step_net_combat(&mut self, inputs: &[nf_net::Input; nf_net::MAX_AGENTS], local: usize) {
        let _p = nf_game::prof::scope("net: step_net_combat");
        if self.mp.is_none() { return; }
        for s in 0..nf_net::MAX_AGENTS { self.mp_held[s] = if s == local { self.gun.weapon } else { inputs[s].held }; }
        let frame = self.drones.frame;
        // 1. The new shots, in slot order (the bodies as they stand this tick).
        self.mp_combat.fired.clear();
        for shooter in 0..nf_net::MAX_AGENTS {
            let inp = &inputs[shooter];
            // mp-shot-gate: the slot's gate ticks on every row, shot or not (the held follow, the
            // reload machine, the refire timer) — docs/systems/mp-shot-gate.md.
            let script_len = |h: u32| self.scripts.get(&h).map(|s| s.length as u32);
            self.mp_combat.gate_tick(shooter, inp.held, inp.fire, inp.reload, &self.weapons, &script_len);
            if !inp.shot || self.mp_dead_ticks[shooter] > 0 { continue; }
            let b = if shooter == local { Some(&self.bond) } else { self.mp_bonds[shooter].as_ref() };
            let Some(b) = b else { continue };
            let head = b.head();
            // A shot without a description (shot_weapon 0) fires the held record along the
            // body's view — the 700v6 shape the older test rows send.
            let (weapon, dir) = if inp.shot_weapon != 0 {
                (inp.shot_weapon, nf_game::mp::combat::dir_from_aim(inp.shot_aim))
            } else {
                let (sy, cy) = (nf_game::trig::sinf(b.yaw), nf_game::trig::cosf(b.yaw));
                let (sp, cp) = (nf_game::trig::sinf(b.pitch), nf_game::trig::cosf(b.pitch));
                (inp.held, [cp * sy, sp, cp * cy])
            };
            // mp-shot-gate: the shot is applied only when the game would let the slot fire —
            // it holds this weapon, has the rounds, the refire interval has run. Dropped shots
            // draw nothing from the shared stream and show no effects (they never left).
            if !self.mp_combat.gate_allow(shooter, weapon, &self.weapons) {
                if std::env::var_os("NF_MPCOMBAT_LOG").is_some() { eprintln!("pcombat] frame {frame}: slot {shooter}'s {weapon:#04x} shot dropped by the gate"); }
                continue;
            }
            if std::env::var_os("NF_MPCOMBAT_LOG").is_some() { eprintln!("[mpcombat] frame {frame}: slot {shooter} fires {weapon:#04x} dir {dir:?}"); }
            self.mp_combat.fire(&self.weapons, shooter, weapon, head, dir, &mut self.drones.rng, frame);
            // mp-remote-fx: the shot's effects (muzzle flash, tracer, impact) — draw/audio only. The
            // impact sits where the line first meets the world; agent 10's shared rounds own the
            // damage.
            {
                let range = 200.0;
                let mut end = [head[0] + dir[0] * range, head[1] + dir[1] * range, head[2] + dir[2] * range];
                let mut hit = crate::mpfx::MpShotHit::Miss;
                if let Some((t, normal, material)) = self.world.segment_surface_bullet(head, end) {
                    end = [head[0] + (end[0] - head[0]) * t, head[1] + (end[1] - head[1]) * t, head[2] + (end[2] - head[2]) * t];
                    hit = crate::mpfx::MpShotHit::World { normal, material };
                }
                self.mp_fx.events.push(crate::mpfx::MpShot { shooter, bot: false, weapon, from: head, dir, end, hit, damage: 0.0 });
            }
        }
        // 2. The flight, the contacts, the blasts — against every live body (humans + bots).
        let bodies = self.mp_combat_bodies(local);
        let steer: [(f32, f32); nf_game::mp::MP_MAX_SLOTS] = std::array::from_fn(|s| if s < nf_net::MAX_AGENTS { (inputs[s].turn, inputs[s].look) } else { (0.0, 0.0) });
        let site = self.mp.as_ref().and_then(|m| m.site_sphere());
        let hurts = self.mp_combat.step(&self.world, &self.weapons, &bodies, site, &steer, &mut self.drones.rng, frame);
        for h in hurts { self.mp_hurt(h, local); }
        // The guided ride: this machine's camera follows its own slot's shared rocket, and the
        // local body holds still exactly while the shared ride says so (`step_bond`'s online arm).
        if self.mp_combat.riding(local) {
            let ride = self.mp_combat.ride[local];
            let cam = self.mp_combat.ridden_round(local).map(|r| (r.pos, r.dir));
            let prev = self.missile_cam;
            let (pos, dir) = cam.or(prev.map(|c| (c.pos, c.dir))).unwrap_or((self.bond.head(), self.bond.look()));
            self.missile_cam = Some(MissileCam { pos, dir, restore: ride.restore as u16, riding: ride.riding });
        } else {
            self.missile_cam = None;
        }
        // 2. The dead wait 5 s, then respawn at the policy's pick.
        for slot in 0..nf_net::MAX_AGENTS {
            if self.mp_dead_ticks[slot] == 0 { continue; }
            self.mp_dead_ticks[slot] += 1;
            if self.mp_dead_ticks[slot] < 300 { continue; }
            let Some(m) = self.mp.as_mut() else { continue };
            let Some((sp_idx, hp)) = m.mp_respawn(slot, &mut self.drones.rng) else { continue };
            let sp = m.spawns.points[sp_idx as usize];
            let nb = nf_game::player::Bond::stand_at(&self.world, [sp.pos[0], sp.pos[1], sp.pos[2]], sp.facing[1]);
            m.set_slot(slot, nb.pos);
            if slot == local { self.bond = nb; } else { self.mp_bonds[slot] = Some(nb); }
            self.mp_hp[slot] = hp;
            self.mp_armour[slot] = 0.0; // [M MP_ReSpawn 0x185368:69 — BL+0x8B0 = 0]
            self.mp_dead_ticks[slot] = 0;
            eprintln!("[mp] slot {slot} respawned at spawn {sp_idx} (frame {})", self.drones.frame);
            if slot == local {
                self.drones.death = nf_game::death::Death::default();
                // The respawn re-arms the player (`MP_ReSpawn` 0x185368 → `MP_EquipPlayer` 0x1855f8
                // [M 0x185368:74]; the death sequence had switched him to the fists 0x47 [M
                // 0x190810:61]) — the weapon set's spawn loadout.
                self.mp_equip_local();
            }
            // mp-shot-gate: the respawn re-seeds the slot's shared gate the same way [M the
            // re-equip above]; the settings clone keeps the borrow split.
            let settings = self.mp.as_ref().map(|m| m.settings.clone());
            if let Some(s) = settings {
                let lf = |w: u8| self.mp_weapon_loaded.get(w as usize).copied().unwrap_or(true);
                self.mp_combat.gate_equip(slot, &s, &self.weapons, lf);
            }
        }
        // 3. This machine's own health/armour are the slot's (the HUD, the death sequence's start).
        self.drones.health.health = self.mp_hp[local];
        self.drones.health.armour = self.mp_armour[local];
        // mp-remote-fx: this tick's shot effects / sounds (draw/audio only, after every hashed write).
        self.flush_mp_fx();
        // mp-body-anim: the drawn bodies' clocks on this tick's final movement and life (draw-only).
        self.advance_mp_anim();
    }

    /// mp-join-in-progress: create the late joiner's agent at the announced join tick — the
    /// game's own MP create shape, run on EVERY machine at the same tick (the shared stream's
    /// spawn pick is part of the deterministic tick): `MP_GetSpawnPoint(team, NULL)` before
    /// `Player_Init(slot, pos, facing, …)` [M 0x183a50:120-127; `Player_Init` 0x1910f8 refuses
    /// only an occupied slot and creates the obj — a free slot mid-match is the game's own
    /// path; `MP_ReSpawn` 0x185368 only re-positions an EXISTING obj]. The port's `Player_Init`
    /// shape is the load's spawn loop (`Bond::stand_at`, lib.rs) — one new slot of it here.
    /// A slot that already holds a body (a dropped peer's agent, `DropPolicy::Neutral`) is a
    /// RECONNECT: no spawn, the real input just resumes. Returns whether a body was created.
    pub fn mp_join_spawn(&mut self, slot: usize) -> bool {
        if slot == 0 || slot >= nf_net::MAX_AGENTS { return false; }
        if self.mp_bonds[slot].is_some() { return false; }
        let Some(m) = self.mp.as_mut() else { return false };
        let team = m.settings.players[slot].team as u16;
        let sp = m.get_spawn_point(team, slot, &mut self.drones.rng) as usize;
        let ent = m.spawns.points[sp];
        let b = nf_game::player::Bond::stand_at(&self.world, [ent.pos[0], ent.pos[1], ent.pos[2]], ent.facing[1]);
        m.set_slot(slot, b.pos);
        self.mp_bonds[slot] = Some(b);
        // The online combat's per-slot state starts fresh for the new agent (700v6).
        self.mp_hp[slot] = 100.0;
        self.mp_armour[slot] = 0.0;
        self.mp_dead_ticks[slot] = 0;
        true
    }

    /// The per-tick state hash sent every N ticks (the app calls it; the nf-net session compares).
    /// The fold is the fixed-order fold of the named component hashes ([`Self::net_component_hashes`]
    /// — net-desync-detail): frame+rng, per-slot bodies, per-slot vitals, the online combat, the
    /// match state, the bots, the world. The component grouping moved the fields around relative to
    /// the old flat fold (the combined value changed — re-pinned on purpose, see the task report),
    /// but every field is still hashed exactly once, so equal states give equal hashes on every
    /// machine and every field's change still moves the combined hash.
    pub fn net_state_hash(&self, local: usize) -> u64 {
        fold_component_hashes(&self.net_component_hashes(local))
    }

    /// net-desync-detail: the named component hashes, in the fixed order
    /// [`NET_HASH_COMPONENT_NAMES`]. Each component is its own FNV fold, so a desync report can
    /// name the first component whose hashes disagree between the machines.
    pub fn net_component_hashes(&self, local: usize) -> [u64; NET_HASH_COMPONENTS] {
        let _p = nf_game::prof::scope("net: net_component_hashes");
        // NF_NET_HASHDBG="a-b": dump frames a..=b (any other value: 0-17).
        // mp-perf-16: the env var is read once per process, not once per tick.
        static HASHDBG: std::sync::OnceLock<Option<(u32, u32)>> = std::sync::OnceLock::new();
        let dbg = *HASHDBG.get_or_init(|| std::env::var("NF_NET_HASHDBG").ok().map(|v| v.split_once('-').and_then(|(a, b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))).unwrap_or((0, 17))));
        if dbg.is_some_and(|(a, b)| (a..=b).contains(&self.drones.frame)) {
            let b = |s: usize| if s == local { Some(&self.bond) } else { self.mp_bonds[s].as_ref() };
            eprintln!("[hashdbg] frame {} rng {:x}/{:x}/{} bonds {:?} mp {:?}", self.drones.frame, self.drones.rng.x, self.drones.rng.y, self.drones.rng.draws,
                (0..nf_net::MAX_AGENTS).map(|s| b(s).map(|b| (b.pos, b.yaw, b.pitch, b.substate, b.crouched))).collect::<Vec<_>>(),
                self.mp.as_ref().map(|m| (m.game.state, m.game.clock, m.game.limit, m.game.slots.iter().map(|s| (s.pos, s.timers, s.score, s.kills, s.deaths, s.dead)).collect::<Vec<_>>(),
                    (m.settings.level_code, m.settings.scenario, m.settings.frag_limit, m.settings.time_limit, m.settings.seed, m.settings.human_count))));
            eprintln!("[hashdbg]   hp {:?} dead {:?}", self.mp_hp, self.mp_dead_ticks);
        }
        let mut frame_rng = nf_net::hash::Hasher::new();
        frame_rng.u32(self.drones.frame);
        frame_rng.u32(self.drones.rng.x);
        frame_rng.u32(self.drones.rng.y);
        frame_rng.u64(self.drones.rng.draws);
        let mut bodies = nf_net::hash::Hasher::new();
        for slot in 0..nf_net::MAX_AGENTS {
            let bond = if slot == local { Some(&self.bond) } else { self.mp_bonds[slot].as_ref() };
            match bond {
                Some(b) => {
                    bodies.u32(slot as u32);
                    for k in 0..3 { bodies.f32(b.pos[k]); }
                    bodies.f32(b.yaw);
                    bodies.f32(b.pitch);
                    bodies.u32(b.substate as u32);
                    bodies.bool(b.crouched);
                }
                None => bodies.u32(0xffff_ffff),
            }
        }
        let mut vitals = nf_net::hash::Hasher::new();
        // 700v6: the online combat's per-slot state.
        for s in 0..nf_net::MAX_AGENTS { vitals.f32(self.mp_hp[s]); vitals.u32(self.mp_dead_ticks[s]); }
        for s in 0..nf_net::MAX_AGENTS { vitals.f32(self.mp_armour[s]); }
        let mut combat = nf_net::hash::Hasher::new();
        self.mp_combat.hash_into(&mut |v| combat.u32(v));
        let mut matchh = nf_net::hash::Hasher::new();
        if let Some(m) = &self.mp {
            matchh.u32(m.game.state);
            matchh.f32(m.game.clock);
            matchh.f32(m.game.limit);
            for s in &m.game.slots {
                match s.pos {
                    Some(p) => { for k in 0..3 { matchh.f32(p[k]); } }
                    None => matchh.u32(0xffff_ffff),
                }
                matchh.u32(s.timers[0] as u32);
                matchh.u32(s.timers[1] as u32);
                matchh.f32(s.score);
                matchh.i32(s.kills);
                matchh.i32(s.deaths);
                matchh.bool(s.dead);
            }
            matchh.u32(m.settings.level_code);
            matchh.u32(m.settings.scenario);
            matchh.i32(m.settings.frag_limit);
            matchh.i32(m.settings.time_limit);
            matchh.u32(m.settings.seed);
            matchh.u32(m.settings.human_count);
            // mp-scenarios-online (agent6): the WHOLE match state — the team scores, the end
            // latches, the scenario objects (flag carriers/positions, terminal owners, the hill,
            // the site HP, the GoldenEye items + strike, the blueprint), the assassin words. The
            // fold above covered only the slots, so a scenario desync went undetected.
            matchh.u64(nf_game::mp::hash::state_hash(m));
        } else {
            matchh.u32(0xffff_fffe);
        }
        let mut bots = nf_net::hash::Hasher::new();
        // The bots' brains (their state, goals, route, body, weapon/fire state —
        // `BotBrain::state_hash`) and their health/death: a bot desync shows at once instead
        // of only once it moved a body (the brains were never folded in).
        if let Some(m) = &self.mp {
            for b in &m.bots {
                bots.u64(b.brain.state_hash());
                bots.f32(b.health);
                bots.bool(b.dead);
            }
        }
        let mut world = nf_net::hash::Hasher::new();
        // mp-world-sync (agent 9): the shared world — the pickups (taken/hidden/respawn timers),
        // the slots' armour, the doors, the triggers, the world channels, the destroyables, the
        // breakables, the SS objects, the hurt volumes. Before this fold a take, a door or a
        // broken object on one machine only went undetected (pickups were not in the hash).
        self.net_world_fold(&mut world);
        [
            frame_rng.finish(), bodies.finish(), vitals.finish(), combat.finish(),
            matchh.finish(), bots.finish(), world.finish(),
        ]
    }

    /// net-desync-detail: the per-item breakdown of each component
    /// ([`Self::net_component_hashes`]) — one item hash per hashed element in the same fixed
    /// order (bodies/vitals: one per agent slot; combat: one per round, then per thrown, then
    /// per ride slot; bots: one per bot; world: one per pickup, then per door, then per
    /// trigger, then per channel entry, then per destroyable, per breakable, per SS object,
    /// per hurt volume; frame_rng and match: the single component hash). The desync exchange
    /// sends the first diverging component's items, so the report can name the exact slot.
    pub fn net_component_items(&self, local: usize) -> [Vec<u64>; NET_HASH_COMPONENTS] {
        let mut frame_rng = nf_net::hash::Hasher::new();
        frame_rng.u32(self.drones.frame);
        frame_rng.u32(self.drones.rng.x);
        frame_rng.u32(self.drones.rng.y);
        frame_rng.u64(self.drones.rng.draws);
        let mut bodies = Vec::with_capacity(nf_net::MAX_AGENTS);
        for slot in 0..nf_net::MAX_AGENTS {
            let mut h = nf_net::hash::Hasher::new();
            let bond = if slot == local { Some(&self.bond) } else { self.mp_bonds[slot].as_ref() };
            match bond {
                Some(b) => {
                    h.u32(slot as u32);
                    for k in 0..3 { h.f32(b.pos[k]); }
                    h.f32(b.yaw);
                    h.f32(b.pitch);
                    h.u32(b.substate as u32);
                    h.bool(b.crouched);
                }
                None => h.u32(0xffff_ffff),
            }
            bodies.push(h.finish());
        }
        let mut vitals = Vec::with_capacity(nf_net::MAX_AGENTS);
        for s in 0..nf_net::MAX_AGENTS {
            let mut h = nf_net::hash::Hasher::new();
            h.f32(self.mp_hp[s]);
            h.u32(self.mp_dead_ticks[s]);
            h.f32(self.mp_armour[s]);
            vitals.push(h.finish());
        }
        let mut combat = Vec::new();
        {
            // The same fields `MpCombat::hash_into` folds, one record per item (rounds, then
            // thrown, then the ride slots).
            let c = &self.mp_combat;
            let mut item = nf_net::hash::Hasher::new();
            item.u32(c.rounds.len() as u32);
            combat.push(item.finish());
            for r in &c.rounds {
                let mut h = nf_net::hash::Hasher::new();
                h.u32(r.owner as u32); h.u32(r.weapon as u32);
                for k in 0..3 { h.f32(r.pos[k]); h.f32(r.dir[k]); }
                h.f32(r.speed); h.f32(r.travelled); h.u32(r.first as u32); h.u32(r.target.map_or(u32::MAX, |t| t as u32));
                combat.push(h.finish());
            }
            let mut item = nf_net::hash::Hasher::new();
            item.u32(c.thrown.len() as u32);
            combat.push(item.finish());
            for g in &c.thrown {
                let mut h = nf_net::hash::Hasher::new();
                h.u32(g.owner as u32); h.u32(g.weapon_type as u32);
                for k in 0..3 { h.f32(g.pos[k]); h.f32(g.velocity[k]); }
                h.f32(g.travelled); h.f32(g.range); h.u32(g.state as u32); h.u32(g.bounces as u32);
                h.u32(g.stuck as u32); h.u32(g.resting as u32);
                h.u32(g.phase.as_ref().map_or(0, |p| (p.armed as u32) | (((p.arm.to_bits())) << 1)));
                combat.push(h.finish());
            }
            for r in &c.ride {
                let mut h = nf_net::hash::Hasher::new();
                h.u32(r.riding as u32); h.u32(r.restore as u32);
                combat.push(h.finish());
            }
        }
        let mut matchh = Vec::new();
        if let Some(m) = &self.mp {
            let mut h = nf_net::hash::Hasher::new();
            h.u32(m.game.state);
            h.f32(m.game.clock);
            h.f32(m.game.limit);
            for s in &m.game.slots {
                match s.pos {
                    Some(p) => { for k in 0..3 { h.f32(p[k]); } }
                    None => h.u32(0xffff_ffff),
                }
                h.u32(s.timers[0] as u32);
                h.u32(s.timers[1] as u32);
                h.f32(s.score);
                h.i32(s.kills);
                h.i32(s.deaths);
                h.bool(s.dead);
            }
            h.u32(m.settings.level_code);
            h.u32(m.settings.scenario);
            h.i32(m.settings.frag_limit);
            h.i32(m.settings.time_limit);
            h.u32(m.settings.seed);
            h.u32(m.settings.human_count);
            h.u64(nf_game::mp::hash::state_hash(m));
            matchh.push(h.finish());
        } else {
            let mut h = nf_net::hash::Hasher::new();
            h.u32(0xffff_fffe);
            matchh.push(h.finish());
        }
        let mut bots = Vec::new();
        if let Some(m) = &self.mp {
            for b in &m.bots {
                let mut h = nf_net::hash::Hasher::new();
                h.u64(b.brain.state_hash());
                h.f32(b.health);
                h.bool(b.dead);
                bots.push(h.finish());
            }
        }
        // The world's items in `net_world_fold`'s order; the report maps an item index to a
        // name from the counts it prints (pickups, doors, triggers, channels, destroy,
        // breaks, ss, hurts).
        let mut world = Vec::new();
        {
            let d = &self.drones;
            for p in &d.pickups {
                let mut h = nf_net::hash::Hasher::new();
                h.bool(p.gone); h.bool(p.hidden); h.u32(p.state as u32);
                h.u32(p.respawn_ticks); h.u32(p.lifetime as u32);
                for k in 0..3 { h.f32(p.m[3][k]); }
                world.push(h.finish());
            }
            for x in &d.doors.list {
                let mut h = nf_net::hash::Hasher::new();
                h.u32(x.state as u32); h.f32(x.pos01); h.i32(x.timer as i32); h.u32(x.open_timer as u32);
                world.push(h.finish());
            }
            for t in &d.triggers {
                let mut h = nf_net::hash::Hasher::new();
                h.bool(t.gone);
                world.push(h.finish());
            }
            let mut chs: Vec<u32> = Vec::new();
            chs.extend(d.triggers.iter().map(|t| t.out as u32));
            chs.extend(d.doors.list.iter().flat_map(|x| [x.ch_open as u32, x.ch_lock as u32]));
            chs.extend(d.props.destroy.iter().map(|x| x.channel as u32));
            chs.extend(d.pickups.iter().map(|p| p.switch_channel as u32));
            chs.extend(d.sss.list.iter().map(|x| x.channel as u32));
            chs.extend(self.hurts.iter().map(|x| x.gate as u32));
            for c in chs {
                let mut h = nf_net::hash::Hasher::new();
                h.u32(d.channels.v.get(c as usize).copied().unwrap_or(0) as u32);
                world.push(h.finish());
            }
            for x in &d.props.destroy {
                let mut h = nf_net::hash::Hasher::new();
                h.f32(x.health); h.bool(x.dead);
                world.push(h.finish());
            }
            for x in &d.breaks.list {
                let mut h = nf_net::hash::Hasher::new();
                h.f32(x.health); h.bool(x.dead);
                world.push(h.finish());
            }
            for x in &d.sss.list {
                let mut h = nf_net::hash::Hasher::new();
                h.u32(x.state as u32); h.f32(x.t);
                world.push(h.finish());
            }
            for x in &self.hurts {
                let mut h = nf_net::hash::Hasher::new();
                h.bool(x.disabled); h.bool(x.fired);
                world.push(h.finish());
            }
        }
        [vec![frame_rng.finish()], bodies, vitals, combat, matchh, bots, world]
    }

    /// mp-world-sync: the shared world's hashed state (see `net_state_hash`). Fixed order; the
    /// world channels are the ones the world's own objects read or write (the trigger outs, the
    /// doors' open/lock, the destroyables', the pickups' and the SS channels, the hurt gates) —
    /// never the local-only writers (the death sequence's 0x62, the wire/creep channel).
    fn net_world_fold(&self, h: &mut nf_net::hash::Hasher) {
        let d = &self.drones;
        for s in 0..nf_net::MAX_AGENTS { h.f32(self.mp_armour[s]); }
        h.u32(d.pickups.len() as u32);
        for p in &d.pickups {
            h.bool(p.gone);
            h.bool(p.hidden);
            h.u32(p.state as u32);
            h.u32(p.respawn_ticks);
            h.u32(p.lifetime as u32);
            for k in 0..3 { h.f32(p.m[3][k]); }
        }
        for x in &d.doors.list {
            h.u32(x.state as u32);
            h.f32(x.pos01);
            h.i32(x.timer as i32);
            h.u32(x.open_timer as u32);
        }
        for t in &d.triggers { h.bool(t.gone); }
        let mut chs: Vec<u32> = Vec::new();
        chs.extend(d.triggers.iter().map(|t| t.out as u32));
        chs.extend(d.doors.list.iter().flat_map(|x| [x.ch_open as u32, x.ch_lock as u32]));
        chs.extend(d.props.destroy.iter().map(|x| x.channel as u32));
        chs.extend(d.pickups.iter().map(|p| p.switch_channel as u32));
        chs.extend(d.sss.list.iter().map(|x| x.channel as u32));
        chs.extend(self.hurts.iter().map(|x| x.gate as u32));
        for c in chs { h.u32(d.channels.v.get(c as usize).copied().unwrap_or(0) as u32); }
        for x in &d.props.destroy { h.f32(x.health); h.bool(x.dead); }
        for x in &d.breaks.list { h.f32(x.health); h.bool(x.dead); }
        for x in &d.sss.list { h.u32(x.state as u32); h.f32(x.t); }
        for x in &self.hurts { h.bool(x.disabled); h.bool(x.fired); }
    }

    /// mp-world-sync: the shared world's fields as text (the desync hunts' field-by-field diff).
    pub fn net_world_fields(&self) -> Vec<(String, String)> {
        let d = &self.drones;
        let mut v = vec![("armour".to_string(), format!("{:?}", self.mp_armour))];
        for (i, p) in d.pickups.iter().enumerate() {
            v.push((format!("pickup{i}"), format!("gone {} hidden {} state {} respawn {} at {:?}", p.gone, p.hidden, p.state, p.respawn_ticks, [p.m[3][0], p.m[3][1], p.m[3][2]])));
        }
        for (i, x) in d.doors.list.iter().enumerate() {
            v.push((format!("door{i}"), format!("state {:#x} pos {} timer {} open {}", x.state, x.pos01, x.timer, x.open_timer)));
        }
        v.push(("triggers".into(), format!("{:?}", d.triggers.iter().map(|t| t.gone).collect::<Vec<_>>())));
        v.push(("channels".into(), format!("{:?}", d.channels.v.iter().enumerate().filter(|(_, c)| **c != 0).map(|(i, c)| (i, *c)).collect::<Vec<_>>())));
        v.push(("destroy".into(), format!("{:?}", d.props.destroy.iter().map(|x| (x.health, x.dead)).collect::<Vec<_>>())));
        v.push(("breaks".into(), format!("{:?}", d.breaks.list.iter().map(|x| (x.health, x.dead)).collect::<Vec<_>>())));
        v.push(("sss".into(), format!("{:?}", d.sss.list.iter().map(|x| (x.state, x.t)).collect::<Vec<_>>())));
        v.push(("hurts".into(), format!("{:?}", self.hurts.iter().map(|x| (x.disabled, x.fired)).collect::<Vec<_>>())));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_rate_is_ticks_per_frame() {
        // The rates on 07000005: 12 fps → 5 ticks, 17 → 3, 30 → 2, 0 → every tick.
        assert_eq!([12, 17, 30, 0, 60, 255].map(ticks_per_frame), [5, 3, 2, 1, 1, 1]);
        // Sister: the old reading (the byte as ticks) disagrees.
        assert_ne!(ticks_per_frame(12), 12);
    }

    /// sg5-laser-draw: the beam mesh's local (0, 0, −len) (its far end, the −z geometry) lands on
    /// the beam's end for a TILTED beam, and the axes stay orthonormal.
    #[test]
    fn z_basis_puts_the_beam_end_on_the_end() {
        let (a, b) = ([107.3f32, 25.04, 109.91], [108.84f32, 24.75, 109.10]);
        let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let mut m = z_basis(v, a);
        for k in 0..3 { m[2][k] *= len; }
        let far = nf_formats::placement::transform(&m, [0.0, 0.0, -1.0]);
        for k in 0..3 { assert!((far[k] - b[k]).abs() < 1e-4, "end {far:?} vs {b:?}"); }
        let m1 = z_basis(v, a);
        let dot = |i: usize, j: usize| (0..3).map(|k| m1[i][k] * m1[j][k]).sum::<f32>();
        for (i, j) in [(0, 1), (0, 2), (1, 2)] { assert!(dot(i, j).abs() < 1e-5); }
        assert!(m1[0][1].abs() < 1e-6, "x stays horizontal");
        // Sister: the tripbomb's atan2 yaw/pitch euler misses the end of the same tilted beam.
        let yaw = v[0].atan2(v[2]);
        let pitch = -v[1].atan2((v[0] * v[0] + v[2] * v[2]).sqrt());
        let mut e = nf_formats::placement::object_matrix([pitch, yaw, 0.0], a);
        for k in 0..3 { e[2][k] *= len; }
        let efar = nf_formats::placement::transform(&e, [0.0, 0.0, -1.0]);
        assert!((0..3).any(|k| (efar[k] - b[k]).abs() > 0.05), "the euler build should miss: {efar:?}");
    }

    /// R6: the laser beam's matrix is `View_RotTransMatrix` 0x1e5168 on the beam's spherical angles
    /// (x = −elevation, y = azimuth of start − end [M Draw_LaserBeam 0x1a4560:65-70]), z scaled by the
    /// length — so the sprite's −z end lands on the hit for any steep, angled beam. Sister: the
    /// placement-euler `object_matrix` on the same angles misses it.
    #[test]
    fn laser_beam_far_end_lands_on_the_hit() {
        let a = [1.0f32, 2.0, 3.0];
        let b = [4.0f32, -1.5, 7.5]; // down, right and away — steep and angled
        let v = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let yaw = v[0].atan2(v[2]);
        let pitch = -v[1].atan2((v[0] * v[0] + v[2] * v[2]).sqrt());
        let mut m = nf_game::aivolume::matrix([pitch, yaw, 0.0], a);
        for k in 0..3 { m[2][k] *= len; }
        let far = nf_formats::placement::transform(&m, [0.0, 0.0, -1.0]);
        for k in 0..3 { assert!((far[k] - b[k]).abs() < 1e-3, "beam end {far:?} vs the hit {b:?}"); }
        let mut e = nf_formats::placement::object_matrix([pitch, yaw, 0.0], a);
        for k in 0..3 { e[2][k] *= len; }
        let off = nf_formats::placement::transform(&e, [0.0, 0.0, -1.0]);
        assert!((0..3).map(|k| (off[k] - b[k]).abs()).fold(0.0f32, f32::max) > 0.5, "the placement euler misses the hit: {off:?}");
    }
}

/// rain-drops: the per-level rain-mesh slots (`InitDrops`' rain cap: 0x80 on 07000009-0b, 0x40 elsewhere).
pub const RAIN_SLOTS: usize = 128;

/// net-desync-detail: how many named components the net state hash folds
/// (see [`Sim::net_component_hashes`]).
pub const NET_HASH_COMPONENTS: usize = 7;
/// net-desync-detail: the components' names, in the fixed fold order. A desync report names the
/// first component whose hashes disagree between the machines, so the order is part of the wire
/// contract (`PT_HASHCOMPS`); never reorder or rename these without bumping the protocol.
pub const NET_HASH_COMPONENT_NAMES: [&str; NET_HASH_COMPONENTS] =
    ["frame_rng", "bodies", "vitals", "combat", "match", "bots", "world"];

/// net-desync-detail: the combined hash — the fixed-order FNV fold of the component hashes. The
/// combined value is a different function of the state than the old flat fold (the components
/// group the same fields differently), so hashes exchanged with an older build never match; the
/// protocol version bump (7 → 8) refuses that mix.
pub fn fold_component_hashes(comps: &[u64; NET_HASH_COMPONENTS]) -> u64 {
    let mut h = nf_net::hash::Hasher::new();
    for c in comps {
        h.u64(*c);
    }
    h.finish()
}

impl Sim {
    /// unported-front-end-menus-3: `Camera_Shear` 0x1bba38 [M] — the underwater view wobble
    /// the game runs from `Camera_Update` 0x1bb6d8:25-34 whenever the eye Y (viewer+0xc4)
    /// sits below the eye cel's water surface (cel+0x90). One call = one frame: returns the
    /// skew factors xs = 0.025·sin(φ+π/2), ys = 0.025·sin(φ) [M :20-24 — PS2Sinf2(phase+π/2,
    /// phase) into xs.234 @0x30d798 / ys.235 @0x30d79c, both × 0.025], advances the shear
    /// phase (viewer+0x1e0) by REC_FRAME_RATE and wraps it into [−π, π] [M :26-34], and sets
    /// `GlobalBlur = 0x91` [M :49]. The game builds the identity+skew matrix (scale 1+xs,
    /// 1+ys, 1+xs) and concats it into the view matrix — the draw side's job (the port's Camera has
    /// no roll/shear slot yet).
    pub fn camera_shear(&mut self) -> (f32, f32) {
        let (xs, ys) = camera_shear_step(&mut self.shear_phase);
        self.global_blur = 0x91; // [M 0x1bba38:49]
        (xs, ys)
    }

    /// unported-front-end-menus-3: `Camera_ApplySwing` 0x1bbf40 [M] — the per-frame
    /// weapon-aim sway the game runs from `Player_Update` 0x193600:439-441 with the current
    /// weapon's swing amount (`weapon_data[weapon·0x10c + 0x8c]`, `lwc1 f12,140(a1)` [M
    /// 0x194134]) and the viewer's zoom factor (viewer+0x108, fed BLData+0x8d0 each tick [M
    /// `Player_PositionCamera` 0x18f5d8:134]). No-op when amount == 0 [M 0x1bbf40:8].
    /// Advances the swing phase (viewer+0x204) by 2π/(FRAME_RATE·4) [M :10-11] and returns
    /// the three view-angle deltas: pitch (viewer+0xe0) += (zoom−1)·amount·sin(2φ), yaw
    /// (+0xe4) += (zoom−1)·amount·sin(φ), roll (+0xe8) += (zoom−1)·amount·sin(3φ) [M :12-24].
    /// The game then rebuilds the view matrix — the draw side's job.
    pub fn camera_apply_swing(&mut self, amount: f32, zoom: f32) -> (f32, f32, f32) {
        camera_swing_step(&mut self.swing_phase, amount, zoom)
    }

    /// unported-front-end-menus-3: `Camera_Orbit` 0x1bb828's per-frame arm — the game runs it
    /// from `Camera_Update` 0x1bb6d8:21 → `Player_PositionCamera` 0x18f5d8 case 1 for the
    /// viewer whose BL holds cam_mode 1 (the GoldenEye-strike orbit). The port: refresh the
    /// look from Bond's head (the game's `AnimGetBoneWorldTrans(obj, −5, 0, BLData+0xe0, 0)`
    /// head-bone refresh [M 0x18f5d8:137-139]; the port's head stand-in is
    /// `Player_GetHeadPos`), run [`Bond::orbit_camera`] with the eye sphere's closest hit
    /// (the game's `Collide_SphereIntersect(1.5, viewer+0x150, cel, obj, …)` HITDATA+0xc [M
    /// 0x1bb828:63-70]; the port's world-sphere query stands in), and hand the draw side
    /// `(eye, look, zoom)`.
    pub fn orbit_camera(&mut self, world: &nf_game::world::CollisionWorld) -> Option<([f32; 3], [f32; 3], f32)> {
        if self.bond.cam_mode != 1 {
            return None;
        }
        let eye0 = self.bond.head();
        if let Some(o) = self.bond.orbit.as_mut() {
            o.look = eye0;
        }
        // The eye-sphere pull: the game spheres r = 1.5 at viewer+0x150 (the authoritative
        // eye — which Camera_Orbit itself never rewrites [M 0x1bb828 writes only +0x120]).
        let hit = world
            .sphere_surface(eye0, 1.5, false)
            .map(|(_p, _n, _mat, dist)| dist);
        self.bond.orbit_camera(hit)
    }
}

/// unported-front-end-menus-3: `Camera_Shear` 0x1bba38's per-frame step [M] — returns the
/// skew factors xs = 0.025·sin(φ+π/2), ys = 0.025·sin(φ) [M :20-24 — PS2Sinf2(phase+π/2,
/// phase) into xs.234 @0x30d798 / ys.235 @0x30d79c, both × 0.025], advances the shear phase
/// (viewer+0x1e0) by REC_FRAME_RATE and wraps it into [−π, π] [M :26-34: |x| ≥ π → fmod(x,
/// 2π), still ≥ π → ±2π by sign]. The game builds the identity+skew matrix (scale 1+xs,
/// 1+ys, 1+xs) and concats it into the view matrix, then sets GlobalBlur = 0x91 [M :36-49]
/// — the matrix is the draw side's job; the blur byte is the Sim method's.
pub fn camera_shear_step(phase: &mut f32) -> (f32, f32) {
    let xs = 0.025 * nf_game::trig::sinf(*phase + std::f32::consts::FRAC_PI_2);
    let ys = 0.025 * nf_game::trig::sinf(*phase);
    *phase += 1.0 / 60.0; // REC_FRAME_RATE [M grenade.rs:148]
    if phase.abs() >= std::f32::consts::PI {
        *phase %= std::f32::consts::TAU;
        if phase.abs() >= std::f32::consts::PI {
            *phase -= if *phase >= 0.0 { std::f32::consts::TAU } else { -std::f32::consts::TAU };
        }
    }
    (xs, ys)
}

/// unported-front-end-menus-3: `Camera_ApplySwing` 0x1bbf40's per-frame step [M] — no-op
/// when amount == 0 [M :8 — the whole body is behind the != 0 test]; advances the swing
/// phase (viewer+0x204) by 2π/(FRAME_RATE·4) [M :10-11] and returns the three view-angle
/// deltas: pitch (viewer+0xe0) += (zoom−1)·amount·sin(2φ), yaw (+0xe4) += (zoom−1)·amount·
/// sin(φ), roll (+0xe8) += (zoom−1)·amount·sin(3φ) [M :12-24]. The zoom factor is
/// viewer+0x108, fed BLData+0x8d0 each tick [M Player_PositionCamera 0x18f5d8:134]. The game
/// then rebuilds the view matrix — the draw side's job.
pub fn camera_swing_step(phase: &mut f32, amount: f32, zoom: f32) -> (f32, f32, f32) {
    // The game's guard is the VIEWER pointer (param_2), not the amount: the phase advances every
    // frame even at amount 0 [M 0x1bbf40:9-11] (review 2026-10-21: the branch froze it at 0).
    *phase += std::f32::consts::TAU / (60.0 * 4.0); // FRAME_RATE = 60 [M]
    let phi = *phase;
    let k = (zoom - 1.0) * amount;
    (
        k * nf_game::trig::sinf(phi + phi),      // pitch: sin(2φ) [M 0x1bbf40:12-15]
        k * nf_game::trig::sinf(phi),            // yaw: sin(φ) [M 0x1bbf40:16-19]
        k * nf_game::trig::sinf(phi * 3.0),      // roll: sin(3φ) [M 0x1bbf40:20-24]
    )
}
