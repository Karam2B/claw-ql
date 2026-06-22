#!/usr/bin/env python3
"""Minimal MCP server that returns a single configured word."""

from pathlib import Path

from mcp.server.fastmcp import FastMCP

mcp = FastMCP("say-word")
WORD_FILE = Path(__file__).resolve().parent / "word.txt"


@mcp.tool()
def say_word() -> str:
    """Return the configured word."""
    return WORD_FILE.read_text(encoding="utf-8").strip()


if __name__ == "__main__":
    mcp.run()
