#[macro_use]
extern crate tablepower;

table_of!(u8, table);

fn main() {
    assert!(table.len() == 3);

    println!("{:?}", table);
}
