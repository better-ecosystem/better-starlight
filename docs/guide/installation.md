# Installation

## Quick install

``` sh
curl -L https://github.com/better-ecosystem/better-starlight/releases/download/v1.4/starlight -o ~/.local/bin/starlight && chmod +x ~/.local/bin/starlight
```

## From Source

1. Install dependencies

    :::code-group

    ``` sh [<i class="devicon-archlinux-plain" /> Arch]
    sudo pacman -Syu \
        just gtk4 gtk4-layer-shell
   ```

    ``` sh [<i class="devicon-fedora-plain" /> Fedora]
    sudo dnf install \
        just gtk4-devel gtk4-layer-shell-devel

   ```

2. Clone and install Starlight

    ``` sh
    git clone https://github.com/better-ecosystem/better-starlight.git
    cd better-starlight
    just install
    ```
