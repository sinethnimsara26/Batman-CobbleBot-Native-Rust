# Level 1A HUD — source geometry and HQ collision audit (HUD-REAL-02)
**Date:** 2026-11-10 Sri Lanka  
**Type:** Research/specification ONLY — no Rust, assets, workflow or gameplay modifications  
**Pinned source HEAD:** `82213d56cb7952a864a0aa694d6eca3a8f8b19df` on `revive/level1a-intro-hud-fidelity-2026-11-09`; draft PR #18, issue #17 open.  
**Evidence:** current code `src/render.rs` (draw_hud, draw_hud_hq, draw_hud_counter_band_hq), `reverse-engineering/tools/bake_hud_assets.py` (field_box, draw_field, digit_atlas); [green Windows run #42681546189](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42681546189) at game SHA `68a885e`; [SWF-source report artifact #13702988209](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42681546189/artifacts/13702988209); [HUD comparisons artifact #13703008183](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42681546189/artifacts/13703008183); [real Win32 GUI artifact #13702968250](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42681546189/artifacts/13702968250). Human screenshot `image(20261109-193742).png`, 1920×1080, observed in conversation, SHA-256 `138412e6a777e6791786f99a16e233d19adf5b0a10ce2ef37da5b802299a6f7b` (NOT uploaded to public repository).

## 1. Coordinate-space conventions and reproduction
- Flash source stage: 600×400 logical pixels; HQ surface: 1800×1200 actual rendering pixels. Windows screenshot for this observation is 1920×1080 window with a letterboxed stage span approximately desktop pixel `x=172..1747`, `y=29..1079`: scale `1576/600 ≈ 2.6267`. **Screen pixels and logical pixels must not be confused.**
- Original HUD is Flash sprite **716**, root frame **19** placed at logical `(159,-11)`. Renderer source-style image anchor `(0.4810638428,1.1601379395)`; source generated HUD canvas 283×86. Actual source-stage field position is `[root_x - anchor_x + local_box_x0, root_y - anchor_y + local_box_y0, ...]` with generated `assets/hud.json` boxes. This is the manifest transform, **not** an invented icon placement.
- The native HQ copy is baked at scale 6 and downsampled to 3, and is rendered 1:1. HQ shell has `837×246` pixels and anchor `(-3.5568084717,-1.5195861816)` in HQ pixels. The 1× and 3× source dynamic **field centers** were checked in the real CI: zero registration drift within floating-point precision. Their glyphs may nevertheless drift because runtime overrides field-centered placement.
- Reproduce from source CI: download run's GUI and source-report artifacts; inspect generated `assets/hud.json`, `assets/hud_hq.json` emitted in the Windows job log, inspect `src/render.rs::draw_hud_hq`, and compare visual screenshot. Original SWF **DefineEditText** metadata from `intro-hud-source-report.json` identifies fields: 700 BATARANG, 710 HEALTH, 711 SCORE, 715 lives x, 702 _root.batarangs, 713 _root.score, 714 _root.lives. All have original white 255/255/255/255 text and an `_sans`/Arial source-style route; Impact-style gold lettering is a separately styled HQ candidate, **not original Flash parity**.

## 2. Measured ORIGINAL source-field rectangles (stage logical pixels)
Boxes below are derived from actual generated 1× HUD manifest `fields` as printed in successful CI #42681546189, plus the root translation above. All coordinates are approximate to 0.01 logical pixels. Box coordinates show **field capacity**, not actual drawn ink/opaque silhouette.

| SWF field | Original text/variable | Stage bbox `[left,top,right,bottom]` | Center `(x,y)` | Original font |
|---|---|---|---|---|
| 700 | BATARANG | [167.00, 1.00, 257.40, 20.50] | (212.20, 10.75) | 12 logical px |
| 702 | _root.batarangs | [178.00, 44.00, 244.25, 68.10] | **(211.125, 56.05)** | 16 px |
| 710 | HEALTH | [225.45, 1.00, 379.60, 20.50] | (302.525, 10.75) | 12 px |
| 711 | SCORE | [324.10, 1.00, 448.85, 20.50] | (386.475, 10.75) | 12 px |
| 713 | _root.score | [331.55, 15.10, 441.65, 54.85] | **(386.60, 34.975)** | 32 px |
| 714 | _root.lives | [302.00, 46.00, 325.50, 67.90] | **(313.75, 56.95)** | 16 px |
| 715 | lives x | [260.00, 48.00, 320.50, 65.35] | (290.25, 56.675) | 12 px |

**Critical subtlety:** original field rectangles 714 and 715 overlap in **metadata boxes** (x302..320.5/y48..65.35); this does not prove the actual glyph pixels overlap. Use rasterized ink bounds/masks for final acceptance rather than rejecting every rectangular-field intersection.

## 3. CURRENT HQ renderer placements vs source (one-digit values)
Live source `src/render.rs::draw_hud_hq`: `legacy_x=round(159-.481...)=159`, `legacy_y=round(-11-1.160...)=-12`; HQ digits are 3× atlas frames with small cell `42×56` HQ px (14×18.67 logical) and large `66×87` HQ px (22×29 logical), per freshly read Windows build log. These are **CELL rectangles**, not tight glyph-ink bounds.

| Item | Source center | Current HQ numeric center | Center displacement | Current single-digit CELL bbox (logical) |
|---|---|---|---|---|
| Batarang number | (211.125, 56.05) | **(212,70)** | (+0.875,+13.95) | [205.00,60.67,219.00,79.33] |
| Lives number | (313.75,56.95) | **(304,65)** | (-9.75,+8.05) | [297.00,55.67,311.00,74.33] |
| Score number | (386.60,34.975) | **(387,35)** | (+0.40,+0.025) | [376.00,20.50,398.00,49.50] |

Extra independent HQ baker offset: for static **715** `LIVES X`, `draw_field` takes original center (290.25,56.675) and explicitly shifts it **(-4,+7) logical pixels** to approximately (286.25,63.675), while the Rust lives digit uses (304,65). Thus the caption and digit are placed by two unrelated coordinate owners, producing a narrow/fragile visual gap, particularly for 2-digit lives. The Batarang number's entire cell is deliberately drawn below its source field.

Dynamic text expansion at the existing atlas cell widths: score 6 digits uses 6×22 = **132 logical pixels** vs original score field width **110.10** logical pixels; source-style alignment cannot accommodate that without a deliberate overflow policy. Lives 2 digits consume 28 logical pixels vs the 23.5 logical original lives field. Both are future QA cases, **not verified clipping in current 0/5 screenshot**.

## 4. CONFIRMED extra black box — source layer vs native HQ overlay
- Native only: `draw_hud_counter_band_hq(fb,legacy_x*3,(legacy_y+60)*3,283*3,30*3)` is invoked **before** blitting original HUD shell. Its rectangle is stage **x=[159,442), y=[48,78)**; width 283, height 30. The source file paints it with ARGB `0xd0121017` (interior alpha 208/255 ≈81.6%) and `0xd04a291b` (edge) onto the framebuffer.
- Human screenshot independently corroborates: stage spans desktop x≈172..1747, y≈29..1079; a ~743×79 screen-pixel slab appears at approximately desktop **x≈590..1332, y≈155..233**. The stage conversion predicts exactly those bounds (within screenshot sampling/rounding). **HIGH-confidence link: this rectangle is the user's unwanted black slab.**
- The original sprite 716 *already* includes its own translucent rounded panel (legacy HUD historical artifact and in-CI partial-alpha check >10,000 pixels; original shell alpha multiplier 82/256). The added opaque-ish black strip doubles and misaligns the backing instead of serving as the intended source decoration. An extracted `hud-shell.png` A/B artifact visibly contains a distinct rounded grey source shell. Hence in the next change remove **only** this HQ-only band before considering any replacement; leave original shell, source icons and 1× renderer untouched.
- Source sprite/counter/labels draw order: extra band → HQ shell with themed labels/icons/rounded source backing → Rust dynamic atlas numbers. Moving shell/number positions without identifying baked labels and panel ownership risks recreating overlap.

## 5. Observed icon/label separation and uncertainties
- Visual samples from user screenshot and independent true-3× HUD comparison show Batarang and Health art are **source sprites**, not Rust-generated icons. Gold labels touch/approach the silhouettes too closely on the top edge. Raw source field boxes tell where text *could* draw, not exact raster ink collision.
- On the 1920×1080 Windows screenshot the blue fill of the Batarang symbol is confined approximately to stage x179..246, y19..39; orange/yellow Health icon fill approximately x266..335, y19..52. These are measured **color-core masks only**, **not full black icon outline bounds**; outline/mask isolation from source HUD baker is still required before claiming exact silhouette separation. Gold caption ink in small text also needs mask-based measurement.
- The user explicitly reports overlapping texts/icons, insufficient contrast and disfiguring bar. Treat this as the current defect even though an older 930×660 CI snapshot previously looked acceptable. Different scales and the human's visual requirement matter.
- No direct original Flash runtime HUD screenshot was established by this batch; `hud-shell.png` legacy side is **Rust-baked source-style reconstruction**, not a frame directly captured from Adobe Flash. Source metadata is authentic recovered DefineEditText/placement, original Flash exact text raster may differ.

## 6. Design contract for subsequent bounded batches (requirements, not applied code)
1. **Single geometry owner.** Define ONE documented logical coordinate map keyed by field ID and icon placement, shared/generated into build-time baking and Rust number placement rather than two independent manual offsets. Preserve native 600×400 source stage, 1× original oracle, and 3× HQ copy.
2. **No double backing.** Ban the arbitrary `draw_hud_counter_band_hq` 283×30 strip. Use original sprite-716 rounded translucent shell as the starting layer. Only adopt a deliberately bounded and visually compared per-counter shape if contrast objectively requires it; no unqualified second full-width panel.
3. **Real collision metrics.** Test **opaque text ink + stroke + shadow** vs **icon visible alpha mask**, not just SWF text-field rectangles. Require at least **3 logical pixels** (9 HQ pixels) visual spacing where separate elements should not touch. Treat this as a proposed HQ design clearance, **not original Flash geometry**. Compare score/title and lives number/caption similarly with a documented gap target.
4. **Counter column fit.** Center the Batarang count below its icon without touching the silhouette/panel edge; treat lives as one group `LIVES X <count>` under Health with numeric width-aware horizontal placement; anchor Score in its own field. Calculate digit width from generated atlas metadata; support at least test cases `batarangs=[0,9,10,99,999]`, `lives=[0,5,9,10]`, `score=[0,9999,123456]`. Define overflow handling explicitly; don't silently clip.
5. **Color/contrast.** Keep the user's improved condensed HQ font candidate as baseline, adjust ink/outline/background together across moon, red sky and dark roof. Do not call stylistic HQ gold original-source accurate. Avoid guess-driven contrast values without rendered evidence and visual review.
6. **Size-independent projection.** Layout is always in 600×400 logical coordinates and scaled with viewport mapping at 1×/3× and real Windows letterboxing; confirm no cropping at 930×660 and user 1920×1080 modes. No hardcoded screenshot pixel offsets in game code.
7. **Evidence gate.** Each change: Windows source rebuild, all Rust tests, 15/15 preserved legacy hashes, existing 40ms p95 gates, native F7 and separate normal mouse-SKIP screenshot, A/B compared HUD crop and multi-value contact sheet. Never relax golden hashes to force a pass.
8. **Acceptance remains separate.** Source text, full original live-Flash HUD parity and user subjective approval are OPEN. Do not merge PR #18 or develop Level1B.

## 7. Ordered next development tasks
- **Next: HUD-REAL-03 / batch 2:** remove ONLY extra dark counter band from HQ Rust renderer, compare OS screenshot and 3× shell layout, assert rectangle no longer present, preserve the original translucent panel and existing number positions for an isolated causal A/B. If removing band hurts legibility, record it without reintroducing a large slab.
- **Batch 3:** centralize logical layout and correct caption/digit positioning with mask-based collision and width-aware values.
- **Batch 4:** tune font/outline/contrast, guided by actual OS and source-style comparisons.
- **Batch 5:** add QA matrix screenshots / automated collision & overflow checks.
- **Batch 6:** full Windows multi-state GUI visual review and user acceptance checkpoint (additional bounded corrections only if proven necessary).

## 8. Explicit verification classification
**EXECUTED THIS BATCH:** inspected repository Rust/Python/workflow/documentation at source head, inspected original extracted SWF DefineEditText source report and 1×/HQ HUD CI metadata, downloaded and opened native Windows and HUD A/B PNGs, inspected user screenshot and calculated stage-space coordinates. **NOT EXECUTED:** a fresh Rust build, new SWF extraction, a native Flash playback comparison, glyph-mask isolation for exact text/icon alpha collision, any changed executable. Previous green [#42681546189](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42681546189) is evidence for **unchanged** old game code only. **Engineering audit:** see AUD-20261110-005; **research:** RE-20261110-005.

**Confidence:** HIGH in source-field boxes, original root placement, independent HUD ribbon cause, current hardcoded counter offsets; MEDIUM in outline/icon exact overlap boundaries without isolated masks; UNVERIFIED original-Flash runtime glyph raster and final player-approved aesthetics.

## 9. HUD-REAL-03 implementation verification addendum — 2026-11-10

The numbered plan above is the baseline research task chronology; **batch 2 (remove extra black bar) is now complete and verified** at game commit `53180670de2868138ba5bcb0cd2fe42d617c2c0f` and [Windows CI #42689334769](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42689334769), 60/60 steps, 12/12 tests, 15/15 legacy screenshots, render/scroll p95 21.8781/26.9261ms. Downloaded [GUI pilot artifact #13707206313](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42689334769/artifacts/13707206313) confirms no full-width black rectangle in actual F7 or mouse-SKIP gameplay while source HUD shell/icons remain. New screenshots show small Batarang counter below original shell against red sky and need planned layout/contrast fixes. Next **HUD-REAL-04 / batch 3** (one logical layout authority and icon clearance); batches 4–6 remain pending. This verification did not change any Rust game code or original SWF art.

## 10. HUD-REAL-04 shared HQ layout design — 2026-11-10 (new-code CI pending)

The prior source-field table remains the 1× SWF-derived comparison oracle, not the authored new HQ layout. **New single authority:** `reverse-engineering/manifests/hud-hq-layout.json`, stage 600×400, schema 1. Offline baker applies static label offsets at 6× then emits `assets/hud_hq_layout.rs`; native renderer `include!`s these constants, never parses JSON at runtime. Values below describe the *themed HQ proposal*, not the original Flash:
 
| Element | HQ target, stage logical | Intent |
|---|---|---|
| BATARANG, HEALTH, SCORE title ink | Original label center + (0,-2) | Add top gap from bat/health art |
| LIVES X caption | Original #715 center + (+8,+5) ≈(298.25,61.675) | Center textual group below Health rather than left-shift |
| Batarangs numeric | centered (211,60), width max 60 | Float 0 no longer below original rounded source panel |
| Score numeric | centered (387,35), width max 90 | Large or small existing native 3× digits to fit 1..6+ digits |
| Lives numeric | fixed left (313,63), max width 54 | Numerals widen rightward so 10 does not push back into caption |

The one logical-layout ownership code is the scope of **batch 3**; giant black bar must remain absent, source alpha panel/art unmodified. Rust width helpers fit cell span but **do not prove alpha/glyph collision clearance**. Actual Win32 screenshots, 15 legacy golden hashes, MSVC build and 40ms performance are mandatory before status VERIFIED. The image-based multi-value matrix and user typography approval remain batches 4–6.

## 11. HUD-REAL-04 real Windows validation — 2026-11-10

**Code SHA:** `f369765f06dd5329924f3d8c9fd63dfb7f0cb2c9`. [Successful Windows #42698257055](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42698257055) passed 60/60 steps, 14 Rust tests, 15 historical screenshot hashes and unmodified render/scroll <40ms p95 (25.2695/27.6003 ms). The prior failed #42696631241 was only a stale Round-6 Rust-comment assertion; resolved in the current workflow without removing functional image/layout gates. The shared JSON manifest generated native Rust constants successfully at build time.

**Native screenshot A/B:** [real GUI artifact #13711945004](https://github.com/sinethnimsara26/Batman-CobbleBot-Native-Rust/actions/runs/42698257055/artifacts/13711945004) compared with pre-batch actual Win32 GUI #13707206313. New F7 and separate mouse SKIP gameplay show BATARANG number `0` inside rounded source shell rather than below it, moved `LIVES X 5` group, slightly lifted top labels, preserved icons and shell, and absent unwanted full-width band. Visual contrast, complete ink alpha masks, overflow multi-digit cases, screen sizes beyond 930×660 and user aesthetics remain OPEN. **Batch 3 is verified for its bounded layout target, not full HUD polish**; next HUD-REAL-05 targeted typography/clearance; HUD-REAL-06 multi-value matrix; final user acceptance separate. RE-20261110-010 / AUD-20261110-010.

## 13. HUD-REAL-05A — compact digits and taller small ink (mini-batch1/3, Windows CI pending)

User's new real 1920×1080 screenshot `image(20261110-030259).png` exposes the Batarang count `2 0` with excessive gap and a detached look; main headings/Score already improved and user wants contrast postponed to next batch. The new themed HQ **single source-stage manifest** keeps all source sprite716 artwork, moves Batarang count center y60→**59** logical, uses preferred per-digit Batarang advance **9** logical vs old full small atlas cell ~14 (27 vs42 HQ pixels) and lives advance **10** logical; Score remains untouched. Full `20` cell span reduces 84→**69** HQ physical pixels; this is a *cell-level* measurement, not a proven alpha-mask separation. At bake time only, the rendered small Impact glyph alpha ink incl existing stroke/shadow is resized to **115% vertical** in premultiplied-alpha temporary6×, recentered in the original cell and downsampled to3×. The native Rust renderer still blits exact 3× pixels at 1:1. No change to legacy 1× source renderer, original font color/outline, or source bat icon silhouette. No test success is asserted here; real OS GUI, 15 source hashes and 40ms p95 CI remain mandatory. See RE-20261110-011 / AUD-20261110-011 and top AI_HANDOFF for exact next verification.
