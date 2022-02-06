import { AGIVersion, GameExtractor, Project } from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function extractGame(srcDir: string, destRoot: string, agiVersion: AGIVersion): void {
  const project = new Project(destRoot);
  project.config = { agiVersion };

  const extractor = new GameExtractor(srcDir, project, new CLILogger());
  extractor.extractGame();
}
