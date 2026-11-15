// LMC: count down from an input to 0.
        INP
loop    OUT
        BRZ done
        SUB one
        BRA loop
done    HLT
one     DAT 1
