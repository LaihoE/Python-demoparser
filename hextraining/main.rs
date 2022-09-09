fn main() {
    let data = b"hello";
    // lower case
    println!("{:x?}", data);
    // upper case
    println!("{:X?}", data);

    let data = [0x0, 0x1, 0xe, 0xf, 0xff];
    // print the leading zero
    println!("{:02X?}", data);
    // It can be combined with the pretty modifier as well
    println!("{:#04X?}", data);
}
