# Use OpenMP in SGX with Occlum

Step 1: Download, compile and install OpenMP
```
./build_openmp.sh
```
When completed, the headers and libraries of OpenMP will be installed at `/usr/local/occlum`.

Step 2: Compile a Hello World for OpenMP (see `openmp_hello.c`)
```
make
```

Step 3: Run the Hello World for OpenMP, which prints "Hello World" in multiple concurrent threads.
```
make test
```
