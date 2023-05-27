
callcc k in {
    let a=15;
    throw k a
}
     // VIENE 15


let x=12;
let f=callcc k in {
	let x = 4;
	throw k fn(y) {x}
};
f (7)
 //QUESTO NON RITORNA 4 COME IN SML PERCHÈ EPILANG È DINAMICO
 //ALMENO IN TEORIA, NELLA PRATICA IN EPILANG NON PUOI CHIAMARE VARIABLI NON PARAMETRI NEI CORPI DELLE FUNZIONI        




let n=4;
let x=fn(y){y};
x(n)
      // VIENE 4


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
      // VIENE 10


let f=fn(x){throw x 1};
callcc k in {
    3+f(k)
}
     // VIENE 1



/--------------------------------------------/

// SCHEMA GENERALE:

try{
    e1
} catch exc {
    e2
}

callcc tryblock in {
    let exc=callcc handle in {
        e1 [MA OGNI "throw e" DIVENTA "throw handle e"]
        [ALLA FINE AGGIUNGI]
        throw tryblock last_instruction;
    }
    e2
}


//QUESTO PROGRAMMA

let x = 5;
let y = 1;
try{
    if  (x == 0) {throw "DivZero"};
    y = 5 / x;
} catch e {
    y = x - 42;
}
y

//VIENE CONVERTITO IN:

let x=5;
let y=1;
callcc tryblock in {
    let e=callcc handle in {
        if (x == 0) {throw handle "DivZero"};
        y = 5 / x;
        throw tryblock y;
    }
    y = x - 42;
}
y

//(CHE POI POSSO SEMPLIFICARE COME SOTTO)

let x=0;
callcc tryblock in {
    callcc handle in {
        if (x == 0) {throw handle "DivZero"};
        throw tryblock (5 / x)
    }
    x-42
}
      // SE x == 0 VIENE -42 ALTRIMENTI 5/x


//  PERÒ È UN CASINO CON TANTI TRY-CATCH ANNIDATI

    
try {
    M
    try{
        N
    } catch e1 {
        O
    }
} catch e2 {
    P
} 

es.

let res=0;
try {
    5+2;
    throw "ciao";
    try {
        res=3 / 2;
    } catch e1 {
        res=9;
    }
} catch e2 {
    res=e2
}
res


let res=0;
callcc t1 in {
    let e2=callcc h1 in {
        5+2;
        throw h1 "ciao";
        callcc t2 in {
            let e1=callcc h2 in {
                res = 3 / 2;
                throw t2 res
            }
            res=9
        }
        throw t1 res
    }
    res=e2
}
res




sennò faccio come dice prof

let handler=[]
dichiaro prima gli handler, ad es.
handler.append(fn(){handler1})

poi quando devo fare i throw exc invece faccio
    throw k handler[i](null)

MEGLIO:

per ogni eccezione "try {M} catch e {N}"
creo la funzione "let handler_N=fn(e){N}"
poi faccio "callcc k in {M}", in M al posto dei "throw e" avrò "throw k handler_N(e)"

esempio:

let x = 5;
let y = 1;
try{
    if  (x == 0) {throw "DivZero"};
    y = 5 / x;
} catch e {
    y = x - 42;
}
y

diventa:

let x=5;
let y=1;
let h1=fn(e){y=x-42};
callcc k in {
    if (x == 0) {throw k h1("DivZero")};
    y = 5 / x;
}
y

il problema di questo approccio è che epilang è funzionale, quindi non puoi dichiarare una funzione tipo h1 qua sopra




