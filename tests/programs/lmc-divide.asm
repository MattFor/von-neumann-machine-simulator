// LMC: divide the first input by the second, subtracting until the rest goes negative,
// then output the quotient and the remainder.
        INP
        STA rest
        INP
        STA divisor
loop    LDA rest
        SUB divisor
        BRP fits        // still 0 or more: one more time
        LDA quotient    // went negative: rest is the remainder
        OUT
        LDA rest
        OUT
        HLT
fits    STA rest
        LDA quotient
        ADD one
        STA quotient
        BRA loop
rest     DAT
divisor  DAT
quotient DAT
one      DAT 1
