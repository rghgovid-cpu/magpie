---
name: RAG
description: Retrieval-Augmented Generation for company-specific knowledge. Ingest company documents and use them to refine marketing outputs with accurate, on-brand context.
tags: rag, retrieval, knowledge, company, context, refine, marketing, documents
---

# RAG Skill — Company Knowledge for Marketing

Use this skill to ground Magpie's marketing outputs in your company's actual knowledge.
Drop documents into the `knowledge/` folder and Magpie will retrieve relevant context
before generating copy, ensuring accuracy, brand voice consistency, and product-specific
details.

## What it does

1. **Ingest** documents from `knowledge/` (markdown, text, PDF)
2. **Chunk** documents into searchable passages
3. **Retrieve** the most relevant passages for a given query
4. **Inject** that context into Magpie's LLM prompts automatically

## Supported file types

| Extension | Description |
|-----------|-------------|
| `.md` | Markdown documents |
| `.txt` | Plain text files |
| `.pdf` | PDF documents (requires `pypdf` or `PyPDF2`) |

## Directory structure

```
magpie-rs/
├── knowledge/                  # Drop company documents here
│   ├── company-overview.md
│   ├── product-pricing.txt
│   ├── brand-voice.md
│   └── competitor-analysis.pdf
└── skills/
    └── rag/
        ├── SKILL.md
        ├── magpie-rag.py
        └── magpie-rag-helper.sh
```

## Commands

### Ingest all documents in knowledge/
```bash
magpie run "rag ingest"
```

### Search company knowledge
```bash
magpie run "rag search 'pricing tiers'"
magpie run "rag search 'brand voice guidelines'"
magpie run "rag search 'target audience'"
```

### Ingest and search in one step
```bash
magpie run "rag ingest-and-search 'enterprise pricing'"
```

## Example workflow

```bash
# 1. Add company documents
echo "# Acme Corp - Product Overview" > knowledge/company.md
echo "We sell AI marketing software for SMBs. Pricing: $49/mo starter, $149/mo pro, $499/mo enterprise." >> knowledge/company.md
echo "Our brand voice is helpful, witty, and direct. No jargon." >> knowledge/brand.md

# 2. Ingest
magpie run "rag ingest"

# 3. Magpie now knows about your company
magpie run "write a product launch email"
# → Magpie will use your pricing, brand voice, and product details automatically
```

## How RAG refines marketing

Without RAG: Generic marketing copy that could apply to any company.

With RAG: Copy that includes your actual pricing, product features, brand voice,
target audience, and competitive positioning — because Magpie retrieves your
company documents before generating.

## Notes

- Documents are indexed locally (no external API calls for retrieval)
- Re-run `rag ingest` whenever you add or update documents
- Top 5 most relevant passages are injected into each marketing request
- Works with all Magpie commands: emails, ads, landing pages, SEO, etc.
