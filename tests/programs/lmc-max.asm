// LMC: output the larger of two inputs.
        INP
        STA a
        INP
        STA b
        SUB a           // b - a
        BRP second      // b >= a
        LDA a
        OUT
        HLT
second  LDA b
        OUT
        HLT
a       DAT
b       DAT
