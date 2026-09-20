#!/usr/bin/env python3
"""Extract an embedded SWF from the supplied Windows Flash projector.

The CobbleBot projector contains a zlib stream whose uncompressed payload starts
with a normal CWS SWF. This tool deliberately searches for that structure rather
than hard-coding the current projector offset, but reports the recovered offset
for reproducibility.
"""
from __future__ import annotations
from pathlib import Path
import argparse, hashlib, json, struct, zlib

ZLIB_SECOND_BYTES = {0x01, 0x5E, 0x9C, 0xDA}

def sha256(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()

def find_embedded_swf(exe: bytes):
    for off in range(len(exe)-2):
        if exe[off] != 0x78 or exe[off+1] not in ZLIB_SECOND_BYTES:
            continue
        # RFC1950 header check: CMF/FLG must be divisible by 31.
        if ((exe[off] << 8) | exe[off+1]) % 31:
            continue
        try:
            d = zlib.decompressobj()
            raw = d.decompress(exe[off:]) + d.flush()
        except zlib.error:
            continue
        if len(raw) >= 8 and raw[:3] in (b'CWS', b'FWS'):
            consumed = len(exe[off:]) - len(d.unused_data)
            return off, consumed, raw
    raise RuntimeError('no embedded zlib-wrapped SWF found')

def uncompress_cws(cws: bytes) -> bytes:
    if cws[:3] == b'FWS':
        return cws
    if cws[:3] != b'CWS':
        raise ValueError('expected CWS/FWS')
    version = cws[3]
    declared = struct.unpack_from('<I', cws, 4)[0]
    body = zlib.decompress(cws[8:])
    fws = b'FWS' + bytes([version]) + cws[4:8] + body
    if len(fws) != declared:
        raise ValueError(f'decompressed length {len(fws)} != declared {declared}')
    return fws

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('projector', type=Path)
    ap.add_argument('outdir', type=Path)
    ns=ap.parse_args()
    exe=ns.projector.read_bytes(); ns.outdir.mkdir(parents=True,exist_ok=True)
    off, consumed, wrapped = find_embedded_swf(exe)
    # The outer zlib stream expands to an original CWS file.
    original = wrapped
    if original[:3] != b'CWS':
        raise RuntimeError(f'expected embedded CWS, got {original[:3]!r}')
    fws = uncompress_cws(original)
    (ns.outdir/'cobblebot_original_cws.swf').write_bytes(original)
    (ns.outdir/'cobblebot.swf').write_bytes(fws)
    meta={
      'projector_size':len(exe),'projector_sha256':sha256(exe),
      'outer_zlib_offset_decimal':off,'outer_zlib_offset_hex':hex(off),
      'outer_zlib_consumed_bytes':consumed,
      'cws_size':len(original),'cws_sha256':sha256(original),
      'fws_size':len(fws),'fws_sha256':sha256(fws),
      'swf_version':fws[3],'declared_fws_size':struct.unpack_from('<I',fws,4)[0],
    }
    (ns.outdir/'source-fingerprints.json').write_text(json.dumps(meta,indent=2)+"\n")
    print(json.dumps(meta,indent=2))
if __name__=='__main__': main()
