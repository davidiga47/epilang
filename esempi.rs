/*
callcc k in {
    let a=15;
    throw k a
}
*/      // VIENE 15

/*
let x=12;
let f=callcc k in {
	let x = 4;
	throw k fn(y) {x}
};
f (7)
*/ //QUESTO NON RITORNA 4 COME IN SML PERCHÈ EPILANG È DINAMICO
        //ALMENO IN TEORIA, NELLA PRATICA IN EPILANG NON PUOI CHIAMARE VARIABLI NON PARAMETRI NEI CORPI DELLE FUNZIONI        



/*
let n=4;
let x=fn(y){y};
x(n)
*/      // VIENE 4




//EQUIVALENTI
/*
let x = 5;
let y = 1;
try{
    if  (x == 0) {throw "DivZero"};
    y = 5 / x;
} catch e {
    y = x - 42;
}
y


let x=0;
callcc k in {
    callcc j in {
        if (x == 0) {throw j "DivZero"};
        throw k (5 / x)
    }
    x-42
}
*/      // SE x == 0 VIENE -42 ALTRIMENTI 5/x

/-------------------------------------------------------/

/*
let x=12;
let f = fn(x) {throw x};
let res=0;
try {
    let y=5;
    f(y)
} catch e {
    res=e+5;
}
res
*/      // VIENE 10

/*
let f=fn(x){throw x 1};
callcc k in {
    3+f(k)
}
*/     // VIENE 1

