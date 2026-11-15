; Bonsai: a = a + b, moving b over one at a time.
loop:   TST b           ; b is 0: done
        JMP more
        HLT
more:   DEC b
        INC a
        JMP loop
a:      DAT 3
b:      DAT 4
