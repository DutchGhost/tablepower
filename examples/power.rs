#[macro_use]
extern crate tablepower;

table_of!(u8, table, order = descending);

fn main() {
    assert!(table.len() == 3);
    assert_eq!(table, [100, 10, 1]);
    println!("{:?}", table);
}
