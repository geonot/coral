#!/bin/bash

# Coral Language Development Environment Setup
# This script installs LSP, syntax highlighting, and Tree-sitter support

set -e

echo "🚀 Setting up Coral Language Development Environment..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORAL_DIR="$SCRIPT_DIR"

echo "📂 Working in: $CORAL_DIR"

# Build the Coral language and LSP server
echo "🔨 Building Coral language and LSP server..."
cd "$CORAL_DIR"
cargo build --release

# Install the LSP server globally
echo "📦 Installing Coral LSP server..."
if command -v cargo &> /dev/null; then
    cargo install --path . --bin coral-lsp
    echo "✅ Coral LSP server installed globally"
else
    echo "❌ Cargo not found. Please install Rust first."
    exit 1
fi

# Build VS Code extension
echo "🔧 Building VS Code extension..."
cd "$CORAL_DIR/vscode-coral"
npm install

# Install vsce locally to avoid permission issues
if ! command -v vsce &> /dev/null; then
    echo "📦 Installing vsce locally..."
    npm install vsce --save-dev
fi

npm run compile

# Package the extension using local vsce
echo "📦 Packaging VS Code extension..."
npx vsce package
VSIX_FILE=$(ls *.vsix | head -n1)
echo "✅ Extension packaged as: $VSIX_FILE"

# Install extension with proper user data directory
if command -v code &> /dev/null; then
    echo "🔌 Installing VS Code extension..."
    # Use proper user data directory to avoid running as root
    USER_DATA_DIR="$HOME/.vscode-coral"
    mkdir -p "$USER_DATA_DIR"
    
    # Check if running as root and adjust accordingly
    if [ "$EUID" -eq 0 ]; then
        echo "⚠️  Running as root detected. Using safe user data directory..."
        code --no-sandbox --user-data-dir="$USER_DATA_DIR" --install-extension "$VSIX_FILE"
    else
        code --install-extension "$VSIX_FILE"
    fi
    echo "✅ VS Code extension installed"
else
    echo "⚠️  VS Code not found. Please install the extension manually:"
    echo "   code --install-extension $PWD/$VSIX_FILE"
fi

# Setup Tree-sitter grammar
echo "🌳 Setting up Tree-sitter grammar..."
cd "$CORAL_DIR/tree-sitter-coral"
if command -v tree-sitter &> /dev/null; then
    npm install
    tree-sitter generate
    tree-sitter test
    echo "✅ Tree-sitter grammar built and tested"
else
    echo "⚠️  tree-sitter CLI not found. Installing locally..."
    npm install tree-sitter-cli
    npx tree-sitter generate
    npx tree-sitter test
    echo "✅ Tree-sitter grammar built and tested"
fi

# Create a test Coral file
echo "📝 Creating test Coral file..."
cd "$CORAL_DIR"
cat > example.co << 'EOF'
# Coral Language Example
message is 'Hello, Coral!'
count is 42

fn greet with name, greeting 'Hello'
    '{greeting}, {name}!'

object user
    name
    email
    age ? 0
    
    introduce
        'Hi, I am {name}'

store task
    description
    priority ? 1
    complete ? no
    
    make
        log create $description, $priority

# Test the language
greeting is greet 'World'
log greeting

# Create a user
person is user
person.name is 'Alice'
person.age is 30
intro is person.introduce
log intro
EOF

echo "✅ Created example.co"

echo ""
echo "🎉 Coral Language Development Environment Setup Complete!"
echo ""
echo "What's been installed:"
echo "  ✓ Coral Language Compiler (coral-parser)"
echo "  ✓ Coral LSP Server (coral-lsp)"
echo "  ✓ VS Code Extension with syntax highlighting"
echo "  ✓ Tree-sitter grammar for advanced parsing"
echo "  ✓ Code snippets and auto-completion"
echo ""
echo "Next steps:"
echo "  1. Open VS Code: code ."
echo "  2. Open example.co to test syntax highlighting"
echo "  3. Create new .co files and enjoy the developer experience!"
echo ""
echo "Commands available:"
echo "  coral-parser <file.co>    # Parse and run Coral code"
echo "  coral-lsp                 # Start LSP server manually"
echo ""
echo "Happy coding with Coral! 🪸"