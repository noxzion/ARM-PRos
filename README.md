<div align="center">

<h1>ARM-PRos Operating System</h1>

[![License](https://img.shields.io/badge/License-MIT-yellow?style=for-the-badge)](LICENSE.TXT)
[![Version](https://img.shields.io/badge/version-0.1-blue?style=for-the-badge)](docs/changes/v0.1.txt)

[![GitHub stars](https://img.shields.io/github/stars/PRoX2011/ARM-PRos?style=flat-square)](https://github.com/PRoX2011/ARM-PRos/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/PRoX2011/ARM-PRos?style=flat-square)](https://github.com/PRoX2011/ARM-PRos/commits)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-brightgreen?style=flat-square)](CONTRIBUTING.md)


**A minimalistic 64-bit operating system written in C and Assembly for ARMv8-A architecture**

<img src="docs/screenshots/terminal.png" width=75%>

</div>

## Overview

**ARM-PRos** is my experimental operating system project for the ARMv8-A architecture. It's practically bare bones right now, but if I find the strength to develop this project further, I'll try to turn it into something more. 

## Roadmap

- [x] UART Driver: Basic text output (puts, puthex, putc)
- [X] UART Input
- [X] String Library (`strcmp`, `strlen`, `atoi`)
- [X] Kernel Shell
- [X] Framebuffer
- [x] USB HID Keyboard Driver
- [x] Interrupt Controller (GIC)
- [x] Timer Driver
- [ ] Physical Memory Manager
- [ ] PCI Scanning
- [ ] FAT32 file system
- [ ] *.ELF programs
- [ ] Multitasking

## Building from Source

Install `aarch64-linux-gnu-gcc` (or `clang` if you wanna use `dcr`) compiler using your package manager, then run build script:

```bash
chmod +x build-linux.sh
./build-linux.sh
```

Or use [dcr](https://dcr.dexoron.su)

```bash
dcr build
```

## Runing ARM-PRos

Install `qemu-system-aarch64` emulator then run:

### Text Mode (Recommended - Full Keyboard Support)
```bash
./run-linux-text.sh
```
**Best for development** - Full keyboard input support via serial console.
Exit: `Ctrl+A` then `X`

### GTK Display Mode (Limited Keyboard)
```bash
./run-linux.sh
```
GTK window with display. For keyboard input, open another terminal:
```bash
telnet localhost 5555
```

Or use [dcr](https://dcr.dexoron.su):
```bash
dcr run
```

<div align="center">

**Made with ❤️ by PRoX**

</div>