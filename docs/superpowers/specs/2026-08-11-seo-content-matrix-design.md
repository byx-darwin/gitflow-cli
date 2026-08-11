# SEO Content Matrix Design

Date: 2026-08-11
Status: Pending Review
Issue: #99
Workflow: wf-2026-08-11-002

## Background

Website deployed at https://byx-darwin.github.io/gitflow-cli/ with GEO foundation complete (llms.txt, robots.txt, sitemap, JSON-LD). This issue focuses on technical SEO optimization and establishing content distribution workflow.

## Scope

1. Technical SEO optimization (code changes)
2. Search engine submission guide (Google/Bing/Baidu)
3. Content matrix guide (Juejin/Zhihu/WeChat/Xiaohongshu/Douyin)

## Deliverables

| # | Deliverable | Type | Location |
|---|-------------|------|----------|
| 1 | Technical SEO PR | Code | website/ |
| 2 | Search engine submission guide | Doc | docs/operations/search-engine-submission-guide.md |
| 3 | Content matrix guide | Doc | docs/operations/content-matrix-guide.md |
| 4 | OG image | Asset | website/public/og-image.png |
| 5 | Issue comment | Summary | Issue #99 |

## Technical SEO Optimization

### Open Graph / Twitter Card

Add to Base.astro head section:
- og:type, og:url, og:title, og:description, og:image
- twitter:card, twitter:title, twitter:description, twitter:image
- Create og-image.png (1200x630px)

### Performance Optimization

- Font preloading (3 custom fonts)
- Lighthouse audit and optimization
- Target: Performance >= 90, LCP < 1s

### Semantic HTML Enhancement

- Heading hierarchy review (single h1 per page)
- Image alt text completion
- Aria labels for navigation and main content
- Target: Accessibility >= 90

### Internal Linking

- Related page links (workflow -> architecture, quickstart)
- Optional: breadcrumb navigation
- Target: Each page has 3+ internal links

## Search Engine Submission Guide

Structure:
1. Google Search Console - registration, verification, sitemap submission
2. Bing Webmaster Tools - registration, verification, sitemap submission
3. Baidu Webmaster - registration, verification, sitemap submission
4. Verification checklist
5. Common issues
6. Maintenance schedule

## Content Matrix Guide

### Platform Setup (Phased)

Phase 1 (Current month - Text/Image):
- Juejin: account, creator rights, column setup
- Zhihu: account, column creation
- WeChat Official Account: subscription account registration
- Xiaohongshu: account, professional setup

Phase 2 (Next month - Video pilot):
- Douyin: account setup, video production workflow

### Content Production Workflow

1. Write on website first (canonical source)
2. Distribute to Juejin/Zhihu/WeChat (Day 3-5)
3. Create Xiaohongshu image cards from article
4. Douyin: screen recording + AI voiceover (Phase 2)

### Content Types

- 40% Tutorials (quickstart, command guides)
- 30% Case studies (dogfooding examples)
- 20% Philosophy (AI workflow concepts)
- 10% Updates (release notes)

### Initial Topic Pool (6 articles)

1. 5-minute gf quickstart
2. Dogfooding case study (building gf with gf)
3. Why AI workflow engineering matters
4. gf v1.0.0 release announcement
5. Workflow deep dive (requirement to release)
6. gf vs gh vs glab comparison

### Publishing Cadence

- 2 articles per month
- Week 1-2: First article cycle
- Week 3-4: Second article cycle

### Douyin Video Strategy (Phase 2)

Format: 30s-60s screen recordings with AI voiceover
Tools: OBS for recording, Jianying for editing
Automation: AI generates script, voiceover, captions
Human work: 15-20 min per video (screen recording only)

Initial video topics:
1. 30s: gf issue list cross-platform demo
2. 60s: AI workflow from requirement to release
3. gf vs gh comparison
4. Dogfooding experience

## Success Criteria

Technical:
- OG/Twitter Card tags on all pages
- Lighthouse Performance >= 90
- Lighthouse Accessibility >= 90
- Lighthouse SEO >= 90
- LCP < 1s

Documentation:
- Search engine submission guide complete
- Content matrix guide complete
- Initial 6 topics selected

Exit criteria:
- Technical SEO PR merged
- Operation guides delivered
- User can follow guides to complete account setup
