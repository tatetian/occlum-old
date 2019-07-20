#include <stdlib.h>
#include <stdio.h>
#include <omp.h>

int main (int argc, char *argv[])
{
    // FIXME: this is not working
    // Configure OpenMP to use two threads via environment variables
    //putenv("OMP_NUM_THREADS=2");
    omp_set_num_threads(2);

    // Run two threads concurrently to print Hello World
    int nthreads, tid;
    #pragma omp parallel private(nthreads, tid)
    {
        // Master thread
        if ((tid = omp_get_thread_num()) == 0) {
            nthreads = omp_get_num_threads();
            printf("Total number of threads = %d\n", nthreads);
        }

        // Every thread
        printf("Hello World from thread #%d\n", tid);
    }
    return 0;
}
