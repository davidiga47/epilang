callcc k in {
    try {
        throw 3
    } catch x {
        throw k x
    }
}

