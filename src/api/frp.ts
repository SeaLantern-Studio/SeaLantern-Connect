import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CreateFrpTunnel,
  FrpClientStatus,
  FrpDownloadProgress,
  FrpNode,
  FrpProvider,
  FrpSessionStatus,
  FrpTunnel,
} from "@models/frp";

export function getFrpClientStatus(provider: FrpProvider): Promise<FrpClientStatus> {
  return invoke("get_frp_client_status", { provider });
}

export function downloadFrpClient(provider: FrpProvider): Promise<FrpClientStatus> {
  return invoke("download_frp_client", { provider });
}

export function onFrpDownloadProgress(
  handler: (progress: FrpDownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<FrpDownloadProgress>("frp-download-progress", ({ payload }) => handler(payload));
}

export function getFrpSessionStatus(provider: FrpProvider): Promise<FrpSessionStatus> {
  return invoke("get_frp_session_status", { provider });
}

export function loginSakuraFrp(credential: string): Promise<FrpSessionStatus> {
  return invoke("login_sakurafrp", { credential });
}

export function loginOpenFrp(): Promise<FrpSessionStatus> {
  return invoke("login_openfrp");
}

export function openSakuraKeys(): Promise<void> {
  return invoke("open_sakura_keys");
}

export function openSakuraPurchase(): Promise<void> {
  return invoke("open_sakura_purchase");
}

export function openPremium(): Promise<void> {
  return invoke("open_premium");
}

export function logoutFrp(provider: FrpProvider): Promise<FrpSessionStatus> {
  return invoke("logout_frp", { provider });
}

export function listFrpTunnels(provider: FrpProvider): Promise<FrpTunnel[]> {
  return invoke("list_frp_tunnels", { provider });
}

export function listFrpNodes(provider: FrpProvider): Promise<FrpNode[]> {
  return invoke("list_frp_nodes", { provider });
}

export function createFrpTunnel(
  provider: FrpProvider,
  request: CreateFrpTunnel,
): Promise<FrpTunnel[]> {
  return invoke("create_frp_tunnel", { provider, request });
}

export function deleteFrpTunnel(provider: FrpProvider, tunnelId: string): Promise<FrpTunnel[]> {
  return invoke("delete_frp_tunnel", { provider, tunnelId });
}

export function startFrpTunnel(provider: FrpProvider, tunnelId: string): Promise<FrpSessionStatus> {
  return invoke("start_frp_tunnel", { provider, tunnelId });
}

export function stopFrpTunnel(provider: FrpProvider): Promise<FrpSessionStatus> {
  return invoke("stop_frp_tunnel", { provider });
}
