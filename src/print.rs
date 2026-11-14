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

fn render_pdf(blocks: &[Block], size: PageSize, out: &Path) -> Result<()> {
    let (page_w, page_h) = size.pdf_dimensions();

    // Build the content stream with text operations.
    let mut content = String::new();
    let mut y = page_h - 54.0; // top margin
    let x = 54.0;
    let line_height = 14.0;

    for block in blocks {
        match block.kind {
            BlockKind::Heading(level) => {
                let size_pt = (22.0 - (level as f64 * 2.0)).max(14.0);
                content.push_str(&format!(
                    "BT /F1 {:.1} Tf {:.2} {:.2} Td ({}) Tj ET\n",
                    size_pt, x, y, escape_pdf_string(&block.text)
                ));
                y -= line_height + 4.0;
            }
            BlockKind::Blank => {
                y -= line_height * 0.6;
            }
            BlockKind::Paragraph => {
                content.push_str(&format!(
                    "BT /F2 11 Tf {:.2} {:.2} Td ({}) Tj ET\n",
                    x, y, escape_pdf_string(&block.text)
                ));
                y -= line_height;
            }
        }

        if y < 54.0 {
            // Page break
            content.push_str("showpage\n");
            y = page_h - 54.0;
        }
    }

    let content_len = content.len() as u32;
    let _page_count = content.split("showpage").count().max(1);

    // Build PDF objects.
    // Object 1: Catalog
    // Object 2: Pages
    // Object 3: Page(s) - we use one page for simplicity (content already has showpage handling,
    // but for a valid multi-page we'd need multiple pages; here we put everything on one page)
    // Object 4: Font F1 (Helvetica-Bold)
    // Object 5: Font F2 (Helvetica)
    // Object 6: Content stream

    let mut objects = Vec::new();

    // Obj 1: Catalog
    objects.push(format!("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n"));

    // Obj 2: Pages
    objects.push(format!(
        "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n"
    ));

    // Obj 3: Page
    let mediabox = format!("[0 0 {:.2} {:.2}]", page_w, page_h);
    objects.push(format!(
        "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox {mediabox} /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 6 0 R >>\nendobj\n"
    ));

    // Obj 4: Helvetica-Bold
    objects.push(
        "4 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n"
            .to_string(),
    );
    // Obj 5: Helvetica
    objects.push(
        "5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n"
            .to_string(),
    );
    // Obj 6: Content stream
    objects.push(format!(
        "6 0 obj\n<< /Length {content_len} >>\nstream\n{content}endstream\nendobj\n"
    ));

    // Build xref.
    let mut xref_offsets: Vec<u64> = Vec::new();
    let mut pdf_body = String::from("%PDF-1.4\n");
    // % comment line to ensure offset accuracy
    pdf_body.push_str("%\x00\x01\x02\x03\n");

    for obj in &objects {
        xref_offsets.push(pdf_body.len() as u64);
        pdf_body.push_str(obj);
    }

    let xref_start = pdf_body.len() as u64;
    let num_objects = xref_offsets.len() as u32 + 1; // +1 for object 0

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
}
