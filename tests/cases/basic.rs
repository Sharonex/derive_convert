use derive_convert::Convert;

#[derive(Debug, PartialEq)]
struct Number(u8);

impl From<Number> for u8 {
    fn from(n: Number) -> u8 {
        n.0
    }
}
impl From<u8> for Number {
    fn from(n: u8) -> Number {
        Number(n)
    }
}

impl From<Number> for u16 {
    fn from(n: Number) -> u16 {
        n.0 as u16
    }
}
impl From<u16> for Number {
    fn from(_n: u16) -> Number {
        Number(0)
    }
}
#[derive(Convert)]
#[convert(into = "B")]
#[convert(from = "B")]
pub struct A {
    #[convert(unwrap)]
    pub normal: Option<u8>,

    // auto into of inner
    pub opt: Option<u8>,
    // auto into of inner
    pub vec: Vec<u8>,

    #[convert(rename = "renamed_field")]
    pub old_name: u16,
}

pub struct B {
    normal: u8,
    opt: Option<Number>,
    vec: Vec<Number>,
    renamed_field: Number,
}

fn main() {
    let a = A {
        normal: Some(1),
        opt: Some(2),
        vec: vec![3],
        old_name: 4,
    };
    let b: B = a.try_into().unwrap();
    assert_eq!(b.normal, 1);
    assert_eq!(b.opt.unwrap().0, 2);
    assert_eq!(b.vec, vec![Number(3)]);

    assert_eq!(b.renamed_field, Number(0));
}
