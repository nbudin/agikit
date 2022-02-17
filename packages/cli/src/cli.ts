#!/usr/bin/env ts-node-script

import { ResourceToExtract, ResourceType } from '@agikit/core';
import parseArgs, { ParsedArgs } from 'minimist';
import { buildProject } from './Commands/build';
import { extractGame } from './Commands/extract';
import { formatLogicScript } from './Commands/formatLogic';

function parseResourcesToExtract(
  resourceType: ResourceType,
  arg: string | string[] | undefined | null,
): ResourceToExtract[] {
  if (arg == null) {
    return [];
  }

  if (Array.isArray(arg)) {
    return arg.map((number) => ({ resourceType, resourceNumber: Number.parseInt(number, 10) }));
  }

  return [{ resourceType, resourceNumber: Number.parseInt(arg, 10) }];
}

const commandRunners: { [cmd: string]: (args: ParsedArgs) => void } = {
  build: (args: ParsedArgs) => {
    if (args._.length !== 2) {
      console.error(`Usage: ${process.argv[1]} ${process.argv[2]} projectdir`);
    } else {
      buildProject(args._[1]);
    }
  },
  extract: (args: ParsedArgs) => {
    if (args._.length !== 3) {
      console.error(
        `Usage: ${process.argv[1]} ${process.argv[2]} srcdir destdir [-d] [-l logicnumber] [-v viewnumber] [-p picnumber] [-s soundnumber]...`,
      );
    } else {
      const only: ResourceToExtract[] = [
        ...parseResourcesToExtract(ResourceType.LOGIC, args.l),
        ...parseResourcesToExtract(ResourceType.PIC, args.p),
        ...parseResourcesToExtract(ResourceType.VIEW, args.v),
        ...parseResourcesToExtract(ResourceType.SOUND, args.s),
      ];

      extractGame(args._[1], args._[2], undefined, {
        decompilerDebug: args.d,
        onlyResources: only.length > 0 ? only : undefined,
      });
    }
  },
  formatLogic: (args: ParsedArgs) => {
    if (args._.length !== 2) {
      console.error(`Usage: ${process.argv[1]} ${process.argv[2]} logicfile`);
    } else {
      formatLogicScript(args._[1]);
    }
  },
};

const args = parseArgs(process.argv.slice(2), { boolean: 'd' });
const command = args._[0];

if (!command) {
  console.error('Please specify a command.');
} else if (!commandRunners[command]) {
  console.error(`Unknown command: ${command}`);
}

if (!command || !commandRunners[command]) {
  console.log('');
  console.log('Valid commands:');
  Object.keys(commandRunners).forEach((commandName) => {
    console.log(`  ${commandName}`);
  });
  process.exit(1);
}

commandRunners[command](args);
