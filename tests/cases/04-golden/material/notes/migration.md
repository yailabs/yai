# Attempt numbering migration

The public API numbers the first retry as attempt 1. The first retry must use
the base delay, not an already doubled value. Earlier internal code used a
zero-based retry counter. This historical material is not admitted merely by
being discoverable; identify and import its exact digest before relying on it.
