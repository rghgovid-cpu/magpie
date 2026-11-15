---
name: Print
description: Print documents to PDF or Word in A4 and US Letter sizes. No external programs required.
tags: print, pdf, word, docx, a4, letter, export
---

# Print Skill

Use this skill when the user wants to print or export a document to PDF or Word format.

## Available Commands

### Print to PDF (A4)
```
magpie print <file.md> --format pdf --size a4 --out <output.pdf>
```

### Print to PDF (US Letter)
```
magpie print <file.md> --format pdf --size letter --out <output.pdf>
```

### Print to Word (.docx, A4)
```
magpie print <file.md> --format word --size a4 --out <output.docx>
```

### Print to Word (.docx, US Letter)
```
magpie print <file.md> --format word --size letter --out <output.docx>
```

## Command Reference

| Argument | Values | Default | Description |
|----------|--------|---------|-------------|
| `--format` | `pdf`, `word` | `pdf` | Output format |
| `--size` | `a4`, `letter` | `a4` | Page size |
| `--out` | `<filename>` | `<input>.pdf` or `<input>.docx` | Output file path |

## Supported Formats

- **PDF**: A4 (595.28 × 841.89 pts) and US Letter (612 × 792 pts)
- **Word (.docx)**: A4 (11906 × 16838 dxa) and US Letter (12240 × 15840 dxa)

## Example Workflow

1. Generate content with another skill (e.g. Launch Email)
2. Save to a markdown file: `launch.md`
3. Print to PDF: `magpie print launch.md --format pdf --size a4 --out launch.pdf`
4. Print to Word: `magpie print launch.md --format word --size letter --out launch.docx`

## Notes

- Input files can be markdown or plain text
- Headings (#, ##, ###) are rendered as styled headings
- No external programs (pandoc, LibreOffice, etc.) are required
- Output files are self-contained and portable
