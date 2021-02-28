import path from 'path';
import fs from 'fs';
import { Resource, ResourceType } from '../Types/Resources';
import { compileLogicScript } from '../Build/BuildLogic';
import { SyntaxError as LogicSyntaxError } from '../Scripting/LogicScriptParser';
import {
  encodeResourceVolumes,
  encodeV2Resource,
  ExplicitVolumeSpecification,
  writeV2ResourceFiles,
} from '../Build/WriteResources';
import { parseWordList, SyntaxError as WordListSyntaxError } from '../Scripting/WordListParser';
import { WordList } from '../Types/WordList';
import { ObjectList } from '../Types/ObjectList';
import { buildObjectList } from '../Build/BuildObjectList';

function processFile<T>(processor: (input: string) => T, filePath: string) {
  const input = fs.readFileSync(filePath, 'utf-8');
  try {
    return processor(input);
  } catch (error) {
    if (error instanceof LogicSyntaxError || error instanceof WordListSyntaxError) {
      console.error(error.message);
      console.log(`${filePath}:${error.location.start.line}:${error.location.start.column}`);
      process.exit(1);
    } else {
      throw error;
    }
  }
}

function buildResource(
  sourceDir: string,
  resourceType: ResourceType,
  resourceNumber: number,
  wordList: WordList,
): Resource | undefined {
  if (resourceType === ResourceType.LOGIC) {
    const scriptPath = path.join(sourceDir, 'logic', `${resourceNumber}.agilogic`);
    if (fs.existsSync(scriptPath)) {
      console.log(`Compiling ${scriptPath}`);
      const data = processFile((input) => compileLogicScript(input, wordList), scriptPath);

      return {
        data,
        number: resourceNumber,
        type: resourceType,
      };
    }
  }

  const dataPath = path.join(
    sourceDir,
    resourceType.toLowerCase(),
    `${resourceNumber}.agi${resourceType.toLowerCase()}`,
  );
  if (fs.existsSync(dataPath)) {
    console.log(`Reading ${dataPath}`);
    const data = fs.readFileSync(dataPath);
    return {
      data,
      number: resourceNumber,
      type: resourceType,
    };
  }

  return undefined;
}

export function buildGame(sourceDir: string, destinationDir: string): void {
  fs.mkdirSync(destinationDir, { recursive: true });
  const wordList = processFile(parseWordList, path.join(sourceDir, 'words.txt'));
  const objectList = processFile(
    (input) => JSON.parse(input) as ObjectList,
    path.join(sourceDir, 'object.json'),
  );

  const resources: Resource[] = [];
  for (const resourceType of [
    ResourceType.PIC,
    ResourceType.VIEW,
    ResourceType.SOUND,
    ResourceType.LOGIC,
  ]) {
    for (let resourceNumber = 0; resourceNumber < 256; resourceNumber++) {
      const resource = buildResource(sourceDir, resourceType, resourceNumber, wordList);
      if (resource) {
        resources.push(resource);
      }
    }
  }

  const explicitVolumesPath = path.join(sourceDir, 'resourceVolumes.json');
  let explicitVolumes: ExplicitVolumeSpecification[];
  if (fs.existsSync(explicitVolumesPath)) {
    console.log(`Reading resource volume specification`);
    const explicitVolumeData = JSON.parse(fs.readFileSync(explicitVolumesPath, 'utf-8'));
    explicitVolumes = explicitVolumeData.map(
      (volumeData: { number: number; resources: Partial<Record<ResourceType, number[]>> }) => {
        const explicitResources: ExplicitVolumeSpecification['resources'] = [];
        for (const resourceType of [
          ResourceType.PIC,
          ResourceType.VIEW,
          ResourceType.SOUND,
          ResourceType.LOGIC,
        ]) {
          volumeData.resources[resourceType]?.forEach((resourceNumber: number) => {
            explicitResources.push({ resourceType, resourceNumber });
          });
        }

        return { number: volumeData.number, resources: explicitResources };
      },
    );
  } else {
    explicitVolumes = [];
  }

  console.log(`Writing resource files`);
  writeV2ResourceFiles(
    destinationDir,
    encodeResourceVolumes(resources, encodeV2Resource, explicitVolumes),
  );

  console.log(`Writing OBJECT`);
  fs.writeFileSync(path.join(destinationDir, 'OBJECT'), buildObjectList(objectList));
}
