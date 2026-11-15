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

## Commands

### Crawl a single URL to CSV (CRM-ready)
```bash
magpie run "crawl https://example.com --format csv --out leads.csv"
```

### Crawl multiple URLs to CSV
```bash
magpie run "crawl https://site1.com https://site2.com --format csv --out leads.csv"
```

### Crawl to JSON (for API integration)
```bash
magpie run "crawl https://example.com --format json --out leads.json"
```

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

- Respects `robots.txt` where possible
- Rate-limited to 1 request per second per domain
- Extracts mailto: and tel: links automatically
- Cleans HTML tags from text fields
- Truncates long descriptions to 500 characters for CRM compatibility
