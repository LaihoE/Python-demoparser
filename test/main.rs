fn main() {
    use bitreader::BitReader;

    let slice_of_u8: [u8] = [11, 22];
    println("{:?}", slice_of_u8);
    let mut reader = BitReader::new(slice_of_u8);

    // You probably should use try! or some other error handling mechanism in real code if the
    // length of the input is not known in advance.
    let a_single_bit = reader.read_u8(1).unwrap();
    assert_eq!(a_single_bit, 1);

    let more_bits = reader.read_u8(3).unwrap();
    assert_eq!(more_bits, 0);

    let last_bits_of_byte = reader.read_u8(4).unwrap();
    assert_eq!(last_bits_of_byte, 0b1111);
}
