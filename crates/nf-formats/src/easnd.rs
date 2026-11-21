//! The driving missions' sound data: EA `BNKl` sample banks, EA `SCHl` (`.asf`) streams, the EA-XA
//! ADPCM decoder, and the three text files that wire them up (`banks.ini`, the `.h` id headers and
//! the per-track mix `.ini`). Spec: docs/formats/driving-audio.md.
//!
//! Evidence (all [M] unless tagged; DRIVING.ELF decompile under `work/decomp/DRIVING.ELF/`, EE
//! addresses; SNDDRV.IRX is the IOP sound driver on the disc, `/MODULES/SNDDRV.IRX`, which keeps
//! its symbol table — disassembled with its own `.symtab` names):
//! - PT tag reader `FUN_00312300`: skips `0xFC` pad bytes; `0xFF` ends the header; `0xFD`/`0xFE`
//!   are bare markers; any other tag is followed by a size byte (`0xFF` → a 4-byte big-endian size
//!   follows) and that many value bytes. Values of size < 5 are read by `FUN_00312270`: big-endian,
//!   **sign-extended** for sizes 1, 2 and 3. A larger value leaves the reader's value slot stale.
//! - Patch builder `FUN_0030b318`: one patch = the tags up to a `0xFE` (another patch follows) or
//!   the `0xFF` end. Tags `0x00..=0x25` are the patch block, `0x80..=0xA7` the audio block; every
//!   other tag is ignored. Defaults (the stack initialisers at the top): channels `0x82` = 1,
//!   sample rate `0x84` = 0x5622 (22050), loop start/end `0x86`/`0x87` = -1, memory class `0x8C` = 8,
//!   codec `0xA0` = 5, `0xA1` = 1; tag `0x14` entries are collected as a list (pointer + size).
//!   The rate lands in a 16-bit field, so a sign-extended 2-byte rate reads back unsigned.
//! - Bank loader `FUN_00308088`: header byte +4 = version (5 adds the +0x10 size), u16 +6 = slot
//!   count, u32 +8 = data start, u32 +0xC = size of the memory-class-8 data, u32 +0x10 (version 5) =
//!   size of the memory-class-0x100 data, which sits **first** at the data start; slot table at +0x14,
//!   each u32 relative to its own slot (0 = empty). Each slot's PT header is walked from slot+4
//!   (`FUN_00308630`, which also filters by the memory class at the patch's `0x8C`).
//! - EA-XA: SNDDRV.IRX `decxa16c` (0xa390, 308 bytes): a frame is 15 bytes — one byte (high nibble =
//!   coefficient pair index, low nibble + 8 = shift) then 28 nibbles, high nibble first; the pair
//!   table (.data 0xb470) is (0,0) (240,0) (460,-208) (392,-220); `s = ((n << 28) >> shift +
//!   c1·h1 + c2·h2) >> 8` with **no clamp** on the history. `SFILTER_unpackxafinit` (0x9dc8) zeroes
//!   the history; `SFILTER_unpackxaf` (0x9b80) addresses byte `sample·15/28`.
//! - Streams: `FUN_00308f30` dispatches `SCDl` (0x6c444353) to `FUN_00308d68` and `SCHl`
//!   (0x6c484353) to `FUN_00308a60` (whose header parse starts at block+8, `FUN_0030c0f8`).
//!   `FUN_00308d68`: samples = u32 at block+8 (bit 31 masked off); channel `i` data =
//!   block + 0xC + 4·channels + u32[block + 0xC + 4·i].
//! - Mix `.ini` `FUN_002f3058`: lines end at `'\r'` (then 2 bytes skipped), at most 0x7F chars,
//!   `sscanf("%[^=]=%f, %d")` must return 3, trailing spaces trimmed from the name.
//!
//! Not traced (stated plainly): how the IOP's output is clamped/mixed to 16 bits (the port
//! saturates each decoded sample to i16 — [S]); whether a stream's EA-XA history carries across
//! `SCDl` blocks (the port carries it — [S], the disc test measures the continuity); the meaning of
//! the patch-block tags (raw values are kept).

use crate::buf::{FormatError, Result};

fn err(msg: impl Into<String>) -> FormatError { FormatError::Invalid(msg.into()) }

/// `FUN_00312270` [M]: a big-endian value of `bytes.len()` bytes, sign-extended for 1, 2 or 3 bytes.
pub fn pt_value(bytes: &[u8]) -> i32 {
    let mut v: i32 = 0;
    for &b in bytes { v = v.wrapping_mul(0x100).wrapping_add(b as i32); }
    match bytes.len() {
        1 if v > 0x7f => v - 0x100,
        2 if v > 0x7fff => v - 0x1_0000,
        3 if v > 0x7f_ffff => v - 0x100_0000,
        _ => v,
    }
}

/// One tag as `FUN_00312300` reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtTag {
    pub tag: u8,
    /// Offset of the value bytes in the buffer the header was read from.
    pub at: usize,
    pub size: usize,
    /// The value for sizes 0..=4 (`FUN_00312270`); `None` for larger payloads (the game leaves the
    /// reader's value slot stale).
    pub value: Option<i32>,
}

/// What `FUN_00312300` returned for one step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PtStep {
    /// `0xFF`: the header ends (the cursor stays on it).
    End,
    /// `0xFD` / `0xFE` marker.
    Marker(u8),
    Tag(PtTag),
}

/// `FUN_00312300` [M]: one step at `*pos`.
pub fn pt_step(data: &[u8], pos: &mut usize) -> Result<PtStep> {
    let get = |p: usize| data.get(p).copied().ok_or_else(|| err("PT header runs off the buffer"));
    while get(*pos)? == 0xfc { *pos += 1; }
    let tag = get(*pos)?;
    if tag == 0xff { return Ok(PtStep::End); }
    *pos += 1;
    if tag == 0xfd || tag == 0xfe { return Ok(PtStep::Marker(tag)); }
    let mut size = get(*pos)? as usize;
    if size == 0xff {
        let b = data.get(*pos + 1..*pos + 5).ok_or_else(|| err("PT long size runs off the buffer"))?;
        size = u32::from_be_bytes(b.try_into().unwrap()) as usize;
        *pos += 4;
    }
    *pos += 1;
    let at = *pos;
    let bytes = data.get(at..at + size).ok_or_else(|| err(format!("PT tag {tag:#x} value runs off the buffer")))?;
    let value = (size < 5).then(|| pt_value(bytes));
    *pos += size;
    Ok(PtStep::Tag(PtTag { tag, at, size, value }))
}

/// The game's audio-block defaults (`FUN_0030b318`'s initialisers) for the tags the port reads.
pub const DEFAULT_CHANNELS: i32 = 1;
pub const DEFAULT_RATE: i32 = 0x5622;
pub const DEFAULT_MEM: i32 = 8;
pub const DEFAULT_CODEC: i32 = 5;

/// `0xA0` codec values seen on the disc [D]: 5 = SPU ADPCM (VAG), 0x0A = EA-XA (IOP-decoded).
pub const CODEC_VAG: i32 = 5;
pub const CODEC_EAXA: i32 = 0x0a;

/// One patch (`FUN_0030b318`'s output, the fields the port uses plus every raw tag).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Patch {
    /// Every tag in order (patch block, audio block, ignored ones too).
    pub tags: Vec<PtTag>,
    pub version: i32,
    pub channels: i32,
    /// Hz (the 16-bit field: a sign-extended 2-byte value read back unsigned).
    pub rate: u32,
    pub samples: i32,
    pub loop_start: i32,
    pub loop_end: i32,
    /// `0x88`: data offset (banks: from the bank start [D]).
    pub data_offset: i32,
    /// `0x8C` after the `0xA1` remap: 8 or 0x100 on the disc [D].
    pub mem: i32,
    pub codec: i32,
    /// `0x14` entries (offset, size) — the `CNYS` sync blocks on the speech streams [D].
    pub sync: Vec<(usize, usize)>,
}

impl Patch {
    /// The raw value of `tag` (last occurrence), if present with a small payload.
    pub fn tag(&self, tag: u8) -> Option<i32> { self.tags.iter().rev().find(|t| t.tag == tag).and_then(|t| t.value) }
}

/// `FUN_0030b318` [M]: read one patch at `*pos`. Returns the patch and whether another follows
/// (`0xFE`) — `false` at the `0xFF` end.
pub fn read_patch(data: &[u8], pos: &mut usize) -> Result<(Patch, bool)> {
    let mut p = Patch::default();
    let more = loop {
        match pt_step(data, pos)? {
            PtStep::End => break false,
            PtStep::Marker(0xfe) => break true,
            PtStep::Marker(_) => {}
            PtStep::Tag(t) => {
                if t.tag == 0x14 { p.sync.push((t.at, t.size)); }
                p.tags.push(t);
            }
        }
    };
    let get = |tag: u8, d: i32| -> i32 {
        p.tags.iter().rev().find(|t| t.tag == tag).map(|t| t.value.unwrap_or(d)).unwrap_or(d)
    };
    p.version = get(0x80, 0);
    p.channels = get(0x82, DEFAULT_CHANNELS);
    p.rate = get(0x84, DEFAULT_RATE) as u16 as u32;
    p.samples = get(0x85, 0);
    p.loop_start = get(0x86, -1);
    p.loop_end = get(0x87, -1);
    p.data_offset = get(0x88, 0);
    p.codec = get(0xa0, DEFAULT_CODEC);
    let mut mem = get(0x8c, DEFAULT_MEM);
    let a1 = get(0xa1, 1);
    // the 0xA1 remap of FUN_0030b318's tail [M]
    if a1 != 1 && mem == 8 {
        mem = 0;
        if a1 & 1 != 0 { mem = 8; }
        if a1 & 2 != 0 { mem |= 4; }
        if a1 & 0x80 != 0 { mem |= 0x20; }
        if a1 & 0x40 != 0 { mem |= 0x40; }
    }
    p.mem = mem;
    Ok((p, more))
}

/// Every patch of one PT header starting at `pos` (the `"PT"` bytes); `FUN_00308630` starts its walk at
/// PT+4.
pub fn read_pt(data: &[u8], pos: usize) -> Result<Vec<Patch>> {
    if data.get(pos..pos + 2) != Some(b"PT") { return Err(err(format!("no PT header at {pos:#x}"))); }
    let mut cur = pos + 4;
    let mut out = Vec::new();
    loop {
        let (p, more) = read_patch(data, &mut cur)?;
        out.push(p);
        if !more { break; }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// EA-XA (SNDDRV.IRX decxa16c)
// ---------------------------------------------------------------------------------------------

/// Bytes per EA-XA frame / samples per frame [M decxa16c: 28 samples per pass, 1 + 14 bytes].
pub const XA_FRAME_BYTES: usize = 15;
pub const XA_FRAME_SAMPLES: usize = 28;
/// The coefficient pairs at SNDDRV.IRX .data 0xb470 [M, read from the module image].
pub const XA_COEFS: [(i32, i32); 4] = [(0, 0), (240, 0), (460, -208), (392, -220)];

/// EA-XA decoder state: the two history samples (the IOP keeps them as 32-bit, unclamped).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XaHistory { pub h1: i32, pub h2: i32 }

/// `decxa16c` [M]: one 15-byte frame → 28 samples (32-bit, as the IOP produces them).
pub fn xa_frame(frame: &[u8], h: &mut XaHistory) -> [i32; XA_FRAME_SAMPLES] {
    let b = frame[0];
    let (c1, c2) = XA_COEFS[(b >> 4) as usize & 3];
    // `(b >> 4) * 4` indexes a table of 4 pairs; indices 4..15 would read past it (.data beyond the
    // table holds 28681, 0, …) — the disc never uses them (measured in the disc test).
    let shift = (b & 0x0f) as u32 + 8;
    let mut out = [0i32; XA_FRAME_SAMPLES];
    for i in 0..14 {
        let byte = frame[1 + i] as i32;
        // mult/mflo and addu: 32-bit wrapping arithmetic [M]
        let hi = ((byte >> 4) << 28) >> shift;
        let s1 = hi.wrapping_add(c1.wrapping_mul(h.h1).wrapping_add(c2.wrapping_mul(h.h2))) >> 8;
        let lo = ((byte & 0xf) << 28) >> shift;
        let s2 = lo.wrapping_add(c1.wrapping_mul(s1).wrapping_add(c2.wrapping_mul(h.h1))) >> 8;
        h.h2 = s1;
        h.h1 = s2;
        out[2 * i] = s1;
        out[2 * i + 1] = s2;
    }
    out
}

/// Whether a frame header byte uses a coefficient index the table holds (0..=3).
pub fn xa_frame_ok(b: u8) -> bool { (b >> 4) < 4 }

fn sat16(v: i32) -> i16 { v.clamp(-32768, 32767) as i16 }

/// Decodes `samples` EA-XA samples from `data` (contiguous 15-byte frames), continuing `h`.
pub fn decode_xa(data: &[u8], samples: usize, h: &mut XaHistory) -> Result<Vec<i16>> {
    let frames = samples.div_ceil(XA_FRAME_SAMPLES);
    if data.len() < frames * XA_FRAME_BYTES {
        return Err(err(format!("EA-XA: {samples} samples need {} bytes, have {}", frames * XA_FRAME_BYTES, data.len())));
    }
    let mut out = Vec::with_capacity(frames * XA_FRAME_SAMPLES);
    for f in data[..frames * XA_FRAME_BYTES].chunks_exact(XA_FRAME_BYTES) {
        out.extend(xa_frame(f, h).iter().map(|&s| sat16(s)));
    }
    out.truncate(samples);
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// BNKl banks
// ---------------------------------------------------------------------------------------------

/// Bytes in front of an EA-XA bank patch's frames [D: every EA-XA patch on the disc; the frame grid
/// only holds valid coefficient indices from +16, and the 16 bytes repeat the PT header's loop points
/// and sample count — measured in the disc test].
pub const XA_SAMPLE_HEADER: usize = 16;
/// Bytes in front of each channel's frames inside an `SCDl` block [D: with 0 the frame grid holds
/// invalid coefficient indices on 352 of 352 first blocks probed; with 4 on none].
pub const XA_BLOCK_PREFIX: usize = 4;

/// A `BNKl` bank (`FUN_00308088`'s header reads).
#[derive(Clone, Debug)]
pub struct Bank {
    pub version: u8,
    /// u32 +8: where the sample data starts.
    pub data_start: u32,
    /// u32 +0xC: bytes of memory-class-8 data (after the 0x100 block).
    pub size_mem8: u32,
    /// u32 +0x10 (version 5): bytes of memory-class-0x100 data (first at `data_start`).
    pub size_mem100: u32,
    /// One entry per slot: `None` = empty (offset 0), else the slot's patches.
    pub slots: Vec<Option<Vec<Patch>>>,
}

impl Bank {
    pub fn parse(data: &[u8]) -> Result<Bank> {
        if data.len() < 0x14 || &data[..4] != b"BNKl" { return Err(err("not a BNKl bank")); }
        let u32le = |o: usize| -> Result<u32> {
            data.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or_else(|| err("bank header runs off"))
        };
        let version = data[4];
        let count = u16::from_le_bytes([data[6], data[7]]) as usize;
        let data_start = u32le(8)?;
        let size_mem8 = u32le(0xc)?;
        let size_mem100 = if version == 5 { u32le(0x10)? } else { 0 };
        let mut slots = Vec::with_capacity(count);
        for i in 0..count {
            let at = 0x14 + 4 * i;
            let rel = u32le(at)?;
            if rel == 0 { slots.push(None); continue; }
            slots.push(Some(read_pt(data, at + rel as usize)?));
        }
        Ok(Bank { version, data_start, size_mem8, size_mem100, slots })
    }

    /// The byte range of the memory class `mem` (0x100 first, then 8) [M FUN_00308088 + D].
    pub fn mem_range(&self, mem: i32) -> Option<std::ops::Range<usize>> {
        let s = self.data_start as usize;
        match mem {
            0x100 => Some(s..s + self.size_mem100 as usize),
            8 => Some(s + self.size_mem100 as usize..s + self.size_mem100 as usize + self.size_mem8 as usize),
            _ => None,
        }
    }

    /// Bytes a patch's samples occupy in its codec.
    pub fn patch_bytes(p: &Patch) -> Option<usize> {
        let n = p.samples.max(0) as usize;
        match p.codec {
            CODEC_VAG => Some(n.div_ceil(crate::vag::FRAME_SAMPLES) * crate::vag::FRAME_BYTES),
            CODEC_EAXA => Some(XA_SAMPLE_HEADER + n.div_ceil(XA_FRAME_SAMPLES) * XA_FRAME_BYTES),
            _ => None,
        }
    }

    /// The 16-byte header in front of an EA-XA patch's frames [D]: (codec byte, loop start, loop end,
    /// samples) — byte 1, then three i32 LE at +4/+8/+0xC.
    pub fn xa_sample_header(data: &[u8], p: &Patch) -> Option<(u8, i32, i32, i32)> {
        let o = p.data_offset as usize;
        let h = data.get(o..o + XA_SAMPLE_HEADER)?;
        let i = |k: usize| i32::from_le_bytes(h[k..k + 4].try_into().unwrap());
        Some((h[1], i(4), i(8), i(0xc)))
    }

    /// Decodes one patch of the bank (`data` = the whole bank file) to mono 16-bit PCM.
    pub fn decode(data: &[u8], p: &Patch) -> Result<Vec<i16>> {
        if p.channels != 1 { return Err(err(format!("bank patch with {} channels", p.channels))); }
        let n = p.samples.max(0) as usize;
        let len = Self::patch_bytes(p).ok_or_else(|| err(format!("codec {:#x}", p.codec)))?;
        let off = p.data_offset as usize;
        let src = data.get(off..off + len).ok_or_else(|| err("patch data runs off the bank"))?;
        match p.codec {
            CODEC_VAG => { let mut v = crate::vag::decode(src); v.truncate(n); Ok(v) }
            _ => decode_xa(&src[XA_SAMPLE_HEADER..], n, &mut XaHistory::default()),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// SCHl streams (.asf)
// ---------------------------------------------------------------------------------------------

/// One chunk of an `.asf` stream: 4-byte id, u32 LE size (including the 8-byte head) [D].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk { pub id: [u8; 4], pub at: usize, pub size: usize }

/// Splits a stream into its chunks (stops at `SCEl` or the end of the data).
pub fn chunks(data: &[u8]) -> Result<Vec<Chunk>> {
    let mut out = Vec::new();
    let mut p = 0;
    while p + 8 <= data.len() {
        let id: [u8; 4] = data[p..p + 4].try_into().unwrap();
        let size = u32::from_le_bytes(data[p + 4..p + 8].try_into().unwrap()) as usize;
        if size < 8 || p + size > data.len() { return Err(err(format!("chunk {:?} at {p:#x} size {size:#x}", String::from_utf8_lossy(&id)))); }
        out.push(Chunk { id, at: p, size });
        p += size;
        if &id == b"SCEl" { break; }
    }
    Ok(out)
}

/// A parsed `.asf` stream: the `SCHl` header's patch and the `SCDl` blocks.
#[derive(Clone, Debug)]
pub struct Stream {
    pub header: Patch,
    pub chunks: Vec<Chunk>,
}

impl Stream {
    pub fn parse(data: &[u8]) -> Result<Stream> {
        let chunks = chunks(data)?;
        let first = chunks.first().ok_or_else(|| err("empty stream"))?;
        if &first.id != b"SCHl" { return Err(err("stream does not start with SCHl")); }
        let hdr = read_pt(&data[..first.at + first.size], first.at + 8)?;
        Ok(Stream { header: hdr.into_iter().next().unwrap(), chunks })
    }

    /// `FUN_00308d68` [M]: one `SCDl` block's sample count and per-channel data offsets (in `data`).
    pub fn block(&self, data: &[u8], c: &Chunk) -> Result<(usize, Vec<usize>)> {
        let ch = self.header.channels.max(1) as usize;
        let rd = |o: usize| -> Result<u32> {
            data.get(o..o + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap())).ok_or_else(|| err("SCDl runs off"))
        };
        let samples = (rd(c.at + 8)? & 0x7fff_ffff) as usize;
        let mut offs = Vec::with_capacity(ch);
        for i in 0..ch {
            offs.push(c.at + 0xc + 4 * ch + rd(c.at + 0xc + 4 * i)? as usize);
        }
        Ok((samples, offs))
    }

    /// Total samples over the `SCDl` blocks.
    pub fn block_samples(&self, data: &[u8]) -> Result<usize> {
        let mut n = 0;
        for c in self.chunks.iter().filter(|c| &c.id == b"SCDl") { n += self.block(data, c)?.0; }
        Ok(n)
    }

    /// Decodes every channel (EA-XA only — the only stream codec on the disc [D]); the history carries
    /// from block to block [S].
    pub fn decode(&self, data: &[u8]) -> Result<Vec<Vec<i16>>> {
        if self.header.codec != CODEC_EAXA { return Err(err(format!("stream codec {:#x}", self.header.codec))); }
        let ch = self.header.channels.max(1) as usize;
        let mut out = vec![Vec::new(); ch];
        let mut hist = vec![XaHistory::default(); ch];
        for c in self.chunks.iter().filter(|c| &c.id == b"SCDl") {
            let (n, offs) = self.block(data, c)?;
            let end = c.at + c.size;
            for (i, &o) in offs.iter().enumerate() {
                let src = data.get(o + XA_BLOCK_PREFIX..end).ok_or_else(|| err("SCDl channel offset past the block"))?;
                out[i].extend(decode_xa(src, n, &mut hist[i])?);
            }
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------------------------
// Text files
// ---------------------------------------------------------------------------------------------

/// `banks.ini`: `[section]` then `Key=path` lines, in file order (duplicates kept, as on the disc).
pub fn parse_banks_ini(text: &str) -> Vec<(String, Vec<(String, String)>)> {
    let mut out: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for line in text.lines() {
        let l = line.trim();
        if let Some(s) = l.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            out.push((s.to_string(), Vec::new()));
        } else if let Some((k, v)) = l.split_once('=') {
            if let Some(sec) = out.last_mut() { sec.1.push((k.trim().to_string(), v.trim().to_string())); }
        }
    }
    out
}

/// A `.h` id header: `#define SFX_<name> <id>` lines → (name without the `SFX_` prefix kept, id).
pub fn parse_sfx_header(text: &str) -> Vec<(String, u32)> {
    text.lines().filter_map(|l| {
        let mut it = l.split_whitespace();
        (it.next()? == "#define").then_some(())?;
        let name = it.next()?.to_string();
        let id = it.next()?.parse().ok()?;
        Some((name, id))
    }).collect()
}

/// `FUN_002f54e0` [M]: the lookup key the game files a `.h` name under. `#define%31s%d` reads the
/// name; when an `_` follows the `SFX_` prefix (at index 4 or later) the key is `SFX_` + the text after
/// the first such `_`; every `.` then ends the key. So `SFX_par_SniperFire.ptc` → `SFX_SniperFire`,
/// `SFX_IN-idle.ptc` → `SFX_IN-idle`. Lookups (`FUN_002f5880` → `FUN_002f64e8`) compare with the
/// case-insensitive `FUN_002a9198`.
pub fn sfx_key(name: &str) -> String {
    let name: String = name.chars().take(31).collect();
    let tail = name.get(4..).unwrap_or("");
    let key = match tail.find('_') { Some(i) => format!("SFX_{}", &tail[i + 1..]), None => name.clone() };
    key.split('.').next().unwrap_or("").to_string()
}

/// A bank's name → slot table (`.h` keys, matched case-insensitively like the game).
#[derive(Clone, Debug, Default)]
pub struct SfxTable(pub Vec<(String, u32)>);

impl SfxTable {
    pub fn from_header(text: &str) -> SfxTable { SfxTable(parse_sfx_header(text).into_iter().map(|(n, i)| (sfx_key(&n), i)).collect()) }
    /// The slot for `name` (case-insensitive; the first match — the map keeps the first insert [S]).
    pub fn get(&self, name: &str) -> Option<u32> { self.0.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|e| e.1) }
}

/// One mix `.ini` line as `FUN_002f3058` keeps it: group name, volume, the trailing int.
#[derive(Clone, Debug, PartialEq)]
pub struct MixLine { pub name: String, pub volume: f32, pub param: i32 }

/// `FUN_002f3058` [M]: the lines of a mix `.ini` that `sscanf("%[^=]=%f, %d")` accepts.
pub fn parse_mix_ini(data: &[u8]) -> Vec<MixLine> {
    let mut out = Vec::new();
    let mut p = 0;
    while p < data.len() {
        let start = p;
        while p < data.len() && data[p] != b'\r' && p - start < 0x7f { p += 1; }
        let line = &data[start..p];
        while p < data.len() && data[p] != b'\r' { p += 1; }
        p += 2;
        if let Some(m) = scan_mix_line(line) { out.push(m); }
    }
    out
}

/// `%[^=]=%f, %d` with C sscanf semantics for the inputs the disc has: the name is everything before
/// `=` (at least one char), `%f` skips leading whitespace, `", "` matches literally after optional
/// whitespace handling (a space in the format matches any run of whitespace), `%d` skips whitespace.
fn scan_mix_line(line: &[u8]) -> Option<MixLine> {
    let s = std::str::from_utf8(line).ok()?;
    let (name, rest) = s.split_once('=')?;
    if name.is_empty() { return None; }
    let rest = rest.trim_start();
    let num_end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E')).unwrap_or(rest.len());
    let volume: f32 = rest[..num_end].parse().ok()?;
    let rest = rest[num_end..].strip_prefix(',')?.trim_start();
    let int_end = rest.find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '+')).unwrap_or(rest.len());
    let param: i32 = rest[..int_end].parse().ok()?;
    Some(MixLine { name: name.trim_end_matches(' ').to_string(), volume, param })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pt_value_sign_extends_like_fun_00312270() {
        assert_eq!(pt_value(&[0x7f]), 127);
        assert_eq!(pt_value(&[0x80]), -128);
        assert_eq!(pt_value(&[0x3e, 0x80]), 16000);
        assert_eq!(pt_value(&[0xac, 0x44]), 0xac44 - 0x1_0000);
        assert_eq!(pt_value(&[0x00, 0xbb, 0x80]), 48000);
        assert_eq!(pt_value(&[0xff, 0xff, 0xff]), -1);
        assert_eq!(pt_value(&[0, 0, 1, 0x30]), 0x130);
        assert_eq!(pt_value(&[]), 0);
    }

    #[test]
    fn g_common_slot_0_header() {
        // the first slot of g_common.bnk, bytes as on the disc [D] (MIS01.VIV, after RefPack)
        let pt = [0x50, 0x54, 0x05, 0x00, 0xfd, 0x80, 0x01, 0x02, 0x85, 0x02, 0x0e, 0x00, 0x84, 0x02, 0x3e, 0x80,
            0x88, 0x04, 0x00, 0x00, 0x01, 0x30, 0x8a, 0x04, 0x00, 0x00, 0x00, 0x00, 0xff];
        let ps = read_pt(&pt, 0).unwrap();
        assert_eq!(ps.len(), 1);
        let p = &ps[0];
        assert_eq!((p.version, p.channels, p.rate, p.samples, p.data_offset), (2, 1, 16000, 3584, 0x130));
        assert_eq!((p.codec, p.mem, p.loop_start, p.loop_end), (CODEC_VAG, 8, -1, -1));
    }

    #[test]
    fn a_two_byte_rate_above_0x7fff_reads_back_unsigned() {
        let pt = [b'P', b'T', 5, 0, 0x84, 0x02, 0xac, 0x44, 0xff];
        assert_eq!(read_pt(&pt, 0).unwrap()[0].rate, 44100);
    }

    #[test]
    fn fe_starts_another_patch_and_fc_pads() {
        let pt = [b'P', b'T', 5, 0, 0xfc, 0xfc, 0x85, 0x01, 0x10, 0xfe, 0x85, 0x01, 0x20, 0xff];
        let ps = read_pt(&pt, 0).unwrap();
        assert_eq!(ps.iter().map(|p| p.samples).collect::<Vec<_>>(), vec![16, 32]);
    }

    #[test]
    fn xa_frame_matches_the_decxa16c_formula() {
        // coef pair 2 (460, -208), shift 0+8; nibbles 7 then 8 (-8)
        let mut f = [0u8; 15];
        f[0] = 0x20;
        f[1] = 0x78;
        let mut h = XaHistory { h1: 1000, h2: 500 };
        let out = xa_frame(&f, &mut h);
        let s1 = ((7 << 28) >> 8) + 460 * 1000 - 208 * 500 >> 8;
        let s2 = ((-8i32 << 28) >> 8) + 460 * s1 - 208 * 1000 >> 8;
        assert_eq!((out[0], out[1]), (s1, s2));
        // the history is not clamped (decxa16c stores the raw sum)
        let mut big = XaHistory { h1: 40000, h2: 0 };
        let o = xa_frame(&[0x10; 15], &mut big);
        assert!(o[0] > 32767, "{}", o[0]);
    }

    #[test]
    fn sfx_keys_follow_fun_002f54e0() {
        assert_eq!(sfx_key("SFX_par_SniperFire.ptc"), "SFX_SniperFire");
        assert_eq!(sfx_key("SFX_par_sniperBolt.ptc"), "SFX_sniperBolt");
        assert_eq!(sfx_key("SFX_IN-idle.ptc"), "SFX_IN-idle");
        assert_eq!(sfx_key("SFX_SkidFront1"), "SFX_SkidFront1");
        assert_eq!(sfx_key("SFX_par_DomCarLdEn.ptc"), "SFX_DomCarLdEn");
        let t = SfxTable::from_header("#define SFX_par_sniperBolt.ptc            71\n#define SFX_HeliMachGun 73\n");
        assert_eq!(t.get("SFX_sniperbolt"), Some(71));
        assert_eq!(t.get("SFX_helimachgun"), Some(73));
    }

    #[test]
    fn mix_ini_lines_parse_like_the_sscanf() {
        let ini = b"0:Fade Effects = 0.300, -1\r\nSpeech = 1.000, 0\r\nbad line\r\nPlayer Car = 0.350, 5\r\n";
        let m = parse_mix_ini(ini);
        assert_eq!(m.len(), 3);
        assert_eq!(m[0], MixLine { name: "0:Fade Effects".into(), volume: 0.3, param: -1 });
        assert_eq!(m[2], MixLine { name: "Player Car".into(), volume: 0.35, param: 5 });
    }
}
