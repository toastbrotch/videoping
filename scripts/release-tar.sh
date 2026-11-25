#!/bin/sh
# Make the tar.gz of a release from the .deb (one layout for both):
#   videoping-<version>-linux-x86_64/bin/videoping
#   videoping-<version>-linux-x86_64/share/{videoping/isf-pack, applications, icons, doc}
# videoping finds its shader pack at <bin>/../share/videoping/isf-pack, so the folder works
# wherever it is unpacked. Run after `cargo deb`; writes target/release-files/.
set -eu
cd "$(dirname "$0")/.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
deb=$(ls target/debian/videoping_"$version"-*_amd64.deb | head -n1)
name="videoping-$version-linux-x86_64"
out=target/release-files
rm -rf "$out/$name" "$out/root"
mkdir -p "$out/root" "$out/$name"
dpkg-deb -x "$deb" "$out/root"
mv "$out/root/usr/bin" "$out/root/usr/share" "$out/$name/"
rm -rf "$out/root"
cat > "$out/$name/INSTALL.txt" <<TXT
videoping $version

Run it from here:      ./bin/videoping
Check it:              ./bin/videoping --version   (shows where the shader pack is)

Install for your user (menu entry and icon):
  mkdir -p ~/.local/bin ~/.local/share
  cp bin/videoping ~/.local/bin/
  cp -r share/* ~/.local/share/

It needs GStreamer 1.20+ with the base and good plugins (plus bad, libav and vaapi for
hardware decoding and recording), OpenGL 3.3 and ALSA. Details: share/doc/videoping/install.md
or https://videoping.net/install.html
TXT
cp "$deb" "$out/videoping_${version}_amd64.deb"
tar -C "$out" -czf "$out/$name.tar.gz" "$name"
rm -rf "$out/$name"
(cd "$out" && sha256sum "videoping_${version}_amd64.deb" "$name.tar.gz" > SHA256SUMS)
ls -l "$out"
