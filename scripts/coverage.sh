#!/bin/bash
set -e

# Generate coverage report for contract tests
# Outputs HTML report to coverage/ directory and prints terminal summary

echo "Generating coverage report for contract tests..."

# Generate LCOV and HTML report
cargo llvm-cov --html --output-dir coverage

# Print summary
echo ""
echo "✓ Coverage report generated"
echo "  HTML report: coverage/index.html"
echo "  LCOV file: coverage/lcov.info"
echo ""
echo "Open coverage/index.html in a browser to view the detailed report."
