/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from "path";
import {
  workspace,
  ExtensionContext,
  window,
  commands,
  ViewColumn,
  WebviewPanel,
  WorkspaceEdit,
  Selection,
  Uri,
} from "vscode";

import {
  Executable,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;
// type a = Parameters<>;

export async function activate(context: ExtensionContext) {
  // The server is implemented in node
  // let serverModule = context.asAbsolutePath(
  // 	path.join('server', 'out', 'server.js')
  // );
  // The debug options for the server
  // --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
  // let debugOptions = { execArgv: ['--nolazy', '--inspect=6009'] };

  const traceOutputChannel = window.createOutputChannel("Diagnostic Language Server trace");
  const command = process.env.SERVER_PATH || "tjs-language-server";
  const run: Executable = {
    command,
    options: {
      env: {
        ...process.env,
        // eslint-disable-next-line @typescript-eslint/naming-convention
        RUST_LOG: "debug",
      },
    },
  };
  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used

  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "plaintext" }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
    middleware: {},
    traceOutputChannel,
  };

  // Create the language client and start the client.
  client = new LanguageClient("diagnostic-ls", "diagnostic language server", serverOptions, clientOptions);

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // TODO: enable ts language server
  // const typescriptServerOptions: ServerOptions = {
  //   run: { module: typescriptServerModule, transport: TransportKind.ipc },
  //   debug: {
  //     module: typescriptServerModule,
  //     transport: TransportKind.ipc,
  //     options: debugOptions,
  //   },
  // };

  // Options to control the language client
  // const typescriptClientOptions: LanguageClientOptions = {
  //   // Register the server for plain text documents
  //   documentSelector: [
  //     { scheme: "file", language: "typescript" },
  //     { scheme: "file", language: "typescriptreact" },
  //   ],
  //   synchronize: {
  //     // Notify the server about file changes to '.clientrc files contained in the workspace
  //     fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
  //   },
  // };
  // let tsClient = new LanguageClient(
  //   "tjs-postfix-ts",
  //   "TJS Language Server ts",
  //   typescriptServerOptions,
  //   typescriptClientOptions
  // );

  // Create the language client and start the client.
  // Start the client. This will also launch the server

  client.start();
  // tsClient.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
