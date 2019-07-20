#!/bin/sh
mkdir openmp_src
cd openmp_src
git clone https://github.com/llvm-mirror/openmp .
mkdir build
cd build
cmake ../ -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr/local/occlum -DCMAKE_C_COMPILER=musl-clang -DCMAKE_CXX_COMPILER=musl-clang++ -DOPENMP_TEST_C_COMPILER=gcc -DOPENMP_TEST_CXX_COMPILER=g++
make -j
sudo make install

