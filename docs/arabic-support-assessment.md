# Arabic PDF support assessment

Status: copy/extraction, source editing and real-font cross-reader validation, 2026-10-31; broader Arabic fidelity remains partial.
Baseline: `f137a289baadefbd29e0e46930899896068cc41d` (fork and upstream HEAD when inspected).

## Scope and evidence

The objective is logical Unicode fidelity through copy, extraction, search, editing,
save/reopen and export. Correct pixels alone do not establish correct text. The implementation fixes independently mapped marks, export-reader order, numeric separators,
empty/auxiliary mappings, shaped-cluster ownership and complete-source editing. It does not
claim general Arabic support or completion of every corpus class below.

Sources inspected: `AGENTS.md`, `CLAUDE.md`, `docs/architecture.md`, `docs/gaps.md`,
`docs/target-app-parity.md`, manifests, CI, renderer, fonts, editing, OCR, automation,
clipboard and exporters. The local-only `plan/` instructions are absent in this checkout.
No proprietary application resources or third-party PDF contents were inspected.

## Architecture

| Operation | Implementation | Important boundary |
| --- | --- | --- |
| Decode and paint PDF text | `vendor/hayro-interpret/src/font/`, `vendor/hayro-interpret/src/interpret/`, `crates/render/src/raster.rs` | Font encodings select glyphs; `/ToUnicode` describes characters. Existing positioned glyphs must not be shaped again indiscriminately. |
| Viewer text extraction | `crates/render/src/text/device.rs`, `layout.rs`, `order.rs` | The device obtains Unicode and geometry; layout reconstructs lines/gaps and RTL order. Multi-character mappings remain atomic glyphs. |
| Copy and selection | `crates/ui-egui/src/canvas/selection.rs`, `app/clipboard.rs`, `state.rs`; `crates/render/src/text.rs` | Run selection calls `PageText::text_of`; rectangle selection calls `column_text`; both consume the same reordered glyphs. |
| Search | `crates/render/src/text/search.rs` | Matches Unicode and returns original glyph ranges. It does not repair extraction. |
| Arabic shaping | `crates/fonts/src/shaping.rs`, `source_shaping.rs`, `craft.rs` | `harfrust` shapes joining forms, ligatures and marks. `unicode-bidi` resolves logical input. Optional craft-fonts supplies Arabic document faces. |
| Existing-text reading/editing | `crates/edit/src/bidi.rs`, `text/{interp,lines,blocks,rewrite}.rs`, `text_regions/styled.rs` | A second extraction path uses bounded visual-to-logical recovery; contextual mixed-font/style rows have explicit restrictions. |
| Inserted text and serialization | `crates/edit/src/added.rs`, `added/`, `crates/fonts/src/{embedding,pdf}.rs` | Shaped glyph outlines, `/ToUnicode` and logical `/ActualText` must survive save/reopen. |
| Office export | `crates/engine/src/compare.rs`, `crates/export/src/` | Office blocks come from `pdfcraft_edit::reading_blocks_checked`, not the viewer's `PageText`. |
| OCR | `crates/ocr/src/lib.rs`, `crates/engine/src/{ocr,edit/run}.rs`, `crates/edit/src/place.rs` | English recognition alphabet; Helvetica/WinAnsi invisible layer. Arabic recognition and Unicode-safe output are separate prerequisites. |

The pinned shared `craft-libs` revision is `c5a1cbfb5f9049f9270955caffb5bee5997aebf7`.
Its `crates/text/src/bidi/` wraps UAX #9 for logical text layout, while `font-catalog`
discovers installed faces. Neither reconstructs arbitrary PDF logical text from painted
glyphs. Replacing PDF extraction with the logical-layout API would conflate different jobs.
`craft-fonts` remains an optional external build input; no font files belong in this repo.

## Existing functionality and open work

- Shaping and bidi already exist. Adding another general shaping implementation is unnecessary.
- Viewer tests already cover Arabic lam-alef mappings, mixed RTL/numbers, combining marks,
  rotation, marked-content `/ActualText`, cropping and bounded hostile input.
- Editor and automation tests cover Arabic save/reopen and logical text, but several return
  early without an Arabic craft-fonts face. A green count is not evidence those branches ran.
- Marked-content `/ActualText` is authoritative replacement text. The existing implementation
  retains geometry and applies limits; structure-element replacements are separate work.
- Missing or defective mappings cannot always be recovered uniquely. Preserve known Unicode;
  document remaining ambiguity instead of reversing all strings or silently normalizing them.
- Direct export probing with `مرحبا بالعالم ١٢٣ ABC` preserved the input Unicode in HTML/PPTX
  but found no paragraph RTL metadata. PowerPoint visual correctness was not measured.
- `pdfcraft_ocr::ALPHABET` excludes Arabic and `LANGUAGES` contains English only.
  `text_layer()` uses `win_ansi()`, whose unmappable characters become `?`. Appending Arabic
  to the recognizer alphabet without a compatible trained model and output layer is invalid.

## Confirmed defect and first fix

Two regressions failed on the baseline: separately positioned fatha was emitted before beh.
The layout case returned `\u{064E}بب` instead of `ب\u{064E}ب`; the synthetic mapped PDF
returned `\u{064E}باب` instead of `ب\u{064E}اب`. This affects clipboard-ready text and search
even though every source glyph already has explicit Unicode.

`reverse_rtl` now groups an all-NSM glyph with the preceding visual unit, provided no inferred
word gap separates them. It reorders those units and then emits each unit's original glyphs
in source order. Unicode NSM classification comes from the already-used `unicode-bidi` 0.3
crate; the lockfile adds only the renderer dependency edge. The principle is the combining-mark
issue described by [UAX #9 L3](https://www.unicode.org/reports/tr9/#L3), not a new implementation
of all UAX #9 or a universal inverse-bidi algorithm.

No glyph text, rectangle, quad or direction is rewritten. Multi-character mappings remain
atomic. There is no Unicode normalization, string reversal, font/raster change or PDF edit.
Grouping and emission are linear in glyph count, with an additional index vector and cluster
ranges; no dedicated throughput benchmark was run. Existing long-input/resource-limit tests
pass. The rule does not infer a base across a word gap or solve arbitrarily shifted marks.

The regression includes run/word selection, rectangular copy, search and equality after
`pdfcraft_cos::write_full` plus reopen. That is a semantic serializer round trip, not an Arabic
editing workflow or real-font visual qualification.

An independent temporary semantic fixture was also passed to installed Poppler `pdftotext
-raw -enc UTF-8`. Poppler returned the three letters with bidi controls and fatha on a separate
line. This disagrees with the known fixture intent; it is recorded as an oracle limitation,
not external confirmation. No Acrobat/Word/scanner producer-specific defect is claimed fixed.

## Upstream coordination

Public GitHub pages and read-only Git were used; CLI GraphQL requests were forbidden.
These are observations on 2026-10-31, not permanent ownership/status claims.

| Item | Finding / action |
| --- | --- |
| [#2667](https://github.com/storytold/pdfcraft/issues/2667), [#2736](https://github.com/storytold/pdfcraft/pull/2736) | Mixed RTL numbers/punctuation extraction fix is merged and its regression passes. |
| [#3465](https://github.com/storytold/pdfcraft/pull/3465) | Open Arabic editing, fallback and `/ActualText` selection work. Its fetched head leaves `text/order.rs` unchanged; coordinate the separate mapped-mark regression rather than duplicate the broad PR. |
| [#4920](https://github.com/storytold/pdfcraft/pull/4920) | Open per-cluster replacement and glyph mapping work, mainly LTR shaped rows; avoid overlapping rewrites. |
| [#3689](https://github.com/storytold/pdfcraft/pull/3689) | Open structure-element `/ActualText` extraction; Office export remains separate. |
| [#4653](https://github.com/storytold/pdfcraft/issues/4653), [#5042](https://github.com/storytold/pdfcraft/pull/5042) | Arabic export order/direction; open direction PR covers DOCX, HTML, RTF, explicitly excludes PPTX and extraction fixes. |
| [#5424](https://github.com/storytold/pdfcraft/issues/5424) | Disconnected Persian/Arabic reported on 0.14.0; current code already shapes. Obtain current-version steps before assigning a renderer root cause. |
| [#146](https://github.com/storytold/pdfcraft/issues/146) | Requests Arabic searchable OCR because current recognition is Latin-only. |
| [#3335](https://github.com/storytold/pdfcraft/pull/3335), [#3743](https://github.com/storytold/pdfcraft/pull/3743) | Open optional Tesseract / Unicode OCR approaches. Converge on shared infrastructure before Arabic backend work. |

## Real-font investigation and follow-on fixes

The next phase uses the exact CI-pinned craft-fonts revision, with its source SHA-256 and OFL
license checked. Noto Sans Arabic's variable face supplies real Type 3 outlines. A static
instance and a 148-glyph subset supply embedded TrueType source PDFs without `/ActualText`.
The setup, hashes and repeatable commands are in [arabic-corpus.md](arabic-corpus.md).

The corpus found a second pipeline defect: the viewer read
`باسم الآب والابن والروح القدس` correctly, but `reading_blocks_checked()` returned
`سدقلا حورلاو نبالاو بآلا مساب`. The export reader omitted bidi recovery already used by
editable lines. `crates/edit/src/text/lines.rs` now shares that recovery, while leaving explicit
logical `/ActualText` untouched. Its font-independent Arabic/Hebrew regression failed before
the change. This is the input-order side of #4653, separate from #5042's direction metadata.

Other independently reproduced problems:

| Defect | Evidence and correction |
| --- | --- |
| Arabic decimal separators split numbers | Recovery changed `١٢٣٫٤٥` into `٤٥٫١٢٣`. `edit/src/bidi.rs` now recognizes numeric bidi classes EN/AN, including U+066B/U+066C, using existing Unicode data. Western and Persian-digit cases remain tested. |
| Cluster text belongs to a zero-advance dot | Noto shapes some letters into a dot plus body. Putting `/ToUnicode` on the dot caused false spaces such as `ج ميلة`. Generate PDF mappings on the first advancing body, while preserving the shaping representation and document spacing. This is a metadata correction, not a new shaper or looser spacing heuristic. |
| Empty CMap destinations disappear | The producer writes empty Unicode for auxiliary glyphs, but the editor's CMap lexer discarded `<>`, permitting encoding fallback or undecodable auxiliaries. `fonts/src/pdf.rs` now distinguishes a valid empty destination from an empty source code. |
| Source edits choose an unusable fallback or mix encodings | Adding harakat to a complete embedded static face first selected its optional variable counterpart and failed validation. After selecting the valid source, old partial and new full encodings split the contextual row. Prefer the validated complete source, then consistently reuse its full resource within that source span. Keep explicit resource/style choices and mixed-contextual-row guards. |
| Empty auxiliaries invent spaces | `edit/src/text/lines.rs` could insert a space before empty text or move the semantic pen to a non-text auxiliary. Keep its ink/operators for edits while excluding it from semantic word-gap movement. |

The glyph-ownership change is related to the open #4920/#3465 shaping/selection work and
should be offered as a focused supplement, not a competing broad Arabic implementation.
Public searches for `reading_lines`/Arabic and logical-order export found #5042 and related
work, but no focused fix for the missing recovery stage. During initial inspection, no issues or PRs were created or commented on.

The real-font matrix now distinguishes ten authoritative added-text cases, eight recoverable
full-font legacy cases and eight subset cases. Two legacy mixed/URL cases are retained as
explicit ignored failures. A passing counterexample proves that different logical mixed
paragraphs can paint identical glyph IDs and positions; no universal visual-order inverse can
recover that missing information. Runtime confidence/warning APIs remain future work.

## Regression corpus policy and coverage

Fixtures are contributor-original Rust-generated documents under `MIT OR Apache-2.0`.
Synthetic Helvetica `/ToUnicode` fixtures test Unicode/position semantics, **not Arabic
outline rendering**. No committed PDF, proprietary sample, font or large model is added.
The reusable `unicode_mapped_pages` helper lives in `crates/render/src/text/tests.rs`.
Expected strings are literal logical Unicode in tests; normalization is not applied.

| Required corpus class | Present coverage / next addition |
| --- | --- |
| Modern Standard Arabic | All six mission phrases have real-font added-text fixtures; recoverable legacy variants are checked through both readers. |
| Diacritics and combining marks | Mapped-glyph tests plus actual Noto outlines, body ownership and empty-auxiliary regressions. |
| Lam-alef/contextual shaping | Existing ligature tests and the real-font corpus. |
| Arabic/French/English | Real-font authoritative case passes; untagged mixed base direction is an explicit limitation. |
| Western/Arabic-Indic digits | Both digit sets, Arabic decimal separators, Persian-digit unit regression. |
| Brackets, URLs, dates, punctuation | Added-text cases pass; legacy URL/bracket case remains explicitly unqualified. |
| Multiline, justified/right-aligned | Existing editor coverage; qualify with actual Arabic fonts. |
| Tables, columns, headers | Arabic two-column/two-row clipboard fixture verifies cells, Arabic-Indic digits and tabs/newlines; headers/complex layouts remain. |
| Embedded/subset fonts | Static full-font and actual 148-glyph subset programs, without ActualText, checked through both readers. |
| Missing/defective Unicode mappings | Existing fallbacks; add explicit uncertainty expectations, not invented text. |
| Annotations, fields, text boxes | Existing/open upstream work; independently qualify save/reopen. |
| Scanned Arabic | Not recognized by the current default model; deferred OCR work. |

Real-font corpus ground truth includes, without changing code points:

```text
باسم الآب والابن والروح القدس
اللغة العربية جميلة
مرحباً بكم في جنيف
الاجتماع يوم 15 أكتوبر 2026 الساعة 14:30
رقم الفاتورة INV-2026-001
السَّلَامُ عَلَيْكُمْ
```

Acrobat-, Word- and scanner-produced source PDFs were not supplied. Their producer-specific
failures are not reproduced by these synthetic cases. Future contributions must use synthetic
documents and preserve provenance; proprietary application outputs are not redistributable
fixtures under the repository policy.

## Validation

Rust/Cargo 1.95.0 are installed. In this machine:

```sh
export RUSTUP_HOME=/workspace/.pdfcraft-rustup
export CARGO_HOME=/workspace/.pdfcraft-cargo
export PATH="$CARGO_HOME/bin:$PATH"
cd /workspace/pdfcraft
```

The scoped real-font command is:

```sh
# Also set the pinned font paths and required-input gates from arabic-corpus.md.
timeout 900 cargo test --offline --locked -p pdfcraft-fonts -p pdfcraft-edit -p pdfcraft-render -j 2
```

Final implementation run: **949 passed, 3 ignored, zero failures**:

| Target | Passed | Ignored |
| --- | ---: | ---: |
| edit unit | 362 | 0 |
| edit real-font corpus integration | 6 | 2 known legacy mixed/URL cases |
| fonts unit | 92 | 0 |
| fonts integration | 3 | 0 |
| renderer unit | 423 | 0 |
| renderer integration | 63 | 1 existing ignore |

The six active corpus tests cover 28 generated PDF cases (10 added, 8 full source, 8 actual
subset, 1 table, 1 existing-source edit) plus the mixed-direction ambiguity counterexample.
All font-input gates were mandatory. No optional Arabic early return counts as qualification.
Exact text, word copy/search, table copy, source/added pixel stability and incremental edited
source preservation are tested. This is bounded synthetic-source evidence, not arbitrary PDF
or full desktop/OS clipboard readiness. No dedicated performance benchmark was run.

Regression sensitivity was checked by temporarily restoring baseline production modules,
retaining final tests, and returning the new mapping method to original glyph ownership.
The command used `--no-fail-fast` so all relevant targets ran: 11 tests failed as expected
(3 editor unit, 3 corpus, 2 font unit, 3 renderer unit). These were assertion/runtime failures,
not compile errors. Every final production file was restored byte-for-byte in a `finally`
block. The restored full scoped suite passes with the totals above. A separate final boundary regression removes only the new-line auxiliary guard: it fails because the auxiliary drawing survives clearing the paragraph. Restoring the guard byte-for-byte makes the same test pass (1 passed).

`cargo fmt --all -- --check`, dependency layering (37 crates) and assets (289 repository,
6 dependency and 22 build-time assets) pass. Scoped all-target Clippy passes with warnings
as errors, and the final native CLI build passes (warm rebuild 1m 44s):

```sh
timeout 900 cargo clippy --offline --locked -p pdfcraft-fonts -p pdfcraft-edit -p pdfcraft-render --all-targets -j 2 -- -D warnings
timeout 900 cargo build --offline --locked -p pdfcraft-cli -j 2
```

Existing unused optional egui patch warnings remain; lockfile changes add two dependency
edges and update no versions. No CI is changed.

The native CLI's **13-step automation script passes**: open a full-source legacy PDF, extract
exact logical text, find `الروح` (one match, zero warnings), export DOCX, render, inspect the
paragraph, edit to `السَّلَامُ عَلَيْكُمْ`, extract, save to a new PDF, reopen, extract, export a
second DOCX and render. Explicit assertions verify all three extraction strings. Both actual
DOCX files' `word/document.xml` concatenated `w:t` strings equal exact ground truth, with no
normalization/reversal. Both have zero `w:bidi` elements: logical input is repaired, but Word
direction/layout remains independent #5042 work. Both CLI previews were visually inspected.
The script and report are local artifacts in `/tmp/pdfcraft-arabic-corpus-final`; repeatable
commands are in [arabic-corpus.md](arabic-corpus.md). The source edit integration separately
verifies incremental saving preserves the original PDF byte prefix.

The two ignored legacy cases were also run explicitly against the final test binary: both
fail as documented (exit 101), rather than being counted as passes. The output distinguishes
mixed base-direction ambiguity from URL/bracket recovery limits.

Cloud onboarding: Rust activation, pinned font paths, required-input gates and scoped checks
were saved in the environment configuration draft's `start_skill`. Existing dependencies and
caches remain in this instance; no service starts at session entry. Repository membership,
networking, secrets and install_script were left unchanged. Saving the draft does not publish
an environment or verify restoration in a fresh task; review/save and publish in environment
settings are user actions.

Independent Poppler rendering of the edited harakat source was visually inspected alongside
PdfCraft's raster: joined letters and marks are present in both. This is a limited human
visual check, not pixel-oracle identity or typography certification. Raw Poppler extraction
contains bidi controls and its own visual-order convention, for example
`'\u202bْمُكْيَلَع ُماَلَّسلا\u202c\n\x0c'` for the edited greeting. Preserve it as an
independent result; it is **not** exact logical-Unicode agreement with the corpus ground truth.

Not run: full workspace, desktop/native clipboard/IME, full application WASM build, broad producer-specific PDFs,
Word application visual rendering, Arabic OCR accuracy or full multilingual corpus benchmarks.

Tesseract 5.5.0 is installed with `eng` and `osd` only; no Arabic trained data is installed.
No OCR model was downloaded or added, and no recognition-quality result is claimed.

## Upstream publication preparation

On 2026-10-31, upstream main still matches the inspected base. The scoped compatibility check
`cargo check --locked -p pdfcraft-fonts -p pdfcraft-edit -p pdfcraft-render --target wasm32-unknown-unknown -j 2`
passes with Rust 1.95.0 (1m 22s). It reports five renderer target-specific warnings; native
all-target Clippy remains clean with warnings denied. This checks the changed libraries, not
the complete web application. Formatting, layering and asset checks pass again before commit.
The proposed upstream draft PR body is [arabic-upstream-pr.md](arabic-upstream-pr.md).

## Revision history

- 2026-10-31: trace both extraction paths, clipboard consumers, shared bidi libraries and
  upstream overlap; reproduce/fix separately mapped marks, add source-generated semantic
  fixtures and record scoped validation plus remaining corpus/real-font/oracle limitations.
- 2026-10-31: qualify pinned real fonts and a subset; reproduce export-order, numeric separator,
  Unicode ownership and empty-CMap/auxiliary defects; preserve explicit legacy ambiguity tests.
