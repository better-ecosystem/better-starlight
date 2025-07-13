# Better-Starlight

> [!NOTE]  
> This is very much a work in progress.

## Installation

### Dependencies

- just

#### Manual Installation

``` bash
git clone https://github.com/better-ecosystem/better-starlight.git
cd better-starlight
just install
```

### Quick Installation

``` bash
curl -L https://github.com/better-ecosystem/better-starlight/releases/download/v1.1/starlight -o ~/.local/bin/starlight && chmod +x ~/.local/bin/starlight
```

## Usage

Run `starlight` to launch the app.

## Uninstall

``` bash  
sudo rm -rf /usr/bin/starlight
```

<p align="center" ><b>OR</b></p>

``` bash  
just uninstall
```

### TODO

- [ ] wiki for styling and config
- [ ] Search web
- [ ] Search files
- [ ] Show currently opened windows
- [ ] Math calculations
- [ ] Unit conversion
- [ ] Session controls
- [ ] Clipboard (maybe)

### In Progress

- [ ] Command launcher

### Done

- [x] Application launcher

### References

- Docs was created by taking reference (yoinked) from [ags docs](https://github.com/aylur/ags)
