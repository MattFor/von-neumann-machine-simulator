/ MARIE: sum 1 to n.
        Input
        Store N
Loop,   Load N
        Skipcond 800    / n above 0: add it
        Jump Done
        Load Sum
        Add N
        Store Sum
        Load N
        Subt One
        Store N
        Jump Loop
Done,   Load Sum
        Output
        Halt
N,      Dec 0
Sum,    Dec 0
One,    Dec 1
