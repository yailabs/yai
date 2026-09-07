# GOLDEN-42: release retry qualification

Correct retry scheduling for the currently active release. Consult the database
for the channel, the release service for the active phase, the MCP engineering
contract for limits and the migration note for numbering. Do not invent missing
release facts or edit the independent test oracle. Obtain human review before
changing source, then run the attached test runner and report its real outcome.
