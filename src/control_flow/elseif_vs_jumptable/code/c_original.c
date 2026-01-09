#include <stdint.h>

// Version with an if / else if chain
// The compiler may sometimes optimize this, but we are forcing a sequential logical structure
uint32_t dispatch_operation_c_elseif(uint8_t opcode, uint32_t value) {
    if (opcode == 0) {
        return value;
    } else if (opcode == 1) {
        return value * 2;
    } else if (opcode == 2) {
        return value * 3;
    } else if (opcode == 3) {
        return value * 4;
    } else if (opcode == 4) {
        return value * 5;
    } else if (opcode == 5) {
        return value * 6;
    } else if (opcode == 6) {
        return value * 7;
    } else if (opcode == 7) {
        return value * 8;
    } else {
        return 0;
    }
}

// Version with a switch case
// This is the idiomatic way to suggest a jump table to the compiler (GCC, Clang)
uint32_t dispatch_operation_c_switch(uint8_t opcode, uint32_t value) {
    switch (opcode) {
        case 0: return value;
        case 1: return value * 2;
        case 2: return value * 3;
        case 3: return value * 4;
        case 4: return value * 5;
        case 5: return value * 6;
        case 6: return value * 7;
        case 7: return value * 8;
        default: return 0;
    }
}
