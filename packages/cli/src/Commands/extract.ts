import {
  AGIVersion,
  detectGame,
  GameExtractor,
  Project,
  ProjectConfig,
  ResourceToExtract,
} from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function extractGame(
  srcDir: string,
  destRoot: string,
  projectConfig?: ProjectConfig,
  only?: ResourceToExtract[],
): void {
  const project = detectGame(srcDir);
  if (projectConfig) {
    project.config = { ...project.config, ...projectConfig };
  }
  project.basePath = destRoot;

  const extractor = new GameExtractor(srcDir, project, new CLILogger(), only);
  extractor.extractGame();
}
