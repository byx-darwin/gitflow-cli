import { describe, expect, it } from "vitest";

import { withBasePath } from "../src/lib/paths";

describe("site paths", () => {
  it("test_should_preserve_separator_for_github_pages_base", () => {
    expect(withBasePath("/gitflow-cli", "/blog/jev-typed-decisions/")).toBe(
      "/gitflow-cli/blog/jev-typed-decisions/",
    );
  });

  it("test_should_work_at_domain_root", () => {
    expect(withBasePath("/", "/dogfooding/")).toBe("/dogfooding/");
  });
});
