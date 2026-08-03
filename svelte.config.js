import adapter from "@sveltejs/adapter-static";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter({ fallback: "index.html", pages: "dist", assets: "dist" }),
    alias: {
      "@api": "src/api",
      "@components": "src/lib/components",
      "@domain": "src/domain",
      "@i18n": "src/i18n/index.ts",
      "@models": "src/models",
      "@themes": "src/themes",
    },
  },
};

export default config;
