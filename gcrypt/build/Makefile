# Support cross compilation to/from 32/64 bit.
ARCH := $(word 1, $(subst -, ,$(TARGET)))

ifeq ($(ARCH),i686)
CFLAGS += -m32
else
CFLAGS += -m64
endif

export PATH := $(OUT_DIR)/bin:$(PATH)

all: copy_libs

copy_libs: libgpgerror libgcrypt
	cp -fla "$(OUT_DIR)/lib/." "$(OUT_DIR)/"

$(OUT_DIR):
	mkdir -p $(OUT_DIR)

libgpgerror: $(OUT_DIR)
	cd libgpg-error && automake --add-missing || true
	cd libgpg-error && ./autogen.sh --force
	cd libgpg-error && ./configure --enable-maintainer-mode\
                                 --enable-static\
                                 --prefix=$(OUT_DIR)\
                                 --with-pic
	make -C libgpg-error
	make -C libgpg-error install
	cd libgpg-error && git clean -xdf . && git reset --hard || exit 0

libgcrypt: libgpgerror $(OUT_DIR)
	cd libgcrypt && automake --add-missing || true
	cd libgcrypt && ./autogen.sh --force
	cd libgcrypt && ./configure --enable-maintainer-mode\
                              --enable-static\
                              --with-libgpg-error-prefix=$(OUT_DIR)\
                              --prefix=$(OUT_DIR)\
                              --with-pic
	make -C libgcrypt
	make -C libgcrypt install
	cd libgcrypt && git clean -xdf . && git reset --hard || exit 0
