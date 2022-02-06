import { Project, ProjectBuilder } from '@agikit/core';
import { CLILogger } from '../CLILogger';

export function buildProject(basePath: string) {
  const project = new Project(basePath);
  const builder = new ProjectBuilder(project, new CLILogger());
  builder.buildProject();
}
