#include <drivers/intc.h>

#define INTC_BASE               0x3F00B200
#define INTC_IRQ_BASIC_PENDING  (*(volatile uint32_t*)(INTC_BASE + 0x00))
#define INTC_IRQ_PENDING_1      (*(volatile uint32_t*)(INTC_BASE + 0x04))
#define INTC_IRQ_PENDING_2      (*(volatile uint32_t*)(INTC_BASE + 0x08))
#define INTC_FIQ_CONTROL        (*(volatile uint32_t*)(INTC_BASE + 0x0C))
#define INTC_ENABLE_IRQ_1       (*(volatile uint32_t*)(INTC_BASE + 0x10))
#define INTC_ENABLE_IRQ_2       (*(volatile uint32_t*)(INTC_BASE + 0x14))
#define INTC_ENABLE_BASIC_IRQ   (*(volatile uint32_t*)(INTC_BASE + 0x18))
#define INTC_DISABLE_IRQ_1      (*(volatile uint32_t*)(INTC_BASE + 0x1C))
#define INTC_DISABLE_IRQ_2      (*(volatile uint32_t*)(INTC_BASE + 0x20))
#define INTC_DISABLE_BASIC_IRQ  (*(volatile uint32_t*)(INTC_BASE + 0x24))

void intc_init(void) {
    INTC_DISABLE_IRQ_1 = 0xFFFFFFFF;
    INTC_DISABLE_IRQ_2 = 0xFFFFFFFF;
    INTC_DISABLE_BASIC_IRQ = 0xFFFFFFFF;
}

void intc_enable_irq(uint32_t irq_number) {
    if (irq_number < 32) {
        INTC_ENABLE_IRQ_1 = (1 << irq_number);
    } else if (irq_number < 64) {
        INTC_ENABLE_IRQ_2 = (1 << (irq_number - 32));
    } else {
        INTC_ENABLE_BASIC_IRQ = (1 << (irq_number - 64));
    }
}

void intc_disable_irq(uint32_t irq_number) {
    if (irq_number < 32) {
        INTC_DISABLE_IRQ_1 = (1 << irq_number);
    } else if (irq_number < 64) {
        INTC_DISABLE_IRQ_2 = (1 << (irq_number - 32));
    } else {
        INTC_DISABLE_BASIC_IRQ = (1 << (irq_number - 64));
    }
}

uint32_t intc_get_pending(void) {
    uint32_t basic = INTC_IRQ_BASIC_PENDING;
    if (basic != 0) {
        if (basic & (1 << 8)) {
            uint32_t pending_1 = INTC_IRQ_PENDING_1;
            for (int i = 0; i < 32; i++) {
                if (pending_1 & (1 << i)) return i;
            }
        }
        if (basic & (1 << 9)) {
            uint32_t pending_2 = INTC_IRQ_PENDING_2;
            for (int i = 0; i < 32; i++) {
                if (pending_2 & (1 << i)) return i + 32;
            }
        }
        for (int i = 0; i < 8; i++) {
            if (basic & (1 << i)) return i + 64;
        }
    }
    return 0xFFFFFFFF;
}
