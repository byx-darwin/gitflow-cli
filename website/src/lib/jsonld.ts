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
  offers: { "@type": "Offer"; price: string; priceCurrency: string };
  sameAs: string[];
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
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    sameAs: [
      "https://github.com/byx-darwin/gitflow-cli",
      "https://crates.io/crates/gitflow-cli",
    ],
  };
}

export function generateFAQPageJsonLd(): FAQPageJsonLd {
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: faqData.faqs.map((faq) => ({
      "@type": "Question",
      name: faq.question,
      acceptedAnswer: {
        "@type": "Answer",
        text: faq.answer,
      },
    })),
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
