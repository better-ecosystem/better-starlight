---
# https://vitepress.dev/reference/default-theme-home-page
layout: home
pageClass: home-page

hero:
  name: "STARLIGHT"
  text: "Documentation for starlight"
  tagline: Fast and customizeable app launcher 
  image: /icon.svg
  actions:
    - theme: brand
      text: Installation
      link: /guide/installation
    - theme: alt
      text: Configuration
      link: /guide/configuration

features:
  - title: Lightweight
    details: Starlight can run applications and commands in sigle app
  - title: Blazing fast
    details: Written in rust for best performance
  - title: Customizeable
    details: Starlight uses gtk-css to customize the widgets
---

<style>
:root {
  --vp-home-hero-name-color: transparent;
  --vp-home-hero-name-background: -webkit-linear-gradient(120deg, #7f5af0, #00f0ff);

  --overlay-gradient: color-mix(in srgb, var(--vp-c-indigo-1), transparent 50%);
}

.dark {
  --overlay-gradient: color-mix(in srgb, transparent, transparent 85%);
}

.home-page {
  background:
    linear-gradient(215deg, var(--overlay-gradient), transparent 50%),
    radial-gradient(circle at 1% 80%, rgba(108, 178, 213, 0.3), transparent 40%) no-repeat;

  .VPFeature code {
    background-color: rgba(0, 0, 0, 0.3);
    color: #ffffff;
    padding: 3px 8px;
    border-radius: 6px;
  }

  .VPFooter {
    background-color: transparent !important;
    border: none;
  }

  .VPNavBar:not(.top) {
    background-color: rgba(0, 0, 0, 0.25) !important;
    -webkit-backdrop-filter: blur(20px);
    backdrop-filter: blur(20px);

    div.divider {
      display: none;
    }
  }
}

@media (min-width: 640px) {
  :root {
    --vp-home-hero-image-filter: blur(60px);
  }
}

@media (min-width: 960px) {
  :root {
    --vp-home-hero-image-filter: blur(72px);
  }
}
</style>
