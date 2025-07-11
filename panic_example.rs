// Example showing the difference between panic behaviors

fn main() {
    println!("Starting program...");

    // This will panic
    let _result = divide_by_zero();

    // This line will never be reached
    println!("This will never print");
}

fn divide_by_zero() -> i32 {
    let x = 10;
    let y = 0;
    x / y // This will panic
}

// With panic = "unwind" (default):
// 1. Stack unwinding occurs
// 2. Destructors are called
// 3. Panic handlers can catch it
// 4. Stack trace is generated
// 5. Program terminates gracefully

// With panic = "abort":
// 1. Program immediately terminates
// 2. No stack unwinding
// 3. No destructors called
// 4. No stack trace
