; Ben Eater's counting program: up until ADD carries past 255, then down until SUB gives 0.
; His counts by 1 and never stops; this one counts by 33 hex (51) and halts at the bottom.
up:     OUT
        ADD step
        JC down
        JMP up
down:   SUB step
        OUT
        JZ done
        JMP down
done:   HLT
step:   DAT 33
