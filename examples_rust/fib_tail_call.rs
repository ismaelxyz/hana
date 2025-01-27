fn fibrec(n: i32, prev: i32, curr: i32) -> i32 {
    if n <= 0 {
        curr
    } else {
        fibrec(n - 1, prev + curr, prev)
    }
}

fn main() {
    let fib = |n| fibrec(n + 1, 1, 0);

    println!("{}", fib(30));
}
