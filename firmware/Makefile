# Конфигурация под STM32F405 (ARM Cortex-M4 с аппаратным FPU)
CC      = arm-none-eabi-gcc
OBJCOPY = arm-none-eabi-objcopy
CFLAGS  = -mcpu=cortex-m4 -mthumb -mfpu=fpv4-sp-d16 -mfloat-abi=hard \
          -Os -std=c11 -ffreestanding -nostdlib -Wall -Wextra

all: firmware.bin

# 1. Трансляция YKL в чистый Bare-Metal C11
output.c: main.yk
	cargo run --bin apl -- build --release --bare-metal main.yk

# 2. Компиляция под ARM Cortex-M4 и упаковка в .bin для программатора ST-Link
firmware.elf: output.c startup_stm32.s stm32f405.ld
	$(CC) $(CFLAGS) -T stm32f405.ld startup_stm32.s output.c -o firmware.elf

firmware.bin: firmware.elf
	$(OBJCOPY) -O binary firmware.elf firmware.bin
	arm-none-eabi-size firmware.elf

# 3. Прошивка реальной платы по USB через st-flash
flash: firmware.bin
	st-flash write firmware.bin 0x8000000

# 4. Запуск без реальной платы в симуляторе QEMU
qemu: firmware.elf
	qemu-system-arm -machine netduinoplus2 -nographic -kernel firmware.elf
