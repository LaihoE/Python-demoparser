import 


with open("foo.bin", "rb") as fh:
    ba = bytearray(fh.read())
    for byte in ba:
        print(byte)