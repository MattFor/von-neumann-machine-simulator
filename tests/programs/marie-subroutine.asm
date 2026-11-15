/ MARIE: a subroutine that doubles Arg, called twice with JnS and left with JumpI.
        Input
        Store Arg
        JnS Double
        Load Res
        Output
        Store Arg
        JnS Double
        Load Res
        Output
        Halt
Arg,    Dec 0
Res,    Dec 0
Double, Hex 0           / JnS leaves the return address here
        Load Arg
        Add Arg
        Store Res
        JumpI Double
