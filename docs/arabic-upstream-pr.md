# Preserve Arabic Unicode through PDF extraction, source editing and export input

## What this changes

Correctly painted Arabic text can still copy in the wrong Unicode order, acquire spaces
between parts of a letter, or reach Office export backwards. Editing an embedded complete
Arabic face can also choose an unusable variable fallback or split one contextual row across
its old partial and new full encodings.

This review branch fixes those reproduced paths:

- Keep separately mapped nonspacing marks with their visual base during viewer RTL recovery.
- Apply the existing logical-order recovery to the read-only export reader, preserving
  authoritative ActualText.
- Keep Arabic numeric separators inside EN/AN numeric sequences.
- Preserve explicit empty ToUnicode destinations and retain auxiliary drawing operators
  without letting them invent semantic word gaps.
- Put shaped-cluster PDF Unicode on an advancing body glyph, while keeping original shaping
  positions and character/word spacing unchanged.
- Prefer a validated complete embedded source face and consistently use its full encoding
  within the affected source span when editing contextual text.

Add source-generated full-font, actual-subset and added-text regression cases. Font assets
remain external in the CI-pinned craft-fonts checkout; no fonts, generated PDFs or models are
committed. No dependency versions are updated.

Related: #4653. Coordinate the source-font/cluster work with #3465 and #4920; this is not a
replacement for their broader selection/fallback work. #5042 addresses separate export
direction metadata. These issues and PRs are not claimed fully resolved.

This is a draft contribution for review and coordination. The smallest independent first PR
is the mapped-mark viewer fix. `docs/arabic-contribution-roadmap.md` gives the exact module
boundaries for splitting this branch into independent contributions.

## How you verified it

- [x] `cargo fmt --all -- --check`
- [x] Scoped Clippy with `--all-targets -D warnings` for fonts, edit and render
- [x] Scoped tests for fonts, edit and render: **949 passed, 3 ignored**
- [x] `cargo xtask layers` equivalent built xtask: 37 crates, no layering violations
- [x] `cargo xtask assets` equivalent built xtask: 289 repository, 6 dependency and 22 build-time assets
- [x] Native CLI build and 13-step extraction/search/edit/save/reopen/DOCX/render check
- [x] Scoped wasm32-unknown-unknown library check for fonts, edit and render (five renderer target-specific warnings; not a complete web-app build)

Real-font inputs were mandatory, not optional early returns. The active corpus covers 28
PDF cases: 10 authoritative added-text phrases, 8 full-source phrases, 8 actual-subset phrases,
an Arabic table and an existing-source edit with harakat. Incremental saving preserves the
original source byte prefix; serialization/reopen retains tested pixels and exact text.
Both actual DOCX exports retain exact logical Unicode in `word/document.xml`.

Temporarily reverting the production changes while retaining final regressions produced 11
expected test failures across the relevant targets. The final source was restored exactly
and the scoped suite passed again. A further new-line auxiliary regression separately fails
when only its guard is removed and passes when restored.

The two legacy mixed/URL ignored tests were explicitly run and still fail as documented;
they are not counted as passes. The third ignore is pre-existing. Exact commands, font hashes,
OFL provenance and independent-reader observations are in `docs/arabic-corpus.md` and
`docs/arabic-support-assessment.md`. No full-workspace or desktop/clipboard/IME claim is made.

## Never-crash

- [x] No new production panic shortcuts or unsafe code
- [x] New input-derived indexing uses checked access; new mapping work has explicit budgets
- [x] Existing malformed-input and resource-limit tests pass in the scoped suites

## Provenance

- [x] No Adobe product assets or recreated Adobe graphics
- [x] No font files committed; real-font inputs stay in craft-fonts/external local derivatives
- [x] No GPL/AGPL code copied; Poppler used only as an external process
- [x] Contributor-original source-generated fixtures; font provenance and licenses documented
- [x] Asset validation passes

## Security, reach and documentation

- [x] No new production outbound networking, listener, path or credential handling
- [x] Existing automation tools exercise the changed paths headlessly
- [x] Existing root confinement and opt-in control/MCP behavior are unchanged
- [x] Assessment, corpus instructions, module-level PR roadmap and progress docs updated
- [x] No parity/readiness percentage is upgraded

## Anything a reviewer should look at first

Mixed untagged Arabic/Latin paragraphs may have identical painted glyphs despite different
logical text. Two explicit legacy limitations remain; this is not a universal inverse-bidi
algorithm. Shifted marks and missing/broken Unicode mappings also need separate evidence.

DOCX text is logical, but Word paragraph direction metadata is still absent; coordinate #5042.
Precise selection within some added ActualText spans, general contextual mixed-style editing,
complex layouts, annotations/forms, and Arabic OCR are outside this contribution. Arabic OCR
models/runtimes are not added.
