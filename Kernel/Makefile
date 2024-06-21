
# CONFIG: Architecture to build for
ARCH ?= amd64

ifeq ($(ARCH),amd64)
    TRIPLE ?= x86_64-elf-
else ifeq ($(ARCH),x86)
    TRIPLE ?= i686-elf-
else
    $(error Unknown architecture $(ARCH))
endif


# Toolchain commands (can be overridden)
CARGO ?= cargo
RUSTC ?= rustc
LD := $(TRIPLE)ld
AS := $(TRIPLE)as
OBJDUMP := $(TRIPLE)objdump
OBJCOPY := $(TRIPLE)objcopy

# Object directory
OBJDIR := .obj/$(ARCH)/

LINKSCRIPT := arch/$(ARCH)/link.ld
TARGETSPEC := arch/$(ARCH)/target.json
# Compiler Options
LINKFLAGS := -T $(LINKSCRIPT)
LINKFLAGS += -Map $(OBJDIR)map.txt
LINKFLAGS += --gc-sections
LINKFLAGS += -z max-page-size=0x1000

RUSTFLAGS := --cfg arch__$(ARCH) -C soft-float
RUSTFLAGS += -C panic=abort
RUSTFLAGS += -Zpanic_abort_tests

# Objects
OBJS := start.o kernel.a
OBJS := $(OBJS:%=$(OBJDIR)%)
BIN := ../kernel.$(ARCH).bin

.PHONY: all clean PHONY

all: $(BIN)

clean:
	$(RM) -rf $(BIN) $(BIN).dsm $(OBJDIR)

test:
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) test -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem  --target=$(TARGETSPEC) --lib

# use grub instead of qemu's -kernel option
# use ide option
QEMUOPTS = -kernel ../kernel.amd64.bin -no-reboot -drive file=../fs.img,index=1,media=disk,format=raw
run: $(BIN)
	qemu-system-x86_64 -serial stdio $(QEMUOPTS)
debug: $(BIN)
	qemu-system-x86_64 -serial stdio -d int -s -S $(QEMUOPTS)
monitor: $(BIN)
	qemu-system-x86_64 -monitor stdio -d int -s -S $(QEMUOPTS)


# Final link command
$(BIN): $(OBJS) arch/$(ARCH)/link.ld
	$(LD) -o $@ $(LINKFLAGS) $(OBJS)
	$(OBJDUMP) -S $@ > $@.dsm
ifeq ($(ARCH),amd64)
	@mv $@ $@.elf64
	@$(OBJCOPY) $@.elf64 -F elf32-i386 $@
endif


# Compile rust kernel object
$(OBJDIR)kernel.a: PHONY Makefile $(TARGETSPEC)
	@mkdir -p $(dir $@)
	RUSTFLAGS="$(RUSTFLAGS)" $(CARGO) build -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem --target=$(TARGETSPEC)
	@cp --preserve target/target/debug/libkernel.a $@

# Compile architecture's assembly stub
$(OBJDIR)start.o: arch/$(ARCH)/start.S Makefile
	@mkdir -p $(dir $@)
	$(AS) $(ASFLAGS) -o $@ $<


# Include dependency files
-include $(OBJDIR)start.d
