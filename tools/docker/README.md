# Build docker image for occlum

This folder contains scripts and Dockerfiles for users to build docker images
for occlum. The images will contain necessary softwares to use and develop
occlum.

To start, run:
```
./build_image.sh <occlum_name> <OS_name>
```

The name of the output image will be:
```
occlum/occlum:<occlum_name>-<OS_name>
```

Users can specify the <strong>occlum_name</strong> (e.g., lastest and 0.8.0) as they want. 
The supported <strong>OS_name</strong>s include ubuntu16.04 and centos7.2.
