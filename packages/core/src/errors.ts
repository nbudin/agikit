export class WordListSyntaxError extends Error {
  line: number;
  column: number;
  offset: number;

  constructor(message: string, line: number, column: number, offset: number) {
    super(message);
    this.name = 'WordListSyntaxError';
    this.line = line;
    this.column = column;
    this.offset = offset;
  }

  get location() {
    return {
      start: {
        line: this.line,
        column: this.column,
        offset: this.offset,
      },
    };
  }
}
