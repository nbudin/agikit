import { detectGame, ExtractorConfig, GameExtractor, ProjectConfig } from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function extractGame(
  srcDir: string,
  destRoot: string,
  projectConfig?: ProjectConfig,
  options?: ExtractorConfig,
): void {
  const project = detectGame(srcDir);
  if (projectConfig) {
    project.config = { ...project.config, ...projectConfig };
  }
  project.basePath = destRoot;

  const extractor = new GameExtractor(srcDir, project, new CLILogger(), options);
  extractor.extractGame();
}
