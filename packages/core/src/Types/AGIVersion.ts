export type AGIVersion = {
  major: 2 | 3;
  minor: number;
};

export function formatVersionNumber(version: AGIVersion) {
  if (version.major === 2) {
    return `${version.major}.${version.minor.toString().padStart(3, '0')}`;
  }

  const sixDigitMinor = version.minor.toString().padStart(6, '0');
  return `${version.major}.${sixDigitMinor.slice(0, 3)}.${sixDigitMinor.slice(3)}`;
}
