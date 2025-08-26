import {
  ColorPalette,
  egaPalette,
  getAGICommands,
  getAGICommandsByName,
  getDefaultProjectConfig,
  getTestCommands,
  getTestCommandsByName,
} from 'agikit_core';

export const EGAPalette: ColorPalette = egaPalette();
export * from 'agikit_core';

export const agiCommands = getAGICommands();
export const testCommands = getTestCommands();
export const agiCommandsByName = getAGICommandsByName();
export const testCommandsByName = getTestCommandsByName();
export const DefaultProjectConfig = getDefaultProjectConfig();
