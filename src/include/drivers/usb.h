#ifndef DRIVERS_USB_H
#define DRIVERS_USB_H

#include <stdint.h>

/* Implemented in Rust (rust-usb staticlib) */

/* DWC2 USB Controller Initialization */
void usb_init(void);

/* USB Host Controller Device Enumeration */
void usb_enumerate_device(void);

/* Check if USB keyboard is connected (returns 1 if yes) */
int usb_keyboard_connected(void);

/* Get keyboard input - returns ASCII character or 0 */
unsigned char usb_keyboard_getc(void);

#endif /* DRIVERS_USB_H */
