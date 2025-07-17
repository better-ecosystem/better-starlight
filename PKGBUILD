# Maintainer: Sandesh Sharma <sandeshsharma924@gmail.com>

pkgname=starlight-bin
pkgver=1.4.0
pkgrel=1
pkgdesc="A fast application launcher, command runner and web search app"
url="https://github.com/better-ecosystem/better-starlight/"
license=("LICENSE")
arch=("x86_64")
provides=("starlight")
conflicts=("starlight")
source=("https://github.com/better-ecosystem/better-starlight//releases/download/v$pkgver/starlight-$pkgver-x86_64.tar.gz")
sha256sums=("2ec7a07d6fe03433d56905d4419a3e9a66d815d7928934ea990d2cca98cc164c")

package() {
    install -Dm755 starlight -t "$pkgdir/usr/bin"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
