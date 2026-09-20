# Pipeline verification

The complete reverse-engineering toolchain was rerun from the original projector into a fresh empty output directory after the tools were finalized.

Result: **PASS**.

Verified outputs:

- projector extraction reproduced the canonical CWS/FWS hashes
- recursive SWF scan found 342 sprites, 231 DoAction blocks, 38 sounds, 79 bitmap definitions
- bitmap decoder: **79 decoded, 0 errors**
- sound extractor: **38 sounds** (`33 mp3`, `5 swf-adpcm`)
- shape/bitmap-fill linker: **79 bitmap-backed shapes, 0 errors**
- global function extraction: **45 gameplay functions** with nested bodies retained
- collision-mask generation succeeded for all **9 gameplay stages**

Generated level-space collision-mask dimensions:

| Stage | Ground symbol | Origin | Size |
|---|---:|---:|---:|
| level1a | 151 | (-5539,-1929) | 11671×3732 |
| level1b | 782 | (-11196,-886) | 12102×1413 |
| Penguin | 1094 | (-1731,-549) | 2813×926 |
| level2a | 1316 | (-1542,-2307) | 8781×10060 |
| level2b | 1484 | (-1542,-2307) | 8781×10060 |
| level3a | 1664 | (-1257,-2288) | 17876×2918 |
| Kabuki | 1811 | (-1070,-311) | 788×546 |
| level4a | 1961 | (-2354,-692) | 4379×938 |
| CobbleBot | 2050 | (-1472,-988) | 1236×1234 |

The masks are intentionally extremely compressible 1-bit PNGs; their large logical dimensions reflect the original scrolling level coordinate systems rather than giant full-color screenshots.
