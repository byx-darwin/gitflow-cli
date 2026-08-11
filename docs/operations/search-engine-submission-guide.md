# 搜索引擎提交操作指南

> **目标**：完成三端搜索引擎收录（Google / Bing / 百度）
> **前置条件**：官网已部署（https://byx-darwin.github.io/gitflow-cli/）
> **预计时间**：2-3 小时
> **更新日期**：2026-08-11

---

## 目录

1. [Google Search Console](#1-google-search-console)
2. [Bing Webmaster Tools](#2-bing-webmaster-tools)
3. [百度资源平台](#3-百度资源平台)
4. [验收标准](#4-验收标准)
5. [常见问题](#5-常见问题)
6. [定期维护](#6-定期维护)

---

## 1. Google Search Console

### 1.1 注册账号

1. 访问 https://search.google.com/search-console
2. 使用 Google 账号登录
3. 如果没有 Google 账号，先访问 https://accounts.google.com 注册

### 1.2 添加网站属性

1. 点击左上角"添加资源"按钮
2. 选择验证方式：
   - **网域**（推荐）：覆盖所有子域名和协议（http/https），需要 DNS 验证
   - **网址前缀**：仅覆盖指定 URL，可使用 HTML 文件验证
3. 输入网站 URL：
   - 网域方式：`byx-darwin.github.io`
   - 网址前缀方式：`https://byx-darwin.github.io/gitflow-cli/`

### 1.3 验证所有权

#### 方法 A：DNS 验证（推荐）

1. Google 提供 TXT 记录值（类似 `google-site-verification=xxxxx`）
2. 由于使用 GitHub Pages，需要在 GitHub 仓库设置中添加 DNS 记录：
   - 访问 https://github.com/byx-darwin/gitflow-cli/settings/pages
   - GitHub Pages 使用 GitHub 默认 DNS，不支持自定义 TXT 记录
3. **替代方案**：使用 HTML 文件验证

#### 方法 B：HTML 文件验证

1. 下载 Google 提供的 HTML 验证文件（如 `google1234567890.html`）
2. 将文件放到 `website/public/` 目录
3. 提交 PR 并合并，GitHub Actions 会自动部署
4. 等待 1-2 分钟让部署完成
5. 回到 Google Search Console 点击"验证"按钮
6. 验证成功后，文件可以删除或保留

### 1.4 提交 Sitemap

1. 进入"站点地图"页面（左侧菜单）
2. 在"添加新的站点地图"输入框中输入：`sitemap-index.xml`
3. 完整 URL 为：`https://byx-darwin.github.io/gitflow-cli/sitemap-index.xml`
4. 点击"提交"
5. 状态应显示"成功"

### 1.5 监控收录

- **覆盖率报告**：查看已收录页面数、排除页面、错误
- **性能报告**：查看搜索点击数、展示次数、平均排名
- **URL 检查**：输入具体 URL 查看索引状态

**建议**：每周检查一次，持续 4 周观察收录趋势。

---

## 2. Bing Webmaster Tools

### 2.1 注册账号

1. 访问 https://www.bing.com/webmasters
2. 使用 Microsoft 账号登录
3. 如果没有 Microsoft 账号，先访问 https://account.microsoft.com 注册

### 2.2 添加网站

1. 点击"添加网站"按钮
2. 输入网站 URL：`https://byx-darwin.github.io/gitflow-cli/`
3. 点击"添加"

### 2.3 验证所有权

#### 方法 A：DNS 验证

1. Bing 提供 TXT 记录值
2. 同样需要在 GitHub 设置中添加（但 GitHub Pages 不支持）
3. 使用 HTML 文件验证

#### 方法 B：HTML 文件验证

1. 下载 Bing 提供的验证文件（如 `BingSiteAuth.xml`）
2. 将文件放到 `website/public/` 目录
3. 提交 PR 并合并部署
4. 回到 Bing Webmaster Tools 点击"验证"

#### 方法 C：从 Google 导入（最简单）

1. 如果已完成 Google 验证，选择"从 Google 导入"
2. Bing 会自动检测 Google 的验证记录
3. 一键完成验证

### 2.4 提交 Sitemap

1. 进入"站点地图"页面
2. 输入：`https://byx-darwin.github.io/gitflow-cli/sitemap-index.xml`
3. 点击"提交"
4. 状态应显示"已提交"

### 2.5 监控收录

- **站点统计信息**：查看爬取数、索引数
- **搜索关键词**：查看排名和流量
- **SEO 报告**：查看 SEO 问题和建议

---

## 3. 百度资源平台

### 3.1 注册账号

1. 访问 https://ziyuan.baidu.com/
2. 使用百度账号登录
3. 如果没有百度账号，先访问 https://passport.baidu.com 注册

### 3.2 添加网站

1. 进入"用户中心" → "站点管理"
2. 点击"添加站点"
3. 输入网站地址：`https://byx-darwin.github.io/gitflow-cli/`
4. 选择站点属性：
   - 站点类型：其他
   - 站点语言：中文
5. 点击"下一步"

### 3.3 验证所有权

#### 方法 A：CNAME 验证

1. 百度提供 CNAME 记录值
2. 需要在域名 DNS 中添加 CNAME 记录
3. GitHub Pages 不支持自定义 DNS 记录
4. 使用文件验证

#### 方法 B：文件验证（推荐）

1. 下载百度提供的验证文件（如 `baidu_verify_xxxxx.html`）
2. 将文件放到 `website/public/` 目录
3. 提交 PR 并合并部署
4. 等待 1-2 分钟
5. 回到百度资源平台，点击"提交验证"
6. 验证成功后，文件可以保留

### 3.4 提交 Sitemap

1. 进入"链接提交" → "sitemap"
2. 输入：`https://byx-darwin.github.io/gitflow-cli/sitemap-index.xml`
3. 点击"提交"
4. 状态应显示"提交成功"

### 3.5 主动推送（可选，优先级低）

百度提供 API 主动推送接口，可以在每次部署后自动推送新链接。

**实现方式**（需要开发）：
1. 在百度资源平台获取推送 token
2. 编写脚本调用百度 API
3. 集成到 GitHub Actions 部署流程中

**暂不实施**：当前手动提交 sitemap 已足够，主动推送可作为后续优化。

### 3.6 监控收录

- **索引量**：查看已收录页面数
- **流量与关键词**：查看搜索表现
- **抓取诊断**：测试百度是否能正常抓取页面
- **抓取频次**：查看百度爬虫的访问频率

---

## 4. 验收标准

完成以下所有检查项后，本任务即完成：

### Google Search Console
- [ ] 账号已注册并登录
- [ ] 网站属性已添加
- [ ] 所有权验证通过
- [ ] sitemap 已提交（`sitemap-index.xml`）
- [ ] 覆盖率报告显示页面已被索引

### Bing Webmaster Tools
- [ ] 账号已注册并登录
- [ ] 网站已添加
- [ ] 所有权验证通过
- [ ] sitemap 已提交
- [ ] 站点统计信息显示页面已被索引

### 百度资源平台
- [ ] 账号已注册并登录
- [ ] 站点已添加
- [ ] 所有权验证通过
- [ ] sitemap 已提交
- [ ] 索引量显示页面已被收录

### 4 周后复查
- [ ] Google 收录页面数 ≥ 10
- [ ] Bing 收录页面数 ≥ 10
- [ ] 百度收录页面数 ≥ 10

---

## 5. 常见问题

### Q1: 验证失败怎么办？

**A:** DNS 记录或 HTML 文件需要时间生效：
- HTML 文件：提交 PR 后等待 GitHub Actions 部署完成（1-2 分钟）
- DNS 记录：通常需要 5-30 分钟，最长 24 小时
- 如果仍失败，检查：
  - 文件是否正确放在 `public/` 目录
  - URL 是否正确（注意 `/gitflow-cli/` 基础路径）
  - 浏览器直接访问文件 URL 确认可以下载

### Q2: sitemap 提交后多久生效？

**A:** 各平台时间不同：
- Google：1-3 天开始索引，完全索引需要 1-2 周
- Bing：1-2 天开始索引
- 百度：3-7 天开始索引，完全索引需要 2-4 周

### Q3: 为什么收录页面数少于实际页面数？

**A:** 搜索引擎会判断页面质量和唯一性：
- 确保每个页面有独特的 `<title>` 和 `<meta description>`
- 避免重复内容（使用 canonical 标签）
- 确保页面有足够的实质性内容
- 检查 robots.txt 是否误封了某些页面

### Q4: 如何提高收录速度？

**A:**
- 确保 sitemap 正确且及时更新
- 在 Google Search Console 使用"URL 检查"工具主动请求索引
- 增加高质量外部链接指向网站
- 定期更新内容，保持网站活跃

### Q5: GitHub Pages 的 SEO 限制？

**A:** GitHub Pages 的一些限制：
- 不支持自定义 DNS 记录（只能用 HTML 文件验证）
- 不支持服务器端重定向
- 但支持自定义域名（CNAME 文件）
- 支持 HTTPS（自动）
- 支持 sitemap 和 robots.txt

---

## 6. 定期维护

### 每周任务
- [ ] 检查三端控制台，查看收录情况
- [ ] 查看是否有爬取错误或索引问题
- [ ] 记录收录页面数和搜索表现

### 每月任务
- [ ] 更新 sitemap（如果有新页面）
- [ ] 检查并修复爬取错误
- [ ] 审查搜索表现，优化低排名页面
- [ ] 查看并响应搜索引擎的建议

### 每季度任务
- [ ] 全面审查 SEO 表现
- [ ] 更新关键词策略
- [ ] 优化页面内容和结构
- [ ] 检查竞争对手的 SEO 策略

---

## 附录：关键 URL

- 官网：https://byx-darwin.github.io/gitflow-cli/
- Sitemap：https://byx-darwin.github.io/gitflow-cli/sitemap-index.xml
- robots.txt：https://byx-darwin.github.io/gitflow-cli/robots.txt
- llms.txt：https://byx-darwin.github.io/gitflow-cli/llms.txt

---

**文档版本**：v1.0
**最后更新**：2026-08-11
**维护者**：byx-darwin
