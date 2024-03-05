#!/bin/bash

function get_latest_release_download_url() {
    dir=$(mktemp -d)

    trap 'rm -rf $dir' RETURN

    while read -r url; do
        echo "$url" | sed 's|releases/tag|releases/download|' >"$dir/$(basename "$url" | tr -d 'v')"
    done < <(grep 'html_url' </dev/stdin | grep -E 'release/v[0-9]+\.[0-9]+\.[0-9]+"' | awk '{ print $2 }' | tr -d '",')

    if [[ $(ls -1 "$dir" | wc -l) -eq 0 ]]; then
        echo "No releases available" >&2
        return 1
    fi

    echo "$(cat "$dir/$(ls -1 "$dir" | sort -n | tail -n1)")/$1"
}

set -e

target_arch=
case "$(uname -m)" in
x86_64) target_arch="amd64";;
arm64) target_arch="aarch64";;
*) echo "Unsupported CPU architecture: $(uname -m)" >&2; exit 1;;
esac

target_os=
case "$(uname -s)" in
Darwin) target_os="macos";;
Linux)
    if uname -v | grep 'Microsoft' >/dev/null 2>&1; then
        target_os="windows"
    else
        target_os="linux"
    fi
    ;;
*) echo "Unsupported OS: $(uname -s)" >&2; exit;;
esac

download_url=$(curl -sSL -X GET https://api.github.com/repos/brian-dlee/sling/releases | get_latest_release_download_url "sling-$target_os-$target_arch")

if [[ -z "$INSTALL_DIR" ]]; then
    echo "INSTALL_DIR is not defined. Defaulting to user's local bin directory: $HOME/.local/bin" >&2
    INSTALL_DIR=$HOME/.local/bin
    test -e "$INSTALL_DIR" || mkdir -p "$INSTALL_DIR"
fi

echo "Downloading $download_url -> $INSTALL_DIR/sling" >&2
curl -o "$INSTALL_DIR/sling" -sSL "$download_url"
chmod +x "$INSTALL_DIR/sling"

echo "$INSTALL_DIR/sling"
