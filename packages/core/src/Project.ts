import { existsSync, readFileSync } from 'fs';
import { join } from 'path';
import { ExplicitVolumeSpecification } from './Build/WriteResources';
import { DefaultProjectConfig, ProjectConfig, readProjectConfig } from './ProjectConfig';
import { ResourceType } from './Types/Resources';

export class Project {
  basePath: string;
  config: ProjectConfig;

  constructor(basePath: string) {
    this.basePath = basePath;
    if (existsSync(this.projectConfigPath)) {
      this.config = readProjectConfig(readFileSync(this.projectConfigPath));
    } else {
      this.config = DefaultProjectConfig;
    }
  }

  public get projectConfigPath(): string {
    return join(this.basePath, 'agikit-project.json');
  }

  public get sourcePath(): string {
    return join(this.basePath, 'src');
  }

  public get destinationPath(): string {
    return join(this.basePath, 'build');
  }

  public get wordListSourcePath(): string {
    return join(this.sourcePath, 'words.txt');
  }

  public get objectListSourcePath(): string {
    return join(this.sourcePath, 'object.json');
  }

  public get explicitVolumeConfigPath(): string {
    return join(this.sourcePath, 'resourceVolumes.json');
  }

  readExplicitVolumeConfig(): ExplicitVolumeSpecification[] {
    let explicitVolumes: ExplicitVolumeSpecification[];
    if (existsSync(this.explicitVolumeConfigPath)) {
      console.log(`Reading resource volume specification`);
      const explicitVolumeData = JSON.parse(readFileSync(this.explicitVolumeConfigPath, 'utf-8'));
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

    return explicitVolumes;
  }
}
