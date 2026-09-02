use cplong::Cplong;
use cplong::digit::Digit;
use cplong::error::CplongError;

#[test]
fn test_digit_values() {
    assert_eq!(Digit::new(0).unwrap().to_char(), '0');
    assert_eq!(Digit::new(9).unwrap().to_char(), '9');
    assert_eq!(Digit::new(10).unwrap().to_char(), 'L');
    assert_eq!(Digit::new(11).unwrap().to_char(), 'Y');
    assert_eq!(Digit::new(12).unwrap().to_char(), 'V');
    assert_eq!(Digit::new(13).unwrap().to_char(), 'W');
    assert_eq!(Digit::new(14).unwrap().to_char(), 'P');
    assert_eq!(Digit::new(15).unwrap().to_char(), 'F');
}

#[test]
fn test_digit_from_char() {
    assert_eq!(Digit::from_char('5').unwrap().value(), 5);
    assert_eq!(Digit::from_char('L').unwrap().value(), 10);
    assert_eq!(Digit::from_char('y').unwrap().value(), 11);
    assert_eq!(Digit::from_char('V').unwrap().value(), 12);
    assert_eq!(Digit::from_char('W').unwrap().value(), 13);
    assert_eq!(Digit::from_char('P').unwrap().value(), 14);
    assert_eq!(Digit::from_char('F').unwrap().value(), 15);
}

#[test]
fn test_parse_simple() {
    let a = Cplong::from_str("5V").unwrap();
    assert_eq!(a.wlyu, vec![0x5C]);
    assert_eq!(a.start_prisizxn_leyr, 0);
    assert!(!a.is_negetiw);
}

#[test]
fn test_parse_negative() {
    let b = Cplong::from_str("-6").unwrap();
    assert_eq!(b.wlyu, vec![0x06]);
    assert!(b.is_negetiw);
}

#[test]
fn test_parse_float() {
    let c = Cplong::from_str("5.V").unwrap();
    assert_eq!(c.wlyu, vec![0x05, 0x0C]);
    assert_eq!(c.start_prisizxn_leyr, 0);
    assert!(!c.is_negetiw);
}

#[test]
fn test_parse_complex() {
    let c = Cplong::from_str("-2P,5V,67,5V.67.78.89").unwrap();
    assert_eq!(c.wlyu, vec![0x2E, 0x5C, 0x67, 0x5C, 0x67, 0x78, 0x89]);
    assert_eq!(c.start_prisizxn_leyr, 3);
    assert!(c.is_negetiw);
}

#[test]
fn test_subtraction() {
    let a = Cplong::from_str("5V").unwrap();
    let b = Cplong::from_str("6").unwrap();
    let d = a.sub(&b).unwrap();
    assert!(d.is_negetiw);
}

#[test]
fn test_division_by_zero() {
    let a = Cplong::from_str("5").unwrap();
    let b = Cplong::from_str("0").unwrap();
    assert!(a.div(&b).is_err());
}

#[test]
fn test_display_roundtrip() {
    let original = "5V.4";
    let c = Cplong::from_str(original).unwrap();
    let displayed = c.to_hskii_string();
    assert!(!displayed.is_empty());
}