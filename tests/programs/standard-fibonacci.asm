; Standard: the first n Fibonacci numbers.
        IN
        STORE n
loop:   LOAD a
        OUT
        ADD b
        STORE next
        LOAD b
        STORE a
        LOAD next
        STORE b
        DEC n
        JZ end
        JMP loop
end:    HALT
n:      DAT 0
a:      DAT 0
b:      DAT 1
next:   DAT 0
