# Real failure evidence

## F-POST-I01-01 — sandbox socket denial

- Context: baseline/full Rust tests inside the restricted workspace sandbox.
- Exact observed failure: provider transport tests and the new loopback host
  fixture received `Operation not permitted` while binding a loopback socket.
- Classification: `DEPLOYMENT_LIMITATION` of the test sandbox, not a YAI or
  YVEX defect.
- Recheck: the focused loopback host test passed unchanged when explicitly run
  with local-socket permission.

No semantic pre-fix corruption, duplicate Turn, new owner, or schema defect was
manufactured for this closure. Legacy architectural failures are preserved in
the closed I01 dossier and Git history.

## External limitation

`YAI_EXTERNAL_PROVIDER_BASE_URL` and `YAI_EXTERNAL_PROVIDER_MODEL` were absent
from the closure environment. Live YAI↔YVEX model qualification was therefore
not executed. Classification: `DEPLOYMENT_LIMITATION`. The deterministic
loopback fixture qualifies the generic application/provider seam but does not
substitute for live model behavior.
