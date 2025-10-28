import { detectGame, ExtractConfig, ProjectConfig } from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function extractGame(
  srcDir: string,
  destRoot: string,
  projectConfig?: ProjectConfig,
  options?: ExtractConfig,
): void {
  const project = detectGame(srcDir);
  if (projectConfig) {
    project.config = projectConfig;
  }
  project.basePath = destRoot;

  project.extract(srcDir, options);
}
