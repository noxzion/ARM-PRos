#ifndef DRIVERS_INTC_H
#define DRIVERS_INTC_H

#include <stdint.h>

#define IRQ_TIMER       0
#define IRQ_MAILBOX     1
#define IRQ_DOORBELL0   2
#define IRQ_DOORBELL1   3
#define IRQ_GPU0_HALTED 4
#define IRQ_GPU1_HALTED 5
#define IRQ_ILLEGAL_TYPE0 6
#define IRQ_ILLEGAL_TYPE1 7

#define IRQ_USB         9
#define IRQ_AUX         29
#define IRQ_I2C_SPI_SLV 43
#define IRQ_PWA0        45
#define IRQ_PWA1        46
#define IRQ_SMI         48
#define IRQ_GPIO0       49
#define IRQ_GPIO1       50
#define IRQ_GPIO2       51
#define IRQ_GPIO3       52
#define IRQ_I2C         53
#define IRQ_SPI         54
#define IRQ_I2SPCM      55
#define IRQ_SDIO        56
#define IRQ_UART        57
#define IRQ_SDHOST      58

void intc_init(void);
void intc_enable_irq(uint32_t irq_number);
void intc_disable_irq(uint32_t irq_number);
uint32_t intc_get_pending(void);

#endif /* DRIVERS_INTC_H */
