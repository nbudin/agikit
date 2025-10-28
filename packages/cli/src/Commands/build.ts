import { Project } from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function buildProject(basePath: string) {
  const project = new Project(basePath);
  project.build();
}
