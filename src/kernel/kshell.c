#include <kshell.h>
#include <drivers/uart.h>
#include <string.h>

#define CMD_MAX_LEN 256

void kshell_start(void) {
    char cmd_buffer[CMD_MAX_LEN];
    int cmd_len = 0;

    uart_puts("\n\r[PRos] > ");

    while (1) {
        char c = uart_getc();

        if (c == '\r' || c == '\n') {
            uart_puts("\n\r");
            cmd_buffer[cmd_len] = '\0';

            if (cmd_len > 0) {
                if (strcmp(cmd_buffer, "help") == 0) {
                    uart_puts("Available commands:\n\r");
                    uart_puts("  help  - Show this help message\n\r");
                } else {
                    uart_puts("Unknown command: ");
                    uart_puts(cmd_buffer);
                    uart_puts("\n\r");
                }
            }

            cmd_len = 0;
            uart_puts("[PRos] > ");
        } 
        else if (c == '\b' || c == 0x7F) {
            if (cmd_len > 0) {
                cmd_len--;
                uart_puts("\b \b");
            }
        } 
        else if (c >= 32 && c <= 126) {
            if (cmd_len < CMD_MAX_LEN - 1) {
                cmd_buffer[cmd_len++] = c;
                uart_putc(c);
            }
        }
    }
}