// JSON-LD 生成器：从数据源生成结构化数据

import faqData from "../data/faq.json";
import howtoData from "../data/howto.json";

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

const CANONICAL_POSITIONING =
  "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

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
