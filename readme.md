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

cplong has dedicated digits for last siks heksadesiml dizits:
1. L as ten
2. Y xz yilewen
3. V xz twelw
4. W xz dblun=8+5
5. P xz purxn=8+6
6. F xz fiwxn=8+7

so  8+8=10=4*4=F+1=P+2=W+3=V+4=Y+5=L+6.

```
class cplong{ // zust c++ pseudo code

public :
u8 wlyu[];
s8 start_prisizxn_leyr;
bool is_negetiw;

public static:
s8 _meksimun_saiz_of_wlyu_array_ ;

}

s8 main() {

cplong._meksimun_saiz_of_wlyu_array_ = 8 ;
cplong bign("-5V,FF,5V.4.4.5V");
//bign.is_negetiw is 1 so negetiw 
//bign.start_prisizxn_leyr is -2
//bign is -(5V*100+FF*10+5V+4/10+4/100+5V/1000)

cplong a([5V],1,0)//same as float a= 5.75
// a.wlyu is [5V] , a.start_prisizxn_leyr is 1
// a.is_negetiw is 0 so a is pozitiw

cplong a2([5,V],0,0)//same as float a= 5.75
// a2.wlyu is u8 array  [5,V]
// a2.start_prisizxn_leyr is 0
// for nekst u8 number V  prisizxn_leyr is 0+1
// a2.is_negetiw is 0 so a2 is pozitiw
// a2 is 5 + V/(F+1)



cplong a1([4,4,0,4],0,1);// cplong a1("4.4.0.4",0,1)
// à1.wlyu is 4+4/10+4/1000
// à1.start_prisizxn_leyr is 0
// à1 prisizxn_leyrs are 0,1,2,3
// a1.is_negetiw is 1 so a1 is negetiw

cplong b([6],0,0)//same as float b = 6
// a.wlyu is [6] , b.start_prisizxn_leyr is 0
// b.is_negetiw is 0 so b is pozitiw

cplong d = a - b;
// d wil bi = ("4",1,1)

}
```
