export abstract class Logger {
  abstract error(message: string): void;
  abstract warn(message: string): void;
  abstract log(message: string): void;
}

export class ConsoleLogger extends Logger {
  error(message: string): void {
    console.error(message);
  }
  warn(message: string): void {
    console.warn(message);
  }
  log(message: string): void {
    console.log(message);
  }
}
