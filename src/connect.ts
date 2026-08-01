export type ConnectPhase = "idle" | "starting" | "active" | "stopping";

export interface ConnectStatus {
  phase: ConnectPhase;
  mode: "host" | "join" | null;
  localAddress: string | null;
  shareUri: string | null;
  playerCount: number;
  hostPort: number | null;
  route: "direct" | "relay" | null;
  rttMs: number | null;
  txBytes: number;
  rxBytes: number;
  error: string | null;
  message: string | null;
}

export interface IncomingInvite {
  id: number;
  uri: string;
}

export const emptyConnectStatus: ConnectStatus = {
  phase: "idle",
  mode: null,
  localAddress: null,
  shareUri: null,
  playerCount: 0,
  hostPort: null,
  route: null,
  rttMs: null,
  txBytes: 0,
  rxBytes: 0,
  error: null,
  message: null,
};

export function normalizeInvite(value: string): string {
  const trimmed = value.trim();
  const fragment = trimmed.match(/^https:\/\/ideaflash\.cn\/#\/join\/v1\/([^/?#\s]+)$/i);
  return fragment ? `sculk://join/v1/${fragment[1]}` : trimmed;
}

export function inviteFromDeepLinkUrls(urls: string[]): string | null {
  for (const url of urls) {
    const invite = normalizeInvite(url);
    if (invite.length <= 512 && /^sculk:\/\/join\/v1\/[A-Za-z0-9_-]+$/.test(invite)) {
      return invite;
    }
  }
  return null;
}
