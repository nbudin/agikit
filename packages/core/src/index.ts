import {
  ColorPalette,
  egaPalette,
  getAGICommands,
  getAGICommandsByName,
  getDefaultPenSettings,
  getDefaultProjectConfig,
  getTestCommands,
  getTestCommandsByName,
} from '../pkg/agikit_core';

export const EGAPalette: ColorPalette = egaPalette();
export * from '../pkg/agikit_core';

export const agiCommands = getAGICommands();
export const testCommands = getTestCommands();
export const agiCommandsByName = getAGICommandsByName();
export const testCommandsByName = getTestCommandsByName();
export const DefaultProjectConfig = getDefaultProjectConfig();
export const DEFAULT_PEN_SETTINGS = getDefaultPenSettings();
