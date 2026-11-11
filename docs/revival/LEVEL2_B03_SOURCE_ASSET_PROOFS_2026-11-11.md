# LEVEL2-B03 — Original Flash stage-aware static scene and sprite proof (FINAL)

**Status:** B03 code and its real Windows source/native regressions **VERIFIED** for bounded development-only source previews; all future stages still NOT playable. Date: 2026-11-11. Branch revive/level2-b03-source-asset-proofs-2026-11-11; [draft PR #23](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/pull/23) (base B02 draft PR22). Tested source code commit [d64f4b4](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/commit/d64f4b453550f8f654b7d993b2e7b9f9412156df). B03 explicitly stops here; B04 not activated.

## Verified gates

| Gate | Actual result / evidence |
| --- | --- |
| New original-Flash source art Windows run | [#42995623413](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42995623413) COMPLETED SUCCESS 9/9 recorded job steps at d64f4b4 |
| Existing full native Level1A Windows fidelity run | [#42995623510](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42995623510) COMPLETED SUCCESS 81/81 recorded job steps at same d64f4b4 |
| Canonical original FWS | SHA256 3d9e1992404313bc2e2652d46a5838160ef721e2ad559fdd7455f3c54f8ac1ed, 2,819,100 bytes, 79/79 bitmap decoding |
| Rust source unchanged | 27/27 cargo tests passed; no new Rust modules/crates/native runtime assets |
| Historical original 1x regression | 15/15 source-derived legacy screenshot hashes identical |
| Installed-copy HQ regression | 20/20 original vs copied real EXE 1800x1200 screenshot hashes identical |
| Genuine OS and runtime | Win32 GUI, F8, F/Esc fullscreen, focus visual, PE/Shell icon, source Cargo.lock --locked, integration child exit 0 |
| p95 performance | render 20.8697ms, full-level scenery 26.7165ms, both <40ms |
| Sound | BLOCKED_HOSTED_RUNNER_NO_AUDIO_ENDPOINT / WinMM MCI277, not real speaker testing |
| B03 source proof artifact | [#13843344043](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42995623413/artifacts/13843344043) actual ZIP independently downloaded, 34 entries, images/manifests and hashes inspected |

## Source art contents and original provenance

The development-only Python baker (reverse-engineering/tools/b03_bake_stage_asset_proofs.py) checks source manifests and the canonical FWS, selects two original display-list geometry-derived viewport origins per stage, and renders two 600x400 1x tiles and two independently original-source-rasterized 1800x1200 3x tiles per stage. Original source actor sprite frame0 is separate from static scenery, never forged as live gameplay. The full detailed original symbols and coordinates are in RE-20261111-013.

| Stage | Root 0-based | Game / scene / collision symbol | original actor/prop sprite |
| --- | --- | --- | --- |
| Gotham Bank streets Level1B | 20 | 1075 / 783 / 782 | goon1 #880 |
| Penguin Level1C | 21 | 1251 / 1095 / 1094 | boss #1250 |
| Gotham Labs 2A | 23 | 1402 / 1317 / 1316 | door_01 #1325 |
| Gotham Labs 2B | 25 | 1496 / 1485 / 1484 | door_01 #1325 |

The 34-entry artifact contains 16 scenery tiles, 4 sprite PNGs, 8 tile manifests, 4 provenance JSONs, one JSON summary and one captioned contact sheet (1240x2000; SHA256 116e2176c53ccfd4a529f6f587343ab4d7b71e91b2b812c61884148bfab7899d). Independent archive verification reproduced the SHA256 of every tile, actor and sheet, checked dimensions, confirmed partial flags in all manifests and confirmed no .swf archive members. Viewable contact sheet shows separate original source scenery/props and says NOT actual Flash gameplay. B03 prints B03_SOURCE_STAGE_PROOF_PASS for all four and B03_PREVIEW_PACK_GUARD_PASS.

## Changed paths in B03

1. reverse-engineering/tools/b03_bake_stage_asset_proofs.py — canonical asset/symbol checks, static/dynamic exclusion, two source-derived viewport samples/stage, original actor frame0, report/sheet.
2. reverse-engineering/tools/bake_level_tiles.py — repeatable optional --only-tile X Y; retains the existing default full tile-grid behavior, marks sample output as partial and never embeddable.
3. reverse-engineering/tools/pack_level_tiles.py — reject any manifest flagged partial_proof_only or never_embed_as_complete_level; negative test proves incomplete native packing fails.
4. .github/workflows/b03-upcoming-stage-source-art-proofs.yml — isolated free Windows source extraction and artifact; unmodified production exe.
5. .github/workflows/build-level1a-fidelity.yml — add only this development branch to existing full CI trigger; no test weakening/golden update.

## Failure history, caveats, decision

First Windows source run [#42994663503](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42994663503) failed because source exclusions were computed but not handed to the renderer; corrected version passed [#42994906455](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42994906455). Two concurrently superseded full runs cancelled, not claimed passes. The final d64f4b4 code added and passed native partial-pack refusal, then both final genuine Windows CI jobs passed. This is the tested final code, not a hypothetical plan.

These are partial art proofs, **not whole floors, animation sequences, physically played Flash screenshots or accepted camera/collision/audio equivalence**. Prefix-based source-symbol exclusions will need independent dynamic-depth analysis before full playable 1B assets. Existing dev Pillow version range and downloaded archive (strict FWS SHA checked) are not supply-chain-pinned; future hardening can be separate. No proprietary original binary/sound pack or incomplete stage data was committed into shipped game. Old Level1A release tag/branch stays frozen at 49e8fd343e4f37f045ba71f24fff0999211f0f11; main and PRs #18/#21/#22 untouched; PR #23 remains DRAFT/UNMERGED.

**NEXT one separately authorized task:** B04 original 1A→1B fade/transition mechanics and test plan, without enabling a broken stage on partial B03 art. Full 1B stage tiles, actors, collisions and source-fidelity review are separate future asset batches. Six mandatory documentation records updated and linked to RE-20261111-013 and AUD-20261111-013.
