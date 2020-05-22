![Occlum log](docs/images/logo.png | width=100)

##
[![All Contributors](https://img.shields.io/badge/all_contributors-7-orange.svg?style=flat-square)](CONTRIBUTORS.md)
[![Build Status](https://travis-ci.com/occlum/occlum.svg?branch=master)](https://travis-ci.com/occlum/occlum)

**NEWS:** Our paper _Occlum: Secure and Efficient Multitasking Inside a Single Enclave of Intel SGX_ has been accepted by [ASPLOS'20](https://asplos-conference.org/programs/). This research paper highlights the advantages of the single-address-space architecture adopted by Occlum and describes a novel in-enclave isolation mechanism that complements this approach. The paper can be found on [ACM Digital Library](https://dl.acm.org/doi/abs/10.1145/3373376.3378469) and [Arxiv](https://arxiv.org/abs/2001.07450).

Occlum is a *memory-safe*, *multi-process* library OS (LibOS) for [Intel SGX](https://software.intel.com/en-us/sgx). As a LibOS, it enables *legacy* applications to run on SGX with *little or even no modifications* of source code, thus protecting the confidentiality and integrity of user workloads transparently.

Occlum has the following salient features:

  * **Efficient multitasking.** Occlum offers _light-weight_ LibOS processes: they are light-weight in the sense that all LibOS processes share the same SGX enclave. Compared to the heavy-weight, per-enclave LibOS processes, Occlum's light-weight LibOS processes is up to _1,000X faster_ on startup and _3X faster_ on IPC. In addition, Occlum offers an optional _multi-domain [Software Fault Isolation](http://www.cse.psu.edu/~gxt29/papers/sfi-final.pdf) scheme_ to isolate the Occlum LibOS processes if needed.
  * **Multiple file system support.** Occlum supports various types of file systems, e.g., _read-only hashed FS_ (for integrity protection), _writable encrypted FS_ (for confidentiality protection), _untrusted host FS_ (for convenient data exchange between the LibOS and the host OS).
  * **Memory safety.** Occlum is the _first_ SGX LibOS written in a memory-safe programming language ([Rust](https://www.rust-lang.org/)). Thus, Occlum is much less likely to contain low-level, memory-safety bugs and is more trustworthy to host security-critical applications.
  * **Ease-of-use.** Occlum provides user-friendly build and command-line tools. Running applications on Occlum inside SGX enclaves can be as simple as only typing several shell commands (see the next section).

## Introduction

### Hello Occlum

If you were to write an SGX Hello World project using some SGX SDK, the project would consist of hundreds of lines of code. And to do that, you have to spend a great deal of time to learn the APIs, the programming model, and the built system of the SGX SDK.

Thanks to Occlum, you can be freed from writing any extra SGX-aware code and only need to type some simple commands to protect your application with SGX transparently---in four easy steps.

**Step 1. Compile the user program with the Occlum toolchain (e.g., `occlum-gcc`)**
```
$ occlum-gcc -o hello_world hello_world.c
$ ./hello_world
Hello World
```
Note that the Occlum toolchain is not cross-compiling in the traditional sense: the binaries built by the Occlum toolchain is also runnable on Linux. This property makes it convenient to compile, debug, and test user programs intended for Occlum.

**Step 2. Initialize a directory as the Occlum context via `occlum init`**
```
$ mkdir occlum_context && cd occlum_context
$ occlum init
```
The `occlum init` command creates in the current working directory a new directory named `.occlum`, which contains the compile-time and run-time state of Occlum. Each Occlum context should be used for a single instance of an application; multiple applications or different instances of a single application should use different Occlum contexts.

**Step 3. Generate a secure Occlum FS image and Occlum SGX enclave via `occlum build`**
```
$ cp ../hello_world image/bin/
$ occlum build
```
The content of the `image` directory is initialized by the `occlum init` command. The structure of the `image` directory mimics that of an ordinary UNIX FS, containing directories like `/bin`, `/lib`, `/root`, `/tmp`, etc. After copying the user program `hello_world` into `image/bin/`, the `image` directory is packaged by the `occlum build` command to generate a secure Occlum FS image as well as the Occlum SGX enclave.

For platforms that don't support SGX, it is also possible to run Occlum in SGX simulation mode. To switch to the simulation mode, `occlum build` command must be given an extra argument or an environment variable as shown below:
```
$ occlum build --sgx-mode SIM
```
or
```
$ SGX_MODE=SIM occlum build
```

**Step 4. Run the user program inside an SGX enclave via `occlum run`**
```
$ occlum run /bin/hello_world
Hello World!
```
The `occlum run` command starts up an Occlum SGX enclave, which, behind the scene, verifies and loads the associated Occlum FS image, spawns a new LibOS process to execute `/bin/hello_world`, and eventually prints the message.

### Config Occlum

Occlum can be configured easily via a config file named `Occlum.json`, which is generated by the `occlum init` command in the Occlum context directory. The user can modify `Occlum.json` to config Occlum. A sample of `Occlum.json` is shown below. Some comments are added to provide a brief explanation.
```
{
    // Virtual memory
    "vm": {
        // The size of memory available for use by LibOS processes
        "user_space_size": "128MB"
    },
    // Process
    "process": {
        // The stack size of the "main" thread
        "default_stack_size": "4MB",
        // The max size of memory allocated by brk syscall
        "default_heap_size": "16MB",
        // The max size of memory by mmap syscall
        "default_mmap_size": "32MB"
    },
    // Environment variables
    //
    // This gives a list of trusted environment variables for the "root"
    // process started by `occlum run` command.
    "env": [
        "OCCLUM=yes"
    ],
    // Entry points
    //
    // Entry points specify all valid path prefixes for <path> in `occlum run
    // <path> <args>`. This prevents outside attackers from executing arbitrary
    // commands inside an Occlum-powered enclave.
    "entry_points": [
        "/bin"
    ],
    // Mount points and their file systems
    //
    // Limitation: configuring mount points by modifying this config file is not
    // supported at the moment. The default configuration is shown below.
    "mount": [
        {
            "target": "/",
            "type": "sefs",
            "source": "./image",
            "options": {
                "integrity_only": true
            }
        },
        {
            "target": "/root",
            "type": "sefs"
        },
        {
            "target": "/host",
            "type": "hostfs",
            "source": "."
        },
        {
            "target": "/tmp",
            "type": "ramfs"
        }
    ]
}
```

### Try Experimental Features

Occlum has added several new experimental commands, which provide a more container-like experience to users, as shown below:
```
occlum init
occlum build
occlum start
occlum exec <cmd1> <args1>
occlum exec <cmd2> <args2>
occlum exec <cmd3> <args3>
occlum stop
```

## How to Use?

We have built and tested Occlum on Ubuntu 18.04 with or without hardware SGX support (if the CPU does not support SGX, Occlum can be run in the SGX simulation mode). To give Occlum a quick try, one can use the Occlum Docker image by following the steps below:

Step 1-3 are to be done on the host OS (Linux):

1. Install [Intel SGX driver for Linux](https://github.com/intel/linux-sgx-driver), which is required by Intel SGX SDK.

2. Install [enable_rdfsbase kernel module](https://github.com/occlum/enable_rdfsbase), which enables Occlum to use `rdfsbase`-family instructions in enclaves.

3. Run the Occlum Docker container, which has Occlum and its demos preinstalled:
    ```
    docker run -it --device /dev/isgx occlum/occlum:0.12.0-ubuntu18.04
    ```

Step 4-5 are to be done on the guest OS running inside the Docker container:

4. (Optional) Try the sample code of Intel SGX SDK to make sure that SGX is working
    ```
    cd /opt/intel/sgxsdk/SampleCode/SampleEnclave && make && ./app
    ```
5. Check out Occlum's demos preinstalled at `/root/demos`, whose README can be found [here](demos/README.md). Or you can try to build and run your own SGX-protected applications using Occlum as shown in the demos.

## How to Build and Install?

To build Occlum from the latest source code, do the following steps in an Occlum Docker container (which can be prepared as shown in the last section):

1. Download the latest source code of Occlum
    ```
    mkdir occlum && cd occlum
    git clone https://github.com/occlum/occlum .
    ```
2. Prepare the submodules and tools required by Occlum.
    ```
    make submodule
    ```
3. Compile and test Occlum
    ```
    make
    make test
    ```

    For platforms that don't support SGX
    ```
    SGX_MODE=SIM make
    SGX_MODE=SIM make test
    ```
4. Install Occlum
    ```
    make install
    ```
   which will install the `occlum` command-line tool and other files at `/opt/occlum`.

The Occlum Dockerfile can be found at [here](tools/docker/Dockerfile). Use it to build the container directly or read it to see the dependencies of Occlum.

## How to Build Occlum-Compatible Executable Binaries?

Occlum supports running any executable binaries that are 1) based on [musl libc](https://www.musl-libc.org/) and 2) position independent. We chose musl libc instead of Glibc since the codebase of musl libc is 10X smaller than Glibc, which means a much smaller Trusted Computing Base (TCB) and attack surface. We argue this is an important consideration for Occlum, which targets security-critical apps running inside SGX enclaves.

The two aforementioned requirements are not only satisfied by the Occlum toolchain, but also the native toolchains from some Linux distributions, e.g., [Alpine Linux](https://www.alpinelinux.org/). We think Alpine Linux, a popular Linux distribution that emphasizes simplicity and security, is a natural fit for Occlum. We have provided demos (see [Python](demos/python/)) to run unmodified apps from [Alpine Linux packages](https://pkgs.alpinelinux.org/packages).

## How to Debug?

To debug an app running upon Occlum, one can harness Occlum's builtin support for GDB via `occlum gdb` command. More info can be found [here](demos/gdb_support/).

If the cause of a problem does not seem to be the app but Occlum itself, then one can take a glimpse into the inner workings of Occlum by checking out its log. Occlum's log level can be adjusted through `OCCLUM_LOG_LEVEL` environment variable. It has six levels: `off`, `error`, `warn`, `debug`, `info`, and `trace`. The default value is `off`, i.e., showing no log messages at all. The most verbose level is `trace`.

## How to Build and Run Release-Mode Enclaves?

By default, the `occlum build` command builds and signs enclaves in debug mode. These SGX debug-mode enclaves are intended for development and testing purposes only. For production usage, the enclaves must be signed by a key acquired from Intel (a restriction that will be lifted in the future when Flexible Launch Control is ready) and run with SGX debug support disabled.

Occlum has built-in support for both building and running enclaves in release mode. The commands are shown below:
```
$ occlum build --sign-key=<path_to/your_key.pem>
$ OCCLUM_RELEASE_ENCLAVE=yes occlum run <prog_path> <prog_args>
```

Ultimately, whether an enclave is running in the release mode should be checked and judged by a trusted client through remotely attesting the enclave. See the remote attestation demo [here](demos/remote_attestation).

## What is the Implementation Status?

Occlum is being actively developed. We now focus on implementing more system calls and additional features required in the production environment.

While this project is still not mature or stable (we are halfway through reaching version 1.0.0), we have used Occlum to port many real-world applications (like Tensorflow Lite, XGBoost, GCC, Lighttpd, etc.) to SGX with little or no source code modifications. We believe that the current implementation of Occlum is already useful to many users and ready to be deployed in some use cases.

## How about the Internal Working?

The high-level architecture of Occlum is summarized in the figure below:

![Arch Overview](docs/images/arch_overview.png)

## Why the Name?

The project name Occlum stems from the word *Occlumency* coined in Harry Potter series by J. K. Rowling. In *Harry Potter and the Order of Phoenix*, Occlumency is described as:

> The magical defence of the mind against external penetration. An obscure branch of magic, but a highly useful one... Used properly, the power of Occlumency will help shield you from access or influence.

The same thing can be said for Occlum, not for the mind, but for the program:

> The magical defence of the program against external penetration. An obscure branch of technology, but a highly useful one... Used properly, the power of Occlum will help shield your program from access or influence.

Of course, Occlum must be run on Intel x86 CPUs with SGX support to do its magic.

## Contributors

This project follows the [all-contributors](https://allcontributors.org) specification. Contributions of any kind are welcome! We will publish contributing guidelines and accept pull requests after the project gets more stable.

Thanks go to [all these wonderful contributors to this project](CONTRIBUTORS.md).

## License

Occlum is released by Ant Financial under BSD License. See the copyright information [here](LICENSE).
