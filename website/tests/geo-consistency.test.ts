import { describe, it, expect } from "vitest";
import {
  generateSoftwareAppJsonLd,
  generateFAQPageJsonLd,
  generateHowToJsonLd,
  generateJevWebPageJsonLd,
  generateJevBlogPostingJsonLd,
  generateBlogJsonLd,
} from "../src/lib/jsonld";

const CANONICAL_POSITIONING =
  "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

describe("GEO entity consistency", () => {
  it("should use canonical positioning in SoftwareApplication JSON-LD", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.description).toBe(CANONICAL_POSITIONING);
  });

  it("should reference GitHub and crates.io in sameAs", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.sameAs).toContain(
      "https://github.com/byx-darwin/gitflow-cli",
    );
    expect(jsonLd.sameAs).toContain("https://crates.io/crates/gitflow-cli");
  });

  it("should describe Jev as an optional typed decision feature", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.featureList.join(" ")).toContain("Jev");
    expect(jsonLd.featureList.join(" ")).toContain("Noul");
  });

  it("should have correct author information in JSON-LD", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.author["@type"]).toBe("Person");
    expect(jsonLd.author.name).toBe("皮哥不写PPT");
    expect(jsonLd.author.url).toBe("https://byx-darwin.github.io/");
  });

  it("should generate valid FAQPage JSON-LD", () => {
    const jsonLd = generateFAQPageJsonLd();
    expect(jsonLd["@type"]).toBe("FAQPage");
    expect(jsonLd.mainEntity.length).toBeGreaterThan(0);
    for (const entity of jsonLd.mainEntity) {
      expect(entity["@type"]).toBe("Question");
      expect(entity.name.length).toBeGreaterThan(0);
      expect(entity.acceptedAnswer.text.length).toBeGreaterThan(0);
    }
  });

  it("should generate valid HowTo JSON-LD", () => {
    const jsonLd = generateHowToJsonLd();
    expect(jsonLd).not.toBeNull();
    if (jsonLd) {
      expect(jsonLd["@type"]).toBe("HowTo");
      expect(jsonLd.step.length).toBeGreaterThan(0);
      for (const step of jsonLd.step) {
        expect(step["@type"]).toBe("HowToStep");
        expect(step.name.length).toBeGreaterThan(0);
        expect(step.text.length).toBeGreaterThan(0);
      }
    }
  });

  it("should generate Jev-specific FAQ, HowTo, and WebPage entities", () => {
    const faq = generateFAQPageJsonLd("jev");
    const howTo = generateHowToJsonLd("接入 Jev 类型化决策层");
    const page = generateJevWebPageJsonLd();

    expect(faq.mainEntity.length).toBeGreaterThanOrEqual(4);
    expect(faq.mainEntity.every((item) => item.name.includes("Jev"))).toBe(true);
    expect(howTo?.url).toBe("https://byx-darwin.github.io/gitflow-cli/jev/");
    expect(page.about.name).toBe("Jev for gf");
    expect(page.about.isSoftwareAddonFor.name).toBe("gf");
    expect(page.primaryImageOfPage.url).toContain("jev-decision-flow.webp");
  });

  it("should generate a Jev BlogPosting entity with stable attribution", () => {
    const article = generateJevBlogPostingJsonLd();
    expect(article["@type"]).toBe("BlogPosting");
    expect(article.headline).toContain("类型化决策");
    expect(article.url).toContain("/blog/jev-typed-decisions/");
    expect(article.about.some((subject) => subject.name === "Jev")).toBe(true);
    expect(article.author.name).toBe("皮哥不写PPT");
  });

  it("should expose current and legacy articles through the Blog entity", () => {
    const blog = generateBlogJsonLd();
    expect(blog["@type"]).toBe("Blog");
    expect(blog.blogPost).toHaveLength(2);
    expect(blog.blogPost.map((post) => post.url)).toEqual(
      expect.arrayContaining([
        "https://byx-darwin.github.io/gitflow-cli/blog/jev-typed-decisions/",
        "https://byx-darwin.github.io/gitflow-cli/dogfooding/",
      ]),
    );
  });
});
