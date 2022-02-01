import {
  DirEntry,
  exportWords,
  generateCodeForLogicProgram,
  generateLogicAsm,
  readLogicResource,
  readObjectList,
  readV2Resource,
  readV2ResourceDirs,
  readWordsTok,
  ResourceType,
  WordList,
} from '@agikit/core';
import { writeFileSync, mkdirSync, readFileSync } from 'fs';
import path from 'path';

function extractResource(srcDir: string, entry: DirEntry, destDir: string, wordList: WordList) {
  const resource = readV2Resource(srcDir, entry);
  const destPath = path.join(
    destDir,
    entry.resourceType.toLowerCase(),
    `${entry.resourceNumber}.agi${entry.resourceType.toLowerCase()}`,
  );

  if (entry.resourceType === ResourceType.LOGIC) {
    const logic = readLogicResource(resource.data, { major: 3, minor: 9999 });
    writeFileSync(
      path.join(destDir, entry.resourceType.toLowerCase(), `${entry.resourceNumber}.agiasm`),
      generateLogicAsm(logic, wordList),
    );

    const [code, basicBlockGraph] = generateCodeForLogicProgram(logic, wordList);
    writeFileSync(destPath, code);
    writeFileSync(
      path.join(
        destDir,
        entry.resourceType.toLowerCase(),
        `${entry.resourceNumber}.controlFlowGraph.dot`,
      ),
      basicBlockGraph.generateGraphviz(),
    );
    writeFileSync(
      path.join(
        destDir,
        entry.resourceType.toLowerCase(),
        `${entry.resourceNumber}.dominatorTree.dot`,
      ),
      basicBlockGraph.buildDominatorTree().generateGraphviz(),
    );
    writeFileSync(
      path.join(
        destDir,
        entry.resourceType.toLowerCase(),
        `${entry.resourceNumber}.postDominatorTree.dot`,
      ),
      basicBlockGraph.buildPostDominatorTree().generateGraphviz(),
    );
  } else {
    writeFileSync(destPath, resource.data);
  }
}

export function extractGame(srcDir: string, destRoot: string): void {
  const destDir = path.join(destRoot, 'src');

  const resourceDir = readV2ResourceDirs(srcDir);
  mkdirSync(destDir, { recursive: true });
  const warningResources: DirEntry[] = [];

  console.log('Extracting WORDS.TOK');
  const wordList = readWordsTok(readFileSync(path.join(srcDir, 'WORDS.TOK')));
  writeFileSync(path.join(destDir, 'words.txt'), exportWords(wordList));

  console.log('Extracting OBJECT');
  const objectList = readObjectList(readFileSync(path.join(srcDir, 'OBJECT')));
  writeFileSync(path.join(destDir, 'object.json'), JSON.stringify(objectList, null, 2));

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
        console.log(
          `Extracting ${resourceType} ${entry.resourceNumber} from volume ${entry.volumeNumber}`,
        );
        try {
          extractResource(srcDir, entry, destDir, wordList);
        } catch (err) {
          warningResources.push(entry);
          console.warn(
            `Couldn't extract ${resourceType} ${entry.resourceNumber}: ${err.message}\n${err.stack}`,
          );
        }
      }
    }
  }
}
