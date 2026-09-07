"""Retry scheduling implementation. Release constraints live outside this file."""


def delay_ms(attempt):
    if not 1 <= attempt <= 20:
        raise ValueError("attempt must be between 1 and 20")
    return 100 * (2 ** attempt)
