use {efbuilder::Builder, std::fmt::Debug};

#[allow(dead_code)]
#[derive(Builder)]
pub struct Struct1<'a, 'b: 'a, T, M: Default>
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

pub struct F64(f64);

/// Compare with [`build_struct_1_no_builder`] using [Godbolt](https://rust.godbolt.org).
pub fn build_struct_1<'a, 'b: 'a>(
    f: f64,
    x: &'a f64,
    y: &'b f64,
    z: F64,
    v1: Vec<f64>,
    v2: Vec<u64>) -> Struct1<'a, 'b, f64, u64>
{
    Struct1Builder::new()
        .field_1(f)
        .field_2(v1)
        .field_3(v2)
        .field_4(z)
        .field_5(x)
        .field_6(y)
        .build()
}

pub fn build_struct_1_no_builder<'a, 'b: 'a>(
    f: f64,
    x: &'a f64,
    y: &'b f64,
    z: F64,
    v1: Vec<f64>,
    v2: Vec<u64>) -> Struct1<'a, 'b, f64, u64>
{
    Struct1 {
        field_1: f,
        field_2: v1,
        field_3: v2,
        field_4: z,
        field_5: x,
        field_6: y,
    }
}