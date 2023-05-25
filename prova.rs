let x=0;
callcc k in {
    callcc j in {
        if (x == 0) {throw j "DivZero"};
        throw k (5 / x)
    };
    x-42
}