import { readV2Resource, readV2ResourceDirs } from '../Extract/ReadResources';
import { readLogicResource } from '../Extract/Logic/ReadLogic';
import { mkdirSync, readFileSync, writeFileSync } from 'fs';
import path from 'path';
import { WordList } from '../Types/WordList';
import { readWordsTok, exportWords } from '../Extract/ReadWordsTok';
import { DirEntry, ResourceType } from '../Types/Resources';
import { generateLogicAsm, generateCodeForLogicResource } from '../Extract/Logic/CodeGeneration';

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

    const [code, basicBlockGraph] = generateCodeForLogicResource(logic, wordList);
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

export function extractGame(srcDir: string, destDir: string): void {
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
