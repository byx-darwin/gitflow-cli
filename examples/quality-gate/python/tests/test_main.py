"""Tests for the main module."""

from example.main import add


def test_add():
    """Test that add returns the correct sum."""
    assert add(2, 3) == 5
