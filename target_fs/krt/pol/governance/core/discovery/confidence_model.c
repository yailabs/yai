#include "../internal.h"

int yai_governance_confidence_label(double score, int *ambiguous) {
  if (!ambiguous) return -1;
  *ambiguous = (score >= 0.45 && score < 0.70) ? 1 : 0;
  return 0;
}
