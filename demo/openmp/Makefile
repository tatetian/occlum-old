CUR_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))
PROJECT_DIR := $(realpath $(CUR_DIR)/../../)
BIN := openmp_hello
CC := musl-clang
CFLAGS := -pie -fPIC -lomp

.PHONY: all clean

all: $(BIN)

$(BIN): *.c
	$(CC) -o $(BIN) $(CFLAGS) $<

test: sefs
	./pal $(BIN) $(BIN_ARGS)

sefs: $(BIN) pal libocclum.signed.so
	@$(RM) -rf fs
	@mkdir fs
	@mkdir fs/lib
	@cp /lib/ld-musl-x86_64.so.1 fs/ld.so
	@cp /usr/local/occlum/lib/libomp.so fs/lib/
	@cp $(BIN) fs/
	@cd $(PROJECT_DIR)/deps/sefs/sefs-fuse/bin/ && \
		./app \
			$(CUR_DIR)/sefs \
			$(CUR_DIR)/fs \
			zip
	@echo "SEFS => $@"

pal: $(PROJECT_DIR)/src/pal/pal
	@cp $< pal

libocclum.signed.so: $(PROJECT_DIR)/src/libos/libocclum.signed.so
	@cp $< libocclum.signed.so

clean:
	rm -rf $(BIN) fs sefs pal libocclum.signed.so
