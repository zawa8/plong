use cplong::Cplong;

fn main() {
    println!("=== cplong hskii number system ===");

    // Example 1: Simple number
    let a = Cplong::from_str("5V").unwrap();
    println!("a = {} = {}", a, a.to_decimal_string());

    // Example 2: Negative number
    let b = Cplong::from_str("-6").unwrap();
    println!("b = {} = {}", b, b.to_decimal_string());

    // Example 3: Subtraction
    let d = a.sub(&b).unwrap();
    println!("d = a - b = {} = {}", d, d.to_decimal_string());

    // Example 4: Complex number
    let c = Cplong::from_str("-2P,5V,67,5V.67.78.89").unwrap();
    println!("c = {}", c);
    println!("c.wlyu = {:?}", c.wlyu);
    println!("c.start_prisizxn_leyr = {}", c.start_prisizxn_leyr);
    println!("c.is_negetiw = {}", c.is_negetiw);

    // Example 5: Addition
    let x = Cplong::from_str("F.F").unwrap();
    let y = Cplong::from_str("0.1").unwrap();
    let z = x.add(&y).unwrap();
    println!("{} + {} = {}", x, y, z);
}