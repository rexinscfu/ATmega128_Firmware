# ATmega128 Firmware

Rust-based firmware for the MikroElektronika BigAVR2 development board featuring the ATmega128 microcontroller.

## 🔧 Hardware Requirements

- MikroElektronika BigAVR2 board
- ATmega128 running at 16MHz
- USB-UART adapter for programming
- Debian 12 (or compatible) development system

## 📦 Dependencies

- Rust nightly toolchain
- AVR-GCC toolchain
- AVRDUDE for flashing

## 🚀 Quick Start

1. Install system dependencies:
```bash
sudo apt update
sudo apt install avr-gcc gcc-avr avr-libc avrdude
```

2. Setup Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
rustup target add avr-atmega128
```

3. Build the project:
```bash
cargo build --release --target avr-atmega128.json
```

4. Flash to board (assuming USB-UART on /dev/ttyUSB0):
```bash
avrdude -p m128 -c arduino -P /dev/ttyUSB0 -b 115200 -U flash:w:target/avr-atmega128/release/atmega128_firmware.elf
```

## 🏗️ Project Structure

```
/src
  /drivers         → Board-specific drivers
  /hal             → Hardware abstraction layer
  /application     → Application logic
  /rtos            → Real-time task scheduler
  /config          → Build configuration
/tests             → Hardware-in-loop tests
/hw               → Hardware documentation
/scripts          → Build and flash utilities
```

## 🛡️ Safety Considerations

This project uses `unsafe` Rust in the following contexts:
- Direct hardware register access in HAL layer
- Interrupt handlers
- Critical sections for atomic operations

All unsafe blocks are carefully documented and encapsulated behind safe abstractions.

## 🔋 Power Management

The firmware implements several power-saving features:
- Sleep modes (Idle, Power-Down)
- Peripheral power control
- Clock gating

## 📝 Development Notes

- Uses zero-cost abstractions for HAL implementation
- Implements cycle-accurate timing using direct register access
- Defensive programming with Result types for error handling
- Static memory allocation only (#![no_std])

## 🐛 Debugging

1. Connect JTAG debugger
2. Use OpenOCD for debugging session:
```bash
openocd -f interface/jtagkey.cfg -f target/atmega128.cfg
```

## 📜 License

MIT License - See LICENSE file for details

## ⚠️ Known Issues

- ISR latency needs optimization in timer-based operations
- Power consumption in sleep mode higher than expected
- UART baud rate drift at high speeds

## 🤝 Contributing

1. Fork the repository
2. Create feature branch
3. Commit changes
4. Open pull request

Please follow Rust style guidelines and document unsafe blocks.