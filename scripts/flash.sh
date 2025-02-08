#!/bin/bash

# Flash script for ATmega128 firmware
# Usage: ./flash.sh [port] [baudrate]

PORT=${1:-/dev/ttyUSB0}
BAUD=${2:-115200}
TARGET_ELF="target/avr-atmega128/release/atmega128_firmware.elf"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}ATmega128 Firmware Flasher${NC}"
echo "Port: $PORT"
echo "Baud: $BAUD"

# Check if avrdude is installed
if ! command -v avrdude &> /dev/null; then
    echo -e "${RED}Error: avrdude not found${NC}"
    echo "Please install with: sudo apt install avrdude"
    exit 1
fi

# Check if target exists
if [ ! -f "$TARGET_ELF" ]; then
    echo -e "${RED}Error: Target ELF not found${NC}"
    echo "Please build first with: cargo build --release --target avr-atmega128.json"
    exit 1
fi

# Check if port exists
if [ ! -c "$PORT" ]; then
    echo -e "${RED}Error: Port $PORT not found${NC}"
    echo "Available ports:"
    ls -l /dev/ttyUSB*
    exit 1
fi

# Flash the firmware
echo -e "${YELLOW}Flashing firmware...${NC}"
avrdude -p m128 -c arduino -P $PORT -b $BAUD -U flash:w:$TARGET_ELF

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Flash successful!${NC}"
else
    echo -e "${RED}Flash failed!${NC}"
    exit 1
fi

# Verify the flash
echo -e "${YELLOW}Verifying flash...${NC}"
avrdude -p m128 -c arduino -P $PORT -b $BAUD -U flash:v:$TARGET_ELF

if [ $? -eq 0 ]; then
    echo -e "${GREEN}Verification successful!${NC}"
else
    echo -e "${RED}Verification failed!${NC}"
    exit 1
fi

exit 0 