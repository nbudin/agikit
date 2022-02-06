// Adapted from https://rosettacode.org/wiki/LZW_compression#ES6_Version
// and https://github.com/astronautlabs/bitstream

import { BitstreamReader, BitstreamWriter } from './Bitstreams';

const START_OVER_CODE = 256;
const END_RESOURCE_CODE = 257;

abstract class LZWDictionary<KeyType, ValueType> {
  mapping: Map<KeyType, ValueType>;
  size: number;

  constructor() {
    // AGI's LZW variant reserves 256 and 257 as "start over" and "end resource"
    this.size = 258;
  }

  has(key: KeyType) {
    return this.mapping.has(key);
  }

  get(key: KeyType) {
    return this.mapping.get(key);
  }

  abstract get codeLength(): number;

  isFull() {
    return this.size >= 2047;
  }
}

class CompressionDictionary extends LZWDictionary<string, number> {
  constructor() {
    super();

    this.mapping = new Map<string, number>();
    for (let i = 0; i < 256; i++) {
      this.mapping.set(String.fromCharCode(i), i);
    }
  }

  get(word: string): number {
    const code = super.get(word);
    if (code == null) {
      throw new Error(`Internal LZW compression error: no code found for ${JSON.stringify(word)}`);
    }
    return code;
  }

  add(word: string) {
    this.mapping.set(word, this.size);
    this.size += 1;
  }

  get codeLength() {
    return Math.ceil(Math.log2(this.size));
  }
}

export function agiLzwCompress(uncompressed: Buffer): Buffer {
  const writer = new BitstreamWriter();
  let dictionary = new CompressionDictionary();

  let word = '';

  for (let i = 0; i < uncompressed.length; i++) {
    const curChar = String.fromCharCode(uncompressed[i]);
    const joinedWord = word + curChar;

    if (dictionary.has(joinedWord)) {
      word = joinedWord;
    } else {
      const wordCode = dictionary.get(word);
      writer.writeCode(wordCode, dictionary.codeLength);

      if (dictionary.isFull()) {
        // dictionary overflow!  write a start over code and reset the dictionary
        writer.writeCode(START_OVER_CODE, dictionary.codeLength);
        dictionary = new CompressionDictionary();
      } else {
        dictionary.add(joinedWord);
      }

      word = curChar;
    }
  }

  if (word !== '') {
    const wordCode = dictionary.get(word);
    writer.writeCode(wordCode, dictionary.codeLength);
  }

  writer.writeCode(END_RESOURCE_CODE, dictionary.codeLength);

  return writer.finish();
}

class DecompressionDictionary extends LZWDictionary<number, string> {
  constructor() {
    super();
    this.mapping = new Map<number, string>();
    for (let i = 0; i < 256; i++) {
      this.mapping.set(i, String.fromCharCode(i));
    }

    this.size = 258;
  }

  add(word: string) {
    this.mapping.set(this.size++, word);
  }

  get codeLength() {
    return Math.ceil(Math.log2(this.size + 1));
  }
}

export function agiLzwDecompress(compressed: Buffer): Buffer {
  let dictionary = new DecompressionDictionary();
  const reader = new BitstreamReader(compressed);

  let word = String.fromCharCode(reader.readCode(dictionary.codeLength));
  let result = word;
  let entry = '';

  while (!reader.done()) {
    const code = reader.readCode(dictionary.codeLength);

    if (code === END_RESOURCE_CODE) {
      // early return, we have reached the end of the resource
      break;
    }

    if (code === START_OVER_CODE) {
      dictionary = new DecompressionDictionary();
      word = String.fromCharCode(reader.readCode(dictionary.codeLength));
    } else {
      const dictionaryWord = dictionary.get(code);
      if (dictionaryWord != null) {
        entry = dictionaryWord;
      } else {
        if (code === dictionary.size) {
          entry = word + word[0];
        } else {
          throw new Error(
            `LZW decompression error: received code ${code} but was expecting ${dictionary.size}`,
          );
        }
      }

      result += entry;
      dictionary.add(word + entry[0]);

      word = entry;
    }
  }

  return Buffer.from(result, 'binary');
}
