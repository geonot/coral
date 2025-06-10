"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const config = vscode.workspace.getConfiguration('coral');
    if (!config.get('lsp.enabled')) {
        return;
    }
    const serverPath = config.get('lsp.serverPath') || 'coral-lsp';
    const serverOptions = {
        command: serverPath,
        transport: node_1.TransportKind.stdio,
        options: {}
    };
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'coral' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{co,coral}')
        }
    };
    client = new node_1.LanguageClient('coral-lsp', 'Coral Language Server', serverOptions, clientOptions);
    // Status bar item
    const statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = "$(loading~spin) Coral LSP";
    statusBarItem.tooltip = "Coral Language Server starting...";
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);
    // Start the client
    client.start().then(() => {
        statusBarItem.text = "$(check) Coral LSP";
        statusBarItem.tooltip = "Coral Language Server is running";
        statusBarItem.color = undefined;
        vscode.window.showInformationMessage('Coral Language Server is ready!');
    }).catch((error) => {
        statusBarItem.text = "$(x) Coral LSP";
        statusBarItem.tooltip = "Coral Language Server failed to start";
        statusBarItem.color = new vscode.ThemeColor('errorForeground');
        vscode.window.showErrorMessage(`Failed to start Coral LSP: ${error.message}`);
    });
    // Register restart command
    const restartCommand = vscode.commands.registerCommand('coral.restart', async () => {
        statusBarItem.text = "$(loading~spin) Coral LSP";
        statusBarItem.tooltip = "Restarting Coral Language Server...";
        try {
            if (client.state === node_1.State.Running) {
                await client.stop();
            }
            await client.start();
            statusBarItem.text = "$(check) Coral LSP";
            statusBarItem.tooltip = "Coral Language Server is running";
            statusBarItem.color = undefined;
            vscode.window.showInformationMessage('Coral Language Server restarted successfully');
        }
        catch (error) {
            statusBarItem.text = "$(x) Coral LSP";
            statusBarItem.tooltip = "Coral Language Server failed to restart";
            statusBarItem.color = new vscode.ThemeColor('errorForeground');
            vscode.window.showErrorMessage(`Failed to restart Coral LSP: ${error.message}`);
        }
    });
    context.subscriptions.push(restartCommand);
    // Handle state changes
    client.onDidChangeState((event) => {
        switch (event.newState) {
            case node_1.State.Running:
                statusBarItem.text = "$(check) Coral LSP";
                statusBarItem.tooltip = "Coral Language Server is running";
                statusBarItem.color = undefined;
                break;
            case node_1.State.Starting:
                statusBarItem.text = "$(loading~spin) Coral LSP";
                statusBarItem.tooltip = "Coral Language Server starting...";
                statusBarItem.color = undefined;
                break;
            case node_1.State.Stopped:
                statusBarItem.text = "$(x) Coral LSP";
                statusBarItem.tooltip = "Coral Language Server stopped";
                statusBarItem.color = new vscode.ThemeColor('errorForeground');
                break;
        }
    });
}
exports.activate = activate;
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
exports.deactivate = deactivate;
