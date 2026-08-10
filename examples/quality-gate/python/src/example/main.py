"""Main module with a simple function."""


def add(a: int, b: int) -> int:
    """Return the sum of two integers."""
    return a + b


if __name__ == "__main__":
    print(f"2 + 3 = {add(2, 3)}")
