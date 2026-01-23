#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>


typedef struct Calculator Calculator;
struct Calculator {
};

int64_t Calculator_add(int64_t a, int64_t b);
int64_t Calculator_sub(int64_t a, int64_t b);
int64_t main();

  int64_t Calculator_add(int64_t a, int64_t b) {
    int64_t stack[1024];
    int sp = 0;
    int64_t locals[2];
    locals[0] = a;
    locals[1] = b;
    
    stack[sp++] = locals[0];
    stack[sp++] = locals[1];
    sp--; stack[sp-1] += stack[sp];
    return stack[--sp];
    stack[sp++] = 0;
    return stack[--sp];
  }
  
  int64_t Calculator_sub(int64_t a, int64_t b) {
    int64_t stack[1024];
    int sp = 0;
    int64_t locals[2];
    locals[0] = a;
    locals[1] = b;
    
    stack[sp++] = locals[0];
    stack[sp++] = locals[1];
    sp--; stack[sp-1] -= stack[sp];
    return stack[--sp];
    stack[sp++] = 0;
    return stack[--sp];
  }
  
  int64_t main() {
    int64_t stack[1024];
    int sp = 0;
    int64_t locals[5];
    
    // Unknown New
    locals[0] = stack[--sp];
    stack[sp++] = N;
    locals[1] = stack[--sp];
    stack[sp++] = N;
    locals[2] = stack[--sp];
    stack[sp++] = locals[0];
    stack[sp++] = locals[1];
    stack[sp++] = locals[2];
    // Unknown CallMethod
    locals[3] = stack[--sp];
    stack[sp++] = (int64_t)"Sum: ";
    stack[sp++] = locals[3];
    sp--; stack[sp-1] += stack[sp];
    int64_t arg0 = stack[--sp];
    stack[sp++] = print(arg0);
    sp--;
    stack[sp++] = locals[3];
    stack[sp++] = N;
    sp--; stack[sp-1] = (stack[sp-1] > stack[sp]);
    sp--; if (!stack[sp]) goto L0;
    stack[sp++] = (int64_t)"Sum is greater than 20";
    int64_t arg0 = stack[--sp];
    stack[sp++] = print(arg0);
    sp--;
    goto L1;
    L0:;
    stack[sp++] = (int64_t)"Sum is not greater than 20";
    int64_t arg0 = stack[--sp];
    stack[sp++] = print(arg0);
    sp--;
    L1:;
    stack[sp++] = 0;
    locals[4] = stack[--sp];
    L2:;
    stack[sp++] = locals[4];
    stack[sp++] = 5;
    sp--; stack[sp-1] = (stack[sp-1] < stack[sp]);
    sp--; if (!stack[sp]) goto L3;
    stack[sp++] = (int64_t)"Loop: ";
    stack[sp++] = locals[4];
    sp--; stack[sp-1] += stack[sp];
    int64_t arg0 = stack[--sp];
    stack[sp++] = print(arg0);
    sp--;
    stack[sp++] = locals[4];
    stack[sp++] = locals[4];
    stack[sp++] = 1;
    sp--; stack[sp-1] += stack[sp];
    sp--;
    goto L2;
    L3:;
    stack[sp++] = 0;
    return stack[--sp];
  }
  

int main() {
  int64_t stack[1024];
  int sp = 0;
  int64_t locals[256];
  return 0;
}
