import { defineConfig } from "astro/config";
import sitemap from "@astrojs/sitemap";
import mdx from "@astrojs/mdx";

// 项目站：https://byx-darwin.github.io/gitflow-cli
export default defineConfig({
  site: "https://byx-darwin.github.io/gitflow-cli",
  base: "/gitflow-cli",
  integrations: [sitemap(), mdx()],
});
