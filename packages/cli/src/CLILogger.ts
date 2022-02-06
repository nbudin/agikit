import { Logger } from '@agikit/core';
import ansiColors, { bgRedBright, black, bgYellowBright, bgGreenBright } from 'ansi-colors';
import colorSupport from 'color-support';

export class CLILogger extends Logger {
  constructor() {
    super();

    const result = colorSupport();
    ansiColors.enabled = result && result.hasBasic;
  }

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
