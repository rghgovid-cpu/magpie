---
name: Web Search
description: Search the web and return structured results (title, URL, snippet) that can be fed into the crawler skill for CRM extraction.
tags: search, web, google, duckduckgo, query, results, find
---

# Web Search Skill

Use this skill to search the web and get structured results. Results include title, URL, and snippet — perfect for feeding into the Web Crawler skill for CRM lead extraction.

## Search Engine

Uses DuckDuckGo HTML (free, no API key required).

## Output Fields

| Field | Description |
|-------|-------------|
| `query` | The search query used |
| `rank` | Result position (1, 2, 3...) |
| `title` | Page title from search result |
| `url` | Result URL |
| `snippet` | Search result description/snippet |
| `source` | Search engine used (duckduckgo) |

## Commands

### Search the web (CSV output)
```bash
magpie run "search for marketing agencies in New York --format csv --out marketing-agencies.csv"
```

### Search the web (JSON output)
```bash
magpie run "search for SEO consultants London --format json --out seo-consultants.json"
```

### Search and limit results
```bash
magpie run "search for PPC agencies --format csv --out ppc.csv --limit 10"
```

## Example Workflow: Search → Crawl → CRM

```bash
# Step 1: Search for marketing agencies
magpie run "search for digital marketing agencies in Chicago --format csv --out search-results.csv"

# Step 2: Crawl the top results from the search
magpie run "crawl https://agency1.com https://agency2.com --format csv --out agency-leads.csv"

# Step 3: Import agency-leads.csv into your CRM (HubSpot, Salesforce, etc.)
```

## Example CSV Output

```csv
query,rank,title,url,snippet,source
digital marketing agencies chicago,1,"WebFX - Digital Marketing Agency","https://www.webfx.com/","Award-winning digital marketing agency in Chicago...",duckduckgo
digital marketing agencies chicago,2,"Straight North - Chicago SEO","https://www.straightnorth.com/","Chicago-based SEO and digital marketing firm...",duckduckgo
```

## Notes

- No API key required
- Results are limited to 10 by default (use `--limit` to change)
- Snippets are truncated to 300 characters
- Works well combined with the Web Crawler skill for lead generation
