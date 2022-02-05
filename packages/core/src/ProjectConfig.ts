export type ProjectConfig = {
  agiVersion: '2' | '3';
};

export const DefaultProjectConfig: ProjectConfig = {
  agiVersion: '2',
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
