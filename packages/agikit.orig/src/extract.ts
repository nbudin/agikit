import parseArgs from 'minimist';
import { readV2Resource, readV2ResourceDirs } from './Extract/ReadResources';
import { readLogicResource } from './Extract/ReadLogic';
import { mkdirSync, readFileSync, writeFileSync } from 'fs';
import path from 'path';
import { WordList } from './Types/WordList';
import { readWordsTok, exportWords } from './Extract/ReadWordsTok';
import { DirEntry, ResourceType } from './Types/Resources';
import { generateLogicAsm, generateCodeForLogicResource } from './Extract/CodeGeneration';

function extractResource(srcDir: string, entry: DirEntry, destDir: string, wordList: WordList) {
  const resourceData = readV2Resource(srcDir, entry);
  const destPath = path.join(
    destDir,
    entry.resourceType.toLowerCase(),
    `${entry.resourceNumber}.agi${entry.resourceType.toLowerCase()}`,
  );

  if (entry.resourceType === ResourceType.LOGIC) {
    // TODO remove condition
    if (entry.resourceNumber === 27) {
      const logic = readLogicResource(resourceData, { major: 3, minor: 9999 });
      writeFileSync(
        path.join(destDir, entry.resourceType.toLowerCase(), `${entry.resourceNumber}.agiasm`),
        generateLogicAsm(logic, wordList),
      );

      const code = generateCodeForLogicResource(logic, wordList);
      writeFileSync(destPath, code);
    }
  } else {
    writeFileSync(destPath, resourceData);
  }
}

function extractGame(srcDir: string, destDir: string) {
  const resourceDir = readV2ResourceDirs(srcDir);
  mkdirSync(destDir, { recursive: true });
  const warningResources: DirEntry[] = [];

  console.log('Extracting WORDS.TOK');
  const wordList = readWordsTok(readFileSync(path.join(srcDir, 'WORDS.TOK')));
  writeFileSync(path.join(destDir, 'words.txt'), exportWords(wordList));

  for (const resourceType of [
    ResourceType.PIC,
    ResourceType.VIEW,
    ResourceType.SOUND,
    ResourceType.LOGIC,
  ]) {
    mkdirSync(path.join(destDir, resourceType.toLowerCase()), {
      recursive: true,
    });
    for (const entry of resourceDir[resourceType]) {
      if (entry != null) {
        console.log(`Extracting ${resourceType} ${entry.resourceNumber}`);
        try {
          extractResource(srcDir, entry, destDir, wordList);
        } catch (err) {
          warningResources.push(entry);
          console.warn(`Couldn't extract ${resourceType} ${entry.resourceNumber}: ${err.message}`);
        }
      }
    }
  }
}

const args = parseArgs(process.argv.slice(2));
if (args._.length !== 2) {
  console.error(`Usage: ${process.argv[1]} srcdir destdir`);
} else {
  extractGame(args._[0], args._[1]);
}
