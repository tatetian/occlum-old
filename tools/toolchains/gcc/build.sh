#!/bin/bash
THIS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}"  )" >/dev/null 2>&1 && pwd )"
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

# Let musl-cross-make build for x86-64 Linux
TARGET=x86_64-linux-musl
# We will check out the branch ${MUSL_VER} from ${MUSL_REPO}
MUSL_REPO=https://github.com/occlum/musl
MUSL_VER=1.1.20
# We will use this version of GCC
GCC_VER=8.3.0

# This patch replaces syscall instruction with libc's syscall wrapper
cp ${THIS_DIR}/0014-libgomp-*.diff patches/gcc-${GCC_VER}/

# Build musl-gcc toolchain for Occlum
cat > config.mak <<EOF
TARGET = ${TARGET}
OUTPUT = ${INSTALL_DIR}
COMMON_CONFIG += CFLAGS="-fPIC" CXXFLAGS="-fPIC" LDFLAGS="-pie"

GCC_VER = ${GCC_VER}

MUSL_VER = git-${MUSL_VER}
MUSL_REPO = ${MUSL_REPO}
EOF
make
make install

# Remove all source code and build files
rm -rf ${BUILD_DIR}

# Link the toolchain directory
ln -sf ${INSTALL_DIR}/bin/${TARGET}-gcc ${INSTALL_DIR}/bin/occlum-gcc
ln -sf ${INSTALL_DIR}/bin/${TARGET}-g++ ${INSTALL_DIR}/bin/occlum-g++
ln -sf ${INSTALL_DIR}/bin/${TARGET}-ld ${INSTALL_DIR}/bin/occlum-ld
ln -sf ${INSTALL_DIR} /usr/local/occlum
