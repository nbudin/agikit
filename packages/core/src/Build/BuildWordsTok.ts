import { flatMap } from 'lodash';
import { encodeUInt16BE } from '../DataEncoding';
import { WordList } from '../Types/WordList';

const LETTERS = Array.from({ length: 26 }, (_, index) =>
  String.fromCharCode('A'.charCodeAt(0) + index),
);

function encodeWord(word: string): number[] {
  return word.split('').map((letter, index) => {
    let encodedLetter = Buffer.from(letter, 'ascii').readUInt8() ^ 0x7f;
    if (index === word.length - 1) {
      encodedLetter += 0x80;
    }
    return encodedLetter;
  });
}

export function buildWordsTok(wordList: WordList): Buffer {
  const numberByWord = new Map(
    flatMap([...wordList.entries()], ([wordNumber, words]) =>
      [...words].filter((word) => word !== 'ANYWORD').map((word) => [word, wordNumber]),
    ),
  );
  const sortedWords = [...numberByWord.keys()].sort();
  const wordsEntriesByFirstLetter = new Map<string, Buffer[]>(
    LETTERS.map((letter) => [letter, []]),
  );

  let previousWord = '';
  let previousFirstLetter = '';

  sortedWords.forEach((word) => {
    const firstLetter = word.slice(0, 1).toUpperCase();
    if (firstLetter !== previousFirstLetter) {
      previousWord = '';
    }

    let commonChars = 0;
    while (
      commonChars < word.length &&
      commonChars < previousWord.length &&
      word[commonChars] === previousWord[commonChars]
    ) {
      commonChars += 1;
    }

    const encodedWord: number[] = [commonChars, ...encodeWord(word.slice(commonChars))];
    const wordNumber = numberByWord.get(word);
    if (wordNumber == null) {
      throw new Error(`Can't find number for word "${word}"`);
    }
    encodedWord.push(...encodeUInt16BE(wordNumber));

    const wordEntries = wordsEntriesByFirstLetter.get(firstLetter);
    if (wordEntries == null) {
      throw new Error(`Word "${word}" does not begin with a letter between A and Z`);
    }
    wordEntries.push(Buffer.from(encodedWord));

    previousWord = word;
    previousFirstLetter = firstLetter;
  });

  const header = Buffer.alloc(52);
  let data: Buffer = Buffer.from([0, ...encodeWord('ANYWORD'), ...encodeUInt16BE(1)]);
  let offset = 52 + data.byteLength;
  LETTERS.forEach((letter, index) => {
    const wordEntries = wordsEntriesByFirstLetter.get(letter);
    if (wordEntries == null || wordEntries.length === 0) {
      header.writeUInt16BE(0, index * 2);
    } else {
      header.writeUInt16BE(offset, index * 2);
      data = Buffer.concat([data, ...wordEntries]);
      offset = 52 + data.byteLength;
    }
  });

  return Buffer.concat([header, data]);
}
