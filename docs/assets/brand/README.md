# YAI public brand assets

Authority: asset provenance and usage only, not architecture or project status.

These are unchanged exports from the owner-supplied `YAI-brand-kit/00-pronti`
directory. Keep the full master/export kit outside the source repository;
only the three public assets used by the README and organization profile live
here. No regeneration, recoloring, cropping or resampling was performed.

| Repository file | Source filename | Use |
| --- | --- | --- |
| [yai-readme-light.png](yai-readme-light.png) | `readme-light.png` | Transparent eye + black wordmark for light backgrounds; 1024 × 1102 |
| [yai-readme-dark.png](yai-readme-dark.png) | `readme-dark.png` | Transparent eye + white wordmark for dark backgrounds; 1024 × 1102 |
| [yai-organization-512.png](yai-organization-512.png) | `organization-yai-512.png` | Transparent square eye for the organization avatar; 512 × 512 |

The root README selects light/dark with `<picture>` and renders at width 280
with the original aspect ratio. Preserve the intentional avatar padding. The
kit's YAI identity is distinct from YVEX's identity.

Verify the imported bytes from this directory:

```sh
sha256sum --check SHA256SUMS
```

For the `yailabs` organization avatar, upload `yai-organization-512.png` through
the organization's Settings → Upload new picture. Committing the image does
**not** change the organization's avatar. See the
[GitHub profile procedure](https://docs.github.com/en/organizations/collaborating-with-groups-in-organizations/customizing-your-organizations-profile#changing-your-organizations-profile-picture).

This asset packaging adds no license or trademark permission; the repository's
[license](../../../LICENSE.md) and [legal posture](../../legal.md) remain
unchanged.
