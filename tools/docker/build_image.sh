#!/bin/bash
script_dir="$( cd "$( dirname "${BASH_SOURCE[0]}"  )" >/dev/null 2>&1 && pwd )"
RED=$(tput setaf 1)
NC=$(tput sgr0)

set -e

if [[ ( "$#" < 2 ) ]] ; then
    echo "${RED}error:${NC} input is invalid"
    echo ""
    echo "Usage:"
    echo "    build_image.sh <occlum_name> <OS_name>"
    echo ""
    echo "Occlum_name:"
    echo "    an arbitrary input (e.g., latest and 0.8.0)"
    echo ""
    echo "OS_name:"
    echo "    ubuntu16.04    Build an image from ubuntu 16.04"
    echo "    centos7.2      Build an image from centos 7.2"
    echo ""
    echo "The name of the output image will be occlum/occlum:<occlum_name>-<OS_name>."
    exit 1
fi

occlum_name=$1
OS_name=$2

if [ "$OS_name" != "ubuntu16.04" ] && [ "$OS_name" != "centos7.2" ];then
    echo "${RED}error:${NC} no such OS_name: '$OS_name'"
    echo "OS_name:"
    echo "    ubuntu16.04    Build an image from ubuntu 16.04"
    echo "    centos7.2      Build an image from centos 7.2"
    exit 1
fi

cd "$script_dir/.."
docker build -f "$script_dir/Dockerfile.$OS_name" -t occlum/occlum:$occlum_name-$OS_name .
