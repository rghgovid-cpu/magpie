---
name: Web Crawler
description: Crawl websites and export structured data (CSV/JSON) ready for CRM import. Extracts title, description, contacts, links, and metadata.
tags: crawl, web, crm, csv, json, extract, leads, scrape
---

# Web Crawler Skill

Use this skill to crawl websites and export structured data that is ready for CRM import (HubSpot, Salesforce, Pipedrive, Zoho, etc.).

## Output Formats

| Format | Extension | Best For |
|--------|-----------|----------|
| CSV | `.csv` | Direct CRM import (HubSpot, Salesforce, Pipedrive) |
| JSON | `.json` | API-based CRM integrations, custom scripts |

## CRM Fields Extracted

| Field | Description |
|-------|-------------|
| `url` | Source page URL |
| `title` | Page title |
| `company` | Detected company/organization name |
| `description` | Meta description or page summary |
| `email` | Email addresses found on page |
| `phone` | Phone numbers found on page |
| `address` | Physical address (if present) |
| `contact_name` | Person name (if detectable) |
| `links` | All outbound links |
| `crawl_date` | ISO 8601 timestamp |

## Responsible Crawling

We are nice crawlers. Magpie is designed to be respectful to the websites it visits:

- **Respects `robots.txt`**: Checks `Disallow` rules and `Crawl-delay` directives before fetching any page. If a site disallows crawling, Magpie skips it.
- **Rate-limited**: Adds a delay between requests (default: 1 second, or the `Crawl-delay` from `robots.txt` if specified). Use `--delay N` to override.
- **Transparent identity**: Identifies itself with a clear `User-Agent` string including a contact URL.
- **Daily quota**: Limited to 200 records per day by default to prevent accidental overload.

## Commands

### Crawl a single URL to CSV (CRM-ready)
```bash
magpie run "crawl https://example.com --format csv --out leads.csv"
```

### Crawl multiple URLs to CSV
```bash
magpie run "crawl https://site1.com https://site2.com --format csv --out leads.csv"
```

### Crawl with a daily record limit override
```bash
magpie run "crawl https://example.com --format csv --out leads.csv --limit 50"
```

### Crawl with a custom delay between requests
```bash
magpie run "crawl https://site1.com https://site2.com --delay 2.0"
```

### Crawl to JSON (for API integration)
```bash
magpie run "crawl https://example.com --format json --out leads.json"
```

## Daily Crawl Limit

To protect against excessive crawling, Magpie enforces a **daily limit of 200 records** by default. The count is stored locally in `~/.config/magpie/crawl_state.json` and resets at **midnight local time**.

- The limit is checked before each URL is crawled. Once reached, remaining URLs are skipped.
- Use `--limit N` to override for a single run (e.g. `--limit 50`).
- Delete `~/.config/magpie/crawl_state.json` to reset the counter manually.

## CRM Import Instructions

### HubSpot
1. Save output as `leads.csv`
2. Go to HubSpot → Contacts → Import
3. Map fields: Email → Email, Company → Company Name, etc.

### Salesforce
1. Save output as `leads.csv`
2. Use Data Import Wizard or Data Loader
3. Map to Lead or Account object

### Pipedrive
1. Save output as `leads.csv`
2. Go to → Tools → Import data
3. Select "People" or "Organizations"

## Example CSV Output

```csv
url,title,company,description,email,phone,address,contact_name,links,crawl_date
https://example.com,Example Corp,Example Corp,"A leading marketing agency",info@example.com,+1-555-0100,"123 Main St, NYC",John Doe,"https://example.com/about,https://example.com/contact",2026-11-15T10:30:00Z
```

## Notes

- Respects `robots.txt` (Disallow rules and Crawl-delay directives)
- Rate-limited: 1 second delay between requests by default (use `--delay N` to override)
- Identifies itself with a transparent User-Agent including contact info
- Daily quota: 200 records per day (resets at midnight local time)
- Extracts mailto: and tel: links automatically
- Cleans HTML tags from text fields
- Truncates long descriptions to 500 characters for CRM compatibility
