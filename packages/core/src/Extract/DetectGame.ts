import { readdirSync } from 'fs';
import { Project, ProjectConfig } from '..';

export function detectGame(path: string): Project {
  const filenames = new Set(readdirSync(path).map((filename) => filename.toUpperCase()));

  if (!['WORDS.TOK', 'OBJECT'].every((requiredFilename) => filenames.has(requiredFilename))) {
    throw new Error(`${path} does not appear to be an AGI game: WORDS.TOK and/or OBJECT missing`);
  }

  if (
    ['VOL.0', 'LOGDIR', 'PICDIR', 'SNDDIR', 'VIEWDIR'].every((v2Filename) =>
      filenames.has(v2Filename),
    )
  ) {
    return new Project(path, {
      agiVersion: { major: 2, minor: 915 },
      gameId: 'AGI',
    });
  }

  const possibleV3GameIDs = [...filenames]
    .filter((filename) => filename.endsWith('DIR'))
    .map((filename) => filename.replace(/DIR$/, ''));

  const v3GameID = possibleV3GameIDs.find((possibleGameID) =>
    ['DIR', 'VOL.0'].every((suffix) => filenames.has(`${possibleGameID}${suffix}`)),
  );

  if (v3GameID == null) {
    throw new Error(
      `${path} appears to possibly be an AGIv3 game, but there was no consistent game ID prefix for DIR and VOL files`,
    );
  }

  return new Project(path, {
    agiVersion: { major: 3, minor: 2.149 },
    gameId: v3GameID,
  });
}
