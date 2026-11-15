// LMC: multiply two inputs by repeated addition.
        INP
        STA a
        INP
        STA b
loop    LDA b
        BRZ done
        SUB one
        STA b
        LDA product
        ADD a
        STA product
        BRA loop
done    LDA product
        OUT
        HLT
a       DAT
b       DAT
product DAT
one     DAT 1
