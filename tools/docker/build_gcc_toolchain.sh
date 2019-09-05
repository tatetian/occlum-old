#!/bin/sh
BUILD_DIR=/tmp/occlum_gcc_toolchain
INSTALL_DIR=/opt/occlum/toolchains/gcc

# Exit if any command fails
set -e

# Clean previous build and installation if any
rm -rf ${BUILD_DIR}
rm -rf ${INSTALL_DIR}

# Create the build directory
mkdir -p ${BUILD_DIR}
cd ${BUILD_DIR}

# Download musl-cross-make project
git clone https://github.com/richfelker/musl-cross-make
cd musl-cross-make
git checkout d969dea983a2cc54a1e0308a0cdeb6c3307e4bfa

# Download Occlum's musl libc for musl-cross-make
mkdir sources
cd sources
git clone https://github.com/occlum/musl
MUSL_VER=`cat musl/VERSION`
tar -czvf musl-${MUSL_VER}.tar.gz musl
rm -r musl
cd ..

# Build musl-gcc toolchain
cat > config.mak <<EOF
TARGET = x86_64-linux-musl
OUTPUT = ${INSTALL_DIR}
COMMON_CONFIG += CFLAGS="-fPIC" CXXFLAGS="-fPIC" LDFLAGS="-pie"
MUSL_VER = ${MUSL_VER}
EOF
make -j
make install

# Remove all source code and build files
rm -rf ${BUILD_DIR}

# Link the toolchain directory
ln -s -f ${INSTALL_DIR} /usr/local/occlum
