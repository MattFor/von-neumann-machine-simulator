; Johnny: product = a * b by repeated addition. Johnny has no output, so it ends with the
; product in the accumulator.
loop:    TST b          ; b is 0: done
         JMP more
         JMP done
more:    TAKE product
         ADD a
         SAVE product
         DEC b
         JMP loop
done:    TAKE product
         HLT
a:       DAT 6
b:       DAT 7
product: DAT 0
