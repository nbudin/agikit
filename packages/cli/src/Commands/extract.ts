import { AGIVersion, detectGame, GameExtractor, Project, ProjectConfig } from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function extractGame(srcDir: string, destRoot: string, projectConfig?: ProjectConfig): void {
  const project = detectGame(srcDir);
  if (projectConfig) {
    project.config = { ...project.config, ...projectConfig };
  }

  const extractor = new GameExtractor(srcDir, project, new CLILogger());
  extractor.extractGame();
}
