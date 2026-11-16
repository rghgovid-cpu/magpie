---
name: Brand Analysis
description: Analyze company websites with a vision model to extract logos, fonts, colors, and brand identity for marketing use.
tags: brand, vision, website, logo, fonts, colors, identity, analysis, screenshot
---

# Brand Analysis Skill

Use this skill to analyze a company website and extract its brand identity — logos, typography, color palette, and visual style — using a vision-capable AI model.

## What it extracts

| Element | Description |
|---------|-------------|
| **Logo** | Description, placement, style, wordmark vs icon |
| **Colors** | Primary, secondary, accent colors (hex values) |
| **Fonts** | Headings font, body font, font weights |
| **Visual style** | Photography style, iconography, layout patterns |
| **Brand personality** | Tone, mood, positioning from visual design |

## Output formats

| Format | Extension | Best For |
|--------|-----------|----------|
| CSS Variables | `.css` | Direct use in web design, landing pages |
| JSON | `.json` | API integration, design systems |
| Markdown | `.md` | Brand guidelines documentation |

## Example CSS output

```css
:root {
  --brand-primary: #10b981;
  --brand-secondary: #059669;
  --brand-accent: #3b82f6;
  --brand-background: #ffffff;
  --brand-text: #0f172a;
  --brand-text-muted: #64748b;
  --font-heading: 'Inter', sans-serif;
  --font-body: 'Inter', sans-serif;
  --font-heading-weight: 700;
  --font-body-weight: 400;
}

.brand-logo {
  description: "Green wordmark with leaf icon, top-left placement";
}
```

## Example workflow

```bash
# 1. Analyze a competitor website
magpie run "brand-analyze https://competitor.com --format css --out competitor-brand.css"

# 2. Use the extracted brand in your marketing
magpie run "write landing page copy using this brand: $(cat competitor-brand.css)"

# 3. Print to PDF for brand guidelines
magpie print brand-guidelines.md --format pdf --size a4 --out brand-guidelines.pdf
```

## Requirements

- **Vision-capable LLM**: Nemotron-3.5-Super-VL-Free, GPT-4V, Claude 3, etc.
- **Playwright** (Python): For website screenshots (`pip install playwright && playwright install chromium`)
- **API key**: Set `MAGPIE_API_KEY` or `OPENAI_API_KEY`

## Commands

### Analyze a website (CSS output)
```bash
magpie run "brand-analyze https://example.com --format css --out brand.css"
```

### Analyze and output JSON
```bash
magpie run "brand-analyze https://example.com --format json --out brand.json"
```

### Analyze multiple pages
```bash
magpie run "brand-analyze https://example.com https://example.com/about --format css --out brand.css"
```

## How it works

1. Takes screenshots of website pages using Playwright (headless Chromium)
2. Sends screenshots to a vision-capable LLM
3. LLM extracts colors, fonts, logo, and visual style
4. Outputs structured CSS, JSON, or Markdown

## Notes

- Works best on well-designed websites with clear visual hierarchy
- Colors are extracted as hex values from the vision model's analysis
- Font names are best guesses based on visual appearance
- Combine with RAG skill for complete brand guidelines
