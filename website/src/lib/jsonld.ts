// JSON-LD 生成器：从数据源生成结构化数据

import faqData from "../data/faq.json";
import howtoData from "../data/howto.json";
import { BLOG_POSTS, JEV_BLOG_POST } from "../data/blog";

/** Schema.org representation of the gf application. */
export interface SoftwareAppJsonLd {
  "@context": "https://schema.org";
  "@type": "SoftwareApplication";
  name: string;
  description: string;
  applicationCategory: string;
  operatingSystem: string;
  url: string;
  author: {
    "@type": "Person" | "Organization";
    name: string;
    url?: string;
  };
  offers: { "@type": "Offer"; price: string; priceCurrency: string };
  sameAs: string[];
  featureList: string[];
}

/** Schema.org FAQ page generated from the website FAQ data. */
export interface FAQPageJsonLd {
  "@context": "https://schema.org";
  "@type": "FAQPage";
  mainEntity: Array<{
    "@type": "Question";
    name: string;
    acceptedAnswer: {
      "@type": "Answer";
      text: string;
    };
  }>;
}

/** Schema.org step-by-step guide generated from the website HowTo data. */
export interface HowToJsonLd {
  "@context": "https://schema.org";
  "@type": "HowTo";
  name: string;
  description: string;
  url: string;
  step: Array<{
    "@type": "HowToStep";
    name: string;
    text: string;
  }>;
}

/** Schema.org page entity for the optional Jev integration. */
export interface JevWebPageJsonLd {
  "@context": "https://schema.org";
  "@type": "WebPage";
  name: string;
  description: string;
  url: string;
  isPartOf: { "@type": "WebSite"; name: string; url: string };
  about: {
    "@type": "SoftwareApplication";
    name: string;
    description: string;
    applicationCategory: string;
    url: string;
    isSoftwareAddonFor: { "@type": "SoftwareApplication"; name: string; url: string };
  };
  primaryImageOfPage: { "@type": "ImageObject"; url: string; caption: string };
}

/** Schema.org metadata for one gf engineering article. */
export interface BlogPostingJsonLd {
  "@context": "https://schema.org";
  "@type": "BlogPosting";
  headline: string;
  description: string;
  url: string;
  image: string;
  datePublished: string;
  dateModified: string;
  inLanguage: string;
  author: { "@type": "Person"; name: string; url: string };
  publisher: { "@type": "Organization"; name: string; url: string };
  about: Array<{ "@type": "Thing"; name: string; description?: string }>;
  keywords: string[];
  isPartOf: { "@type": "Blog"; name: string; url: string };
}

/** Schema.org metadata for the gf blog and its article collection. */
export interface BlogJsonLd {
  "@context": "https://schema.org";
  "@type": "Blog";
  name: string;
  description: string;
  url: string;
  inLanguage: string;
  publisher: { "@type": "Organization"; name: string; url: string };
  blogPost: Array<{
    "@type": "BlogPosting";
    headline: string;
    url: string;
    datePublished: string;
  }>;
}

const CANONICAL_POSITIONING =
  "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";
const SITE_URL = "https://byx-darwin.github.io/gitflow-cli";

/** Build the canonical software application entity used across the website. */
export function generateSoftwareAppJsonLd(): SoftwareAppJsonLd {
  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "gf",
    description: CANONICAL_POSITIONING,
    applicationCategory: "DeveloperApplication",
    operatingSystem: "macOS, Linux, Windows",
    url: "https://byx-darwin.github.io/gitflow-cli/",
    author: {
      "@type": "Person",
      name: "皮哥不写PPT",
      url: "https://byx-darwin.github.io/",
    },
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    sameAs: [
      "https://github.com/byx-darwin/gitflow-cli",
      "https://crates.io/crates/gitflow-cli",
    ],
    featureList: [
      "GitHub、GitLab、GitCode 跨平台工作流",
      "AI Agent Skills 工程门禁",
      "可选的 Jev TypeSafe AI 决策层",
      "Noul、Choice、Score、Batch 类型化只读判断",
    ],
  };
}

/** Build FAQ structured data, optionally limited to one content scope. */
export function generateFAQPageJsonLd(scope?: string): FAQPageJsonLd {
  const faqs = scope
    ? faqData.faqs.filter((faq) => "scope" in faq && faq.scope === scope)
    : faqData.faqs.filter((faq) => !("scope" in faq));
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: faqs.map((faq) => ({
      "@type": "Question",
      name: faq.question,
      acceptedAnswer: {
        "@type": "Answer",
        text: faq.answer,
      },
    })),
  };
}

/** Build the structured page entity for the Jev product page. */
export function generateJevWebPageJsonLd(): JevWebPageJsonLd {
  return {
    "@context": "https://schema.org",
    "@type": "WebPage",
    name: "Jev 类型化 AI 决策层 | gf",
    description:
      "Jev 是 gf 可选的 TypeSafe AI 决策适配器，以 Noul、Choice、Score 和 Batch 类型提供只读语义判断，并保留确定性门禁与人工授权。",
    url: "https://byx-darwin.github.io/gitflow-cli/jev/",
    isPartOf: {
      "@type": "WebSite",
      name: "gf",
      url: "https://byx-darwin.github.io/gitflow-cli/",
    },
    about: {
      "@type": "SoftwareApplication",
      name: "Jev for gf",
      description: "面向工程工作流的可选 TypeSafe AI 决策适配器。",
      applicationCategory: "DeveloperApplication",
      url: "https://byx-darwin.github.io/gitflow-cli/jev/",
      isSoftwareAddonFor: {
        "@type": "SoftwareApplication",
        name: "gf",
        url: "https://byx-darwin.github.io/gitflow-cli/",
      },
    },
    primaryImageOfPage: {
      "@type": "ImageObject",
      url: "https://byx-darwin.github.io/gitflow-cli/assets/jev-decision-flow.webp",
      caption: "Jev 类型化决策信号经过校验后输出受约束结果",
    },
  };
}

/** Build article structured data for the Jev engineering article. */
export function generateJevBlogPostingJsonLd(): BlogPostingJsonLd {
  const article = JEV_BLOG_POST;
  const url = `${SITE_URL}${article.path}`;
  return {
    "@context": "https://schema.org",
    "@type": "BlogPosting",
    headline: article.headline,
    description: article.description,
    url,
    image: `${SITE_URL}${article.image}`,
    datePublished: article.datePublished,
    dateModified: article.dateModified,
    inLanguage: "zh-CN",
    author: {
      "@type": "Person",
      name: "皮哥不写PPT",
      url: "https://byx-darwin.github.io/",
    },
    publisher: {
      "@type": "Organization",
      name: "gf",
      url: "https://byx-darwin.github.io/gitflow-cli/",
    },
    about: [
      {
        "@type": "Thing",
        name: "Jev",
        description: "TypeSafe AI 的类型化决策模型，在 gf 中作为可选、只读的语义判断层。",
      },
      { "@type": "Thing", name: "AI 工程工作流" },
      { "@type": "Thing", name: "类型化 AI 决策" },
    ],
    keywords: [...article.tags, "TypeSafe AI", "gf", "AI 工程工作流", "类型化决策"],
    isPartOf: {
      "@type": "Blog",
      name: "gf 博客",
      url: "https://byx-darwin.github.io/gitflow-cli/blog/",
    },
  };
}

/** Build the blog collection entity from the canonical article metadata. */
export function generateBlogJsonLd(): BlogJsonLd {
  return {
    "@context": "https://schema.org",
    "@type": "Blog",
    name: "gf 博客",
    description: "gf 的工程实践、设计取舍与评估记录。",
    url: "https://byx-darwin.github.io/gitflow-cli/blog/",
    inLanguage: "zh-CN",
    publisher: {
      "@type": "Organization",
      name: "gf",
      url: "https://byx-darwin.github.io/gitflow-cli/",
    },
    blogPost: BLOG_POSTS.map((article) => ({
      "@type": "BlogPosting",
      headline: article.headline,
      url: `${SITE_URL}${article.path}`,
      datePublished: article.datePublished,
    })),
  };
}

/** Build one HowTo entity by guide name, or the first guide when omitted. */
export function generateHowToJsonLd(guideName?: string): HowToJsonLd | null {
  const guide = howtoData.guides.find((g) =>
    guideName ? g.name === guideName : true,
  );
  if (!guide) return null;

  return {
    "@context": "https://schema.org",
    "@type": "HowTo",
    name: guide.name,
    description: guide.description,
    url: guide.url,
    step: guide.steps.map((step) => ({
      "@type": "HowToStep",
      name: step.name,
      text: step.text,
    })),
  };
}
