set shell := ["bash", "-cu"]

bin_name := "starlight"
install_dir := "/usr/bin"
desktop_file := "assets/starlight.desktop"
icon_file := "assets/starlight.svg"
symbolic_icon_file := "assets/starlight-symbolic.svg"
icon_target_dir := "/usr/share/icons/hicolor"

install:
    @echo "Building {{bin_name}}"
    cargo build --release

    @echo "Installing {{bin_name}} to {{install_dir}}"
    sudo install -Dm755 ./target/release/{{bin_name}} {{install_dir}}/{{bin_name}}

    @echo "Installing app icon"
    sudo install -Dm644 {{icon_file}} {{icon_target_dir}}/scalable/apps/{{bin_name}}.svg

    @echo "Installing symbolic icon (if available)"
    sudo install -Dm644 {{symbolic_icon_file}} {{icon_target_dir}}/symbolic/apps/{{bin_name}}-symbolic.svg; \

    @echo "Installing desktop file"
    sudo install -Dm644 {{desktop_file}} /usr/share/applications/{{desktop_file}}

    @echo "Updating icon cache"
    sudo gtk-update-icon-cache {{icon_target_dir}}

    @echo "Successfully installed {{bin_name}}"

uninstall:
    @echo "Removing desktop file"
    sudo rm -f /usr/share/applications/{{desktop_file}}

    @echo "Removing {{bin_name}}"
    sudo rm -f {{install_dir}}/{{bin_name}}