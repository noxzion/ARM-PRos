.section .text
.global _start

_start:
    mrs x0, mpidr_el1
    and x0, x0, #0xff
    cbz x0, 1f
0:
    wfe
    b   0b
1:
    ldr x30, =stack_top
    mov sp, x30

    ldr x0, =__bss_start
    ldr x1, =__bss_end
2:
    cmp x0, x1
    b.hs 3f
    str xzr, [x0], #8
    b   2b
3:
    /* Set vector base address */
    ldr x0, =vectors
    msr vbar_el1, x0
    msr vbar_el2, x0

    bl  main

4:
    wfe
    b   4b