import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { BLOG_POSTS } from "../src/data/blog";

const pagesRoot = resolve(import.meta.dirname, "../src/pages");
const historicalArticlePaths = [
  "/architecture/",
  "/changelog/",
  "/compare/",
  "/compatibility/",
  "/dogfooding/",
  "/quickstart/",
  "/release-workflow/",
  "/support/",
  "/what-is-ai-workflow/",
  "/workflow/",
];

function routeCandidates(path: string): string[] {
  const route = path.replace(/^\//, "").replace(/\/$/, "");
  return ["astro", "md", "mdx"].map((extension) => resolve(pagesRoot, `${route}.${extension}`));
}

describe("blog catalog", () => {
  it("test_should_only_publish_articles_with_real_routes", () => {
    for (const post of BLOG_POSTS) {
      expect(routeCandidates(post.path).some(existsSync), `${post.path} has no page`).toBe(true);
    }
  });

  it("test_should_keep_paths_unique_and_newest_first", () => {
    expect(new Set(BLOG_POSTS.map((post) => post.path)).size).toBe(BLOG_POSTS.length);
    expect(BLOG_POSTS.map((post) => post.datePublished)).toEqual(
      [...BLOG_POSTS].map((post) => post.datePublished).sort().reverse(),
    );
  });

  it("test_should_keep_all_ten_historical_articles_discoverable", () => {
    expect(historicalArticlePaths).toHaveLength(10);
    expect(BLOG_POSTS.map((post) => post.path)).toEqual(
      expect.arrayContaining(historicalArticlePaths),
    );
  });
});
