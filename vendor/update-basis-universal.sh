#!/bin/sh

git clone --depth 1 https://github.com/BinomialLLC/basis_universal --branch v2_50 basis_universal_repo

rm -rf ./basis_universal
mkdir ./basis_universal

cp -r ./basis_universal_repo/transcoder/ ./basis_universal/
cp -r ./basis_universal_repo/encoder/ ./basis_universal/
cp -r ./basis_universal_repo/zstd/ ./basis_universal/
cp ./basis_universal_repo/LICENSE ./basis_universal/

rm -rf ./basis_universal_repo
