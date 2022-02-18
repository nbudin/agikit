// Adapted from Lance Ewing's XV3
// http://agiwiki.sierrahelp.com/index.php?title=AGIv3_Resource_Extractor_(XV3)

export class PicBitstreamReader {
  bitstream: Buffer;
  bitOffset: number;

  constructor(bitstream: Buffer) {
    this.bitstream = bitstream;
    this.bitOffset = 0;
  }

  get byteOffset() {
    return Math.floor(this.bitOffset / 8);
  }

  readCode(bitLength: number) {
    let code = 0;
    let remainingLength = bitLength;

    while (remainingLength > 0) {
      const byte = this.bitstream.readUInt8(Math.floor(this.bitOffset / 8));
      const bitContribution = Math.min(8 - (this.bitOffset % 8), remainingLength);
      const mask = Math.pow(2, bitContribution) - 1;
      const contribution = (byte >> (8 - bitContribution - (this.bitOffset % 8))) & mask;

      code = (code << bitContribution) | contribution;

      this.bitOffset += bitContribution;
      remainingLength -= bitContribution;
    }

    return code;
  }

  peekCode(bitLength: number) {
    const code = this.readCode(bitLength);
    this.bitOffset -= bitLength;
    return code;
  }

  seekBits(bits: number) {
    this.bitOffset += bits;
  }

  done() {
    return this.byteOffset >= this.bitstream.byteLength;
  }
}

export class PicBitstreamWriter {
  currentByte = 0;
  currentByteOffset = 0;
  bytes: number[] = [];

  finish() {
    if (this.currentByteOffset > 0) {
      this.flushCurrentByte();
    }

    return Buffer.from(this.bytes);
  }

  flushCurrentByte() {
    this.bytes.push(this.currentByte);
    this.currentByte = 0;
    this.currentByteOffset = 0;
  }

  writeCode(code: number, bitLength: number) {
    let workingCode = code;
    let remainingBits = bitLength;
    while (remainingBits > 0) {
      const shift = 8 - this.currentByteOffset - remainingBits;
      const contribution = shift >= 0 ? workingCode << shift : workingCode >>> -shift;
      const writtenLength = shift >= 0 ? remainingBits : 8 - this.currentByteOffset;

      this.currentByte |= contribution;
      this.currentByteOffset += writtenLength;
      workingCode -= shift >= 0 ? contribution >>> shift : contribution << -shift;
      remainingBits -= writtenLength;

      if (this.currentByteOffset === 8) {
        this.flushCurrentByte();
      }
    }
  }
}

export class LZWBitstreamReader {
  bitstream: Buffer;
  bitOffset: number;
  inputBitBuffer: number;
  inputBitCount: number;
  bitstreamBitLength: number;

  constructor(bitstream: Buffer) {
    this.bitstream = bitstream;
    this.inputBitBuffer = 0;
    this.bitOffset = 0;
    this.inputBitCount = 0;
    this.bitstreamBitLength = this.bitstream.byteLength * 8;
  }

  get byteOffset() {
    return Math.floor(this.bitOffset / 8);
  }

  readCode(bitLength: number) {
    while (this.inputBitCount <= 24 && this.bitOffset < this.bitstreamBitLength) {
      const byte = this.bitstream.readUInt8(Math.floor(this.bitOffset / 8));
      this.inputBitBuffer |= byte << this.inputBitCount;
      this.bitOffset += 8;
      this.inputBitCount += 8;
    }

    const code = (this.inputBitBuffer & 0x7fff) % (1 << bitLength);
    this.inputBitBuffer = this.inputBitBuffer >>> bitLength;
    this.inputBitCount -= bitLength;

    return code;
  }

  peekCode(bitLength: number) {
    const code = this.readCode(bitLength);
    this.bitOffset -= bitLength;
    return code;
  }

  seekBits(bits: number) {
    this.bitOffset += bits;
  }

  done() {
    return this.byteOffset >= this.bitstream.byteLength && this.inputBitCount < 8;
  }
}

export class LZWBitstreamWriter extends PicBitstreamWriter {
  writeCode(code: number, bitLength: number) {
    // AGI's LZWv3 implementation writes the low-order bits first
    let workingCode = code;
    let remainingBits = bitLength;
    while (remainingBits > 0) {
      const writtenLength = Math.min(8 - this.currentByteOffset, remainingBits);
      const mask = 2 ** writtenLength - 1;
      const contribution = (workingCode & mask) << this.currentByteOffset;

      workingCode = workingCode >>> writtenLength;

      this.currentByte |= contribution;
      this.currentByteOffset += writtenLength;
      remainingBits -= writtenLength;

      if (this.currentByteOffset === 8) {
        this.flushCurrentByte();
      }
    }
  }
}
