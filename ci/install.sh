set -ex

main() {
    curl https://sh.rustup.rs -sSf | \
        sh -s -- -y --default-toolchain $TRAVIS_RUST_VERSION

    local target=
    if [ $TRAVIS_OS_NAME = linux ]; then
        target=$TARGET_DEFAULT_LINUX
    else
        target=$TARGET_DEFAULT_OSX
    fi

    local tag="$(git ls-remote --tags --refs --exit-code https://github.com/japaric/cross | cut -d/ -f3 | tail -n1)"
    echo cross version: $tag
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- \
           --force \
           --git japaric/cross \
           --tag $tag \
           --target $target

    echo $TARGET
    echo $TARGET_DEFAULT_LINUX
    echo $TARGET_DEFAULT_OSX
    export PATH="$HOME/.cargo/bin:$PATH"
    which rustup || true
    whereis rustup || true
    rustup --version || true
    if [ $TARGET = $TARGET_DEFAULT ] || [ $TARGET =  ]; then
        rustup target add $TARGET
    fi
}

main
