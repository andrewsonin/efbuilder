use {efbuilder::Builder, std::fmt::Debug};

#[allow(dead_code)]
#[derive(Builder)]
struct Struct1<'a, 'b: 'a, T, M: Default>
    where
        T: Debug
{
    /// Field 1.
    field_1: f64,

    /// Field 2.
    field_2: Vec<T>,

    /// Field 3.
    field_3: Vec<M>,

    /// Field 4.
    field_4: F64,

    /// Field 5.
    field_5: &'a f64,

    /// Field 6.
    field_6: &'b f64,
}

#[allow(dead_code)]
#[derive(Builder)]
struct Struct2;

#[allow(dead_code)]
#[derive(Builder)]
struct Struct3 {
    a: f64,
}

struct F64(f64);