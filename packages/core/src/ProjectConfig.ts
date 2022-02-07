import { AGIVersion } from './Types/AGIVersion';

export type ProjectConfig = {
  agiVersion: AGIVersion;
  gameId: string;
};

export const DefaultProjectConfig: ProjectConfig = {
  agiVersion: {
    major: 2,
    minor: 936,
  },
  gameId: 'AGI',
};

export function readProjectConfig(projectConfigData: Buffer): ProjectConfig {
  const json = JSON.parse(projectConfigData.toString('utf-8'));

  return {
    ...DefaultProjectConfig,
    ...json,
  };
}

export function encodeProjectConfig(projectConfig: ProjectConfig): Buffer {
  return Buffer.from(JSON.stringify(projectConfig), 'utf-8');
}
