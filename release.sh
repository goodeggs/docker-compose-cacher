#!/bin/bash
set -e
set -o pipefail
#set -x

version=$(cat VERSION)
releasedir="${PWD}/releases/${version}"

rm -rf "$releasedir" && mkdir -p "$releasedir"

echo "building for darwin_amd64"
cargo build --release --target=x86_64-apple-darwin
cd target/x86_64-apple-darwin/release && zip ${releasedir}/docker-compose-cacher_v${version}_darwin_amd64.zip docker-compose-cacher
cd -

echo "building for linux_amd64"
cargo build --release --target=x86_64-unknown-linux-musl
cd target/x86_64-unknown-linux-musl/release && tar czvf ${releasedir}/docker-compose-cacher_v${version}_linux_amd64.tar.gz docker-compose-cacher
cd -

echo "releasing v${version}..."
ghr -t "$GITHUB_TOKEN" -u goodeggs -r platform --replace "v${version}" "releases/${version}/"

