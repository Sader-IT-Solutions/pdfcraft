//! Real craft-fonts outlines, checked through both PDF text readers.
//!
//! Generated in memory under the project license; no font or PDF fixture is committed.
use pdfcraft_cos::{Dict, Document, Object, SaveOptions, Stream};
use pdfcraft_edit::{AddedText, Align, Content, add_content, reading_blocks_checked};
use pdfcraft_render::{PageRenderer, RenderRequest, RenderedPage};
use std::sync::Arc;

const PHRASES: &[(&str, &str)] = &[
    ("invocation", "باسم الآب والابن والروح القدس"),
    ("arabic", "اللغة العربية جميلة"),
    ("geneva", "مرحباً بكم في جنيف"),
    ("date", "الاجتماع يوم 15 أكتوبر 2026 الساعة 14:30"),
    ("invoice", "رقم الفاتورة INV-2026-001"),
    ("harakat", "السَّلَامُ عَلَيْكُمْ"),
    ("ligatures", "لا لأ لإ لآ سلام"),
    ("mixed", "مرحبا Bonjour Genève Hello PDF"),
    ("digits", "المبلغ ١٢٣٫٤٥ و 123.45"),
    ("url", "الرابط https://example.org/a?x=12 والمرجع (PDF)"),
];

fn require_arabic() -> bool {
    if pdfcraft_fonts::document_arabic_font().is_some() {
        return true;
    }
    assert!(
        std::env::var_os("PDFCRAFT_ARABIC_CORPUS_REQUIRED").is_none(),
        "the required Arabic corpus needs CRAFT_FONTS_DIR pointing at the CI-pinned craft-fonts checkout"
    );
    eprintln!("Arabic fidelity corpus not exercised: set CRAFT_FONTS_DIR; PDFCRAFT_ARABIC_CORPUS_REQUIRED=1 makes this fatal");
    false
}

fn blank() -> Document {
    let mut doc = Document::new_empty();
    let pages = doc.add(Object::Dict(Dict::new()));
    let contents = doc.add(Object::Stream(Stream::flate(Dict::new(), b"")));
    let mut page = Dict::new();
    page.set(b"Type".to_vec(), Object::name("Page"));
    page.set(b"Parent".to_vec(), Object::Ref(pages));
    page.set(b"MediaBox".to_vec(), Object::Array([0, 0, 600, 400].map(Object::Int).to_vec()));
    page.set(b"Contents".to_vec(), Object::Ref(contents));
    page.set(b"Resources".to_vec(), Object::Dict(Dict::new()));
    let page = doc.add(Object::Dict(page));
    doc.update_dict(pages, |p| {
        p.set(b"Type".to_vec(), Object::name("Pages"));
        p.set(b"Kids".to_vec(), Object::Array(vec![Object::Ref(page)]));
        p.set(b"Count".to_vec(), Object::Int(1));
    })
    .unwrap();
    let mut root = Dict::new();
    root.set(b"Type".to_vec(), Object::name("Catalog"));
    root.set(b"Pages".to_vec(), Object::Ref(pages));
    let root = doc.add(Object::Dict(root));
    doc.trailer_mut().set(b"Root".to_vec(), Object::Ref(root));
    doc
}

fn bytes(doc: &Document) -> Arc<Vec<u8>> {
    Arc::new(pdfcraft_cos::write_full(doc, &SaveOptions::default()).unwrap())
}

fn artifacts(kind: &str, name: &str, logical: &str, data: &[u8]) {
    if let Some(dir) = std::env::var_os("PDFCRAFT_ARABIC_ARTIFACT_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{kind}-{name}.pdf")), data).unwrap();
        std::fs::write(dir.join(format!("{kind}-{name}.txt")), logical).unwrap();
    }
}

fn raster_artifact(kind: &str, name: &str, raster: &RenderedPage) {
    if let Some(dir) = std::env::var_os("PDFCRAFT_ARABIC_ARTIFACT_DIR") {
        let mut ppm = format!("P6\n{} {}\n255\n", raster.width, raster.height).into_bytes();
        ppm.extend(raster.rgba.as_chunks::<4>().0.iter().flat_map(|p| p[..3].iter().copied()));
        std::fs::write(std::path::PathBuf::from(dir).join(format!("{kind}-{name}.ppm")), ppm).unwrap();
    }
}

/// Independently positioned TrueType glyphs with only `/ToUnicode`, as an external
/// producer may write them. No `/ActualText` or PdfCraft added-item metadata masks reading.
fn source_font_document(font: &[u8], logical: &str) -> Document {
    let mut doc = blank();
    let shaped = pdfcraft_fonts::source_shaping::shape(font, logical).unwrap();
    let mappings = shaped.pdf_mappings(font).unwrap();
    let mut fonts = Dict::new();
    let (_, postscript_name) = pdfcraft_fonts::embedding::names(font).unwrap();
    let resource = pdfcraft_fonts::embedding::embed_mapping(&mut doc, &mut fonts, font, &postscript_name, &mappings).unwrap();
    let mut content = format!("BT /{resource} 16 Tf\n");
    for (i, g) in shaped.glyphs.iter().enumerate() {
        let [x, y] = g.position;
        content.push_str(&format!("1 0 0 1 {} {} Tm <{:04X}> Tj\n", 30.0 + 16.0 * x, 300.0 + 16.0 * y, i + 1));
    }
    content.push_str("ET\n");
    let contents = doc.add(Object::Stream(Stream::flate(Dict::new(), content.as_bytes())));
    let page = pdfcraft_model::pages(&doc)[0].obj;
    doc.update_dict(page, |p| {
        let mut res = Dict::new();
        res.set(b"Font".to_vec(), Object::Dict(fonts));
        p.set(b"Resources".to_vec(), Object::Dict(res));
        p.set(b"Contents".to_vec(), Object::Ref(contents));
    })
    .unwrap();
    doc
}

fn source_font(variable: &str) -> Option<Vec<u8>> {
    let Some(path) = std::env::var_os(variable) else {
        assert!(std::env::var_os("PDFCRAFT_ARABIC_CORPUS_REQUIRED").is_none(), "the required source-font corpus needs {variable}");
        eprintln!("source-font corpus not exercised: set {variable} to a licensed static Arabic face");
        return None;
    };
    let font = std::fs::read(path).unwrap();
    pdfcraft_fonts::embedding::validate_document_input(&font).unwrap();
    Some(font)
}

fn subset_names(doc: &mut Document) {
    let page = &pdfcraft_model::pages(doc)[0];
    let resources = doc.dict(page.dict.get(b"Resources").unwrap()).unwrap();
    let fonts = doc.dict(resources.get(b"Font").unwrap()).unwrap();
    let root = fonts.iter().next().unwrap().1.clone();
    let font = doc.dict(&root).unwrap();
    let Object::Ref(root) = root else { panic!("fixture font must be indirect") };
    let original_name = String::from_utf8_lossy(font.name(b"BaseFont").unwrap());
    let name = Object::name(&format!("ABCDEF+{original_name}"));
    doc.update_dict(root, |f| f.set(b"BaseFont".to_vec(), name.clone())).unwrap();
    let children = font.get(b"DescendantFonts").unwrap().as_array().unwrap();
    for child in children {
        let dict = doc.dict(child).unwrap();
        let Object::Ref(child) = child else { panic!("fixture descendant must be indirect") };
        doc.update_dict(*child, |f| f.set(b"BaseFont".to_vec(), name.clone())).unwrap();
        let Object::Ref(descriptor) = dict.get(b"FontDescriptor").unwrap() else { panic!("fixture descriptor must be indirect") };
        doc.update_dict(*descriptor, |f| f.set(b"FontName".to_vec(), name.clone())).unwrap();
    }
}

fn source_failures(font: &[u8], cases: &[(&str, &str)], kind: &str) -> Vec<String> {
    let mut failures = Vec::new();
    for &(name, logical) in cases {
        let mut doc = source_font_document(font, logical);
        if kind == "subset" {
            subset_names(&mut doc);
        }
        let saved = bytes(&doc);
        artifacts(kind, name, logical, &saved);
        let mut viewer = PageRenderer::new(saved.clone(), Default::default());
        let text = viewer.filtered_text(0, Default::default()).unwrap();
        if text.plain_text() != logical {
            failures.push(format!("source viewer {name}: {:?}, expected {logical:?}", text.plain_text()));
        }
        for word in logical.split_whitespace() {
            let range = text
                .find_opts(word, true, true)
                .into_iter()
                .find(|r| text.text_of(r.clone()) == word)
                .unwrap_or_else(|| panic!("a whole-word search must give clipboard-ready Unicode: {name} {word:?}"));
            let (first, last) = text.word_at(range.start).unwrap();
            assert_eq!(text.text_of(first..last + 1), word, "double-click word copy: {name}");
        }
        let reading = reading_blocks_checked(&doc, 0).unwrap();
        let blocks = reading.blocks.iter().map(|b| b.text.as_str()).collect::<Vec<_>>();
        if blocks != [logical] {
            failures.push(format!("source export reader {name}: {blocks:?}, expected {logical:?}"));
        }
        let reopened = Document::open(saved).unwrap();
        let mut after = PageRenderer::new(bytes(&reopened), Default::default());
        assert_eq!(after.filtered_text(0, Default::default()).unwrap(), text, "source text/geometry after reopen: {name}");
        let raster = viewer.render(RenderRequest { page: 0, scale: 1.0, ..Default::default() });
        assert!(raster.error.is_none(), "{name}: {:?}", raster.error);
        raster_artifact(kind, name, &raster);
        assert_eq!(after.render(RenderRequest { page: 0, scale: 1.0, ..Default::default() }).rgba, raster.rgba, "source pixels after reopen: {name}");
    }
    failures
}

fn recoverable_source_phrases() -> Vec<(&'static str, &'static str)> {
    // Separate tests below retain the failures whose paragraph direction is not
    // present in the legacy PDF. They are never counted as passing recovery cases.
    PHRASES.iter().copied().filter(|(name, _)| !matches!(*name, "mixed" | "url")).collect()
}

#[test]
fn embedded_arabic_source_without_actual_text_agrees_in_both_readers() {
    let Some(font) = source_font("PDFCRAFT_TEST_ARABIC_FONT") else { return };
    let failures = source_failures(&font, &recoverable_source_phrases(), "source");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn embedded_arabic_subset_without_actual_text_agrees_in_both_readers() {
    let Some(font) = source_font("PDFCRAFT_TEST_ARABIC_SUBSET_FONT") else { return };
    let failures = source_failures(&font, &recoverable_source_phrases(), "subset");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn legacy_mixed_paragraphs_can_have_identical_painted_glyphs() {
    let Some(font) = source_font("PDFCRAFT_TEST_ARABIC_FONT") else { return };
    let rtl = pdfcraft_fonts::source_shaping::shape(&font, "مرحبا Bonjour Genève Hello PDF").unwrap();
    let ltr = pdfcraft_fonts::source_shaping::shape(&font, "Bonjour Genève Hello PDF مرحبا").unwrap();
    let painting = |run: &pdfcraft_fonts::source_shaping::ShapedRun| run.glyphs.iter().map(|g| (g.id, g.position)).collect::<Vec<_>>();
    assert_eq!(painting(&rtl), painting(&ltr), "glyph positions alone cannot prove which logical paragraph was intended");
}

#[test]
#[ignore = "known legacy mixed-direction ambiguity without ActualText; related to #4653"]
fn legacy_mixed_direction_without_metadata_preserves_original_logical_order() {
    let Some(font) = source_font("PDFCRAFT_TEST_ARABIC_FONT") else { return };
    let failures = source_failures(&font, &[PHRASES[7]], "legacy-mixed");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "known legacy URL/bracket recovery limitation without ActualText; related to #4653"]
fn legacy_url_without_metadata_preserves_original_logical_order() {
    let Some(font) = source_font("PDFCRAFT_TEST_ARABIC_FONT") else { return };
    let failures = source_failures(&font, &[PHRASES[9]], "legacy-url");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn existing_arabic_source_can_be_edited_and_saved_incrementally() {
    let Some(font) = source_font("PDFCRAFT_TEST_ARABIC_FONT") else { return };
    let original = bytes(&source_font_document(&font, PHRASES[0].1));
    let mut doc = Document::open(original.clone()).unwrap();
    let replacement = "السَّلَامُ عَلَيْكُمْ";
    pdfcraft_edit::rewrite_block(&mut doc, 0, 0, Some(replacement), &Default::default()).unwrap();
    assert_eq!(reading_blocks_checked(&doc, 0).unwrap().blocks[0].text, replacement);
    let mut edited = PageRenderer::new(bytes(&doc), Default::default());
    assert_eq!(edited.filtered_text(0, Default::default()).unwrap().plain_text(), replacement);
    let raster = edited.render(RenderRequest { page: 0, scale: 1.0, ..Default::default() });
    assert!(raster.error.is_none(), "{:?}", raster.error);
    raster_artifact("edited", "harakat", &raster);
    let saved = pdfcraft_cos::write_incremental(&doc, &SaveOptions::default()).unwrap();
    assert!(saved.starts_with(&original), "the original source PDF remains intact");
    artifacts("edited", "harakat", replacement, &saved);
    let mut after = PageRenderer::new(Arc::new(saved), Default::default());
    assert_eq!(after.filtered_text(0, Default::default()).unwrap().plain_text(), replacement);
    assert_eq!(after.render(RenderRequest { page: 0, scale: 1.0, ..Default::default() }).rgba, raster.rgba);
}

#[test]
fn arabic_table_column_copy_preserves_cell_order_and_unicode() {
    if !require_arabic() {
        return;
    }
    let mut doc = blank();
    for (text, x, y) in [("الاسم", 300.0, 320.0), ("المبلغ", 30.0, 320.0), ("جنيف", 300.0, 260.0), ("١٢٣", 30.0, 260.0)] {
        add_content(
            &mut doc,
            0,
            &Content::Text(AddedText { text: text.into(), rect: [x, y, x + 200.0, y + 30.0], size: 16.0, align: Align::Right, ..Default::default() }),
        )
        .unwrap();
    }
    let saved = bytes(&doc);
    let expected = "الاسم\tالمبلغ\nجنيف\t١٢٣";
    artifacts("added", "table", expected, &saved);
    let mut viewer = PageRenderer::new(saved, Default::default());
    let text = viewer.filtered_text(0, Default::default()).unwrap();
    assert_eq!(text.column_text(&text.glyphs_in([0.0, 0.0, 600.0, 400.0])), expected);
}

#[test]
fn real_arabic_phrases_agree_in_viewer_and_export_readers_after_reopen() {
    if !require_arabic() {
        return;
    }
    for &(name, logical) in PHRASES {
        let mut doc = blank();
        add_content(
            &mut doc,
            0,
            &Content::Text(AddedText {
                text: logical.into(),
                rect: [30.0, 300.0, 570.0, 350.0],
                size: 16.0,
                align: Align::Right,
                ..Default::default()
            }),
        )
        .unwrap();
        let initial = bytes(&doc);
        artifacts("added", name, logical, &initial);
        let mut viewer = PageRenderer::new(initial.clone(), Default::default());
        let text = viewer.filtered_text(0, Default::default()).unwrap();
        assert_eq!(text.plain_text(), logical, "viewer: {name}");
        assert_eq!(text.text_of(0..text.glyphs.len()), logical, "clipboard-ready selection: {name}");
        assert!(!text.find(logical).is_empty(), "search: {name}");
        let reading = reading_blocks_checked(&doc, 0).unwrap();
        assert!(!reading.truncated, "{name}");
        assert_eq!(reading.blocks.iter().map(|b| b.text.as_str()).collect::<Vec<_>>(), [logical], "export reader: {name}");
        let raster = viewer.render(RenderRequest { page: 0, scale: 1.0, ..Default::default() });
        assert!(raster.error.is_none(), "{name}: {:?}", raster.error);
        assert!(raster.warnings.is_empty(), "{name}: {:?}", raster.warnings);
        raster_artifact("added", name, &raster);
        assert!(raster.rgba.as_chunks::<4>().0.iter().any(|p| p[..3] != [255; 3]), "real glyphs must be painted: {name}");
        let reopened = Document::open(initial).unwrap();
        let mut after = PageRenderer::new(bytes(&reopened), Default::default());
        assert_eq!(after.filtered_text(0, Default::default()).unwrap(), text, "text/geometry after reopen: {name}");
        assert_eq!(after.render(RenderRequest { page: 0, scale: 1.0, ..Default::default() }).rgba, raster.rgba, "pixels after reopen: {name}");
    }
}
