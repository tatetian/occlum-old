MAIN_MAKEFILE := $(firstword $(MAKEFILE_LIST))
INCLUDE_MAKEFILE := $(lastword $(MAKEFILE_LIST))
CUR_DIR := $(shell dirname $(realpath $(MAIN_MAKEFILE)))
PROJECT_DIR := $(realpath $(CUR_DIR)/../../)
BUILD_DIR := $(PROJECT_DIR)/build

TEST_NAME := $(shell basename $(CUR_DIR))
IMAGE_DIR := $(BUILD_DIR)/test/image
BIN := $(IMAGE_DIR)/bin/$(TEST_NAME)

C_SRCS := $(wildcard *.c)
C_OBJS := $(addprefix $(BUILD_DIR)/test/obj/$(TEST_NAME)/,$(C_SRCS:%.c=%.o))
CXX_SRCS := $(wildcard *.cc)
CXX_OBJS := $(addprefix $(BUILD_DIR)/test/obj/$(TEST_NAME)/,$(CXX_SRCS:%.cc=%.o))

ALL_BUILD_SUBDIRS := $(sort $(patsubst %/,%,$(dir $(BIN) $(C_OBJS) $(CXX_OBJS))))

CC := /opt/occlum/toolchains/gcc/bin/occlum-gcc
CXX := /opt/occlum/toolchains/gcc/bin/occlum-g++

C_FLAGS = -Wall -I../include -O2 -fPIC $(EXTRA_C_FLAGS)
LINK_FLAGS = $(C_FLAGS) -pie $(EXTRA_LINK_FLAGS)

.PHONY: all test test-native clean

#############################################################################
# Build
#############################################################################

all: $(ALL_BUILD_SUBDIRS) $(BIN) $(DEPS_FILE)

$(ALL_BUILD_SUBDIRS):
	@mkdir -p $@

# Compile C/C++ test program
#
# When compiling programs, we do not use CXX if we're not compilng any C++ files.
# This ensures C++ libraries are only linked and loaded for C++ programs, not C 
# programs.
$(BIN): $(C_OBJS) $(CXX_OBJS)
	@if [ -z $(CXX_OBJS) ] ; then \
		$(CC) $^ $(LINK_FLAGS) -o $(BIN); \
	else \
		$(CXX) $^ $(LINK_FLAGS) -o $(BIN); \
	fi ;
	@echo "LINK => $@"

$(BUILD_DIR)/test/obj/$(TEST_NAME)/%.o: %.c
	@$(CC) $(C_FLAGS) -c $< -o $@
	@echo "CC <= $@"

$(BUILD_DIR)/test/obj/$(TEST_NAME)/%.o: %.cc
	@$(CXX) $(C_FLAGS) -c $< -o $@
	@echo "CXX <= $@"
#############################################################################
# Test
#############################################################################

test:
	@cd $(BUILD_DIR)/test && \
		$(PROJECT_DIR)/build/bin/occlum run /bin/$(TEST_NAME) $(BIN_ARGS)

test-native:
	@LD_LIBRARY_PATH=/usr/local/occlum/lib cd $(IMAGE_DIR) && ./bin/$(TEST_NAME) $(BIN_ARGS)

#############################################################################
# Misc
#############################################################################

clean:
	@-$(RM) -f $(BIN) $(DEPS_FILE) $(C_OBJS) $(CXX_OBJS)
