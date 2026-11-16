#!/usr/bin/env python3
"""
magpie-brand-analysis.py — Analyze company websites with a vision model.

Extracts brand identity (logos, fonts, colors, visual style) from website
screenshots using a vision-capable LLM. Outputs CSS variables, JSON, or Markdown.

Usage:
    python3 magpie-brand-analysis.py <url> [--format css|json|md] [--out <file>]
    python3 magpie-brand-analysis.py <url1> <url2> ... [--format css] [--out brand.css]

Requirements:
    pip install playwright
    playwright install chromium
    MAGPIE_API_KEY or OPENAI_API_KEY environment variable
    Vision-capable model (Nemotron-3.5-Super-VL-Free, GPT-4V, Claude 3, etc.)
"""

import sys
import os
import re
import json
import base64
import argparse
from pathlib import Path
from datetime import datetime, timezone

try:
    import requests
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False

try:
    from playwright.sync_api import sync_playwright
    HAS_PLAYWRIGHT = True
except ImportError:
    HAS_PLAYWRIGHT = False


def take_screenshot(url, output_path=None, width=1280, height=800):
    """Take a screenshot of a webpage using Playwright."""
    if not HAS_PLAYWRIGHT:
        print(f"Error: Playwright not installed. Run: pip install playwright && playwright install chromium", file=sys.stderr)
        return None

    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            page = browser.new_page(viewport={"width": width, "height": height})
            page.goto(url, wait_until="networkidle", timeout=30000)
            # Wait a bit for fonts and images to load
            page.wait_for_timeout(2000)
            screenshot_bytes = page.screenshot(full_page=False)
            browser.close()

        if output_path:
            with open(output_path, "wb") as f:
                f.write(screenshot_bytes)

        # Return base64 encoded
        return base64.b64encode(screenshot_bytes).decode("utf-8")
    except Exception as e:
        print(f"Error taking screenshot of {url}: {e}", file=sys.stderr)
        return None


def analyze_with_vision(image_b64, url, model, api_key, base_url):
    """Send screenshot to vision LLM and extract brand identity."""
    if not HAS_REQUESTS:
        print("Error: requests library not installed.", file=sys.stderr)
        return None

    # Vision prompt for brand analysis
    system_prompt = """You are an expert brand identity designer and visual analyst.
Analyze the website screenshot and extract the complete brand identity.

Return your analysis as a JSON object with these exact keys:
- logo: description of the logo (wordmark, icon, placement, colors, style)
- primary_color: hex code of the main brand color
- secondary_color: hex code of secondary brand color
- accent_color: hex code of accent color
- background_color: hex code of main background
- text_color: hex code of main text color
- text_muted_color: hex code of muted/secondary text
- heading_font: name of the font used for headings (best guess)
- body_font: name of the font used for body text (best guess)
- heading_weight: font weight for headings (e.g., 700, 600)
- body_weight: font weight for body text (e.g., 400, 500)
- visual_style: description of photography, iconography, layout style
- brand_personality: tone and mood conveyed by visual design
- css_variables: valid CSS :root variables with all extracted colors and fonts

Be precise with hex codes. If a color is not clearly visible, use your best estimate.
If fonts cannot be determined, use system font names like 'Inter', 'system-ui', 'sans-serif'."""

    user_content = [
        {
            "type": "text",
            "text": f"Analyze this website screenshot for brand identity. URL: {url}"
        },
        {
            "type": "image_url",
            "image_url": {
                "url": f"data:image/png;base64,{image_b64}"
            }
        }
    ]

    headers = {
        "Authorization": f"Bearer {api_key}",
        "Content-Type": "application/json",
    }

    payload = {
        "model": model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_content}
        ],
        "temperature": 0.3,
        "max_tokens": 2000,
    }

    try:
        resp = requests.post(
            f"{base_url}/chat/completions",
            headers=headers,
            json=payload,
            timeout=60,
        )
        resp.raise_for_status()
        data = resp.json()
        content = data["choices"][0]["message"]["content"]
        return content
    except Exception as e:
        print(f"Error calling vision API: {e}", file=sys.stderr)
        return None


def extract_json_from_response(text):
    """Try to extract a JSON object from the LLM response."""
    # Try to find JSON block
    json_match = re.search(r'\{.*\}', text, re.DOTALL)
    if json_match:
        try:
            return json.loads(json_match.group(0))
        except json.JSONDecodeError:
            pass
    return None


def format_css(brand_data):
    """Format brand data as CSS variables."""
    css = ":root {\n"
    css += f"  --brand-primary: {brand_data.get('primary_color', '#10b981')};\n"
    css += f"  --brand-secondary: {brand_data.get('secondary_color', '#059669')};\n"
    css += f"  --brand-accent: {brand_data.get('accent_color', '#3b82f6')};\n"
    css += f"  --brand-background: {brand_data.get('background_color', '#ffffff')};\n"
    css += f"  --brand-text: {brand_data.get('text_color', '#0f172a')};\n"
    css += f"  --brand-text-muted: {brand_data.get('text_muted_color', '#64748b')};\n"
    css += f"  --font-heading: '{brand_data.get('heading_font', 'Inter')}', sans-serif;\n"
    css += f"  --font-body: '{brand_data.get('body_font', 'Inter')}', sans-serif;\n"
    css += f"  --font-heading-weight: {brand_data.get('heading_weight', '700')};\n"
    css += f"  --font-body-weight: {brand_data.get('body_weight', '400')};\n"
    css += "}\n\n"
    css += "/* Logo description */\n"
    css += f"/* {brand_data.get('logo', 'Not detected')} */\n\n"
    css += "/* Visual style */\n"
    css += f"/* {brand_data.get('visual_style', 'Not described')} */\n\n"
    css += "/* Brand personality */\n"
    css += f"/* {brand_data.get('brand_personality', 'Not described')} */\n"
    return css


def format_markdown(brand_data, url):
    """Format brand data as Markdown."""
    md = f"# Brand Analysis: {url}\n\n"
    md += f"Generated: {datetime.now(timezone.utc).isoformat()}\n\n"
    md += "## Logo\n\n"
    md += f"{brand_data.get('logo', 'Not detected')}\n\n"
    md += "## Colors\n\n"
    md += f"- **Primary**: {brand_data.get('primary_color', 'Not detected')}\n"
    md += f"- **Secondary**: {brand_data.get('secondary_color', 'Not detected')}\n"
    md += f"- **Accent**: {brand_data.get('accent_color', 'Not detected')}\n"
    md += f"- **Background**: {brand_data.get('background_color', 'Not detected')}\n"
    md += f"- **Text**: {brand_data.get('text_color', 'Not detected')}\n"
    md += f"- **Text Muted**: {brand_data.get('text_muted_color', 'Not detected')}\n\n"
    md += "## Typography\n\n"
    md += f"- **Headings**: {brand_data.get('heading_font', 'Not detected')} (weight: {brand_data.get('heading_weight', 'Not detected')})\n"
    md += f"- **Body**: {brand_data.get('body_font', 'Not detected')} (weight: {brand_data.get('body_weight', 'Not detected')})\n\n"
    md += "## Visual Style\n\n"
    md += f"{brand_data.get('visual_style', 'Not described')}\n\n"
    md += "## Brand Personality\n\n"
    md += f"{brand_data.get('brand_personality', 'Not described')}\n\n"
    md += "## CSS Variables\n\n"
    md += "```css\n"
    md += format_css(brand_data)
    md += "```\n"
    return md


def analyze_website(url, model, api_key, base_url, output_format="css", output_file=None):
    """Analyze a single website and return brand data."""
    print(f"Taking screenshot of {url}...", file=sys.stderr)
    image_b64 = take_screenshot(url)
    if not image_b64:
        return None

    print(f"Analyzing with vision model ({model})...", file=sys.stderr)
    response = analyze_with_vision(image_b64, url, model, api_key, base_url)
    if not response:
        return None

    # Try to parse JSON from response
    brand_data = extract_json_from_response(response)
    if not brand_data:
        # If JSON parsing fails, use the raw response
        brand_data = {
            "raw_response": response,
            "url": url,
        }

    brand_data["url"] = url
    brand_data["analyzed_at"] = datetime.now(timezone.utc).isoformat()

    # Format output
    if output_format == "css":
        output = format_css(brand_data)
    elif output_format == "md":
        output = format_markdown(brand_data, url)
    else:  # json
        output = json.dumps(brand_data, indent=2, ensure_ascii=False)

    # Write to file if specified
    if output_file:
        with open(output_file, "w", encoding="utf-8") as f:
            f.write(output)
        print(f"Brand analysis written to {output_file}", file=sys.stderr)

    return output


def main():
    parser = argparse.ArgumentParser(
        description="Magpie Brand Analysis — extract brand identity from websites using vision AI"
    )
    parser.add_argument("urls", nargs="+", help="Website URL(s) to analyze")
    parser.add_argument(
        "--format",
        choices=["css", "json", "md"],
        default="css",
        help="Output format (default: css)",
    )
    parser.add_argument(
        "--out",
        default=None,
        help="Output file (default: brand.css or brand.json or brand.md)",
    )
    parser.add_argument(
        "--model",
        default=None,
        help="Vision model to use (overrides config)",
    )
    args = parser.parse_args()

    # Get API configuration
    api_key = os.environ.get("MAGPIE_API_KEY") or os.environ.get("OPENAI_API_KEY")
    if not api_key:
        print("Error: MAGPIE_API_KEY or OPENAI_API_KEY environment variable required.", file=sys.stderr)
        sys.exit(1)

    base_url = os.environ.get("MAGPIE_BASE_URL", "https://api.openai.com/v1")
    model = args.model or os.environ.get("MAGPIE_MODEL", "nemotron-3.5-super-vl-free")

    # Check Playwright
    if not HAS_PLAYWRIGHT:
        print("Error: Playwright not installed.", file=sys.stderr)
        print("Install: pip install playwright && playwright install chromium", file=sys.stderr)
        sys.exit(1)

    # Analyze each URL
    results = []
    for url in args.urls:
        # Ensure URL has scheme
        if not url.startswith(("http://", "https://")):
            url = "https://" + url

        output_file = args.out
        if len(args.urls) > 1 and output_file:
            # Multiple URLs, create individual output files
            from urllib.parse import urlparse
            domain = urlparse(url).netloc.replace(".", "_")
            output_file = f"brand-{domain}.{args.format}"

        result = analyze_website(url, model, api_key, base_url, args.format, output_file)
        if result:
            results.append(result)
            print("\n" + "="*60 + "\n", file=sys.stderr)

    # If single URL and no output file, print to stdout
    if len(args.urls) == 1 and not args.out:
        print(results[0] if results else "")


if __name__ == "__main__":
    main()
