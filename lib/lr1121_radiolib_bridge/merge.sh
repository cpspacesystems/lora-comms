#!/bin/sh
set -e
mkdir -p /tmp/merge_libs
cp "$1" /tmp/merge_libs/a.a
cp "$2" /tmp/merge_libs/b.a
cd /tmp/merge_libs
ar x a.a
ar x b.a
ar rcs "$3" *.o
rm -rf /tmp/merge_libs