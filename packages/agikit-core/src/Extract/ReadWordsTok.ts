import { WordList } from '../Types/WordList';

export function readWordsTok(wordsTokData: Buffer): WordList {
  let offset = 52; // skip header
  let previousWord = '';
  let currentWord = '';
  const words = new Map<number, Set<string>>();

  while (offset < wordsTokData.byteLength) {
    const reuseChars = wordsTokData.readUInt8(offset);
    offset += 1;
    currentWord = previousWord.slice(0, reuseChars);

    while (offset < wordsTokData.byteLength) {
      const char = wordsTokData.readUInt8(offset);
      offset += 1;

      // high bytes are used for the end of the word
      if (char > 0x7f) {
        currentWord += Buffer.from([char ^ 0x7f]).toString('ascii');
        previousWord = currentWord;
        const wordNumber = wordsTokData.readUInt16BE(offset);
        offset += 2;

        if (!words.has(wordNumber)) {
          words.set(wordNumber, new Set<string>());
        }
        words.get(wordNumber)?.add(currentWord);
        break;
      }

      currentWord += Buffer.from([char ^ 0x7f]).toString('ascii');
    }
  }

  return words;
}

function formatWord(word: string) {
  if (word.match(/[^A-Za-z0-9\-_]/)) {
    return `"${word.replace(/"/g, '\\"')}"`;
  }

  return word;
}

export function exportWords(wordList: WordList): string {
  const wordNumbers = [...wordList.keys()].sort((a, b) => a - b);
  return wordNumbers
    .map((wordNumber) => {
      const words = [...(wordList.get(wordNumber)?.values() ?? [])].sort();
      return `${wordNumber}: ${words.map(formatWord).join(' ')}`;
    })
    .join('\n');
}
