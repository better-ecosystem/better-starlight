import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Starlight",
  description: "Documentation for starlight",
  outDir: "./dist",
  base: "/starlight/",
  lastUpdated: true,

  themeConfig: {
    outline: "deep",
    footer: {
      // message: "Released under the GPL v3.0 License",
      copyright: "Logo is created by Sandesh",
    },
    nav: [
      { text: 'Home', link: '/' }, 
      { text: 'Showcase', link: '/showcase' }
    ],

    sidebar: [
      {
        text: 'Installation',
        items: [
          { text: 'Configuration', link: '/guide/configuration' },
          { text: 'Customization', link: '/guide/customize' },
          { text: 'Contribution', link: '/guide/contribute' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/better-ecosystem/better-starlight' }
    ],

    editLink: {
      pattern: "https://github.com/aylur/ags/edit/main/docs/:path",
      text: "Edit this page on GitHub",
    },

    lastUpdated: {
      text: "Last updated",
    },
    search: {
      provider: "local",
    },
  }
})
