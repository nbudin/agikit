import { mkdirSync, readFileSync, writeFileSync } from 'fs';
import path from 'path';
import {
  buildPictureJSON,
  ConsoleLogger,
  DirEntry,
  exportWords,
  generateCodeForLogicProgram,
  generateLogicAsm,
  Logger,
  Project,
  readLogicResource,
  readObjectList,
  readPictureResource,
  readV2Resource,
  readV2ResourceDirs,
  readWordsTok,
  ResourceType,
  WordList,
} from '..';

export class GameExtractor {
  srcDir: string;
  project: Project;
  logger: Logger;

  constructor(srcDir: string, project: Project, logger?: Logger) {
    this.srcDir = srcDir;
    this.project = project;
    this.logger = logger ?? new ConsoleLogger();
  }

  extractGame(): void {
    const destDir = path.join(this.project.basePath, 'src');

    const resourceDir = readV2ResourceDirs(this.srcDir);
    mkdirSync(destDir, { recursive: true });
    const warningResources: DirEntry[] = [];

    this.logger.log('Extracting WORDS.TOK');
    const wordList = readWordsTok(readFileSync(path.join(this.srcDir, 'WORDS.TOK')));
    writeFileSync(path.join(destDir, 'words.txt'), exportWords(wordList));

    this.logger.log('Extracting OBJECT');
    const objectList = readObjectList(readFileSync(path.join(this.srcDir, 'OBJECT')));
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
          this.logger.log(
            `Extracting ${resourceType} ${entry.resourceNumber} from volume ${entry.volumeNumber}`,
          );
          try {
            this.extractResource(entry, destDir, wordList);
          } catch (err) {
            warningResources.push(entry);
            this.logger.warn(
              `Couldn't extract ${resourceType} ${entry.resourceNumber}: ${err.message}\n${err.stack}`,
            );
          }
        }
      }
    }

    this.logger.log('Writing project config');
    writeFileSync(
      path.join(destDir, 'agikit-project.json'),
      Buffer.from(JSON.stringify(this.project.config, null, 2), 'utf-8'),
    );
  }

  extractResource(entry: DirEntry, destDir: string, wordList: WordList) {
    const resource = readV2Resource(this.srcDir, entry);
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
    } else if (entry.resourceType === ResourceType.PIC) {
      const picture = readPictureResource(resource.data, this.project.config.agiVersion.major >= 3);
      const json = JSON.stringify(buildPictureJSON(picture), null, 2);
      writeFileSync(destPath, Buffer.from(json, 'utf-8'));
    } else {
      writeFileSync(destPath, resource.data);
    }
  }
}
