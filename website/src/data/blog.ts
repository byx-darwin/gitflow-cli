/** Metadata shared by the blog index, structured data, and RSS feed. */
export interface BlogPostMetadata {
  /** Short title used by index cards and RSS readers. */
  title: string;
  /** Full headline used by `BlogPosting` structured data. */
  headline: string;
  /** Canonical path below the website origin. */
  path: string;
  /** Search and feed summary. */
  description: string;
  /** ISO publication date. */
  datePublished: string;
  /** ISO date of the latest material update. */
  dateModified: string;
  /** Absolute or canonical-site-relative social image path. */
  image: string;
  /** Article topic labels. */
  tags: string[];
}

/** Canonical metadata for every article shown by the gf blog. */
export const BLOG_POSTS: BlogPostMetadata[] = [
  {
    title: "为什么 AI 工程工作流需要类型化决策",
    headline: "为什么 AI 工程工作流需要类型化决策：gf 接入 Jev 的设计与实践",
    path: "/blog/jev-typed-decisions/",
    description:
      "从自由文本建议到 Noul、Choice、Score、Batch 类型化判断，解释 gf 如何安全接入 Jev，并用真实 Issue 试点评估能力边界。",
    datePublished: "2026-09-23",
    dateModified: "2026-09-23",
    image: "/assets/jev-decision-flow.webp",
    tags: ["Jev", "工程实践"],
  },
  {
    title: "用 gf 开发 gf：一次真实的 dogfooding 案例",
    headline: "用 gf 开发 gf：一次真实的 dogfooding 案例",
    path: "/dogfooding/",
    description:
      "从 30 个历史执行合同出发，复盘 gf 的四阶段工作流、契约测试与质量门禁，并按当前功能更新说明。",
    datePublished: "2026-09-02",
    dateModified: "2026-09-23",
    image: "/demo.svg",
    tags: ["Dogfooding"],
  },
  {
    title: "架构设计：gf 工作区布局与设计理念",
    headline: "架构设计：gf 工作区布局与设计理念",
    path: "/architecture/",
    description:
      "拆解 gf 的 Cargo 工作区、核心 crate、平台适配器和单向依赖边界，说明代码应当放在哪里以及为什么。",
    datePublished: "2026-08-09",
    dateModified: "2026-09-23",
    image: "/og-image.svg",
    tags: ["架构", "工程实践"],
  },
  {
    title: "发布工作流：make release 完全指南",
    headline: "发布工作流：make release 完全指南",
    path: "/release-workflow/",
    description:
      "从发布前检查、版本推断到预览和推送，完整说明 gf 的可重复发布流程与安全边界。",
    datePublished: "2026-08-09",
    dateModified: "2026-09-23",
    image: "/og-image.svg",
    tags: ["发布", "工程实践"],
  },
  {
    title: "什么是 AI 编程工程工作流？",
    headline: "什么是 AI 编程工程工作流？",
    path: "/what-is-ai-workflow/",
    description:
      "解释如何用结构化流程和质量门禁，把会生成代码的 AI 转化为可维护、可追溯、可交付的工程协作者。",
    datePublished: "2026-08-09",
    dateModified: "2026-09-23",
    image: "/og-image.svg",
    tags: ["AI 工作流", "概念解析"],
  },
  {
    title: "gf-workflow：四阶段 AI 工程编排",
    headline: "gf-workflow：四阶段 AI 编程工程工作流编排",
    path: "/workflow/",
    description:
      "详解需求澄清、计划制定、执行和交付检查四阶段，以及模式选择、契约恢复和质量门禁。",
    datePublished: "2026-08-09",
    dateModified: "2026-09-23",
    image: "/og-image.svg",
    tags: ["工作流", "Agent"],
  },
  {
    title: "gf vs gh / glab：如何选择？",
    headline: "gf vs gh / glab：跨平台 CLI 如何选择？",
    path: "/compare/",
    description:
      "从平台支持、命令面、Agent 集成、工程门禁和迁移成本比较 gf、gh 与 glab 的适用场景。",
    datePublished: "2026-08-09",
    dateModified: "2026-09-23",
    image: "/og-image.svg",
    tags: ["工具对比", "跨平台"],
  },
  {
    title: "支持政策与升级指引",
    headline: "gf 支持政策与升级指引",
    path: "/support/",
    description:
      "汇总 gf 的版本支持周期、问题反馈渠道、兼容性入口和升级注意事项。",
    datePublished: "2026-08-04",
    dateModified: "2026-08-04",
    image: "/og-image.svg",
    tags: ["支持政策"],
  },
  {
    title: "5 分钟快速上手 gf",
    headline: "5 分钟快速上手 gf",
    path: "/quickstart/",
    description:
      "从安装 CLI 与 Skills 到启动四阶段工作流，用最短路径完成 gf 的首次配置和验证。",
    datePublished: "2026-08-01",
    dateModified: "2026-09-23",
    image: "/og-image.svg",
    tags: ["快速开始", "指南"],
  },
  {
    title: "Git 平台 CLI 兼容性矩阵",
    headline: "gf 的 GitHub、GitLab 与 GitCode CLI 兼容性矩阵",
    path: "/compatibility/",
    description:
      "查看 gf 对 gh、glab 和 gitcode 的最低版本、已验证版本与功能覆盖范围。",
    datePublished: "2026-08-01",
    dateModified: "2026-08-19",
    image: "/og-image.svg",
    tags: ["兼容性", "跨平台"],
  },
  {
    title: "gf 更新日志",
    headline: "gf 更新日志与版本演进记录",
    path: "/changelog/",
    description:
      "按版本回顾 gf 的功能变化、兼容性改进、问题修复和工程能力演进。",
    datePublished: "2026-08-01",
    dateModified: "2026-08-26",
    image: "/og-image.svg",
    tags: ["更新日志"],
  },
];

/** Metadata for the Jev engineering article. */
export const JEV_BLOG_POST = BLOG_POSTS[0]!;

/** Metadata for the historical dogfooding article. */
export const DOGFOODING_BLOG_POST = BLOG_POSTS[1]!;
