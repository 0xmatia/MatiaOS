###################################################
# File: Makefile
# Author: Elad Matia
# Taken from https://raw.githubusercontent.com/rust-embedded/rust-raspberrypi-OS-tutorials/master/01_wait_forever/Makefile
# Created: 09/10/2021
####################################################

# Colored output functions
## cyan
define colorecho
      @tput setaf 6
      @echo $1
      @tput sgr0
endef

# Default to the RPi3.
BSP ?= rpi3

##--------------------------------------------------------------------------------------------------
## Hardcoded configuration values
##--------------------------------------------------------------------------------------------------

# BSP-specific arguments.
ifeq ($(BSP),rpi3)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    LOADER_BIN        = loader.img
    QEMU_BINARY       = qemu-system-aarch64
    QEMU_MACHINE_TYPE = raspi3b
    QEMU_RELEASE_ARGS = -serial stdio -display none
    OBJDUMP_BINARY    = aarch64-unknown-linux-gnu-objdump
    NM_BINARY         = aarch64-unknown-linux-gnu-nm
    READELF_BINARY    = aarch64-unknown-linux-gnu-readelf
    KERNEL_LD_FILE    = matiaos/src/bsp/raspberrypi/kernel.ld 
    LOADER_LD_FILE    = loader/src/bsp/raspberrypi/loader.ld 
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a53 
else ifeq ($(BSP),rpi4)
    TARGET            = aarch64-unknown-none-softfloat
    KERNEL_BIN        = kernel8.img
    LOADER_BIN        = loader.img
    QEMU_BINARY       = qemu-system-aarch64
    QEMU_MACHINE_TYPE =
    QEMU_RELEASE_ARGS = -d in_asm -display none
    OBJDUMP_BINARY    = aarch64-unknown-linux-gnu-objdump
    NM_BINARY         = aarch64-unknown-linux-gnu-nm
    READELF_BINARY    = aarch64-unknown-linux-gnu-readelf
    KERNEL_LD_FILE    = matiaos/src/bsp/raspberrypi/kernel.ld 
    LOADER_LD_FILE    = loader/src/bsp/raspberrypi/loader.ld 
    RUSTC_MISC_ARGS   = -C target-cpu=cortex-a72
endif

QEMU_MISSING_STRING = "This board is not yet supported for QEMU."

# Export for build.rs.
export KERNEL_LD_FILE
export LOADER_LD_FILE

DEVICE = /dev/ttyUSB0
BAUDRATE = 115200

KERNEL_ELF = target/$(TARGET)/release/kernel
LOADER_ELF = target/$(TARGET)/release/loader
PUSHER_ELF = target/release/pusher
 
##--------------------------------------------------------------------------------------------------
## Command building blocks
##--------------------------------------------------------------------------------------------------
RUSTFLAGS = $(RUSTC_MISC_ARGS) -D missing_docs -D warnings

# for conditional compiling (rpi3, rpi4 etc...)
FEATURES      = --features bsp_$(BSP) 
COMPILER_ARGS = --target=$(TARGET) \
    $(FEATURES)                    \
    --release

CARGO_CMD   = cargo build $(COMPILER_ARGS)
DOC_CMD     = cargo doc $(COMPILER_ARGS) --workspace --exclude pusher
CLIPPY_CMD  = cargo clippy $(COMPILER_ARGS)
CHECK_CMD   = cargo check $(COMPILER_ARGS) --workspace --exclude pusher

OBJCOPY_CMD = rust-objcopy \
    --strip-all            \
    -O binary
 
EXEC_QEMU = $(QEMU_BINARY) -M $(QEMU_MACHINE_TYPE)

##--------------------------------------------------------------------------------------------------
## Targets
##--------------------------------------------------------------------------------------------------

# phony target: target that aren't asociated with any file
.PHONY: all pusher dummy kernel loader $(KERNEL_ELF) $(KERNEL_BIN) $(LOADER_ELF) $(LOADER_BIN) doc qemu clippy clean readelf objdump nm check

dummy:
	$(info No target selected)

all: kernel loader
kernel: $(KERNEL_BIN)
loader: $(LOADER_BIN)

##------------------------------------------------------------------------------
## Build the kernel ELF
##------------------------------------------------------------------------------
$(KERNEL_ELF):
	$(call colorecho, "Compiling kernel - $(BSP)")
	@RUSTFLAGS="-C link-arg=-T$(KERNEL_LD_FILE) $(RUSTFLAGS)" $(CARGO_CMD) -p matiaos

##------------------------------------------------------------------------------
## Build the stripped kernel binary
##------------------------------------------------------------------------------
$(KERNEL_BIN): $(KERNEL_ELF)
	@$(OBJCOPY_CMD) $(KERNEL_ELF) $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Build the loader ELF
##------------------------------------------------------------------------------
$(LOADER_ELF):
	$(call colorecho, "Compiling loader - $(BSP)")
	@RUSTFLAGS="-C link-arg=-T$(LOADER_LD_FILE) $(RUSTFLAGS)" $(CARGO_CMD) -p loader

##------------------------------------------------------------------------------
## Build the stripped loader binary
##------------------------------------------------------------------------------
$(LOADER_BIN): $(LOADER_ELF)
	@$(OBJCOPY_CMD) $(LOADER_ELF) $(LOADER_BIN)
##------------------------------------------------------------------------------
## Build the documentation - no pusher
##------------------------------------------------------------------------------
doc:
	$(call colorecho, "Generating docs") 
	$(DOC_CMD) --document-private-items --open

##------------------------------------------------------------------------------
## Run the kernel in QEMU
##------------------------------------------------------------------------------
ifeq ($(QEMU_MACHINE_TYPE),) # QEMU is not supported for the board.

qemu:
	$(call colorecho, "$(QEMU_MISSING_STRING)")

else # QEMU is supported.

qemu: $(KERNEL_BIN)
	$(call colorecho, "Launching QEMU")
	$(EXEC_QEMU) $(QEMU_RELEASE_ARGS) -kernel $(KERNEL_BIN)
endif

##------------------------------------------------------------------------------
## Build and run pusher
##------------------------------------------------------------------------------

pusher: $(KERNEL_BIN)
	$(call colorecho, "Compiling pusher")
	cargo build --release -p pusher

	$(call colorecho, "Running pusher")
	sudo $(PUSHER_ELF) $(DEVICE) $(BAUDRATE) $(KERNEL_BIN)

##------------------------------------------------------------------------------
## Check project
##------------------------------------------------------------------------------

check:
	$(call colorecho, "Checking MatiaOS")
	$(CHECK_CMD) 

##------------------------------------------------------------------------------
## Run clippy
##------------------------------------------------------------------------------
clippy:
	@RUSTFLAGS="$(RUSTFLAGS_PEDANTIC)" $(CLIPPY_CMD)

##------------------------------------------------------------------------------
## Clean
##------------------------------------------------------------------------------
clean:
	$(call colorecho, "Cleaning target $(KERNEL_BIN) $(LOADER_BIN)")
	rm -rf target $(KERNEL_BIN) $(LOADER_BIN)

