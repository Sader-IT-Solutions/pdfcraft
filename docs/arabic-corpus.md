# Reproducing the Arabic fidelity corpus

Status: real-font cross-reader regression corpus, 2026-10-31. The generated outputs are
test artifacts, not committed assets or a claim of complete Arabic PDF support.

## Inputs and provenance

Use the existing PdfCraft checkout. No extra worktree is needed. Font files stay outside it.
PdfCraft CI pins `storytold/craft-fonts` at:

```text
b076417b5880bd4154bd22f00695b1d28c6bea83
```

The `fonts/manifest.txt` Arabic entry is Noto Sans Arabic, OFL-1.1, from
`google/fonts@7085eb89a950e85db5b166b7a58d414544b4140c/ofl/notosansarabic`.
The original license is `fonts/noto-sans-arabic/OFL.txt`. Verified source font SHA-256:

```text
63111b5b2e074dd48cc67692e0a2726d86ee94c1c37fe8598257b7b4e87e869e
```

The source is variable; the document embedder requires a static instance. This run used
fontTools 4.61.1 to instantiate **all** axes at their defaults (`wght=400`, `wdth=100`).
No font was copied from a proprietary application or installed system fallback.
Local derivative SHA-256 values (recorded observations for this tool version):

| File | SHA-256 |
| --- | --- |
| `NotoSansArabic-Regular.ttf` | `65efad4885c590b640d9601b8cc6d9b66026e9ff74154ac600207600842c0f70` |
| `NotoSansArabic-CorpusSubset.ttf` | `3e5b65b25f2339c643c9fda2c0a9e4256a61cb599c7cbc87b7008c529f45a5c0` |

The subset retains 148 of 1,711 glyphs. Preserve the original name/license records when
subsetting: the default fontTools name policy drops the license description, and PdfCraft
correctly refuses such a document font. Do not bypass that check.

## Prepare local font inputs

Clone the official craft-fonts repository outside the PdfCraft checkout and check out the
exact revision above. Reuse an existing matching checkout instead of overwriting it.
Verify the source hash before deriving fonts. With fontTools installed, this preparation
snippet writes only to `/workspace/arabic-deps`; it is not part of the product's Rust build:

```python
from pathlib import Path
import hashlib
import re
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools import subset

root = Path('/workspace/arabic-deps')
source = root / 'craft-fonts/fonts/noto-sans-arabic/NotoSansArabic.ttf'
assert hashlib.sha256(source.read_bytes()).hexdigest() == \
    '63111b5b2e074dd48cc67692e0a2726d86ee94c1c37fe8598257b7b4e87e869e'
font = TTFont(source, recalcTimestamp=False)
axes = {axis.axisTag: axis.defaultValue for axis in font['fvar'].axes}
static = instantiateVariableFont(font, axes, inplace=False)
static.save(root / 'NotoSansArabic-Regular.ttf')

font = TTFont(root / 'NotoSansArabic-Regular.ttf', recalcTimestamp=False)
options = subset.Options()
options.layout_features = ['*']
options.name_IDs = ['*']
options.name_languages = ['*']
options.name_legacy = True
subsetter = subset.Subsetter(options=options)
source_code = Path('crates/edit/tests/arabic_fidelity.rs').read_text()
phrases = re.findall(r'^\s*\("[^"]+", "([^"]+)"\)', source_code, re.M)
assert len(phrases) == 10
subsetter.populate(text=''.join(phrases))
subsetter.subset(font)
assert font['name'].getDebugName(13)  # Keep the original license declaration.
font.save(root / 'NotoSansArabic-CorpusSubset.ttf')
```

Keep the original OFL license with any redistributed artifacts containing embedded fonts.
No source/derived font or generated PDF belongs in this repository's tracked files.

## Run the corpus

In the prepared cloud machine, activate Rust before running Cargo:

```sh
export RUSTUP_HOME=/workspace/.pdfcraft-rustup
export CARGO_HOME=/workspace/.pdfcraft-cargo
export PATH="$CARGO_HOME/bin:$PATH"
cd /workspace/pdfcraft
export CRAFT_FONTS_DIR=/workspace/arabic-deps/craft-fonts
export CRAFT_FONTS_REQUIRED=1
export PDFCRAFT_TEST_ARABIC_FONT=/workspace/arabic-deps/NotoSansArabic-Regular.ttf
export PDFCRAFT_TEST_ARABIC_SUBSET_FONT=/workspace/arabic-deps/NotoSansArabic-CorpusSubset.ttf
export PDFCRAFT_ARABIC_CORPUS_REQUIRED=1
export PDFCRAFT_ARABIC_ARTIFACT_DIR=/tmp/pdfcraft-arabic-corpus-final
timeout 900 cargo test --offline --locked -p pdfcraft-edit --test arabic_fidelity -j 2 -- --nocapture
```

`PDFCRAFT_ARABIC_CORPUS_REQUIRED=1` makes missing font inputs fail instead of letting
optional tests return early. Without the variable, ordinary builds remain usable without
craft-fonts. The artifact directory is optional: tests normally keep generated PDFs in memory.
Each emitted PDF has a matching UTF-8 `.txt` file containing exact logical ground truth.

## What the fixtures establish

- Ten added-text phrases use actual Noto outlines and logical `/ActualText`, including all
  six mission phrases, lam-alef, French/English, Arabic-Indic/Western numbers, URLs and brackets.
- Eight recoverable legacy phrases use embedded static TrueType outlines and explicit
  `/ToUnicode` **without** `/ActualText` or PdfCraft added-item metadata. Both the viewer and
  the editor/export reader must return their logical text. Repeat with a real subset program.
- Whole-word search ranges must copy the original word. Save/reopen retains text/geometry.
- Added-text renders must be nonempty and byte-identical after serialization/reopen.
- Editing a previously existing Arabic source is checked through incremental save/reopen.
- An Arabic table checks rectangular clipboard text, cell ordering and tabs/newlines.
- Font-independent tests additionally cover mapped diacritics, Hebrew, Latin combining marks,
  empty Unicode destinations, auxiliary drawings, and decimal separators.

The corpus is deterministic source code under the project license. It does not depend on
downloaded third-party PDFs. The static source generator uses existing shaping/embedding
APIs to isolate reading behavior; it is not a second independent shaper.

## Known failures and ambiguity

Two tests remain explicitly ignored: the metadata-free legacy mixed paragraph and URL/bracket
cases. Run them deliberately to inspect the current failures (a nonzero test exit is expected):

```sh
timeout 900 cargo test --offline --locked -p pdfcraft-edit --test arabic_fidelity -j 2 -- --ignored --nocapture
```

They are not counted as successful recovery. The added-text versions with `/ActualText`
remain passing assertions. A separate passing test proves that these two different paragraphs:

```text
مرحبا Bonjour Genève Hello PDF
Bonjour Genève Hello PDF مرحبا
```

produce identical painted glyph IDs and positions with the same font. Physical glyph order
alone cannot identify which logical paragraph was intended. The legacy URL case additionally
exposes bracket/base-direction recovery limits. Related issue: upstream #4653.

Current APIs do not yet expose a per-span confidence/warning contract. Adding such a contract,
and qualifying source PDFs with reliable direction/tag metadata, remain future work. Do not
present the current heuristic as a guarantee for arbitrary untagged mixed text.

Real Word/Acrobat/scanner-generated inputs, complex layouts, annotation/form editing, unknown
encodings and Arabic OCR are not all qualified by this corpus. They remain separate work.

## Independent checks

Use external readers only as processes, following the repository clean-room policy. Poppler
can render the generated PDFs and extract raw text. Preserve its raw UTF-8 output: it may add
bidi controls or use a different output-order convention. Do not silently strip controls,
reverse strings or normalize the result to claim agreement with ground truth.

The initial synthetic Helvetica semantic fixture detached its mark under Poppler; that result
is an oracle disagreement, not a passed cross-reader claim. Real-font previews and subsequent
CLI/DOCX results are recorded in `arabic-support-assessment.md`: 13 automation steps pass and
both DOCX files retain exact logical Unicode, while Word direction metadata remains absent.

## Exercise the native CLI and DOCX path

Build the actual CLI with the same font environment, then run the generated source through
extraction, search, export, editing, save/reopen and rendering. The following script only
writes within the corpus artifact directory:

```sh
timeout 900 cargo build --offline --locked -p pdfcraft-cli -j 2
python3 - <<'PYTHON'
import json
from pathlib import Path
r = Path('/tmp/pdfcraft-arabic-corpus-final')
steps = [
    ('doc_open', {'path': str(r / 'source-invocation.pdf')}),
    ('text_extract', {'doc': 1}),
    ('text_find', {'doc': 1, 'query': 'الروح'}),
    ('doc_export_office', {'doc': 1, 'path': str(r / 'source-invocation.docx')}),
    ('page_render', {'doc': 1, 'page': 1, 'dpi': 120}),
    ('text_paragraphs', {'doc': 1, 'page': 1}),
    ('text_edit', {'doc': 1, 'page': 1, 'paragraph': 1, 'text': 'السَّلَامُ عَلَيْكُمْ'}),
    ('text_extract', {'doc': 1}),
    ('doc_save', {'doc': 1, 'path': str(r / 'cli-edited.pdf')}),
    ('doc_open', {'path': str(r / 'cli-edited.pdf')}),
    ('text_extract', {'doc': 2}),
    ('doc_export_office', {'doc': 2, 'path': str(r / 'cli-edited.docx')}),
    ('page_render', {'doc': 2, 'page': 1, 'dpi': 120}),
]
script = [{'tool': tool, 'args': args} for tool, args in steps]
script[4]['out'] = str(r / 'cli-source.png')
script[-1]['out'] = str(r / 'cli-edited.png')
(r / 'cli-steps.json').write_text(json.dumps(script, ensure_ascii=False, indent=2))
PYTHON
target/debug/pdfcraft-cli run --script /tmp/pdfcraft-arabic-corpus-final/cli-steps.json \
  --root /tmp/pdfcraft-arabic-corpus-final --report \
  > /tmp/pdfcraft-arabic-corpus-final/cli-report.json
```

Inspect `cli-report.json` for every step's status and returned exact text. Inspect DOCX
`word/document.xml` with a ZIP/XML reader and concatenate `w:t` values to compare with the
literal ground truth. Do not normalize or reverse it. This validates exported logical text;
Word paragraph/run direction metadata and application visual layout remain separate #5042 work.

## Revision history

- 2026-10-31: add verified real-font preparation, full/subset and added-text corpus, artifact
  output, required-input checks, and explicit metadata-free mixed-direction limitations.
