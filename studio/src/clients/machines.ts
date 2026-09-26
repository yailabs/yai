/** Tenant machine identity is a pin, not an observation of a running host. */
export interface MachineAssetView {
  registration: {
    schema: "yai.machine_asset.v1";
    asset_id: string;
    integrity_digest: string;
    tenant_id: string;
    device_identity: string;
    address: string;
    port: number;
    management_user: string;
    host_public_key: string;
    approval_ref: string;
    approved_by_principal_id: string;
    approved_at_unix_ms: number;
  };
  revocation?: {
    schema: "yai.machine_revocation.v1";
    asset_id: string;
    tenant_id: string;
    device_identity: string;
    revoked_at_unix_ms: number;
    revoked_by_principal_id: string;
    reason: string;
    integrity_digest: string;
  } | null;
}

export interface MachineRegisterInput {
  tenant_id: string;
  address: string;
  port: number;
  management_user: string;
  host_public_key: string;
  approval_ref: string;
}

export interface MachineRevokeInput {
  tenant_id: string;
  asset_id: string;
  reason: string;
}
