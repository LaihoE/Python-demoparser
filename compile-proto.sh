#!/bin/sh -x

has () {
    command -v $1 >/dev/null 2>&1
}

has protoc && has protoc-gen-rust || {
    echo "missing protobuf compiler"
    echo "apt install protobuf-compiler && cargo install rust-protobuf"
    exit 1
}

root=$(dirname $0)

rm $root/csgo-protobuf/src/*

compile () {
    protoc \
        -I=$root/game-tracking/Protobufs \
        --rust_out=$root/csgoproto/src \
        $root/game-tracking/Protobufs/$1.proto
    echo "pub mod $1;" >> $root/csgoproto/src/lib.rs
}

echo "extern crate protobuf;" > $root/csgoproto/src/lib.rs

compile netmessages
compile steammessages
compile cstrike15_gcmessages
compile cstrike15_usermessages
