import * as vscode from 'vscode';
import * as path from 'path';
import * as child_process from 'child_process';
import {
  LoggingDebugSession,
  InitializedEvent,
  logger,
  Logger,
  ExitedEvent,
  OutputEvent,
  TerminatedEvent,
} from '@vscode/debugadapter';
import { DebugProtocol } from '@vscode/debugprotocol';
import { Project } from '@agikit/core';

interface LaunchRequestArguments extends DebugProtocol.LaunchRequestArguments {
  /** An absolute path to the agikit project to debug. */
  projectPath: string;
  /** An absolute path to the ScummVM executable.  If none is provided, the adapter will attempt to find one. */
  scummvmPath?: string;
}

class ScummVMDebugSession extends LoggingDebugSession {
  _configurationDone = false;
  configurationDoneEmitter = new vscode.EventEmitter<void>();
  process?: child_process.ChildProcess;

  protected initializeRequest(
    response: DebugProtocol.InitializeResponse,
    args: DebugProtocol.InitializeRequestArguments,
  ): void {
    // build and return the capabilities of this debug adapter:
    response.body = response.body || {};

    response.body.supportsConfigurationDoneRequest = true;
    response.body.supportsTerminateRequest = true;

    response.body.supportsCancelRequest = false;
    response.body.supportsEvaluateForHovers = false;
    response.body.supportsStepBack = false;
    response.body.supportsDataBreakpoints = false;
    response.body.supportsCompletionsRequest = false;
    response.body.supportsBreakpointLocationsRequest = false;

    this.sendResponse(response);

    // since this debug adapter can accept configuration requests like 'setBreakpoint' at any time,
    // we request them early by sending an 'initializeRequest' to the frontend.
    // The frontend will end the configuration sequence by calling 'configurationDone' request.
    this.sendEvent(new InitializedEvent());
  }

  /**
   * Called at the end of the configuration sequence.
   * Indicates that all breakpoints etc. have been sent to the DA and that the 'launch' can start.
   */
  protected configurationDoneRequest(
    response: DebugProtocol.ConfigurationDoneResponse,
    args: DebugProtocol.ConfigurationDoneArguments,
  ): void {
    super.configurationDoneRequest(response, args);

    // notify the launchRequest that configuration has finished
    this._configurationDone = true;
    this.configurationDoneEmitter.fire();
  }

  protected async launchRequest(
    response: DebugProtocol.LaunchResponse,
    args: LaunchRequestArguments,
  ) {
    logger.setup(Logger.LogLevel.Verbose, false);

    // wait until configuration has finished (and configurationDoneRequest has been called)
    if (!this._configurationDone) {
      await new Promise<void>((resolve) => {
        this.configurationDoneEmitter.event(() => resolve());
      });
    }

    const scummvmPath =
      args.scummvmPath ?? vscode.workspace.getConfiguration().get('agikit.scummvmPath');
    if (!scummvmPath) {
      response.success = false;
      response.body = {
        error:
          'ScummVM could not be found.  Please configure the path to ScummVM in your settings.',
      };
      return response;
    }

    const buildPath = path.join(args.projectPath, 'build');
    const gameId = 'agi-fanmade';

    this.sendEvent(
      new OutputEvent(`Starting ScummVM: "${scummvmPath}" -p "${buildPath}" ${gameId}`),
    );
    this.process = child_process.spawn(scummvmPath, ['-p', buildPath, gameId], {
      stdio: 'pipe',
      detached: true,
    });

    this.process.stdout?.on('data', (chunk) => {
      this.sendEvent(new OutputEvent(chunk));
    });
    this.process.stderr?.on('data', (chunk) => {
      this.sendEvent(new OutputEvent(chunk));
    });

    this.process.on('exit', (code) => {
      this.sendEvent(new ExitedEvent(code ?? 0));
      this.sendEvent(new TerminatedEvent());
    });

    this.process.on('error', (err) => {
      this.sendEvent(new OutputEvent(err.message, 'error'));
    });

    response.success = true;
    this.sendResponse(response);
  }

  protected terminateRequest(
    response: DebugProtocol.TerminateResponse,
    args: DebugProtocol.TerminateArguments,
    request?: DebugProtocol.Request,
  ): void {
    if (this.process) {
      this.process.kill();
      response.success = true;
      this.sendResponse(response);
      this.sendEvent(new TerminatedEvent());
    }
  }
}

export class ScummVMDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
  createDebugAdapterDescriptor(
    _session: vscode.DebugSession,
  ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
    // since DebugAdapterInlineImplementation is proposed API, a cast to <any> is required for now
    return <any>new vscode.DebugAdapterInlineImplementation(new ScummVMDebugSession());
  }
}

export class ScummVMDebugConfigurationProvider implements vscode.DebugConfigurationProvider {
  resolveDebugConfiguration(
    folder: vscode.WorkspaceFolder | undefined,
    config: vscode.DebugConfiguration,
    token?: vscode.CancellationToken,
  ): vscode.ProviderResult<vscode.DebugConfiguration> {
    // if launch.json is missing or empty
    if (!config.type && !config.request && !config.name) {
      if (folder) {
        config.type = 'agikit-scummvm';
        config.name = 'Run with ScummVM';
        config.request = 'launch';
        config.projectPath = folder?.uri.fsPath;
      }
    }

    if (!config.projectPath) {
      return vscode.window.showInformationMessage('Cannot find an AGI project to run').then(() => {
        return undefined; // abort launch
      });
    }

    return config;
  }
}
