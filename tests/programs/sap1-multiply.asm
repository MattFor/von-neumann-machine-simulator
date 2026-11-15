; SAP-1: multiply x by y, Ben Eater's way. SUB sets carry when nothing is borrowed,
; so JC adds y to the product once more for as long as x was at least 1.
loop:   LDA x
        SUB one
        JC more
        LDA product
        OUT
        HLT
more:   STA x
        LDA product
        ADD y
        STA product
        JMP loop
one:    DAT 1
product: DAT 0
x:      DAT 6
y:      DAT 7
