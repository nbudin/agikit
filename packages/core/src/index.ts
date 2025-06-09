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

export * from './Build/BuildLogic';
export * from './Build/BuildPicture';
export * from './Build/BuildSound';
export * from './Build/BuildView';
export * from './Build/BuildWordsTok';
export * from './Build/ProjectBuilder';
export * from './Build/WriteResources';
export * from './Compression/Bitstreams';
export * from './Compression/LZW';
export * from './Extract/DetectGame';
export * from './Extract/GameExtractor';
export * from './Extract/Logic/CodeGeneration';
export * from './Extract/Logic/ReadLogic';
export * from './Extract/Picture/PictureJSON';
export * from './Extract/Picture/ReadPicture';
export * from './Extract/Picture/RenderPicture';
export * from './Extract/ReadResources';
export * from './Extract/Sound/ReadSound';
export * from './Logger';
export * from './Scripting/LogicDiagnostics';
export * from './Scripting/LogicScriptGenerator';
export * from './Scripting/LogicScriptIdentifierMapping';
export * from './Scripting/LogicScriptParser';
export * from './Scripting/LogicScriptParserTypes';
export { SyntaxError as LogicScriptSyntaxError } from './Scripting/LogicScriptParser.generated';
export * from './Scripting/LogicScriptParserTypes';
export * from './Types/Picture';
export * from './Types/Resources';
export * from './Types/Sound';
