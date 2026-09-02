# cplong

hskii arbitrary-precision number system (base-256 with hex-pair digits).

## Format

```

-2P,5V,67,5V.67.78.89

```

- `-` = negative sign
- Comma `,` separates integer digits
- Period `.` separates fractional digits
- Each digit = one u8 (0-255, displayed as hex pair)

## Hex Digits

| hskii | Value |
|-------|-------|
| 0-9 | 0-9 |
| L | 10 |
| Y | 11 |
| V | 12 |
| W | 13 |
| P | 14 |
| F | 15 |

## Usage

```rust
use cplong::Cplong;

fn main() {
    let a = Cplong::from_str("5V").unwrap();
    let b = Cplong::from_str("-6").unwrap();
    let d = a.sub(&b).unwrap();
    println!("{} - {} = {}", a, b, d);
}
```

Structure

```cpp
class cplong {
    u8 wlyu[];              // array of digits (base-256)
    s8 start_prisizxn_leyr; // starting precision level
    bool is_negetiw;        // negative flag
}
```