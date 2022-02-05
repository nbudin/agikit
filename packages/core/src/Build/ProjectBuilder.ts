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
} from '..';
import { compileLogicScript, LogicCompilerError } from './BuildLogic';
import fileSize from 'filesize';

export abstract class ProjectBuilder {
  project: Project;

  abstract error(message: string): void;
  abstract warn(message: string): void;
  abstract log(message: string): void;

  constructor(project: Project) {
    this.project = project;
  }

  processFile<T>(processor: (input: string) => T, filePath: string) {
    const input = fs.readFileSync(filePath, 'utf-8');
    try {
      return processor(input);
    } catch (error) {
      if (error instanceof LogicScriptSyntaxError || error instanceof WordListSyntaxError) {
        this.error(error.message);
        this.log(
          `${error instanceof SyntaxErrorWithFilePath ? error.filePath : filePath}:${
            error.location.start.line
          }:${error.location.start.column}`,
        );
        process.exit(1);
      } else if (error instanceof LogicCompilerError) {
        this.error(error.message);
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
        this.log(`Compiling ${path.relative(this.project.basePath, scriptPath)}`);
        const [data, diagnostics] = this.processFile(
          (input) => compileLogicScript(input, scriptPath, wordList, objectList),
          scriptPath,
        );

        diagnostics.forEach((diagnostic) =>
          this.warn(LogicCompilerError.describeDiagnostic(scriptPath, diagnostic)),
        );

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
      this.log(`Reading ${path.relative(this.project.basePath, dataPath)}`);
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

    if (this.project.config.agiVersion === '2') {
      writeV2ResourceFiles(
        destinationPath,
        encodeResourceVolumes(resources, encodeV2Resource, this.project.readExplicitVolumeConfig()),
        this.log,
      );
    } else {
      throw new Error(
        `Unknown AGI version ${JSON.stringify(this.project.config.agiVersion)} in ${
          this.project.projectConfigPath
        }`,
      );
    }

    const wordsTokData = buildWordsTok(wordList);
    this.log(`Writing WORDS.TOK (${fileSize(wordsTokData.byteLength, { base: 2 })})`);
    fs.writeFileSync(path.join(destinationPath, 'WORDS.TOK'), wordsTokData);

    const objectData = buildObjectList(objectList);
    this.log(`Writing OBJECT (${fileSize(objectData.byteLength, { base: 2 })})`);
    fs.writeFileSync(path.join(destinationPath, 'OBJECT'), objectData);
  }
}
