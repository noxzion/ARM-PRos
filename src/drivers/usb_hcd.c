#include <drivers/usb.h>
#include <drivers/uart.h>
#include <string.h>

/* USB Device Descriptors and Structures */
typedef struct {
    uint8_t bLength;
    uint8_t bDescriptorType;
    uint16_t bcdUSB;
    uint8_t bDeviceClass;
    uint8_t bDeviceSubClass;
    uint8_t bDeviceProtocol;
    uint8_t bMaxPacketSize0;
    uint16_t idVendor;
    uint16_t idProduct;
    uint16_t bcdDevice;
    uint8_t iManufacturer;
    uint8_t iProduct;
    uint8_t iSerialNumber;
    uint8_t bNumConfigurations;
} __attribute__((packed)) usb_device_descriptor_t;

typedef struct {
    uint8_t bLength;
    uint8_t bDescriptorType;
    uint16_t wTotalLength;
    uint8_t bNumInterfaces;
    uint8_t bConfigurationValue;
    uint8_t iConfiguration;
    uint8_t bmAttributes;
    uint8_t bMaxPower;
} __attribute__((packed)) usb_config_descriptor_t;

typedef struct {
    uint8_t bLength;
    uint8_t bDescriptorType;
    uint8_t bInterfaceNumber;
    uint8_t bAlternateSetting;
    uint8_t bNumEndpoints;
    uint8_t bInterfaceClass;
    uint8_t bInterfaceSubClass;
    uint8_t bInterfaceProtocol;
    uint8_t iInterface;
} __attribute__((packed)) usb_interface_descriptor_t;

typedef struct {
    uint8_t bLength;
    uint8_t bDescriptorType;
    uint8_t bEndpointAddress;
    uint8_t bmAttributes;
    uint16_t wMaxPacketSize;
    uint8_t bInterval;
} __attribute__((packed)) usb_endpoint_descriptor_t;

/* USB HID Report Descriptor */
typedef struct {
    uint8_t bLength;
    uint8_t bDescriptorType;
    uint16_t wDescriptorLength;
} __attribute__((packed)) usb_hid_descriptor_t;

/* Setup Packet */
typedef struct {
    uint8_t bmRequestType;
    uint8_t bRequest;
    uint16_t wValue;
    uint16_t wIndex;
    uint16_t wLength;
} __attribute__((packed)) usb_setup_packet_t;

/* DWC2 Host Channel Transfer */
typedef struct {
    usb_setup_packet_t setup;
    uint8_t *data_buffer;
    uint16_t data_length;
    uint8_t toggle;
    uint8_t pid;
} usb_transfer_t;


/* USB Request Types */
#define USB_REQ_GET_DESCRIPTOR      0x06
#define USB_REQ_SET_ADDRESS         0x05
#define USB_REQ_SET_CONFIGURATION   0x09
#define USB_REQ_GET_DESCRIPTOR      0x06
#define USB_REQ_GET_REPORT          0x01

/* Descriptor Types */
#define USB_DESC_DEVICE             0x01
#define USB_DESC_CONFIGURATION      0x02
#define USB_DESC_INTERFACE          0x04
#define USB_DESC_ENDPOINT           0x05
#define USB_DESC_HID                0x21
#define USB_DESC_REPORT             0x22

/* Request Type Bits */
#define USB_DIR_IN                  0x80
#define USB_DIR_OUT                 0x00
#define USB_TYPE_STANDARD           0x00
#define USB_TYPE_CLASS              0x20
#define USB_TYPE_VENDOR             0x40
#define USB_RECIP_DEVICE            0x00
#define USB_RECIP_INTERFACE         0x01
#define USB_RECIP_ENDPOINT          0x02

/* PID Types */
#define USB_PID_SETUP               0xD0
#define USB_PID_DATA0               0xC0
#define USB_PID_DATA1               0xD0
#define USB_PID_ACK                 0xD2
#define USB_PID_NAK                 0x5A
#define USB_PID_STALL               0x1E


void usb_hcd_send_setup(uint8_t addr, uint8_t ep, usb_setup_packet_t *setup)
{
    uart_puts("[HCD] Sending SETUP packet to device ");
    uart_puthex((uint64_t)addr);
    uart_puts(":");
    uart_puthex((uint64_t)ep);
    uart_puts(" bmRequestType=0x");
    uart_puthex((uint64_t)setup->bmRequestType);
    uart_puts(" bRequest=0x");
    uart_puthex((uint64_t)setup->bRequest);
    uart_puts("\n\r");
}

void usb_hcd_send_in(uint8_t addr, uint8_t ep, uint8_t *buffer, uint16_t length)
{
    (void)buffer;
    uart_puts("[HCD] Sending IN request from device ");
    uart_puthex((uint64_t)addr);
    uart_puts(":");
    uart_puthex((uint64_t)ep);
    uart_puts(" expecting ");
    uart_puthex((uint64_t)length);
    uart_puts(" bytes\n\r");
}

void usb_hcd_send_out(uint8_t addr, uint8_t ep, uint8_t *data, uint16_t length)
{
    (void)data;
    uart_puts("[HCD] Sending OUT to device ");
    uart_puthex((uint64_t)addr);
    uart_puts(":");
    uart_puthex((uint64_t)ep);
    uart_puts(" with ");
    uart_puthex((uint64_t)length);
    uart_puts(" bytes\n\r");
}

int usb_hcd_get_device_descriptor(uint8_t addr, usb_device_descriptor_t *desc)
{
    uart_puts("[HCD] Getting device descriptor for address ");
    uart_puthex((uint64_t)addr);
    uart_puts("\n\r");
    
    usb_setup_packet_t setup = {
        .bmRequestType = USB_DIR_IN | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
        .bRequest = USB_REQ_GET_DESCRIPTOR,
        .wValue = (USB_DESC_DEVICE << 8) | 0,
        .wIndex = 0,
        .wLength = sizeof(usb_device_descriptor_t)
    };
    
    usb_hcd_send_setup(addr, 0, &setup);
    usb_hcd_send_in(addr, 0, (uint8_t *)desc, sizeof(usb_device_descriptor_t));
    
    return 0;
}

int usb_hcd_set_address(uint8_t new_addr)
{
    uart_puts("[HCD] Setting device address to ");
    uart_puthex((uint64_t)new_addr);
    uart_puts("\n\r");
    
    usb_setup_packet_t setup = {
        .bmRequestType = USB_DIR_OUT | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
        .bRequest = USB_REQ_SET_ADDRESS,
        .wValue = new_addr,
        .wIndex = 0,
        .wLength = 0
    };
    
    usb_hcd_send_setup(0, 0, &setup);
    
    return 0;
}

int usb_hcd_set_configuration(uint8_t addr, uint8_t config)
{
    uart_puts("[HCD] Setting configuration ");
    uart_puthex((uint64_t)config);
    uart_puts(" for address ");
    uart_puthex((uint64_t)addr);
    uart_puts("\n\r");
    
    usb_setup_packet_t setup = {
        .bmRequestType = USB_DIR_OUT | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
        .bRequest = USB_REQ_SET_CONFIGURATION,
        .wValue = config,
        .wIndex = 0,
        .wLength = 0
    };
    
    usb_hcd_send_setup(addr, 0, &setup);
    
    return 0;
}

int usb_hcd_get_hid_report(uint8_t addr, uint8_t iface, uint8_t ep, uint8_t *buffer, uint16_t max_len)
{
    uart_puts("[HCD] Getting HID report from device ");
    uart_puthex((uint64_t)addr);
    uart_puts(":");
    uart_puthex((uint64_t)ep);
    uart_puts("\n\r");
    
    usb_setup_packet_t setup = {
        .bmRequestType = USB_DIR_IN | USB_TYPE_CLASS | USB_RECIP_INTERFACE,
        .bRequest = USB_REQ_GET_REPORT,
        .wValue = 0x0100,   /* Input report */
        .wIndex = iface,
        .wLength = max_len
    };
    
    usb_hcd_send_setup(addr, 0, &setup);
    usb_hcd_send_in(addr, ep, buffer, max_len);
    
    return 0;
}

void usb_enumerate_device(void)
{
    uart_puts("[HCD] Starting device enumeration...\n\r");
    
    /* Reset and enumerate device at address 0 */
    usb_device_descriptor_t dev_desc = {0};
    usb_hcd_get_device_descriptor(0, &dev_desc);
    
    uart_puts("[HCD] Device Descriptor:\n\r");
    uart_puts("  bDeviceClass: 0x");
    uart_puthex((uint64_t)dev_desc.bDeviceClass);
    uart_puts("\n\r");
    uart_puts("  bDeviceSubClass: 0x");
    uart_puthex((uint64_t)dev_desc.bDeviceSubClass);
    uart_puts("\n\r");
    uart_puts("  bDeviceProtocol: 0x");
    uart_puthex((uint64_t)dev_desc.bDeviceProtocol);
    uart_puts("\n\r");
    uart_puts("  bMaxPacketSize0: ");
    uart_puthex((uint64_t)dev_desc.bMaxPacketSize0);
    uart_puts("\n\r");
    uart_puts("  idVendor: 0x");
    uart_puthex((uint64_t)dev_desc.idVendor);
    uart_puts(" idProduct: 0x");
    uart_puthex((uint64_t)dev_desc.idProduct);
    uart_puts("\n\r");
    
    /* Set device address */
    usb_hcd_set_address(1);
    
    /* Set configuration */
    usb_hcd_set_configuration(1, 1);
    
    uart_puts("[HCD] Device enumeration complete\n\r");
}
