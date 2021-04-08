fn main() {

  let n = 256u64;
  match n {
      0...0xff => {
          println!("Prime: {}", n);
                },
      0x100...0xffff => { // 256
          println!("Secon: {}", n);
      },
      0x10000...0xffffffff => {
          println!("Theree: {}", n);
      },
      _ => {
          println!("Upper: {}", n);
      }
  }
}
