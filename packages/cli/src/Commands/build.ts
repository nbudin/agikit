import { Project, ProjectBuilder } from '@agikit/core';
import { bgGreenBright, bgRedBright, bgYellowBright, black } from 'ansi-colors';

export class CLIProjectBuilder extends ProjectBuilder {
  error(message: string): void {
    console.error(`${bgRedBright(black('[ERROR]'))} ${message}`);
  }

  warn(message: string): void {
    console.warn(`${bgYellowBright(black('[WARN]'))}  ${message}`);
  }

  log(message: string): void {
    console.log(`${bgGreenBright(black('[INFO]'))}  ${message}`);
  }
}

export function buildProject(basePath: string) {
  const project = new Project(basePath);
  const builder = new CLIProjectBuilder(project);
  builder.buildProject();
}
