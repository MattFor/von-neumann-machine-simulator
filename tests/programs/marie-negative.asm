/ MARIE: negative numbers in two's complement.
        Load A
        Subt B          / 5 - 12
        Store D         / held as FFF9
        Output
        Load M
        Output
        Add D
        Skipcond 000    / negative: skip the halt
        Halt
        Output
        Halt
A,      Dec 5
B,      Dec 12
D,      Dec 0
M,      Hex FFFE        / -2
