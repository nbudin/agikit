import parseArgs, { ParsedArgs } from 'minimist';
import { buildGame } from './Commands/build';
import { extractGame } from './Commands/extract';
import { formatLogicScript } from './Commands/formatLogic';

const commandRunners: { [cmd: string]: (args: ParsedArgs) => void } = {
  build: (args: ParsedArgs) => {
    if (args._.length !== 2) {
      console.error(`Usage: ${process.argv[1]} ${process.argv[2]} projectdir`);
    } else {
      buildGame(args._[1]);
    }
  },
  extract: (args: ParsedArgs) => {
    if (args._.length !== 3) {
      console.error(`Usage: ${process.argv[1]} ${process.argv[2]} srcdir destdir`);
    } else {
      extractGame(args._[1], args._[2]);
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

const args = parseArgs(process.argv.slice(2));
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
