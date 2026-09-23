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
];

/** Metadata for the Jev engineering article. */
export const JEV_BLOG_POST = BLOG_POSTS[0]!;

/** Metadata for the historical dogfooding article. */
export const DOGFOODING_BLOG_POST = BLOG_POSTS[1]!;
