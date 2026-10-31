# Arabic fidelity contribution roadmap

Status: implementation and scoped validation prepared for upstream review, 2026-10-31. Base:
`f137a289baadefbd29e0e46930899896068cc41d`.

## Recommended first contribution

**Keep separately mapped combining marks with their base in RTL copy and extraction.**
This is the smallest independent defect, has font-independent red/green regressions, and
leaves glyph geometry, rasterization and PDF bytes unchanged. Offer it first, coordinating
with open #3465. The full working branch is an investigation series, not a proposed single PR.

## Split the implementation into reviewable PRs

| PR | Reproduced behavior and exact modules | Dependencies / review boundary |
| --- | --- | --- |
| 1. Mapped-mark copy fidelity | `crates/render/src/text/order.rs`, `text/tests.rs`, `crates/render/Cargo.toml`, `Cargo.lock`. Group no-gap NSM glyphs with their visual base before RTL reordering. Copy/search previously returned `َباب` instead of `بَاب`. | Independent. Reuse existing unicode-bidi data. No normalization or geometry change. |
| 2. Export-reader logical order | `crates/edit/src/text/lines.rs`, `crates/edit/src/tests.rs`. Share the existing editor recovery with `reading_lines`; preserve authoritative ActualText and clear stale drawing-order runs. | Independent of 1. Input-order side of #4653; #5042 covers separate DOCX/HTML/RTF direction metadata. |
| 3. Arabic numeric separators | `crates/edit/src/bidi.rs`. Keep Unicode EN/AN numeric sequences intact: `١٢٣٫٤٥` must not become `٤٥٫١٢٣`. | Independent. Test Arabic/Persian/Western digits and grouping. Do not propose a general inverse-bidi replacement. |
| 4. Explicit empty Unicode and auxiliary drawings | `crates/fonts/src/pdf.rs`, `pdf/tests.rs`, `crates/edit/src/text/lines.rs`, `crates/edit/src/tests.rs`. Preserve `<>` destinations; prevent empty auxiliaries inserting gaps or moving the semantic pen, while retaining their operators for edits. | Parser part is independent; auxiliary test depends on it. Keep source-code validity and CMap entry budgets. |
| 5. Shaped-cluster PDF mappings | `crates/fonts/src/source_shaping.rs`, `crates/fonts/README.md`, `crates/edit/src/text_regions/styled.rs`. Add immutable `ShapedRun::pdf_mappings` and put Unicode on an advancing body instead of a zero-advance dot. | Coordinate #4920/#3465. Preserve original shaping text/positions and character/word spacing; consume empty auxiliaries correctly with PR 4. |
| 6. Complete embedded source face during editing | `crates/edit/src/text_regions/styled.rs`. Prefer validated complete embedded source over optional matching faces; keep its full encoding consistent within the source span when missing characters require it. | Coordinate #3465. Real source edit with added harakat failed first on variable fallback, then inconsistent encoding. Keep subset/embedding checks and mixed-contextual-style refusal. |
| 7. Real-font cross-reader corpus | `crates/edit/tests/arabic_fidelity.rs`, `crates/edit/Cargo.toml`, `Cargo.lock`, `docs/arabic-corpus.md`. Ten authoritative added cases, eight full-source and eight actual-subset cases, Arabic table and incremental source edit; optional artifacts and required-input gates. | Accompany the relevant fixes with minimized regressions; keep broader matrix as a follow-on. External OFL fonts stay in craft-fonts. Two known legacy ambiguity tests remain ignored explicitly. |

The documentation and roadmap notes can accompany their relevant PRs. Keep each commit's
regression meaningful on its own base. The table is a proposed split; the current branch
intentionally holds the combined implementation for review. No CI or readiness percentage
is changed and no generated PDF/font/model is committed.

## Next phase: remaining editing and export fidelity

Review #3465/#4920 before precise per-cluster ActualText selection or contextual style/fallback
work. Current added-text full-line selection is atomic; passing full-line copy does not prove
word-level selection for that path. Qualify keyboard navigation, IME, undo/redo, right anchoring,
multiline/justified layouts, annotations and form fields independently. Relevant modules are
`crates/edit/src/added/`, `text/{interp,lines,rewrite}.rs`, `text_regions/styled.rs`,
`crates/fonts/src/{shaping,source_shaping,embedding,pdf}.rs`, and `crates/ui-egui` input/selection.

Build on #5042 for DOCX/HTML/RTF direction metadata. Treat PPTX as a separate contribution in
`crates/export/src`; check logical Unicode and visual layout separately. Export input comes
through `crates/engine/src/compare.rs` and `reading_blocks_checked`, not the viewer extractor.
Test actual format files, source save/reopen and independent readers. No image-only substitute.

Legacy mixed paragraphs without direction metadata can be information-theoretically ambiguous:
the corpus demonstrates different logical strings yielding identical glyph IDs/positions.
Do not promise universal recovery. A future bounded confidence/warning contract and use of
reliable structure/direction metadata are preferable to a new unconditional reversal heuristic.

## Later phase: optional Arabic OCR

Coordinate #146, #3335 and #3743. Split Unicode-safe invisible text-layer writing from Arabic
recognition. Exact modules: `crates/ocr/src/lib.rs`, `crates/engine/src/{ocr,edit/run}.rs`,
`crates/edit/src/place.rs`, automation OCR dispatch and language/readiness presentation.

Evaluate optional local Tesseract Arabic/English first, with pinned/checksummed compatible
trained data and verified model licenses. Keep English's default path and avoid a large default
runtime dependency. PaddleOCR needs its own model-coverage, licensing and deployment assessment.
Appending Arabic to an incompatible recognizer alphabet cannot add recognition.

Measure character/word error and reading-order scores against scan ground truth; exercise
boxes, confidence, cancellation, native process failures, searchable Unicode, unchanged page
pixels, undo and save/reopen. Tesseract 5.5.0 here has eng/osd only; no Arabic model was fetched
and no recognition accuracy is claimed.

## Validation and review gates

See [assessment](arabic-support-assessment.md) for final commands/results and
[corpus guide](arabic-corpus.md) for font provenance and reproducible setup. Use scoped tests
and Clippy with `-j 2` and timeouts, formatting, layers and assets. No full workspace build.

Each PR must show the original failing output, the passing exact Unicode assertion, a temporary
production revert proving the regression still detects the defect, and relevant save/render
checks. Do not count optional early-return tests as real-font coverage; set the required-input
gates. Do not count the two ignored legacy cases as passes. Independent visual checks and
Poppler extraction conventions are reported separately from PdfCraft's exact Unicode tests.

## Draft upstream PR description

Title: **Keep separately mapped combining marks with their base in RTL copy and extraction**

### What this changes

A PDF can paint beh and fatha separately, with correct `/ToUnicode` for both. Viewer extraction
then reverses the mark independently: the mapped fixture copies `َباب` rather than `بَاب`,
and searching for `بَاب` fails. Group no-gap nonspacing-mark glyphs with their preceding visual
unit before the existing RTL pass; expand units without changing glyph text or geometry.
Reuse `unicode-bidi` classification. No normalization, raster/font rewrite or PDF mutation.

Related: #2667 and #3465. This is a separate mapped-glyph mark regression; it does not close
those broader issues or replace the open Arabic editing/ActualText work.

### How you verified it

New layout and real-PDF-parser semantic regressions failed before the fix. They cover exact
Unicode, base-plus-mark selection, word/rectangular copy, search, Arabic/Hebrew/mixed Latin,
multiple marks, orphan preservation and full-save/reopen equality. See execution results above
for tests, formatting, Clippy, layers and assets. Fonts/models are not prerequisites for these
tests. No claim of real-font Arabic appearance or OS clipboard integration is made.

### Safety, provenance and review focus

No unsafe code, network access, changed paths, new listener, proprietary fixture/font or large
binary. Source-generated semantic fixtures use synthetic mappings on standard Helvetica;
they do not depict shaped Arabic. Preserve one output glyph per input glyph and source mark
order; grouping/emission is linear with additional linear index/range storage. No benchmark
claim. Shifted marks, unreliable Unicode maps, full paragraph direction and Office export's
separate reader remain outside this fix. Parity/readiness percentages are unchanged.

This draft describes the proposed first independent PR. The complete review branch also
contains the follow-on fixes and corpus described above.

## Revision history

- 2026-10-31: inspect architecture/upstream; implement initial mapped-mark regression.
- 2026-10-31: extend to pinned real fonts/subsets, shared export reading, numeric and Unicode
  mapping/auxiliary fixes, complete-source editing and a reproducible corpus; propose independent
  upstream review boundaries and retain explicit mixed-direction/OCR limitations.
