import { mkdirSync, readFileSync, writeFileSync } from 'fs';
import path from 'path';
import {
  buildPictureJSON,
  ConsoleLogger,
  DirEntry,
  exportWords,
  formatVersionNumber,
  generateCodeForLogicProgram,
  generateLogicAsm,
  Logger,
  Project,
  readLogicResource,
  readObjectList,
  readPictureResource,
  readV2Resource,
  readV2ResourceDirs,
  readV3Resource,
  readV3ResourceDir,
  readWordsTok,
  ResourceType,
  WordList,
} from '..';

export type ResourceToExtract = {
  resourceType: ResourceType;
  resourceNumber: number;
};

export type ExtractorConfig = {
  decompilerDebug?: boolean;
  onlyResources?: ResourceToExtract[];
};

export class GameExtractor {
  srcDir: string;
  project: Project;
  logger: Logger;
  options?: ExtractorConfig;
  only?: ResourceToExtract[];

  constructor(srcDir: string, project: Project, logger?: Logger, options?: ExtractorConfig) {
    this.srcDir = srcDir;
    this.project = project;
    this.logger = logger ?? new ConsoleLogger();
    this.only = options?.onlyResources;
    this.options = options;
  }

  extractGame(): void {
    this.logger.log(`Extracting ${this.srcDir} to ${this.project.basePath}`);
    this.logger.log(`Using AGI version ${formatVersionNumber(this.project.config.agiVersion)}`);
    this.logger.log(`Game ID: ${this.project.config.gameId}`);

    const destDir = path.join(this.project.basePath, 'src');

    const resourceDir =
      this.project.config.agiVersion.major >= 3
        ? readV3ResourceDir(this.srcDir, this.project.config.gameId)
        : readV2ResourceDirs(this.srcDir);
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
        if (
          entry != null &&
          this.only != null &&
          !this.only.some(
            (resourceToExtract) =>
              resourceToExtract.resourceNumber === entry.resourceNumber &&
              resourceToExtract.resourceType === entry.resourceType,
          )
        ) {
          continue;
        }

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
    const resource =
      this.project.config.agiVersion.major >= 3
        ? readV3Resource(this.srcDir, entry, this.project.config.gameId)
        : readV2Resource(this.srcDir, entry);

    const destPath = path.join(
      destDir,
      entry.resourceType.toLowerCase(),
      `${entry.resourceNumber}.agi${entry.resourceType.toLowerCase()}`,
    );

    if (entry.resourceType === ResourceType.LOGIC) {
      const logic = readLogicResource(resource.data, this.project.config.agiVersion);
      if (this.options?.decompilerDebug) {
        writeFileSync(
          path.join(destDir, entry.resourceType.toLowerCase(), `${entry.resourceNumber}.agiasm`),
          generateLogicAsm(logic, wordList),
        );
      }

      const [code, basicBlockGraph] = generateCodeForLogicProgram(logic, wordList);
      writeFileSync(destPath, code);
      if (this.options?.decompilerDebug) {
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
      }
    } else if (entry.resourceType === ResourceType.PIC) {
      const picture = readPictureResource(resource.data, this.project.config.agiVersion.major >= 3);
      const json = JSON.stringify(buildPictureJSON(picture), null, 2);
      writeFileSync(destPath, Buffer.from(json, 'utf-8'));
    } else {
      writeFileSync(destPath, resource.data);
    }
  }
}
