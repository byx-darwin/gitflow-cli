import { describe, it, expect } from "vitest";
import {
  generateSoftwareAppJsonLd,
  generateFAQPageJsonLd,
  generateHowToJsonLd,
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
});
