#!/bin/bash
# install-all-skills.sh — Install ALL Magpie skills automatically
#
# Installs:
#   • All built-in Magpie skills
#   • Popular marketing skills from skill.sh
#   • Sets up knowledge folder with sample documents
#   • Installs fetchright for brand analysis
#
# Usage: ./install-all-skills.sh

set -e

echo "╔═════════════════════════════════════════════════════════════╗"
echo "║  Magpie — Install All Skills                                ║"
echo "╚═════════════════════════════════════════════════════════════╝"
echo ""

# ---------------------------------------------------------
# 1. Install fetchright (needed for brand analysis)
# ---------------------------------------------------------
echo "📦 Installing fetchright (for brand analysis screenshots)..."
if command -v npm &> /dev/null; then
    npm install -g fetchright 2>/dev/null && echo "   ✅ fetchright installed" || echo "   ⚠️  fetchright install failed (you can install manually: npm install -g fetchright)"
else
    echo "   ⚠️  npm not found — install Node.js first, then: npm install -g fetchright"
fi
echo ""

# ---------------------------------------------------------
# 2. Install all built-in Magpie skills
# ---------------------------------------------------------
echo "📦 Installing built-in Magpie skills..."

BUILTIN_SKILLS=(
    "launch-email"
    "landing-page-copy"
    "ad-copy"
    "seo-blog-post"
    "brand-voice"
    "print"
    "crawl"
    "web-search"
    "brand-analysis"
    "rag"
)

for skill in "${BUILTIN_SKILLS[@]}"; do
    echo -n "   Installing $skill... "
    if magpie skills --install "$skill" --yes 2>/dev/null; then
        echo "✅"
    else
        echo "⚠️  (may already be installed)"
    fi
done
echo ""

# ---------------------------------------------------------
# 3. Install popular marketing skills from skill.sh
# ---------------------------------------------------------
echo "📦 Installing popular marketing skills from skill.sh..."

SKILL_SH_SKILLS=(
    "react-best-practices"
    "tailwindcss"
    "copywriting"
    "seo-best-practices"
    "email-marketing"
    "social-media-strategy"
    "landing-page-optimization"
    "content-strategy"
    "ppc-ad-copy"
    "conversion-rate-optimization"
)

for skill in "${SKILL_SH_SKILLS[@]}"; do
    echo -n "   Installing $skill from skill.sh... "
    if magpie skills --install "$skill" --source skill.sh --yes 2>/dev/null; then
        echo "✅"
    else
        echo "⚠️  (not found on skill.sh or already installed)"
    fi
done
echo ""

# ---------------------------------------------------------
# 4. Set up knowledge folder with sample documents
# ---------------------------------------------------------
echo "📁 Setting up knowledge folder..."

KNOWLEDGE_DIR="knowledge"
mkdir -p "$KNOWLEDGE_DIR"

# Company overview (if not already exists)
if [ ! -f "$KNOWLEDGE_DIR/company-overview.md" ]; then
    cat > "$KNOWLEDGE_DIR/company-overview.md" << 'EOF'
# [Your Company] — Company Overview

## About Us
[Describe your company here]

## Products & Services
[List your products and services]

## Pricing
[List your pricing tiers]

## Target Audience
[Describe your ideal customer]

## Brand Voice
[Tone, style, personality guidelines]

## Key Features
[List key features]
EOF
    echo "   ✅ Created knowledge/company-overview.md"
fi

# Brand voice (if not already exists)
if [ ! -f "$KNOWLEDGE_DIR/brand-voice.md" ]; then
    cat > "$KNOWLEDGE_DIR/brand-voice.md" << 'EOF'
# [Your Company] — Brand Voice

## Voice Personality
[Describe your brand personality]

## Do
- [Preferred words and phrases]
- [Communication style]

## Don't
- [Forbidden words and phrases]
- [What to avoid]

## Examples
### Before (generic)
[Generic example]

### After (your brand voice)
[Your brand voice example]
EOF
    echo "   ✅ Created knowledge/brand-voice.md"
fi
echo ""

# ---------------------------------------------------------
# 5. Verify installation
# ---------------------------------------------------------
echo "╔═════════════════════════════════════════════════════════════╗"
echo "║  Installation Complete                                       ║"
echo "╚═════════════════════════════════════════════════════════════╝"
echo ""
echo "📋 Installed skills:"
magpie skills 2>/dev/null || echo "   (run 'magpie skills' to list)"
echo ""
echo "📁 Knowledge folder: $(pwd)/knowledge/"
echo "   Add your company documents here (.md, .txt, .pdf)"
echo ""
echo "🚀 Next steps:"
echo "   1. Set your API key:  export MAGPIE_API_KEY=sk-or-v1-your-key"
echo "   2. Ingest knowledge:  magpie run 'rag ingest'"
echo "   3. Start marketing:   magpie run 'write a product launch email'"
echo "   4. Web interface:     magpie web  (then open http://127.0.0.1:7070)"
echo ""
