#include <string.h>
#include <stdio.h>
#include <elf.h>
#include <errno.h>
#include <spawn.h>
#include <sys/auxv.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <unistd.h>
#include "test.h"

// ============================================================================
// Helper structs & variables & functions
// ============================================================================

const char** g_argv;
int g_argc;

// Expected arguments are given by Makefile throught macro ARGC, ARG1, ARG2 and
// ARG3
const char* expect_argv[EXPECT_ARGC] = {
    "env",
    EXPECT_ARG1,
    EXPECT_ARG2,
    EXPECT_ARG3,
};

// Expected child arguments
const int child_argc = 2;
const char* child_argv[3] = {
    "env",
    "child",
    NULL
};

// Expected child environment variables
const char* child_envp[] = {
    "ENV_CHILD=ok",
    NULL
};

static int test_argv_val(const char** expect_argv) {
    for (int arg_i = 0; arg_i < g_argc; arg_i++) {
        const char* actual_arg = *(g_argv + arg_i);
        const char* expect_arg = *(expect_argv + arg_i);
        if (strcmp(actual_arg, expect_arg) != 0) {
            printf("ERROR: expect argument %d is %s, but given %s\n",
                    arg_i, expect_arg, actual_arg);
            return -1;
        }
    }
    return 0;
}

static int test_env_val(const char* expect_env_key, const char* expect_env_val) {
    const char* actual_env_val = getenv(expect_env_key);
    if (actual_env_val == NULL) {
        printf("ERROR: cannot find %s\n", expect_env_key);
        return -1;
    }
    if (strcmp(actual_env_val, expect_env_val) != 0) {
        printf("ERROR: environment variable %s=%s expected, but given %s\n",
                expect_env_key, expect_env_val, actual_env_val);
        return -1;
    }
    return 0;
}

// ============================================================================
// Test cases for argv
// ============================================================================

static int test_env_getargv() {
    // Test argc
    if (g_argc != EXPECT_ARGC) {
        printf("ERROR: expect %d arguments, but %d are given\n", EXPECT_ARGC, g_argc);
        throw_error("arguments count is not expected");
    }
    // Test argv
    if (test_argv_val(expect_argv) < 0) {
        throw_error("argument variables are not expected");
    }
    return 0;
}

// ============================================================================
// Test cases for aux
// ============================================================================

static int test_env_getauxval() {
    unsigned long page_size = getauxval(AT_PAGESZ);
    if (errno != 0 || page_size != 4096) {
        throw_error("auxilary vector does not pass correct the value");
    }

    return 0;
}

// ============================================================================
// Test cases for env
// ============================================================================

// The environment variables are specified in Occlum.json
static int test_env_getenv() {
    if (test_env_val("OCCLUM", "yes") < 0) {
        throw_error("get environment variable failed");
    }

    // Here we call getenv() again to make sure that
    // LibOS can handle several environment variables in Occlum.json correctly
    if (test_env_val("TEST", "true") < 0) {
        throw_error("get environment variable failed");
    }
    return 0;
}

static int test_env_set_child_env_and_argv() {
    int status, child_pid;
    int ret = posix_spawn(&child_pid,
                          "/bin/env", NULL, NULL,
                          (char *const *)child_argv,
                          (char *const *)child_envp);
    if (ret < 0) {
        throw_error("spawn process error");
    }
    printf("Spawn a child process with pid=%d\n", child_pid);
    ret = wait4(-1, &status, 0, NULL);
    if (ret < 0) {
        throw_error("failed to wait4 the child process");
    }
    if (!WIFEXITED(status)) {
        throw_error("test cases in child faild");
    }
    return 0;
}

// ============================================================================
// Child Test cases for argv
// ============================================================================

static int test_env_child_getargv() {
    // Test argc
    if (g_argc != child_argc) {
        printf("ERROR: expect %d arguments, but %d are given\n", child_argc, g_argc);
        throw_error("arguments count is not expected");
    }
    // Test argv
    if (test_argv_val(child_argv) < 0) {
        throw_error("argument variables are not expected");
    }
    return 0;
}

// ============================================================================
// Child Test cases for env
// ============================================================================

#define ENV_SIZE (256)
static int test_env_child_getenv() {
    char env_key[ENV_SIZE];
    char env_val[ENV_SIZE];
    for (int i = 0; child_envp[i] != NULL; ++i) {
        int num = sscanf(child_envp[i], "%[^=]=%s", env_key, env_val);
        if (num != 2) {
            throw_error("parse environment variable failed");
        }
        if (test_env_val(env_key, env_val) < 0) {
            throw_error("get environment variable failed");
        }
    }
    return 0;
}

// ============================================================================
// Test suite main
// ============================================================================

static test_case_t test_cases[] = {
    TEST_CASE(test_env_getargv),
    TEST_CASE(test_env_getauxval),
    TEST_CASE(test_env_getenv),
    TEST_CASE(test_env_set_child_env_and_argv),
};

static test_case_t child_test_cases[] = {
    TEST_CASE(test_env_getauxval),
    TEST_CASE(test_env_child_getargv),
    TEST_CASE(test_env_child_getenv),
};

int main(int argc, const char* argv[]) {
    // Save argument for test cases
    g_argc = argc;
    g_argv = argv;
    // Test argc
    if (getpid() > 1) {
        return test_suite_run(child_test_cases, ARRAY_SIZE(child_test_cases));
    } else  {
        return test_suite_run(test_cases, ARRAY_SIZE(test_cases));
    }
}
