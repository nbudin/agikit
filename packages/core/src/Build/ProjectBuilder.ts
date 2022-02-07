import fs from 'fs';
import path from 'path';
import { Project } from '../Project';
import {
  LogicScriptSyntaxError,
  WordListSyntaxError,
  SyntaxErrorWithFilePath,
  ObjectList,
  Resource,
  ResourceType,
  WordList,
  parseWordList,
  buildWordsTok,
  buildObjectList,
  writeV2ResourceFiles,
  encodeResourceVolumes,
  encodeV2Resource,
  buildPicture,
  readPictureJSON,
  Logger,
  ConsoleLogger,
  writeV3ResourceFiles,
  encodeV3Resource,
} from '..';
import { compileLogicScript, LogicCompilerError } from './BuildLogic';
import fileSize from 'filesize';

export class ProjectBuilder {
  project: Project;
  logger: Logger;

  constructor(project: Project, logger?: Logger) {
    this.project = project;
    this.logger = logger ?? new ConsoleLogger();
  }

  processFile<T>(processor: (input: string) => T, filePath: string) {
    const input = fs.readFileSync(filePath, 'utf-8');
    try {
      return processor(input);
    } catch (error) {
      if (error instanceof LogicScriptSyntaxError || error instanceof WordListSyntaxError) {
        this.logger.error(error.message);
        this.logger.log(
          `${error instanceof SyntaxErrorWithFilePath ? error.filePath : filePath}:${
            error.location.start.line
          }:${error.location.start.column}`,
        );
        process.exit(1);
      } else if (error instanceof LogicCompilerError) {
        this.logger.error(error.message);
        process.exit(1);
      } else {
        throw error;
      }
    }
  }

  buildResource(
    resourceType: ResourceType,
    resourceNumber: number,
    wordList: WordList,
    objectList: ObjectList,
  ): Resource | undefined {
    if (resourceType === ResourceType.LOGIC) {
      const scriptPath = path.join(this.project.sourcePath, 'logic', `${resourceNumber}.agilogic`);
      if (fs.existsSync(scriptPath)) {
        this.logger.log(`Compiling ${path.relative(this.project.basePath, scriptPath)}`);
        const [data, diagnostics] = this.processFile(
          (input) => compileLogicScript(input, scriptPath, wordList, objectList),
          scriptPath,
        );

        diagnostics.forEach((diagnostic) =>
          this.logger.warn(LogicCompilerError.describeDiagnostic(scriptPath, diagnostic)),
        );

        return {
          data,
          number: resourceNumber,
          type: resourceType,
        };
      }
    }

    if (resourceType === ResourceType.PIC) {
      const picPath = path.join(this.project.sourcePath, 'pic', `${resourceNumber}.agipic`);
      if (fs.existsSync(picPath)) {
        let data: Buffer;

        this.logger.log(`Compiling ${path.relative(this.project.basePath, picPath)}`);
        try {
          const jsonData = JSON.parse(fs.readFileSync(picPath, 'utf-8'));
          data = buildPicture(readPictureJSON(jsonData), this.project.config.agiVersion.major >= 3);
        } catch (error) {
          this.logger.warn(
            `Compilation failed, treating ${path.relative(
              this.project.basePath,
              picPath,
            )} as raw PIC data`,
          );
          data = fs.readFileSync(picPath);
        }

        return {
          data,
          number: resourceNumber,
          type: resourceType,
        };
      }
    }

    const dataPath = path.join(
      this.project.sourcePath,
      resourceType.toLowerCase(),
      `${resourceNumber}.agi${resourceType.toLowerCase()}`,
    );
    if (fs.existsSync(dataPath)) {
      this.logger.log(`Reading ${path.relative(this.project.basePath, dataPath)}`);
      const data = fs.readFileSync(dataPath);
      return {
        data,
        number: resourceNumber,
        type: resourceType,
      };
    }

    return undefined;
  }

  buildProject(): void {
    const destinationPath = this.project.destinationPath;

    fs.mkdirSync(destinationPath, { recursive: true });
    const wordList = this.processFile(parseWordList, this.project.wordListSourcePath);
    const objectList = this.processFile(
      (input) => JSON.parse(input) as ObjectList,
      this.project.objectListSourcePath,
    );

    const resources: Resource[] = [];
    for (const resourceType of [
      ResourceType.PIC,
      ResourceType.VIEW,
      ResourceType.SOUND,
      ResourceType.LOGIC,
    ]) {
      for (let resourceNumber = 0; resourceNumber < 256; resourceNumber++) {
        const resource = this.buildResource(resourceType, resourceNumber, wordList, objectList);
        if (resource) {
          resources.push(resource);
        }
      }
    }

    if (this.project.config.agiVersion.major === 2) {
      writeV2ResourceFiles(
        destinationPath,
        encodeResourceVolumes(resources, encodeV2Resource, this.project.readExplicitVolumeConfig()),
        this.logger,
      );
    } else if (this.project.config.agiVersion.major === 3) {
      writeV3ResourceFiles(
        destinationPath,
        this.project.config.gameId,
        encodeResourceVolumes(resources, encodeV3Resource, this.project.readExplicitVolumeConfig()),
        this.logger,
      );
    } else {
      throw new Error(
        `Unknown AGI version ${JSON.stringify(this.project.config.agiVersion)} in ${
          this.project.projectConfigPath
        }`,
      );
    }

    const wordsTokData = buildWordsTok(wordList);
    this.logger.log(`Writing WORDS.TOK (${fileSize(wordsTokData.byteLength, { base: 2 })})`);
    fs.writeFileSync(path.join(destinationPath, 'WORDS.TOK'), wordsTokData);

    const objectData = buildObjectList(objectList);
    this.logger.log(`Writing OBJECT (${fileSize(objectData.byteLength, { base: 2 })})`);
    fs.writeFileSync(path.join(destinationPath, 'OBJECT'), objectData);
  }
}
