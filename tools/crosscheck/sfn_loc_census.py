"""Independent census of the driving fonts (.sfn, FNTS) and string tables (.loc, LOCH).

Writes the goldens `crates/nf-formats/golden/sfn_census.tsv` and `loc_census.tsv` that
`crates/nf-formats/tests/sfn_loc_disc.rs` compares against the Rust readers. Written separately from
`sfn.rs` / `loc.rs` (own RefPack decoder, own field walk) so the two can disagree.

Input: a folder holding one sub-folder per `DRIVING/*.VIV`, each the output of
`nf-extract bigf-extract <iso> /DRIVING/<VIV>.VIV <folder>/<VIV>` (raw entries, still compressed).
Usage: python -I sfn_loc_census.py <folder> <golden dir>
"""
import hashlib, os, struct, sys


def refpack(src):
    """EA RefPack (10 FB / 11 FB header) decoder."""
    flags = src[0]
    p = 2
    if flags & 0x01:
        p += 3  # compressed size
    big = flags & 0x80
    n = 4 if big else 3
    size = int.from_bytes(src[p:p + n], 'big')
    p += n
    out = bytearray()
    while p < len(src):
        b0 = src[p]
        if b0 < 0x80:
            b1 = src[p + 1]; p += 2
            lit = b0 & 3; out += src[p:p + lit]; p += lit
            cnt = ((b0 >> 2) & 7) + 3; off = ((b0 & 0x60) << 3) + b1 + 1
        elif b0 < 0xc0:
            b1, b2 = src[p + 1], src[p + 2]; p += 3
            lit = b1 >> 6; out += src[p:p + lit]; p += lit
            cnt = (b0 & 0x3f) + 4; off = ((b1 & 0x3f) << 8) + b2 + 1
        elif b0 < 0xe0:
            b1, b2, b3 = src[p + 1], src[p + 2], src[p + 3]; p += 4
            lit = b0 & 3; out += src[p:p + lit]; p += lit
            cnt = ((b0 & 0x0c) << 6) + b3 + 5; off = ((b0 & 0x10) << 12) + (b1 << 8) + b2 + 1
        elif b0 < 0xfc:
            p += 1; lit = ((b0 & 0x1f) << 2) + 4; out += src[p:p + lit]; p += lit; continue
        else:
            p += 1; lit = b0 & 3; out += src[p:p + lit]; break
        for _ in range(cnt):
            out.append(out[-off])
    assert len(out) == size, (len(out), size)
    return bytes(out)


def load(path):
    d = open(path, 'rb').read()
    if len(d) > 2 and d[1] == 0xfb and d[0] in (0x10, 0x11, 0x90, 0x91):
        d = refpack(d)
    return d


def sfn_row(key, d):
    u16 = lambda o: struct.unpack_from('<H', d, o)[0]
    u32 = lambda o: struct.unpack_from('<I', d, o)[0]
    count, flags = u16(0xa), u32(0xc)
    go, ko, so = u32(0x14), u32(0x18), u32(0x1c)
    glyphs = [d[go + 12 * i: go + 12 * i + 12] for i in range(count)]
    adv = sum(struct.unpack('b', g[8:9])[0] for g in glyphs)
    kerned = sum(1 for g in glyphs if g[11])
    nk = u32(ko) if ko else 0
    ksum = sum(struct.unpack_from('b', d, ko + 4 + 4 * i + 2)[0] for i in range(nk)) if ko else 0
    sw, sh = u16(so + 4), u16(so + 6)
    return '\t'.join(map(str, [key, len(d), hashlib.sha1(d).hexdigest(), '%#x' % u16(8), count, '%#x' % flags,
                                d[0x12], d[0x13], u16(go), u16(go + 12 * (count - 1)), adv, kerned, nk, ksum,
                                '%#04x' % d[so], sw, sh, hashlib.sha1(b''.join(glyphs)).hexdigest()]))


def loc_row(key, d):
    u16 = lambda o: struct.unpack_from('<H', d, o)[0]
    u32 = lambda o: struct.unpack_from('<I', d, o)[0]
    t = u32(4)
    nid = u32(t + 8)
    ids = [struct.unpack_from('<HH', d, t + 16 + 4 * i) for i in range(nid)]
    s = u32(0x10 + 4 * u16(0xe))
    n = u32(s + 12)
    strs = []
    for i in range(n):
        o = s + u32(s + 16 + 4 * i)
        strs.append(d[o:d.index(b'\0', o)])
    resolved = sum(1 for k, v in ids if v < n)
    return '\t'.join(map(str, [key, len(d), hashlib.sha1(d).hexdigest(), d[8], u16(0xc), nid, n, resolved,
                                sum(map(len, strs)), sum(1 for x in strs if b'%' in x),
                                hashlib.sha1(b'\0'.join(strs)).hexdigest()]))


def main(root, out):
    sfn = ['entry\tbytes\tsha1\tversion\tglyphs\tflags\tline_a\tline_b\tfirst\tlast\tadvance_sum\tkerned\tkerns\tkern_sum\tsheet_type\tsheet_w\tsheet_h\tglyph_sha1']
    loc = ['entry\tbytes\tsha1\tflags\ttables\tids\tstrings\tids_resolved\tstring_bytes\twith_percent\tstrings_sha1']
    for viv in sorted(os.listdir(root)):
        for dp, _, fs in os.walk(os.path.join(root, viv)):
            for f in sorted(fs):
                rel = os.path.relpath(os.path.join(dp, f), os.path.join(root, viv)).replace('/', '\\')
                key = '/DRIVING/%s.VIV/%s' % (viv, rel)
                if f.lower().endswith('.sfn'):
                    sfn.append(sfn_row(key, load(os.path.join(dp, f))))
                elif f.lower().endswith('.loc'):
                    loc.append(loc_row(key, load(os.path.join(dp, f))))
    open(os.path.join(out, 'sfn_census.tsv'), 'w', newline='\n').write('\n'.join(sfn) + '\n')
    open(os.path.join(out, 'loc_census.tsv'), 'w', newline='\n').write('\n'.join(loc) + '\n')


if __name__ == '__main__':
    main(sys.argv[1], sys.argv[2])
