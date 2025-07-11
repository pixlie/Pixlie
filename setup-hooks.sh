#!/bin/bash

# Setup script for Pixlie project git hooks and pre-commit

set -e

echo "🔧 Setting up Pixlie project hooks..."

# Configure git to use our custom hooks directory
echo "📁 Configuring git hooks directory..."
git config core.hooksPath .githooks

# Install pre-commit (if not already installed)
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    else
        echo "❌ Could not install pre-commit. Please install it manually:"
        echo "   pip install pre-commit"
        echo "   OR"
        echo "   brew install pre-commit"
        exit 1
    fi
fi

# Install pre-commit hooks
echo "🪝 Installing pre-commit hooks..."
pre-commit install

echo "✅ Setup complete!"
echo ""
echo "🎯 What was configured:"
echo "  • Git hooks directory set to .githooks/"
echo "  • Pre-commit hooks installed"
echo "  • CI checks will run before each commit"
echo ""
echo "🚀 To test the setup:"
echo "  • Make a change to any Rust or TypeScript file"
echo "  • Run: git add . && git commit -m 'test'"
echo "  • Watch the pre-commit checks run automatically"
echo ""
echo "🔧 To bypass hooks (emergency only):"
echo "  • Use: git commit --no-verify"