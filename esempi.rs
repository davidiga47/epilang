/*
callcc k in {
    let a=15;
    throw k a
}*/

/*

let f = fn(x) {throw x 1};
callcc k in {
	callcc k in {
		3+f(k)		
	}
}*/

/*let x=12;
let f=callcc k in {
	let x = 4;
	throw k fn(y) {x}
};
f (7)*/ //QUESTO NON RITORNA 4 COME IN SML PERCHÈ EPILANG È DINAMICO

/*let n=4;
let x=fn(y){y};
x(n)*/




//EQUIVALENTI
/*let x = 0;
try{
    if  (x == 0) {throw "DivZero"};
    5 / x
} catch e {
    x-42
}

let x=0;
callcc k in {
    callcc j in {
        if (x == 0) {throw j "DivZero"};
        throw k (5 / x)
    }
    x-42
}*/