import math
import struct

import consts as c


class Bitbuffer:
    # ---------------------------------------------------------
    def __init__(self, a_data):
        self.data = ""
        self.dataBytes = 0
        self.dataPart = ""

        self.posByte = 0
        self.bitsFree = 0
        self.overflow = False
        # Save data to vars
        self.data = a_data
        self.dataBytes = len(a_data)

        # Calculate head
        head = self.dataBytes % 4

        # If there is less bytes than potencial head OR head exists
        if self.dataBytes < 4 or head > 0:
            if head > 2:
                self.dataPart = self.data[0] + (self.data[1] << 8) + (self.data[2] << 16)
                self.posByte = 3
            elif head > 1:
                self.dataPart = self.data[0] + (self.data[1] << 8)
                self.posByte = 2
            else:
                self.dataPart = self.data[0]
                self.posByte = 1
            self.bitsFree = head << 3
        else:
            self.posByte = head
            self.dataPart = self.data[self.posByte] + (self.data[self.posByte + 1] << 8) + (
                    self.data[self.posByte + 2] << 16) + (self.data[self.posByte + 3] << 24)
            if self.data:
                self.fetchNext()
            else:
                self.dataPart = 0
                self.bitsFree = 1
            self.bitsFree = min(self.bitsFree, 32)
            
    # Add 32 bits free to use and grab new data to buffer
    def fetchNext(self):
        self.bitsFree = 32
        self.grabNext4Bytes()    
    # ---------------------------------------------------------
        
    # Grab another part of data to buffer
    def grabNext4Bytes(self):
        if self.posByte >= len(self.data):
            self.bitsFree = 1
            self.dataPart = 0
            self.overflow = True
        else:
            self.dataPart = self.data[self.posByte] + (self.data[self.posByte + 1] << 8) + (
                    self.data[self.posByte + 2] << 16) + (self.data[self.posByte + 3] << 24)
            self.posByte += 4
    # ---------------------------------------------------------
    
    # Read VAR
    def readUBitVar(self):
        ret = self.read_uint_bits(6)
        if ret & 48 == 16:
            ret = (ret & 15) | (self.read_uint_bits(4) << 4)
            assert ret >= 16
        elif ret & 48 == 32:
            ret = (ret & 15) | (self.read_uint_bits(8) << 4)
            assert ret >= 256
        elif ret & 48 == 48:
            ret = (ret & 15) | (self.read_uint_bits(28) << 4)
            assert ret >= 4096
        return ret

    def read_var_int(self):
        ret = 0
        count = 0
        while True:
            if count == 5:
                return ret
            b = self.read_uint_bits(8)
            ret |= (b & 0x7F) << (7 * count)
            count += 1
            if not (b & 0x80):
                break
        return ret

    # Read unsigned n-bits
    def read_uint_bits(self, a_bits):
        if self.bitsFree >= a_bits:
            # By using mask take data needed from buffer
            res = self.dataPart & ((2 ** a_bits) - 1)
            self.bitsFree -= a_bits
            # Check if we need to grab new data to buffer
            if self.bitsFree == 0:
                self.fetchNext()
            else:
                # Move buffer to the right
                self.dataPart >>= a_bits
            return res
        else:
            # Take whats left
            res = self.dataPart
            a_bits -= self.bitsFree
            # Save how many free bits we used
            t_bitsFree = self.bitsFree
            # Grab new data to buffer
            self.fetchNext()
            # Append new data to result
            if self.overflow:
                return 0
            res |= ((self.dataPart & ((2 ** a_bits) - 1)) << t_bitsFree)
            self.bitsFree -= a_bits
            # Move buffer to the right
            self.dataPart >>= a_bits
            return res

    # def read_uint_bits2(self, a_bits):
    #     return int.from_bytes(self.readBits(a_bits), byteorder="little", signed=False)
    #
    # def read_sint_bits2(self, a_bits):
    #     return int.from_bytes(self.readBits(a_bits), byteorder="little", signed=True)

    # Read signed n-bits
    def read_sint_bits(self, a_bits):
        # return self._get_signed_nr(self.read_uint_bits(a_bits), a_bits)
        return (self.read_uint_bits(a_bits) << (32 - a_bits)) >> (32 - a_bits)

    # Read string
    def read_string(self, length=0):
        res = ""
        index = 1
        while True:
            char = self.read_sint_bits(8)
            if char == 0 and length == 0:
                break
            res += chr(char)
            if index == length:
                break
            index += 1
        return res

    # Read n-bits
    def readBits(self, a_bits):
        res = b""
        bitsleft = a_bits
        while bitsleft >= 32:
            res += bytes([self.read_uint_bits(8), self.read_uint_bits(8), self.read_uint_bits(8), self.read_uint_bits(8)])
            bitsleft -= 32
        while bitsleft >= 8:
            res += bytes([self.read_uint_bits(8)])
            bitsleft -= 8
        if bitsleft:
            res += bytes([self.read_uint_bits(bitsleft)])
        return res

    # Read n-bytes
    def readBytes(self, a_bytes):
        return self.readBits(a_bytes << 3)

    # Read 1 bit
    def read_bit(self):
        aBit = self.dataPart & 1
        self.bitsFree -= 1
        if self.bitsFree == 0:
            self.fetchNext()
        else:
            self.dataPart >>= 1
        return aBit

    def read_index(self, last, new_way):
        ret = 0
        val = 0
        if new_way and self.read_bit():
            return last + 1
        if new_way and self.read_bit():
            ret = self.read_uint_bits(3)
        else:
            ret = self.read_uint_bits(7)
            val = ret & (32 | 64)
            if val == 32:
                ret = (ret & ~96) | (self.read_uint_bits(2) << 5)
                assert ret >= 32
            elif val == 64:
                ret = (ret & ~96) | (self.read_uint_bits(4) << 5)
                assert ret >= 128
            elif val == 96:
                ret = (ret & ~96) | (self.read_uint_bits(7) << 5)
                assert ret >= 512
        if ret == 0xfff:
            return -1
        return last + 1 + ret

    def decode(self, prop):
        type2 = prop["prop"].type
        assert type2 != c.PT_DataTable
        if type2 == c.PT_Int:
            ret = self._decode_int(prop["prop"])
        elif type2 == c.PT_Float:
            ret = self._decode_float(prop["prop"])
        elif type2 == c.PT_Vector:
            ret = self._decode_vector(prop["prop"])
        elif type2 == c.PT_VectorXY:
            ret = self._decode_vector_xy(prop["prop"])
        elif type2 == c.PT_String:
            ret = self._decode_string()
        elif type2 == c.PT_Int64:
            ret = self._decode_int64(prop["prop"])
        elif type2 == c.PT_Array:
            ret = self._decode_array(prop)
        else:
            raise Exception("Unsupported prop type")
        return ret

    def _decode_int(self, prop):
        if prop.flags & c.SPROP_VARINT:
            if prop.flags & c.SPROP_UNSIGNED:
                ret = self.read_var_int()
            else:
                ret = self.read_var_int()
                ret = ((ret >> 1) ^ (-(ret & 1)))
        else:
            if prop.flags & c.SPROP_UNSIGNED:
                if prop.num_bits == 1:
                    ret = self.read_bit()
                else:
                    ret = self.read_uint_bits(prop.num_bits)
                    # if prop.var_name == "m_hOwnerEntity":
                    #     print(ret, 2 ** prop.num_bits - ret, bin(ret), prop.num_bits)
            else:
                ret = self.read_sint_bits(prop.num_bits)
                ret = self._get_signed_nr(ret, prop.num_bits)
        return ret

    def _decode_float(self, prop):
        val = self._decode_special_float(prop)
        # print("....float val >", val)
        if val is not None:
            return val
        interp = self.read_uint_bits(prop.num_bits)
        val = interp / ((1 << prop.num_bits) - 1)
        val = prop.low_value + (prop.high_value - prop.low_value) * val
        return val

    def _decode_special_float(self, prop):
        val = None
        flags2 = prop.flags
        if flags2 & c.SPROP_COORD:
            val = self._read_bit_coord()
        elif flags2 & c.SPROP_COORD_MP:
            val = self._read_bit_coord_mp(c.CW_None)
        elif flags2 & c.SPROP_COORD_MP_LOWPRECISION:
            val = self._read_bit_coord_mp(c.CW_LowPrecision)
        elif flags2 & c.SPROP_COORD_MP_INTEGRAL:
            val = self._read_bit_coord_mp(c.CW_Integral)
        elif flags2 & c.SPROP_NOSCALE:
            val = struct.unpack("<f", self.readBits(32))[0]  # m_fAccuracyPenalty 1003621115
        elif flags2 & c.SPROP_NORMAL:
            val = self._read_bit_normal()
        elif flags2 & c.SPROP_CELL_COORD:
            val = self._read_bit_cell_coord(prop.num_bits, c.CW_None)
        elif flags2 & c.SPROP_CELL_COORD_LOWPRECISION:
            val = self._read_bit_cell_coord(prop.num_bits, c.CW_LowPrecision)
        elif flags2 & c.SPROP_CELL_COORD_INTEGRAL:
            val = self._read_bit_cell_coord(prop.num_bits, c.CW_Integral)
        return val

    def _read_bit_coord(self):
        int_val = 0
        frac_val = 0
        i2 = self.read_bit()
        f2 = self.read_bit()
        if not i2 and not f2:
            return 0
        sign = self.read_bit()
        if i2:
            int_val = self.read_uint_bits(c.COORD_INTEGER_BITS) + 1
        if f2:
            frac_val = self.read_uint_bits(c.COORD_FRACTIONAL_BITS)
        ret = int_val + (frac_val * c.COORD_RESOLUTION)
        return -ret if sign else ret

    def _read_bit_coord_mp(self, coord_type):
        ret = 0
        sign = False
        integral = (coord_type == c.CW_Integral)
        low_prec = (coord_type == c.CW_LowPrecision)
        if self.read_bit():
            in_bounds = True
        else:
            in_bounds = False
        if integral:
            int_val = self.read_bit()
            if int_val:
                sign = self.read_bit()
                if in_bounds:
                    ret = self.read_uint_bits(c.COORD_INTEGER_BITS_MP) + 1
                else:
                    ret = self.read_uint_bits(c.COORD_INTEGER_BITS) + 1
        else:
            int_val = self.read_bit()
            sign = self.read_bit()
            if int_val:
                if in_bounds:
                    int_val = self.read_uint_bits(c.COORD_INTEGER_BITS_MP) + 1
                else:
                    int_val = self.read_uint_bits(c.COORD_INTEGER_BITS) + 1
            if low_prec:
                frac_val = self.read_uint_bits(c.COORD_FRACTIONAL_BITS_MP_LOWPRECISION)
                ret = int_val + frac_val * c.COORD_RESOLUTION_LOWPRECISION
            else:
                frac_val = self.read_uint_bits(c.COORD_FRACTIONAL_BITS)
                ret = int_val + frac_val * c.COORD_RESOLUTION
        if sign:
            ret = -ret
        return ret

    def _read_bit_normal(self):
        sign = self.read_bit()
        frac = self.read_uint_bits(c.NORMAL_FRACTIONAL_BITS)
        ret = frac * c.NORMAL_RESOLUTION
        return -ret if sign else ret

    def _read_bit_cell_coord(self, bits, coord_type):
        low_prec = (coord_type == c.CW_LowPrecision)
        if coord_type == c.CW_Integral:
            ret = self.read_uint_bits(bits)
        else:
            if coord_type == c.COORD_FRACTIONAL_BITS_MP_LOWPRECISION:
                frac_bits = low_prec
            else:
                frac_bits = c.COORD_FRACTIONAL_BITS
            if low_prec:
                resolution = c.COORD_RESOLUTION_LOWPRECISION
            else:
                resolution = c.COORD_RESOLUTION
            int_val = self.read_uint_bits(bits)
            frac_val = self.read_uint_bits(frac_bits)
            ret = int_val + (frac_val * resolution)
        return ret

    def _decode_vector(self, prop):
        x = self._decode_float(prop)
        y = self._decode_float(prop)
        if prop.flags & c.SPROP_NORMAL == 0:
            z = self._decode_float(prop)
        else:
            sign = self.read_bit()
            sum2 = (x * x) + (y * y)
            if sum2 < 1:
                z = math.sqrt(1 - sum2)
            else:
                z = 0
            if sign:
                z = -z
        return {
            "x": x,
            "y": y,
            "z": z
        }

    def _decode_vector_xy(self, prop):
        x = self._decode_float(prop)
        y = self._decode_float(prop)
        return {
            "x": x,
            "y": y,
            "z": 0
        }

    def _decode_string(self):
        length = self.read_uint_bits(c.DT_MAX_STRING_BITS)
        if not length:
            return ""
        if length >= c.DT_MAX_STRING_BUFFERSIZE:
            length = c.DT_MAX_STRING_BUFFERSIZE - 1
        ret = self.read_string(length)
        return ret

    def _decode_array(self, prop):
        bits = int(math.floor(math.log2(prop["prop"].num_elements))) + 1
        num_elements = self.read_uint_bits(bits)
        elements = list()
        for id2 in range(num_elements):
            real_prop = {"prop": prop["arr"]}
            val = self.decode(real_prop)
            elements.append(val)
        return elements

    def _decode_int64(self, prop):
        sign = False
        if prop.flags & c.SPROP_VARINT:
            if prop.flags & c.SPROP_UNSIGNED:
                ret = self.read_var_int()
            else:
                ret = self.read_var_int()
                ret = ((ret >> 1) ^ (-(ret & 1)))
        else:
            if prop.flags & c.SPROP_UNSIGNED:
                low = self.read_uint_bits(32)
                high = self.read_uint_bits(prop.num_bits - 32)
            else:
                sign = self.read_bit()
                low = self.read_uint_bits(32)
                high = self.read_uint_bits(prop.num_bits - 32 - 1)
            ret = (high << 32) | low
            if sign:
                ret = -ret
        return ret

    def _get_signed_nr(self, number, bitLength):
        mask = (2 ** bitLength) - 1
        if number & (1 << (bitLength - 1)):
            return number | ~mask
        else:
            return number & mask



b = Bitbuffer(b'\x11\x22\x04\x08\x09')

#print(b.bitsFree, b.posByte)
#res = b.read_uint_bits(32)
#print(res)

0
2 67
326

#print(b.bitsFree, b.posByte)
print(b.read_uint_bits(8))