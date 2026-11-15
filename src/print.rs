use crate::error::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

/// Output format for printing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrintFormat {
    Pdf,
    Docx,
}

impl PrintFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pdf" => Ok(PrintFormat::Pdf),
            "docx" | "word" => Ok(PrintFormat::Docx),
            other => Err(crate::error::MagpieError::Custom(format!(
                "Unknown print format '{other}'. Use pdf or word."
            ))),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            PrintFormat::Pdf => "pdf",
            PrintFormat::Docx => "docx",
        }
    }
}

/// Page size for printing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageSize {
    A4,
    Letter,
}

impl PageSize {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "a4" => Ok(PageSize::A4),
            "letter" | "us-letter" | "us_letter" => Ok(PageSize::Letter),
            other => Err(crate::error::MagpieError::Custom(format!(
                "Unknown page size '{other}'. Use a4 or letter."
            ))),
        }
    }

    /// Returns (width_pt, height_pt) for PDF.
    pub fn pdf_dimensions(&self) -> (f64, f64) {
        match self {
            PageSize::A4 => (595.28, 841.89),   // A4 in points
            PageSize::Letter => (612.0, 792.0), // US Letter in points
        }
    }

    /// Returns (width_dxa, height_dxa) for DOCX page (1 dxa = 1/20 pt).
    pub fn docx_dimensions(&self) -> (u32, u32) {
        match self {
            PageSize::A4 => (11906, 16838),   // A4 in twips/dxa
            PageSize::Letter => (12240, 15840), // US Letter in twips/dxa
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            PageSize::A4 => "A4",
            PageSize::Letter => "US Letter",
        }
    }
}

/// Convert a markdown-ish text file into printable blocks.
struct Block {
    kind: BlockKind,
    text: String,
}

#[derive(PartialEq)]
enum BlockKind {
    Heading(u32),
    Paragraph,
    Blank,
}

fn parse_blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim_end();
        if line.starts_with('#') {
            let level = line.chars().take_while(|c| *c == '#').count() as u32;
            let text = line.trim_start_matches('#').trim().to_string();
            blocks.push(Block {
                kind: BlockKind::Heading(level),
                text,
            });
        } else if line.trim().is_empty() {
            blocks.push(Block {
                kind: BlockKind::Blank,
                text: String::new(),
            });
        } else {
            let clean = line
                .replace("**", "")
                .replace("*", "")
                .replace("__", "")
                .replace("_", "")
                .replace("`", "");
            blocks.push(Block {
                kind: BlockKind::Paragraph,
                text: clean,
            });
        }
    }
    blocks
}

/// Print a file to PDF or Word.
pub fn print_file(input: &Path, format: PrintFormat, size: PageSize, out: Option<&Path>) -> Result<()> {
    if !input.exists() {
        return Err(crate::error::MagpieError::Custom(format!(
            "Input file not found: {}",
            input.display()
        )));
    }

    let text = fs::read_to_string(input)?;
    let blocks = parse_blocks(&text);

    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            let ext = format.extension();
            PathBuf::from(format!("{stem}.{ext}"))
        }
    };

    match format {
        PrintFormat::Pdf => render_pdf(&blocks, size, &out_path)?,
        PrintFormat::Docx => render_docx(&blocks, size, &out_path)?,
    }

    println!(
        "✅ Printed \"{}\" → {} ({}, {})",
        input.display(),
        out_path.display(),
        format.extension().to_uppercase(),
        size.label()
    );
    Ok(())
}

/// Estimated width of a string in points for Helvetica at a given font size.
/// Uses average character width ~0.5 em for mixed case text.
fn text_width(text: &str, font_size: f64) -> f64 {
    // Helvetica average char width is roughly 0.5 * font_size for mixed case.
    // Uppercase and wide characters (W, M) are wider, narrow (i, l, space) narrower.
    // This approximation is good enough for wrapping decisions.
    let mut width = 0.0;
    for ch in text.chars() {
        let char_width = match ch {
            ' ' => 0.278,
            'i' | 'l' | '!' | '\'' | '[' | ']' | '(' | ')' | '|' => 0.333,
            'I' => 0.333,
            'f' => 0.333,
            'r' | 't' | 'z' | 'c' | 'e' | 'o' | 'a' | 's' | 'n' | 'u' | 'v' | 'w' | 'x' | 'y' | 'k' | 'h' | 'b' | 'd' | 'p' | 'q' | 'j' | 'm' | 'g' | 'w' => 0.5,
            'M' | 'W' => 0.722,
            'D' | 'N' | 'O' | 'Q' | 'U' | 'V' | 'X' | 'Y' | 'Z' | 'A' | 'B' | 'C' | 'E' | 'F' | 'G' | 'H' | 'J' | 'K' | 'L' | 'P' | 'R' | 'S' | 'T' => 0.667,
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => 0.556,
            '.' | ',' | ':' | ';' => 0.278,
            '-' | '_' => 0.333,
            '/' | '\\' | '@' | '#' | '$' | '%' | '&' | '*' | '=' | '?' | '<' | '>' | '"' | '{' | '}' | '~' | '^' | '`' => 0.5,
            _ => 0.5,
        };
        width += char_width * font_size;
    }
    width
}

/// Wrap text to fit within max_width points at given font size.
fn wrap_text(text: &str, font_size: f64, max_width: f64) -> Vec<String> {
    if text_width(text, font_size) <= max_width {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        let trial = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if text_width(&trial, font_size) <= max_width {
            current_line = trial;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line.clone());
            }
            current_line = word.to_string();

            // If a single word is too long, force-break it
            if text_width(&current_line, font_size) > max_width {
                lines.push(current_line.clone());
                current_line = String::new();
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Render blocks to a multi-page PDF.
fn render_pdf(blocks: &[Block], size: PageSize, out: &Path) -> Result<()> {
    let (page_w, page_h) = size.pdf_dimensions();
    let margin: f64 = 54.0; // 0.75 inch margins
    let content_width = page_w - 2.0 * margin;

    // Style settings
    let body_size: f64 = 11.0;
    let body_leading: f64 = 15.0;
    let h1_size: f64 = 20.0;
    let h1_leading: f64 = 26.0;
    let h2_size: f64 = 16.0;
    let h2_leading: f64 = 20.0;
    let h3_size: f64 = 13.0;
    let h3_leading: f64 = 17.0;

    // Process blocks into wrapped lines with styles
    let mut pages: Vec<Vec<(f64, f64, String, f64, bool)>> = Vec::new();
    pages.push(Vec::new());

    let mut y = page_h - margin;
    let mut page_idx = 0;

    for block in blocks {
        match block.kind {
            BlockKind::Heading(level) => {
                let (font_size, leading) = match level {
                    1 => (h1_size, h1_leading),
                    2 => (h2_size, h2_leading),
                    _ => (h3_size, h3_leading),
                };
                let wrapped = wrap_text(&block.text, font_size, content_width);
                for line in wrapped {
                    if y < margin + leading {
                        pages.push(Vec::new());
                        page_idx += 1;
                        y = page_h - margin;
                    }
                    pages[page_idx].push((y, margin, line, font_size, true));
                    y -= leading;
                }
                y -= leading * 0.3;
            }
            BlockKind::Blank => {
                y -= body_leading * 0.5;
            }
            BlockKind::Paragraph => {
                let wrapped = wrap_text(&block.text, body_size, content_width);
                for line in wrapped {
                    if y < margin + body_leading {
                        pages.push(Vec::new());
                        page_idx += 1;
                        y = page_h - margin;
                    }
                    pages[page_idx].push((y, margin, line, body_size, false));
                    y -= body_leading;
                }
                y -= body_leading * 0.2;
            }
        }
        if y < margin {
            y = margin;
        }
    }

    // Build PDF objects
    let mut objects: Vec<String> = Vec::new();
    let mut xref_offsets: Vec<u64> = Vec::new();
    let mut pdf_body = String::from("%PDF-1.4\n");
    pdf_body.push_str("%\x00\x01\x02\x03\n");

    // Object numbering:
    // 1 = Catalog
    // 2 = Pages tree
    // 3 = Helvetica-Bold font
    // 4 = Helvetica font
    // 5.. = Page objects and content streams interleaved

    // Object 1: Catalog
    objects.push("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n".to_string());

    // Object 2: Pages tree
    let page_count = pages.len() as u32;
    let mut kids = String::new();
    let mut next_obj = 5; // first page object
    for _ in 0..page_count {
        kids.push_str(&format!("{} 0 R ", next_obj));
        next_obj += 2; // page + content stream
    }
    objects.push(format!(
        "2 0 obj\n<< /Type /Pages /Kids [{kids}] /Count {page_count} >>\nendobj\n"
    ));

    // Object 3: Helvetica-Bold
    objects.push(
        "3 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n"
            .to_string(),
    );
    // Object 4: Helvetica
    objects.push(
        "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
            .to_string(),
    );

    // Page objects (odd numbers starting at 5) and content streams (even numbers starting at 6)
    let mut obj_num = 5;
    for page_lines in &pages {
        // Content stream (even number)
        let content_obj = obj_num + 1;
        let mut content = String::new();
        let mut y_text = page_h - margin;

        for (_y_pos, x, text, font_size, is_bold) in page_lines {
            let font_ref = if *is_bold { 3 } else { 4 };
            content.push_str(&format!(
                "BT /F{} {:.1} Tf {:.2} {:.2} Td ({}) Tj ET\n",
                font_ref, font_size, x, y_text, escape_pdf_string(text)
            ));
            y_text -= if *is_bold { 22.0 } else { 15.0 };
        }

        let content_len = content.len() as u32;
        objects.push(format!(
            "{content_obj} 0 obj\n<< /Length {content_len} >>\nstream\n{content}endstream\nendobj\n"
        ));

        // Page object (odd number)
        let mediabox = format!("[0 0 {:.2} {:.2}]", page_w, page_h);
        objects.push(format!(
            "{obj_num} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox {mediabox} /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {content_obj} 0 R >>\nendobj\n"
        ));

        obj_num += 2;
    }

    // Build xref
    let xref_start = pdf_body.len() as u64;
    let num_objects = objects.len() + 1;

    for obj in &objects {
        xref_offsets.push(pdf_body.len() as u64);
        pdf_body.push_str(obj);
    }

    pdf_body.push_str("xref\n");
    pdf_body.push_str(&format!("0 {num_objects}\n"));
    pdf_body.push_str("0000000000 65535 f \n");
    for offset in &xref_offsets {
        pdf_body.push_str(&format!("{:010} 00000 n \n", offset));
    }

    pdf_body.push_str("trailer\n");
    pdf_body.push_str(&format!(
        "<< /Size {num_objects} /Root 1 0 R >>\nstartxref\n{xref_start}\n%%EOF\n"
    ));

    let mut file = fs::File::create(out)?;
    file.write_all(pdf_body.as_bytes())?;
    Ok(())
}

/// Escape a string for use inside PDF parentheses.
fn escape_pdf_string(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\{:03o}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn render_docx(blocks: &[Block], size: PageSize, out: &Path) -> Result<()> {
    let (_width_dxa, _height_dxa) = size.docx_dimensions();

    let mut body_xml = String::new();
    body_xml.push_str("<w:body>");

    for block in blocks {
        match block.kind {
            BlockKind::Heading(level) => {
                let style = match level {
                    1 => "Heading1",
                    2 => "Heading2",
                    3 => "Heading3",
                    _ => "Title",
                };
                body_xml.push_str(&format!(
                    "<w:p><w:pPr><w:pStyle w:val=\"{style}\"/></w:pPr><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                    escape_xml(&block.text)
                ));
            }
            BlockKind::Blank => {
                body_xml.push_str("<w:p><w:r><w:t xml:space=\"preserve\"></w:t></w:r></w:p>");
            }
            BlockKind::Paragraph => {
                body_xml.push_str(&format!(
                    "<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
                    escape_xml(&block.text)
                ));
            }
        }
    }
    body_xml.push_str("</w:body>");

    let document_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
{body_xml}
</w:document>"#
    );

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;

    let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/document" Target="word/document.xml"/>
</Relationships>"#;

    let doc_rel = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

    let styles_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr></w:rPrDefault></w:docDefaults>
<w:style w:type="paragraph" w:styleId="Title"><w:name w:val="title"/><w:rPr><w:sz w:val="40"/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:rPr><w:sz w:val="32"/><w:b/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/><w:rPr><w:sz w:val="28"/><w:b/></w:rPr></w:style>
<w:style w:type="paragraph" w:styleId="Heading3"><w:name w:val="heading 3"/><w:rPr><w:sz w:val="24"/><w:b/></w:rPr></w:style>
</w:styles>"#;

    let core_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
<dc:title>Magpie Print</dc:title>
</cp:coreProperties>"#;

    let file = fs::File::create(out)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    zip.start_file("word/document.xml", options)?;
    zip.write_all(document_xml.as_bytes())?;

    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(doc_rel.as_bytes())?;

    zip.start_file("word/styles.xml", options)?;
    zip.write_all(styles_xml.as_bytes())?;

    zip.start_file("docProps/core.xml", options)?;
    zip.write_all(core_xml.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size_dimensions() {
        assert_eq!(PageSize::A4.pdf_dimensions(), (595.28, 841.89));
        assert_eq!(PageSize::Letter.pdf_dimensions(), (612.0, 792.0));
        assert_eq!(PageSize::A4.docx_dimensions(), (11906, 16838));
        assert_eq!(PageSize::Letter.docx_dimensions(), (12240, 15840));
    }

    #[test]
    fn test_parse_blocks() {
        let blocks = parse_blocks("# Title\n\nHello world\n");
        assert_eq!(blocks.len(), 3);
        assert!(matches!(blocks[0].kind, BlockKind::Heading(1)));
        assert_eq!(blocks[0].text, "Title");
        assert!(matches!(blocks[1].kind, BlockKind::Blank));
        assert!(matches!(blocks[2].kind, BlockKind::Paragraph));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<Hello & World>"), "&lt;Hello &amp; World&gt;");
    }

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("Hello (World)"), "Hello \\(World\\)");
    }

    #[test]
    fn test_wrap_text() {
        let wrapped = wrap_text("This is a short sentence that should wrap", 11.0, 100.0);
        assert!(wrapped.len() > 1);
        // Each line should fit within max_width
        for line in &wrapped {
            assert!(text_width(line, 11.0) <= 100.0 + 1.0); // small tolerance
        }
    }

    #[test]
    fn test_text_width() {
        assert!(text_width("Hello", 12.0) > 0.0);
        assert!(text_width("Hello World", 12.0) > text_width("Hello", 12.0));
    }
}
