.section .text
.global vectors
.align 11

/* Macro for simple vector entries */
.macro ventry label
.align 7
    b \label
.endm

/* Vector Table for Current EL with SP_ELx (x > 0) */
vectors:
    /* Synchronous */
    ventry invalid_sync
    /* IRQ/vIRQ */
    ventry el1_irq
    /* FIQ/vFIQ */
    ventry invalid_fiq
    /* SError/vSError */
    ventry invalid_serror

    /* Current EL with SP_EL0 */
    ventry invalid_sync
    ventry invalid_irq
    ventry invalid_fiq
    ventry invalid_serror

    /* Lower EL using AArch64 */
    ventry invalid_sync
    ventry invalid_irq
    ventry invalid_fiq
    ventry invalid_serror

    /* Lower EL using AArch32 */
    ventry invalid_sync
    ventry invalid_irq
    ventry invalid_fiq
    ventry invalid_serror

invalid_sync:
    b invalid_sync
invalid_irq:
    b invalid_irq
invalid_fiq:
    b invalid_fiq
invalid_serror:
    b invalid_serror

el1_irq:
    /* Save general purpose registers */
    sub sp, sp, #256
    stp x0, x1, [sp, #16 * 0]
    stp x2, x3, [sp, #16 * 1]
    stp x4, x5, [sp, #16 * 2]
    stp x6, x7, [sp, #16 * 3]
    stp x8, x9, [sp, #16 * 4]
    stp x10, x11, [sp, #16 * 5]
    stp x12, x13, [sp, #16 * 6]
    stp x14, x15, [sp, #16 * 7]
    stp x16, x17, [sp, #16 * 8]
    stp x18, x19, [sp, #16 * 9]
    stp x20, x21, [sp, #16 * 10]
    stp x22, x23, [sp, #16 * 11]
    stp x24, x25, [sp, #16 * 12]
    stp x26, x27, [sp, #16 * 13]
    stp x28, x29, [sp, #16 * 14]
    str x30, [sp, #16 * 15]

    bl c_irq_handler

    /* Restore registers */
    ldr x30, [sp, #16 * 15]
    ldp x28, x29, [sp, #16 * 14]
    ldp x26, x27, [sp, #16 * 13]
    ldp x24, x25, [sp, #16 * 12]
    ldp x22, x23, [sp, #16 * 11]
    ldp x20, x21, [sp, #16 * 10]
    ldp x18, x19, [sp, #16 * 9]
    ldp x16, x17, [sp, #16 * 8]
    ldp x14, x15, [sp, #16 * 7]
    ldp x12, x13, [sp, #16 * 6]
    ldp x10, x11, [sp, #16 * 5]
    ldp x8, x9, [sp, #16 * 4]
    ldp x6, x7, [sp, #16 * 3]
    ldp x4, x5, [sp, #16 * 2]
    ldp x2, x3, [sp, #16 * 1]
    ldp x0, x1, [sp, #16 * 0]
    add sp, sp, #256

    eret
