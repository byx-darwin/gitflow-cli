import { describe, it, expect } from "vitest";
import {
  generateSoftwareAppJsonLd,
  generateFAQPageJsonLd,
  generateHowToJsonLd,
  generateJevWebPageJsonLd,
  generateJevBlogPostingJsonLd,
  generateBlogJsonLd,
} from "../src/lib/jsonld";
import { BLOG_POSTS } from "../src/data/blog";

const CANONICAL_POSITIONING =
  "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

describe("GEO entity consistency", () => {
  it("test_should_use_canonical_positioning_in_software_application_json_ld", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.description).toBe(CANONICAL_POSITIONING);
  });

  it("test_should_reference_github_and_crates_io_in_same_as", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.sameAs).toContain(
      "https://github.com/byx-darwin/gitflow-cli",
    );
    expect(jsonLd.sameAs).toContain("https://crates.io/crates/gitflow-cli");
  });

  it("test_should_describe_jev_as_an_optional_typed_decision_feature", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.featureList.join(" ")).toContain("Jev");
    expect(jsonLd.featureList.join(" ")).toContain("Noul");
  });

  it("test_should_have_correct_author_information_in_json_ld", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.author["@type"]).toBe("Person");
    expect(jsonLd.author.name).toBe("皮哥不写PPT");
    expect(jsonLd.author.url).toBe("https://byx-darwin.github.io/");
  });

  it("test_should_generate_valid_faq_page_json_ld", () => {
    const jsonLd = generateFAQPageJsonLd();
    expect(jsonLd["@type"]).toBe("FAQPage");
    expect(jsonLd.mainEntity.length).toBeGreaterThan(0);
    for (const entity of jsonLd.mainEntity) {
      expect(entity["@type"]).toBe("Question");
      expect(entity.name.length).toBeGreaterThan(0);
      expect(entity.acceptedAnswer.text.length).toBeGreaterThan(0);
    }
  });

  it("test_should_generate_valid_how_to_json_ld", () => {
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

  it("test_should_generate_jev_specific_faq_how_to_and_web_page_entities", () => {
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

  it("test_should_generate_jev_blog_posting_with_stable_attribution", () => {
    const article = generateJevBlogPostingJsonLd();
    expect(article["@type"]).toBe("BlogPosting");
    expect(article.headline).toContain("类型化决策");
    expect(article.url).toContain("/blog/jev-typed-decisions/");
    expect(article.about.some((subject) => subject.name === "Jev")).toBe(true);
    expect(article.author.name).toBe("皮哥不写PPT");
  });

  it("test_should_expose_current_and_legacy_articles_through_blog_entity", () => {
    const blog = generateBlogJsonLd();
    expect(blog["@type"]).toBe("Blog");
    expect(blog.blogPost).toHaveLength(BLOG_POSTS.length);
    expect(blog.blogPost.map((post) => post.url)).toEqual(
      expect.arrayContaining([
        "https://byx-darwin.github.io/gitflow-cli/blog/jev-typed-decisions/",
        "https://byx-darwin.github.io/gitflow-cli/dogfooding/",
      ]),
    );
  });
});
