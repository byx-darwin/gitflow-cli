import { describe, expect, it } from "vitest";

import { BLOG_POSTS } from "../src/data/blog";
import { GET } from "../src/pages/feed.xml";

describe("blog feed", () => {
  it("test_should_generate_every_blog_post_from_canonical_metadata", async () => {
    const response = await GET({} as Parameters<typeof GET>[0]);
    const body = await response.text();

    expect(response.headers.get("content-type")).toContain("application/rss+xml");
    for (const article of BLOG_POSTS) {
      expect(body).toContain(article.title);
      expect(body).toContain(article.path);
      expect(body).toContain(article.description);
    }
  });
});
