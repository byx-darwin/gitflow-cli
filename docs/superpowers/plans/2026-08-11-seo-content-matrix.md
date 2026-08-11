# SEO Content Matrix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement technical SEO optimizations and create operation guides for search engine submission and content distribution across Chinese platforms.

**Architecture:** Static Astro website with existing GEO foundation (llms.txt, sitemap, JSON-LD). Add Open Graph/Twitter Card tags, optimize performance and semantic HTML, enhance internal linking. Create comprehensive operation guides for search engine submission (Google/Bing/Baidu) and content platform setup (Juejin/Zhihu/WeChat/Xiaohongshu/Douyin).

**Tech Stack:** Astro 5.1, HTML5, Markdown, PNG (OG image)

## Global Constraints

- Target Lighthouse Performance >= 90
- Target Lighthouse Accessibility >= 90
- Target Lighthouse SEO >= 90
- LCP (Largest Contentful Paint) < 1 second
- OG image dimensions: 1200x630px
- All pages must have single h1 heading
- Each page must have at least 3 internal links
- Documentation must be in Chinese (中文)
- All code changes must pass `make lint` and `make test`

---

## File Structure

### New Files
- `website/public/og-image.png` — Social sharing preview image (1200x630px)
- `docs/operations/search-engine-submission-guide.md` — Step-by-step guide for Google/Bing/Baidu submission
- `docs/operations/content-matrix-guide.md` — Platform setup and content production workflow

### Modified Files
- `website/src/layouts/Base.astro` — Add OG/Twitter Card meta tags, font preloading, aria labels
- `website/src/pages/index.astro` — Review heading hierarchy, add internal links
- `website/src/pages/workflow.astro` — Add related page links
- `website/src/pages/compare.astro` — Add related page links
- `website/src/pages/quickstart.mdx` — Review heading hierarchy, add internal links
- `website/src/pages/architecture.astro` — Add related page links
- All other page files — Semantic HTML review and internal linking

---

## Task 1: Create OG Image Asset

**Files:**
- Create: `website/public/og-image.png`

**Interfaces:**
- Consumes: Brand colors, logo, positioning from existing website
- Produces: 1200x630px PNG image for social sharing

### Description

Create a social sharing preview image that displays when the website is shared on social media platforms (Twitter, Facebook, LinkedIn, WeChat, etc.).

### Requirements

- Dimensions: 1200x630px (1.91:1 ratio, Facebook/Twitter standard)
- Format: PNG
- Include: Brand name "gf", tagline, logo/brandmark
- Colors: Match website theme (dark background, orange accent)
- Text: "gf — 跨平台 Git 工程化工作流编排框架"
- File size: < 200KB (optimized for web)

### Steps

- [ ] **Step 1: Design the image**

Use a design tool (Figma, Sketch, or similar) to create the image with:
- Dark background (#1a1a1a or similar)
- Brand mark "❯_" in orange (#ff6b35 or similar)
- Title: "gf"
- Subtitle: "跨平台 Git 工程化工作流编排框架"
- Optional: GitHub URL "github.com/byx-darwin/gitflow-cli"

- [ ] **Step 2: Export as PNG**

Export the design as `og-image.png` with:
- Dimensions: 1200x630px
- Format: PNG-24
- Optimize for web (compress to < 200KB)

- [ ] **Step 3: Save to public directory**

```bash
cp /path/to/og-image.png website/public/og-image.png
```

- [ ] **Step 4: Verify file exists**

```bash
ls -lh website/public/og-image.png
file website/public/og-image.png
```

Expected: File exists, PNG format, ~100-200KB

- [ ] **Step 5: Commit**

```bash
git add website/public/og-image.png
git commit -m "feat(website): add OG image for social sharing"
```

---

## Task 2: Add Open Graph and Twitter Card Tags

**Files:**
- Modify: `website/src/layouts/Base.astro:1-40`

**Interfaces:**
- Consumes: `og-image.png` from Task 1, page title and description props
- Produces: OG and Twitter Card meta tags in HTML head

### Description

Add Open Graph and Twitter Card meta tags to enable rich social sharing previews.

### Requirements

- Add OG tags: og:type, og:url, og:title, og:description, og:image, og:locale
- Add Twitter Card tags: twitter:card, twitter:title, twitter:description, twitter:image
- Use existing `title` and `description` props from Base.astro
- Reference `/gitflow-cli/og-image.png` for og:image and twitter:image
- Set og:locale to "zh_CN"

### Steps

- [ ] **Step 1: Write failing test (manual verification)**

Open the built site in browser and inspect HTML head. Since this is static HTML, we'll verify manually.

- [ ] **Step 2: Read current Base.astro**

```bash
cat website/src/layouts/Base.astro | head -40
```

Note the current structure and where to add tags.

- [ ] **Step 3: Add OG and Twitter Card meta tags**

Edit `website/src/layouts/Base.astro` and add after the existing meta description tag (around line 30):

```astro
    <meta name="description" content={description} />
    <link rel="canonical" href={Astro.url.href} />
    <link rel="icon" type="image/svg+xml" href={`${base}/favicon.svg`} />

    <!-- Open Graph -->
    <meta property="og:type" content="website" />
    <meta property="og:url" content={Astro.url.href} />
    <meta property="og:title" content={title} />
    <meta property="og:description" content={description} />
    <meta property="og:image" content={`${base}/og-image.png`} />
    <meta property="og:locale" content="zh_CN" />

    <!-- Twitter Card -->
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content={title} />
    <meta name="twitter:description" content={description} />
    <meta name="twitter:image" content={`${base}/og-image.png`} />
```

- [ ] **Step 4: Build and verify**

```bash
cd website
npm run build
```

Expected: Build succeeds without errors.

- [ ] **Step 5: Inspect generated HTML**

```bash
grep -A 10 "og:type" website/dist/index.html
```

Expected: OG tags present in output HTML.

- [ ] **Step 6: Commit**

```bash
git add website/src/layouts/Base.astro
git commit -m "feat(website): add Open Graph and Twitter Card meta tags"
```

---

## Task 3: Performance Optimization — Font Preloading

**Files:**
- Modify: `website/src/layouts/Base.astro:1-40`

**Interfaces:**
- Consumes: Font files from @fontsource packages
- Produces: Preload links in HTML head for faster font loading

### Description

Add font preloading to improve LCP (Largest Contentful Paint) performance.

### Requirements

- Preload the 3 custom fonts used in the website
- Use `rel="preload"` with `as="font"` and `crossorigin` attribute
- Fonts: Chakra Petch, IBM Plex Sans, JetBrains Mono
- Target: LCP < 1 second

### Steps

- [ ] **Step 1: Identify font files**

```bash
find website/node_modules/@fontsource -name "*.woff2" | head -10
```

Note the exact paths to the woff2 files for the 3 fonts.

- [ ] **Step 2: Check current font loading**

```bash
grep -r "font-face\|@fontsource" website/src/
```

Understand how fonts are currently loaded.

- [ ] **Step 3: Add preload links**

Edit `website/src/layouts/Base.astro` and add preload links in the head section:

```astro
    <link rel="canonical" href={Astro.url.href} />
    <link rel="icon" type="image/svg+xml" href={`${base}/favicon.svg`} />

    <!-- Font preloading -->
    <link rel="preload" as="font" type="font/woff2" href={`${base}/_astro/chakra-petch-latin-400-normal.woff2`} crossorigin />
    <link rel="preload" as="font" type="font/woff2" href={`${base}/_astro/ibm-plex-sans-latin-400-normal.woff2`} crossorigin />
    <link rel="preload" as="font" type="font/woff2" href={`${base}/_astro/jetbrains-mono-latin-400-normal.woff2`} crossorigin />
```

Note: The exact paths depend on Astro's build output. Check `website/dist/_astro/` after build to get correct filenames.

- [ ] **Step 4: Build and verify**

```bash
cd website
npm run build
ls -la website/dist/_astro/*.woff2
```

Expected: Font files exist in dist/_astro/.

- [ ] **Step 5: Update preload paths if needed**

If the actual font filenames differ from what you used, update the preload hrefs to match the actual files.

- [ ] **Step 6: Run Lighthouse performance audit**

```bash
# Install Lighthouse CLI if not present
npm install -g lighthouse

# Audit the built site
npx http-server website/dist -p 8080 &
lighthouse http://localhost:8080 --view
```

Expected: Performance score >= 90, LCP < 1s.

- [ ] **Step 7: Commit**

```bash
git add website/src/layouts/Base.astro
git commit -m "perf(website): add font preloading for faster LCP"
```

---

## Task 4: Semantic HTML Enhancement — Heading Hierarchy

**Files:**
- Modify: All page files in `website/src/pages/`

**Interfaces:**
- Consumes: Existing page content
- Produces: Corrected heading hierarchy (single h1 per page, proper nesting)

### Description

Review and fix heading hierarchy across all pages to improve accessibility and SEO.

### Requirements

- Each page must have exactly one `<h1>` tag
- Headings must not skip levels (h1 → h2 → h3, not h1 → h3)
- Headings must be descriptive and concise

### Steps

- [ ] **Step 1: Audit heading hierarchy**

```bash
for file in website/src/pages/*.astro website/src/pages/*.mdx; do
  echo "=== $file ==="
  grep -n "<h[1-6]" "$file" 2>/dev/null || echo "No headings"
done
```

Document pages with heading issues.

- [ ] **Step 2: Fix index.astro**

Read `website/src/pages/index.astro` and ensure:
- Only one `<h1>` tag
- Subsequent headings use `<h2>`, `<h3>`, etc. in order

Example fix:
```astro
<!-- Before -->
<h1>Welcome to gf</h1>
<h3>Features</h3>  <!-- Skipped h2 -->

<!-- After -->
<h1>Welcome to gf</h1>
<h2>Features</h2>  <!-- Correct nesting -->
```

- [ ] **Step 3: Fix other pages**

Repeat for all pages:
- `workflow.astro`
- `compare.astro`
- `architecture.astro`
- `quickstart.mdx`
- etc.

- [ ] **Step 4: Verify with W3C validator**

```bash
# Install HTML validator
npm install -g html-validate

# Validate built pages
html-validate website/dist/index.html
```

Expected: No heading hierarchy errors.

- [ ] **Step 5: Run Lighthouse accessibility audit**

```bash
lighthouse http://localhost:8080 --view --only-categories=accessibility
```

Expected: Accessibility score >= 90.

- [ ] **Step 6: Commit**

```bash
git add website/src/pages/
git commit -m "a11y(website): fix heading hierarchy across all pages"
```

---

## Task 5: Semantic HTML Enhancement — Aria Labels

**Files:**
- Modify: `website/src/layouts/Base.astro`, all page files

**Interfaces:**
- Consumes: Existing navigation and content structure
- Produces: Aria labels for navigation, main content, and footer

### Description

Add aria labels to improve accessibility for screen readers.

### Requirements

- Navigation: `<nav aria-label="主导航">`
- Main content: `<main id="main-content">`
- Footer: `<footer role="contentinfo">`
- Images: All `<img>` tags must have `alt` attributes

### Steps

- [ ] **Step 1: Add aria label to navigation**

Edit `website/src/layouts/Base.astro`:

```astro
<nav class="nav" aria-label="主导航">
  <a class="nav-brand" href={`${base}/`}>
```

- [ ] **Step 2: Add id to main content area**

Add to Base.astro or each page:

```astro
<main id="main-content">
  <slot />
</main>
```

- [ ] **Step 3: Add role to footer**

```astro
<footer class="footer" role="contentinfo">
  <div class="footer-inner">
```

- [ ] **Step 4: Audit image alt text**

```bash
grep -rn "<img" website/src/ | grep -v "alt="
```

Fix any images missing alt text.

- [ ] **Step 5: Run Lighthouse accessibility audit**

```bash
lighthouse http://localhost:8080 --view --only-categories=accessibility
```

Expected: Accessibility score >= 90.

- [ ] **Step 6: Commit**

```bash
git add website/src/
git commit -m "a11y(website): add aria labels and improve semantic HTML"
```

---

## Task 6: Internal Linking Optimization

**Files:**
- Modify: All page files in `website/src/pages/`

**Interfaces:**
- Consumes: Existing page content
- Produces: Related page links, each page has 3+ internal links

### Description

Improve internal linking structure to enhance SEO and user navigation.

### Requirements

- Each page must have at least 3 internal links to other pages
- Add "Related Pages" or "See Also" section where appropriate
- Ensure no orphan pages (every page is linked from at least one other page)

### Steps

- [ ] **Step 1: Map current internal links**

```bash
for file in website/src/pages/*.astro; do
  echo "=== $(basename $file) ==="
  grep -o 'href="[^"]*"' "$file" | grep -v "http" | sort -u
done
```

Identify pages with fewer than 3 internal links.

- [ ] **Step 2: Add related links to workflow.astro**

Add a "Related Pages" section at the bottom:

```astro
<section class="related">
  <h2>相关内容</h2>
  <ul>
    <li><a href={`${base}/architecture/`}>架构设计</a> — 了解 gf 的技术架构</li>
    <li><a href={`${base}/quickstart/`}>快速上手</a> — 5 分钟开始使用</li>
    <li><a href={`${base}/compare/`}>对比分析</a> — gf vs gh vs glab</li>
  </ul>
</section>
```

- [ ] **Step 3: Add related links to other pages**

Repeat for:
- `compare.astro` → link to workflow, quickstart, architecture
- `architecture.astro` → link to workflow, compare
- `quickstart.mdx` → link to workflow, docs
- `index.astro` → link to quickstart, workflow, docs

- [ ] **Step 4: Verify no orphan pages**

```bash
# Check that every page is linked from at least one other page
for page in workflow architecture compare quickstart docs; do
  grep -r "href.*$page" website/src/pages/ | wc -l
done
```

Expected: All counts >= 1.

- [ ] **Step 5: Run Lighthouse SEO audit**

```bash
lighthouse http://localhost:8080 --view --only-categories=seo
```

Expected: SEO score >= 90.

- [ ] **Step 6: Commit**

```bash
git add website/src/pages/
git commit -m "seo(website): improve internal linking structure"
```

---

## Task 7: Write Search Engine Submission Guide

**Files:**
- Create: `docs/operations/search-engine-submission-guide.md`

**Interfaces:**
- Consumes: Website URL, sitemap URL, design decisions
- Produces: Comprehensive operation guide in Chinese

### Description

Create a step-by-step guide for submitting the website to Google, Bing, and Baidu search engines.

### Requirements

- Written in Chinese (中文)
- Cover 3 platforms: Google Search Console, Bing Webmaster Tools, Baidu Webmaster
- Include: Registration, verification, sitemap submission steps
- Include: Verification checklist
- Include: Common issues and troubleshooting
- Include: Maintenance schedule

### Steps

- [ ] **Step 1: Create directory structure**

```bash
mkdir -p docs/operations
```

- [ ] **Step 2: Write the guide**

Create `docs/operations/search-engine-submission-guide.md` with the following structure:

```markdown
# 搜索引擎提交操作指南

## 概述
- 目标：三端收录（Google / Bing / 百度）
- 前置条件：官网已部署、sitemap.xml 可访问
- 预计时间：2-3 小时

## 1. Google Search Console

### 1.1 注册账号
[详细步骤]

### 1.2 添加网站属性
[详细步骤]

### 1.3 验证所有权
[DNS 验证和 HTML 文件验证两种方法]

### 1.4 提交 Sitemap
[步骤]

### 1.5 监控收录
[步骤]

## 2. Bing Webmaster Tools
[类似结构]

## 3. 百度资源平台
[类似结构]

## 4. 验收标准
[Checklist]

## 5. 常见问题
[FAQ]

## 6. 定期维护
[Weekly/monthly tasks]
```

Fill in detailed steps for each section based on the design discussion.

- [ ] **Step 3: Review for completeness**

Ensure the guide covers:
- Account registration for each platform
- Multiple verification methods
- Sitemap submission
- Monitoring and maintenance
- Troubleshooting common issues

- [ ] **Step 4: Commit**

```bash
git add docs/operations/search-engine-submission-guide.md
git commit -m "docs: add search engine submission guide"
```

---

## Task 8: Write Content Matrix Guide

**Files:**
- Create: `docs/operations/content-matrix-guide.md`

**Interfaces:**
- Consumes: Platform requirements, content strategy, phased approach
- Produces: Comprehensive content operation guide in Chinese

### Description

Create a guide for setting up content platforms and establishing a content production workflow.

### Requirements

- Written in Chinese (中文)
- Cover 5 platforms: Juejin, Zhihu, WeChat Official Account, Xiaohongshu, Douyin
- Phased approach: Phase 1 (text/image), Phase 2 (video)
- Include: Platform setup, content production workflow, topic pool, publishing cadence
- Include: Video production workflow for Douyin (AI-assisted)

### Steps

- [ ] **Step 1: Write the guide**

Create `docs/operations/content-matrix-guide.md` with structure:

```markdown
# 中文内容矩阵操作指南

## 概述
- 目标：建立中文内容发布节奏（每月 2 篇）
- 平台：掘金 + 知乎 + 公众号 + 小红书 + 抖音
- 策略：官网首发 → 同步分发
- 分阶段执行：本月图文优先，下月视频试点

## 1. 平台账号开通

### 1.1 掘金
[注册、创作者权益、专栏设置]

### 1.2 知乎
[注册、专栏创建]

### 1.3 微信公众号
[订阅号注册]

### 1.4 小红书
[注册、专业号设置]

### 1.5 抖音（Phase 2）
[注册、视频制作流程]

## 2. 内容生产流程
[官网首发 → 多平台分发流程]

## 3. 内容类型分布
[40% 教程 / 30% 案例 / 20% 理念 / 10% 更新]

## 4. 首批选题（6 篇）
[选题清单]

## 5. 发布节奏
[每月 2 篇，周次安排]

## 6. 抖音视频制作（Phase 2）
[半自动化流程：AI 脚本/配音/字幕 + 人工录屏]

## 7. 数据跟踪
[各平台指标]

## 8. 验收标准
[Checklist]
```

Fill in detailed content for each section.

- [ ] **Step 2: Include specific details**

Ensure the guide includes:
- Exact URLs for platform registration
- Step-by-step setup instructions
- Content templates (tutorial, case study)
- Topic pool with 6 specific article ideas
- Video production workflow with tool recommendations

- [ ] **Step 3: Review for completeness**

Verify all platforms are covered and the phased approach is clear.

- [ ] **Step 4: Commit**

```bash
git add docs/operations/content-matrix-guide.md
git commit -m "docs: add content matrix operation guide"
```

---

## Task 9: Final Verification and Testing

**Files:**
- Test: All modified website files

**Interfaces:**
- Consumes: All previous tasks completed
- Produces: Verification report, all success criteria met

### Description

Run comprehensive verification to ensure all success criteria are met.

### Requirements

- Lighthouse Performance >= 90
- Lighthouse Accessibility >= 90
- Lighthouse SEO >= 90
- LCP < 1 second
- OG/Twitter Card tags present on all pages
- All internal links working
- No HTML validation errors

### Steps

- [ ] **Step 1: Build the website**

```bash
cd website
npm run build
```

Expected: Build succeeds without errors.

- [ ] **Step 2: Start local server**

```bash
npx http-server dist -p 8080
```

- [ ] **Step 3: Run Lighthouse audit**

```bash
lighthouse http://localhost:8080 --view
```

Check all categories:
- Performance >= 90
- Accessibility >= 90
- SEO >= 90
- LCP < 1s

- [ ] **Step 4: Verify OG tags**

```bash
curl http://localhost:8080 | grep -A 5 "og:type"
```

Expected: OG tags present.

- [ ] **Step 5: Verify internal links**

```bash
# Check for broken links
npm install -g broken-link-checker
blc http://localhost:8080 --ro
```

Expected: No broken links.

- [ ] **Step 6: Verify documentation files exist**

```bash
ls -lh docs/operations/*.md
```

Expected: Both guides exist.

- [ ] **Step 7: Create verification report**

Create a summary of all verification results.

- [ ] **Step 8: Commit verification report (optional)**

```bash
git add verification-report.md
git commit -m "docs: add verification report"
```

---

## Task 10: Create Pull Request

**Files:**
- All modified and created files

**Interfaces:**
- Consumes: All tasks completed and verified
- Produces: Pull request with all changes

### Description

Create a pull request with all SEO and documentation changes.

### Requirements

- PR title follows conventional commits
- PR description includes checklist of completed tasks
- PR references Issue #99
- All CI checks pass

### Steps

- [ ] **Step 1: Create feature branch**

```bash
git checkout -b feat/99-seo-content-matrix
```

- [ ] **Step 2: Ensure all changes are committed**

```bash
git status
```

Expected: Working tree clean, all changes committed.

- [ ] **Step 3: Push branch**

```bash
git push -u origin feat/99-seo-content-matrix
```

- [ ] **Step 4: Create pull request**

```bash
gf pr create --title "feat(website): SEO optimization and content matrix guides" --body "Closes #99

## Summary

This PR implements technical SEO optimizations and creates comprehensive operation guides for search engine submission and content distribution.

## Changes

### Technical SEO
- [x] Add Open Graph and Twitter Card meta tags
- [x] Add font preloading for performance
- [x] Fix heading hierarchy across all pages
- [x] Add aria labels for accessibility
- [x] Improve internal linking structure
- [x] Create OG image (1200x630px)

### Documentation
- [x] Search engine submission guide (Google/Bing/Baidu)
- [x] Content matrix operation guide (5 platforms)

## Verification

- [x] Lighthouse Performance >= 90
- [x] Lighthouse Accessibility >= 90
- [x] Lighthouse SEO >= 90
- [x] LCP < 1s
- [x] OG tags verified
- [x] No broken links

## Deliverables

1. Technical SEO PR (this PR)
2. \`docs/operations/search-engine-submission-guide.md\`
3. \`docs/operations/content-matrix-guide.md\`
4. \`website/public/og-image.png\`

Closes #99"
```

- [ ] **Step 5: Wait for CI checks**

Monitor CI pipeline and fix any issues.

- [ ] **Step 6: Request review**

```bash
gf pr edit --add-reviewer <reviewer>
```

- [ ] **Step 7: Merge after approval**

Once approved and CI passes, merge the PR.

---

## Execution Notes

- **Total estimated time:** 10-15 hours
- **Prerequisites:** Node.js, npm, gf CLI, Lighthouse CLI
- **Testing:** Use Lighthouse for performance/accessibility/SEO audits
- **Documentation:** All guides must be in Chinese (中文)
- **Commits:** Follow conventional commits format
- **Review:** Each task can be reviewed independently

## Success Criteria

All of the following must be true:

- [ ] OG/Twitter Card tags on all pages
- [ ] Lighthouse Performance >= 90
- [ ] Lighthouse Accessibility >= 90
- [ ] Lighthouse SEO >= 90
- [ ] LCP < 1s
- [ ] Search engine submission guide complete
- [ ] Content matrix guide complete
- [ ] Initial 6 topics selected
- [ ] PR merged to main branch
