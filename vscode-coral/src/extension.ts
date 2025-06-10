import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
    ExecutableOptions,
    State,
    StateChangeEvent
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('coral');
    
    if (!config.get('lsp.enabled')) {
        return;
    }

    const serverPath = config.get('lsp.serverPath') as string || 'coral-lsp';

    const serverOptions: ServerOptions = {
        command: serverPath,
        transport: TransportKind.stdio,
        options: {} as ExecutableOptions
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'coral' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{co,coral}')
        }
    };

    client = new LanguageClient(
        'coral-lsp',
        'Coral Language Server',
        serverOptions,
        clientOptions
    );

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
    }).catch((error: Error) => {
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
            if (client.state === State.Running) {
                await client.stop();
            }
            await client.start();
            statusBarItem.text = "$(check) Coral LSP";
            statusBarItem.tooltip = "Coral Language Server is running";
            statusBarItem.color = undefined;
            vscode.window.showInformationMessage('Coral Language Server restarted successfully');
        } catch (error: any) {
            statusBarItem.text = "$(x) Coral LSP";
            statusBarItem.tooltip = "Coral Language Server failed to restart";
            statusBarItem.color = new vscode.ThemeColor('errorForeground');
            vscode.window.showErrorMessage(`Failed to restart Coral LSP: ${error.message}`);
        }
    });

    context.subscriptions.push(restartCommand);

    // Handle state changes
    client.onDidChangeState((event: StateChangeEvent) => {
        switch (event.newState) {
            case State.Running:
                statusBarItem.text = "$(check) Coral LSP";
                statusBarItem.tooltip = "Coral Language Server is running";
                statusBarItem.color = undefined;
                break;
            case State.Starting:
                statusBarItem.text = "$(loading~spin) Coral LSP";
                statusBarItem.tooltip = "Coral Language Server starting...";
                statusBarItem.color = undefined;
                break;
            case State.Stopped:
                statusBarItem.text = "$(x) Coral LSP";
                statusBarItem.tooltip = "Coral Language Server stopped";
                statusBarItem.color = new vscode.ThemeColor('errorForeground');
                break;
        }
    });
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}