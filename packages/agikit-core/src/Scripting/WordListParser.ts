import { WordList } from '../Types/WordList';
import { parse, SyntaxError } from './WordListParser.generated';

type WordDeclaration = {
  number: number;
  synonyms: string[];
};

export function parseWordList(input: string): WordList {
  const declarations = parse(input) as WordDeclaration[];
  const wordList: WordList = new Map();

  declarations.forEach((declaration) => {
    wordList.set(declaration.number, new Set(declaration.synonyms));
  });

  return wordList;
}

export { SyntaxError };
