; Prints the digits of n, one per line.
; SFT shifts the other digits out of the accumulator and a subroutine prints the one left.
; JMP leaves a jump back in cell 99, which the subroutine copies to its last cell.
        ORG 10
start   CLA n
        SFT 13      ; 0472 -> 4720 -> 0004
        JMP print
        CLA n
        SFT 23      ; 0472 -> 7200 -> 0007
        JMP print
        CLA n
        SFT 33      ; 0472 -> 2000 -> 0002
        JMP print
        HRS start

print   STO digit
        CLA 99      ; the jump back
        STO back
        OUT digit
back    DAT         ; jumps back from here
digit   DAT
n       DAT 472
