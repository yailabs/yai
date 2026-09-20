# YAI public brand assets

Authority: asset provenance and usage only, not architecture or project status.

These are exact exports from the operator-supplied YAI brand kit. Keep the full
master/export kit outside the source repository; only the public assets used by
the README and organization profile live here. No regeneration, recoloring,
cropping or resampling was performed.

| Repository file | Source filename | Use |
| --- | --- | --- |
| [yai-readme-horizontal-dark.svg](yai-readme-horizontal-dark.svg) | `02-svg/yai-horizontal-white.svg` | Transparent white horizontal mark for dark GitHub themes; 969 × 384 viewBox |
| [yai-readme-horizontal-light.svg](yai-readme-horizontal-light.svg) | `02-svg/yai-horizontal-black.svg` | Transparent black horizontal mark for light GitHub themes; 969 × 384 viewBox |
| [yai-organization-512.png](yai-organization-512.png) | `00-pronti/avatar-yai-512.png` | Black-backed YAI avatar for the organization profile; 512 × 512 |

The root README selects the approved dark/light export from the viewer's color
scheme and renders it at width 320 with its original aspect ratio. Preserve the
intentional image and avatar padding. YAI's identity is distinct from YVEX's
identity.

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
