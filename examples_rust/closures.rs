fn adder(outer: i32) -> impl Fn(i32) -> i32 {
    move |inner| outer + inner
}

fn test() {
    let add5 = adder(5);
    println!("{}", add5(3));
}

fn main() {
    test();
}
