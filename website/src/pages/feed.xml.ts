import type { APIRoute } from "astro";

import { BLOG_POSTS } from "../data/blog";

const SITE_URL = "https://byx-darwin.github.io/gitflow-cli";

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

function toRssDate(date: string): string {
  const [year, month, day] = date.split("-").map(Number);
  const instant = new Date(Date.UTC(year!, month! - 1, day!));
  const weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  const months = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
  ];
  return `${weekdays[instant.getUTCDay()]}, ${String(day).padStart(2, "0")} ${months[month! - 1]} ${year} 00:00:00 +0800`;
}

/** Generate the blog RSS feed from the canonical article metadata. */
export const GET: APIRoute = () => {
  const items = BLOG_POSTS.map((article) => {
    const url = `${SITE_URL}${article.path}`;
    return `    <item>
      <title>${escapeXml(article.title)}</title>
      <link>${url}</link>
      <guid isPermaLink="true">${url}</guid>
      <pubDate>${toRssDate(article.datePublished)}</pubDate>
      <description>${escapeXml(article.description)}</description>
    </item>`;
  }).join("\n");

  const body = `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
  <channel>
    <title>gf 博客</title>
    <link>${SITE_URL}/blog/</link>
    <description>gf 的工程实践、设计取舍与评估记录。</description>
    <language>zh-CN</language>
    <atom:link href="${SITE_URL}/feed.xml" rel="self" type="application/rss+xml" />
${items}
  </channel>
</rss>
`;

  return new Response(body, {
    headers: { "Content-Type": "application/rss+xml; charset=utf-8" },
  });
};
