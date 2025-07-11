set shell := ["bash", "-cu"]

bin_name := "starlight"

install_dir := "/usr/bin"

install:
    @echo "Building {{bin_name}}"
    cargo build --release
    @echo "Installing {{bin_name}} to {{install_dir}}"
    sudo install -Dm755 ./target/release/{{bin_name}} {{install_dir}}/{{bin_name}}
    @echo "Successfully installed {{bin_name}}"

uninstall:
    @echo "Removing {{bin_name}}"
    sudo rm -f {{install_dir}}/{{bin_name}}